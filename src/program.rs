//! On-device VM interpreter for LLM-generated projects.
//!
//! The VM runs a flat instruction list with labeled jumps,
//! variable storage, GPIO control, ADC reads, and PWM output.

use crate::gpio::GpioRegistry;
use crate::llm::plan::{Instruction, Project};
use esp_idf_hal::delay::FreeRtos;
use heapless::String;
use log::{info, warn};
use std::sync::mpsc::Receiver;

/// Virtual machine state.
pub struct Vm {
    vars: heapless::Vec<(String<8>, u16), 8>,
    labels: heapless::Vec<(String<16>, usize), 16>,
    pc: usize,
}

impl Vm {
    /// Load a program and build the label table.
    pub fn load(program: &[Instruction]) -> Self {
        let mut labels = heapless::Vec::new();
        for (idx, inst) in program.iter().enumerate() {
            if let Instruction::Label { name } = inst {
                let _ = labels.push((name.clone(), idx));
            }
        }
        Self {
            vars: heapless::Vec::new(),
            labels,
            pc: 0,
        }
    }

    fn get_var(&self, name: &str) -> u16 {
        self.vars
            .iter()
            .find(|(n, _)| n.as_str() == name)
            .map(|(_, v)| *v)
            .unwrap_or(0)
    }

    fn set_var(&mut self, name: String<8>, value: u16) {
        if let Some(pos) = self.vars.iter().position(|(n, _)| n.as_str() == name.as_str()) {
            self.vars[pos].1 = value;
        } else {
            let _ = self.vars.push((name, value));
        }
    }

    fn find_label(&self, name: &str) -> usize {
        self.labels
            .iter()
            .find(|(n, _)| n.as_str() == name)
            .map(|(_, idx)| *idx)
            .unwrap_or(0)
    }

    /// Run program from current PC until end.
    pub fn run(&mut self, program: &[Instruction], registry: &mut GpioRegistry) {
        while self.pc < program.len() {
            self.step(&program[self.pc], registry);
        }
    }

    fn step(&mut self, inst: &Instruction, registry: &mut GpioRegistry) {
        match inst {
            Instruction::ConfigOutput { pin } => {
                info!("[VM] ConfigOutput GPIO{}", pin);
                // Pin is already in InputOutput mode; this is a no-op.
            }
            Instruction::ConfigInput { pin } => {
                info!("[VM] ConfigInput GPIO{}", pin);
                // InputOutput pins are already capable of input reads.
                // Input-only pins (34-39) are lazy-initialised by claim_input.
                let _ = registry.claim_input(*pin);
            }
            Instruction::ConfigPwm { pin } => {
                info!("[VM] ConfigPwm GPIO{}", pin);
                if let Err(e) = registry.config_pwm(*pin) {
                    warn!("[VM] PWM config failed for GPIO{}: {}", pin, e);
                }
            }
            Instruction::SetHigh { pin } => {
                info!("[VM] SetHigh GPIO{}", pin);
                if let Some(p) = registry.claim_output(*pin) {
                    let _ = p.set_high();
                }
            }
            Instruction::SetLow { pin } => {
                info!("[VM] SetLow GPIO{}", pin);
                if let Some(p) = registry.claim_output(*pin) {
                    let _ = p.set_low();
                }
            }
            Instruction::SetPwm { pin, duty } => {
                info!("[VM] SetPwm GPIO{} duty={}", pin, duty);
                registry.set_pwm(*pin, *duty);
            }
            Instruction::ReadAdc { pin, var } => {
                let value = registry.read_adc(*pin).unwrap_or(0);
                info!("[VM] ReadAdc GPIO{} -> {}", pin, value);
                self.set_var(var.clone(), value);
            }
            Instruction::ReadDigital { pin, var } => {
                let value = if let Some(p) = registry.claim_input(*pin) {
                    p.is_high().unwrap_or(false) as u16
                } else {
                    0u16
                };
                info!("[VM] ReadDigital GPIO{} -> {}", pin, value);
                self.set_var(var.clone(), value);
            }
            Instruction::Delay { ms } => {
                info!("[VM] Delay {}ms", ms);
                FreeRtos::delay_ms(*ms);
            }
            Instruction::SetVar { var, value } => {
                info!("[VM] SetVar {} = {}", var.as_str(), value);
                self.set_var(var.clone(), *value);
            }
            Instruction::Label { name: _ } => {
                // No-op
            }
            Instruction::Jump { label } => {
                info!("[VM] Jump -> {}", label.as_str());
                self.pc = self.find_label(label.as_str());
                return;
            }
            Instruction::JumpIfLt { var, value, label } => {
                let current = self.get_var(var.as_str());
                if current < *value {
                    info!(
                        "[VM] JumpIfLt {} ({}) < {} -> {}",
                        var.as_str(), current, value, label.as_str()
                    );
                    self.pc = self.find_label(label.as_str());
                    return;
                }
            }
            Instruction::JumpIfGt { var, value, label } => {
                let current = self.get_var(var.as_str());
                if current > *value {
                    info!(
                        "[VM] JumpIfGt {} ({}) > {} -> {}",
                        var.as_str(), current, value, label.as_str()
                    );
                    self.pc = self.find_label(label.as_str());
                    return;
                }
            }
            Instruction::JumpIfEq { var, value, label } => {
                let current = self.get_var(var.as_str());
                if current == *value {
                    info!(
                        "[VM] JumpIfEq {} ({}) == {} -> {}",
                        var.as_str(), current, value, label.as_str()
                    );
                    self.pc = self.find_label(label.as_str());
                    return;
                }
            }
        }
        self.pc += 1;
    }
}

/// Main entry point for the program task.
/// Receives Projects from Telegram/Serial and runs them.
pub fn run(rx: Receiver<Project>, mut registry: GpioRegistry) {
    info!("[PROGRAM] Task started. Waiting for projects...");

    for project in rx {
        info!("[PROGRAM] New project loaded: {}", project.description);
        info!("[PROGRAM] Wiring guide: {}", project.wiring_instructions);

        // Run setup phase
        if !project.setup.is_empty() {
            info!("[PROGRAM] Running setup ({} instructions)", project.setup.len());
            let mut vm = Vm::load(&project.setup);
            vm.run(&project.setup, &mut registry);
        }

        // Run loop phase
        if project.loop_body.is_empty() {
            warn!("[PROGRAM] No loop_body — project is one-shot.");
            continue;
        }

        info!(
            "[PROGRAM] Entering loop ({} instructions, interval={}ms)",
            project.loop_body.len(),
            project.interval_ms
        );

        let mut loop_vm = Vm::load(&project.loop_body);

        loop {
            loop_vm.pc = 0;
            loop_vm.run(&project.loop_body, &mut registry);
            FreeRtos::delay_ms(project.interval_ms);
        }
    }

    info!("[PROGRAM] Task exiting.");
}
