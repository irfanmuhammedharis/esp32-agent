//! USB-UART serial CLI — runs as a dedicated FreeRTOS task.
//!
//! ALL non-local commands are forwarded to the LLM for absolute control.
//! The LLM returns a Project (wiring guide + program) which is sent to the
//! program task for execution.

use crate::config::Config;
use crate::gpio::registry;
use crate::llm::deepseek::DeepSeekClient;
use crate::llm::plan::Project;
use crate::llm::LlmClient;
use crate::rtos::{Command, PLAN_QUEUE_DEPTH};
use esp_idf_hal::uart::UartDriver;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use heapless::String;
use log::{error, info};
use std::sync::mpsc::SyncSender;

/// Entry point for the serial task.
pub fn run(
    uart: UartDriver<'static>,
    tx: SyncSender<Project>,
    deepseek_key: String<128>,
    nvs: EspDefaultNvsPartition,
) {
    info!("[SERIAL] Task started.");

    let llm = match DeepSeekClient::new(deepseek_key.as_str()) {
        Ok(c) => c,
        Err(e) => {
            error!("[SERIAL] Failed to create LLM client: {:?}", e);
            match DeepSeekClient::new("") {
                Ok(c) => c,
                Err(_) => {
                    error!("[SERIAL] Cannot create placeholder LLM client");
                    return;
                }
            }
        }
    };

    let mut buf = [0u8; 256];
    let mut cursor = 0usize;

    print_prompt(&uart);

    loop {
        let mut byte = [0u8; 1];
        match uart.read(&mut byte, 100) {
            Ok(0) | Err(_) => {
                continue;
            }
            Ok(_) => {
                let b = byte[0];
                if b == b'\r' || b == b'\n' {
                    if cursor > 0 {
                        let _ = uart.write(b"\r\n");
                        let line = core::str::from_utf8(&buf[..cursor])
                            .unwrap_or("")
                            .trim();
                        handle_line(line, &tx, &uart, &llm, &nvs);
                        cursor = 0;
                        print_prompt(&uart);
                    }
                } else if b == 0x7F || b == 0x08 {
                    if cursor > 0 {
                        cursor -= 1;
                        let _ = uart.write(b"\x08 \x08");
                    }
                } else if cursor < buf.len() {
                    buf[cursor] = b;
                    cursor += 1;
                    let _ = uart.write(&[b]);
                }
            }
        }
    }
}

fn print_prompt(uart: &UartDriver) {
    let _ = uart.write(b"> ");
}

fn handle_line(
    line: &str,
    tx: &SyncSender<Project>,
    uart: &UartDriver,
    llm: &DeepSeekClient,
    nvs: &EspDefaultNvsPartition,
) {
    let cmd = parse(line);
    match cmd {
        Command::Help => {
            let msg = b"Local commands:\r\n  help\r\n  gpio list\r\n  config set <k> <v>\r\n  reset\r\n\r\nAny other text is sent to the LLM for project generation.\r\n";
            let _ = uart.write(msg);
        }
        Command::Reset => {
            let _ = uart.write(b"Resetting...\r\n");
            unsafe { esp_idf_sys::esp_restart() };
        }
        Command::GpioList => {
            for spec in registry::PIN_TABLE {
                let mut line_buf = String::<64>::new();
                let _ = core::fmt::write(
                    &mut line_buf,
                    format_args!(
                        "GPIO{:02} | out={} pwm={} in={} adc={} strap={}\r\n",
                        spec.number, spec.can_output, spec.can_pwm, spec.can_input, spec.can_adc, spec.strapping
                    ),
                );
                let _ = uart.write(line_buf.as_bytes());
            }
        }
        Command::LlmTask { text } => {
            let mut msg = String::<64>::new();
            let _ = core::fmt::write(&mut msg, format_args!("[LLM] '{}': ", text));
            let _ = uart.write(msg.as_bytes());

            match llm.complete(text.as_str()) {
                Ok(project) => {
                    let mut ok_msg = String::<256>::new();
                    let _ = core::fmt::write(
                        &mut ok_msg,
                        format_args!("{}\r\nWiring: {}\r\n", project.description, project.wiring_instructions)
                    );
                    let _ = uart.write(ok_msg.as_bytes());
                    let _ = tx.send(project);
                }
                Err(e) => {
                    error!("[SERIAL] LLM error: {:?}", e);
                    let mut err_msg = String::<128>::new();
                    let _ = core::fmt::write(&mut err_msg, format_args!("ERR: {:?}\r\n", e));
                    let _ = uart.write(err_msg.as_bytes());
                }
            }
        }
        Command::Noop => {}
        Command::ConfigSet { key, value } => {
            match Config::set(nvs, key.as_str(), value.as_str()) {
                Ok(()) => {
                    let mut msg = String::<64>::new();
                    let _ = core::fmt::write(&mut msg, format_args!("OK: {}={}\r\n", key, value));
                    let _ = uart.write(msg.as_bytes());
                }
                Err(e) => {
                    let mut msg = String::<128>::new();
                    let _ = core::fmt::write(&mut msg, format_args!("ERR saving {}: {:?}\r\n", key, e));
                    let _ = uart.write(msg.as_bytes());
                }
            }
        }
    }
}

fn parse(line: &str) -> Command {
    let line = line.trim();
    if line.eq_ignore_ascii_case("help") || line.eq_ignore_ascii_case("h") {
        return Command::Help;
    }
    if line.eq_ignore_ascii_case("reset") {
        return Command::Reset;
    }
    if line.eq_ignore_ascii_case("gpio list") {
        return Command::GpioList;
    }
    if let Some(rest) = line.strip_prefix("config set ") {
        let mut parts = rest.splitn(2, ' ');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            let mut key = String::<32>::new();
            let mut value = String::<128>::new();
            let _ = key.push_str(k);
            let _ = value.push_str(v);
            return Command::ConfigSet { key, value };
        }
    }
    if !line.is_empty() {
        let mut text = String::<256>::new();
        let _ = text.push_str(line);
        return Command::LlmTask { text };
    }
    Command::Noop
}
