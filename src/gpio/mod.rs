//! GPIO registry and safe handle store.

use esp_idf_hal::gpio::*;
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::units::Hertz;
use esp_idf_sys::{
    adc1_config_channel_atten, adc1_config_width, adc1_get_raw, adc_atten_t_ADC_ATTEN_DB_11,
    adc_bits_width_t_ADC_WIDTH_BIT_12,
};
use log::{info, warn};

pub mod registry;

/// Owned registry of all usable GPIO handles.
/// Only the executor task should mutate this after creation.
pub struct GpioRegistry {
    // Dual-mode pins (input + output)
    pub pin2: Option<PinDriver<'static, Gpio2, InputOutput>>,
    pub pin4: Option<PinDriver<'static, Gpio4, InputOutput>>,
    pub pin5: Option<PinDriver<'static, Gpio5, InputOutput>>,
    pub pin12: Option<PinDriver<'static, Gpio12, InputOutput>>,
    pub pin13: Option<PinDriver<'static, Gpio13, InputOutput>>,
    pub pin14: Option<PinDriver<'static, Gpio14, InputOutput>>,
    pub pin15: Option<PinDriver<'static, Gpio15, InputOutput>>,
    pub pin16: Option<PinDriver<'static, Gpio16, InputOutput>>,
    pub pin17: Option<PinDriver<'static, Gpio17, InputOutput>>,
    pub pin18: Option<PinDriver<'static, Gpio18, InputOutput>>,
    pub pin19: Option<PinDriver<'static, Gpio19, InputOutput>>,
    pub pin21: Option<PinDriver<'static, Gpio21, InputOutput>>,
    pub pin22: Option<PinDriver<'static, Gpio22, InputOutput>>,
    pub pin23: Option<PinDriver<'static, Gpio23, InputOutput>>,
    pub pin25: Option<PinDriver<'static, Gpio25, InputOutput>>,
    pub pin26: Option<PinDriver<'static, Gpio26, InputOutput>>,
    pub pin27: Option<PinDriver<'static, Gpio27, InputOutput>>,
    pub pin32: Option<PinDriver<'static, Gpio32, InputOutput>>,
    pub pin33: Option<PinDriver<'static, Gpio33, InputOutput>>,

    // Input-only pins (ADC capable) — lazy-initialised on first use
    pin34: Option<PinDriver<'static, Gpio34, Input>>,
    pin35: Option<PinDriver<'static, Gpio35, Input>>,
    pin36: Option<PinDriver<'static, Gpio36, Input>>,
    pin39: Option<PinDriver<'static, Gpio39, Input>>,

    // PWM timer and channel pool
    pwm_timer: Option<LedcTimerDriver<'static, esp_idf_hal::ledc::HTIMER0>>,
    pwm0: Option<(u8, LedcDriver<'static>)>,
    pwm1: Option<(u8, LedcDriver<'static>)>,
    pwm2: Option<(u8, LedcDriver<'static>)>,
    pwm3: Option<(u8, LedcDriver<'static>)>,

    // Raw high-speed LEDC channels available for allocation
    hchannel0: Option<esp_idf_hal::ledc::HCHANNEL0>,
    hchannel1: Option<esp_idf_hal::ledc::HCHANNEL1>,
    hchannel2: Option<esp_idf_hal::ledc::HCHANNEL2>,
    hchannel3: Option<esp_idf_hal::ledc::HCHANNEL3>,
}

impl GpioRegistry {
    /// Construct the registry by safely extracting individual pins from
    /// `Peripherals` using `core::mem::replace` with dummy values.
    pub fn from_peripherals(p: &mut Peripherals) -> anyhow::Result<Self> {
        info!("[GPIO] Initialising registry...");

        let timer = unsafe {
            LedcTimerDriver::new(
                core::mem::replace(&mut p.hledc.timer0, esp_idf_hal::ledc::HTIMER0::new()),
                &TimerConfig::default()
                    .frequency(Hertz(5_000))
                    .resolution(Resolution::Bits8),
            )?
        };

        // Configure ADC1 for 12-bit reads (done once)
        unsafe {
            adc1_config_width(adc_bits_width_t_ADC_WIDTH_BIT_12);
        }

        unsafe {
            Ok(Self {
                pin2: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio2,
                    Gpio2::new(),
                ))?),
                pin4: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio4,
                    Gpio4::new(),
                ))?),
                pin5: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio5,
                    Gpio5::new(),
                ))?),
                pin12: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio12,
                    Gpio12::new(),
                ))?),
                pin13: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio13,
                    Gpio13::new(),
                ))?),
                pin14: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio14,
                    Gpio14::new(),
                ))?),
                pin15: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio15,
                    Gpio15::new(),
                ))?),
                pin16: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio16,
                    Gpio16::new(),
                ))?),
                pin17: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio17,
                    Gpio17::new(),
                ))?),
                pin18: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio18,
                    Gpio18::new(),
                ))?),
                pin19: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio19,
                    Gpio19::new(),
                ))?),
                pin21: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio21,
                    Gpio21::new(),
                ))?),
                pin22: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio22,
                    Gpio22::new(),
                ))?),
                pin23: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio23,
                    Gpio23::new(),
                ))?),
                pin25: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio25,
                    Gpio25::new(),
                ))?),
                pin26: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio26,
                    Gpio26::new(),
                ))?),
                pin27: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio27,
                    Gpio27::new(),
                ))?),
                pin32: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio32,
                    Gpio32::new(),
                ))?),
                pin33: Some(PinDriver::input_output(core::mem::replace(
                    &mut p.pins.gpio33,
                    Gpio33::new(),
                ))?),
                // Input-only pins — created on first use
                pin34: None,
                pin35: None,
                pin36: None,
                pin39: None,
                pwm_timer: Some(timer),
                pwm0: None,
                pwm1: None,
                pwm2: None,
                pwm3: None,
                hchannel0: Some(core::mem::replace(
                    &mut p.hledc.channel0,
                    esp_idf_hal::ledc::HCHANNEL0::new(),
                )),
                hchannel1: Some(core::mem::replace(
                    &mut p.hledc.channel1,
                    esp_idf_hal::ledc::HCHANNEL1::new(),
                )),
                hchannel2: Some(core::mem::replace(
                    &mut p.hledc.channel2,
                    esp_idf_hal::ledc::HCHANNEL2::new(),
                )),
                hchannel3: Some(core::mem::replace(
                    &mut p.hledc.channel3,
                    esp_idf_hal::ledc::HCHANNEL3::new(),
                )),
            })
        }
    }

    /// Claim a mutable reference to an output pin by number.
    pub fn claim_output(
        &mut self,
        pin: u8,
    ) -> Option<&mut dyn embedded_hal::digital::OutputPin<Error = GpioError>> {
        // If pin was previously used for PWM, drop the LEDC driver first
        self.release_pwm(pin);
        match pin {
            2 => { if self.pin2.is_none() { self.pin2 = PinDriver::input_output(unsafe { Gpio2::new() }).ok(); } self.pin2.as_mut().map(|p| p as _) }
            4 => { if self.pin4.is_none() { self.pin4 = PinDriver::input_output(unsafe { Gpio4::new() }).ok(); } self.pin4.as_mut().map(|p| p as _) }
            5 => { if self.pin5.is_none() { self.pin5 = PinDriver::input_output(unsafe { Gpio5::new() }).ok(); } self.pin5.as_mut().map(|p| p as _) }
            12 => { if self.pin12.is_none() { self.pin12 = PinDriver::input_output(unsafe { Gpio12::new() }).ok(); } self.pin12.as_mut().map(|p| p as _) }
            13 => { if self.pin13.is_none() { self.pin13 = PinDriver::input_output(unsafe { Gpio13::new() }).ok(); } self.pin13.as_mut().map(|p| p as _) }
            14 => { if self.pin14.is_none() { self.pin14 = PinDriver::input_output(unsafe { Gpio14::new() }).ok(); } self.pin14.as_mut().map(|p| p as _) }
            15 => { if self.pin15.is_none() { self.pin15 = PinDriver::input_output(unsafe { Gpio15::new() }).ok(); } self.pin15.as_mut().map(|p| p as _) }
            16 => { if self.pin16.is_none() { self.pin16 = PinDriver::input_output(unsafe { Gpio16::new() }).ok(); } self.pin16.as_mut().map(|p| p as _) }
            17 => { if self.pin17.is_none() { self.pin17 = PinDriver::input_output(unsafe { Gpio17::new() }).ok(); } self.pin17.as_mut().map(|p| p as _) }
            18 => { if self.pin18.is_none() { self.pin18 = PinDriver::input_output(unsafe { Gpio18::new() }).ok(); } self.pin18.as_mut().map(|p| p as _) }
            19 => { if self.pin19.is_none() { self.pin19 = PinDriver::input_output(unsafe { Gpio19::new() }).ok(); } self.pin19.as_mut().map(|p| p as _) }
            21 => { if self.pin21.is_none() { self.pin21 = PinDriver::input_output(unsafe { Gpio21::new() }).ok(); } self.pin21.as_mut().map(|p| p as _) }
            22 => { if self.pin22.is_none() { self.pin22 = PinDriver::input_output(unsafe { Gpio22::new() }).ok(); } self.pin22.as_mut().map(|p| p as _) }
            23 => { if self.pin23.is_none() { self.pin23 = PinDriver::input_output(unsafe { Gpio23::new() }).ok(); } self.pin23.as_mut().map(|p| p as _) }
            25 => { if self.pin25.is_none() { self.pin25 = PinDriver::input_output(unsafe { Gpio25::new() }).ok(); } self.pin25.as_mut().map(|p| p as _) }
            26 => { if self.pin26.is_none() { self.pin26 = PinDriver::input_output(unsafe { Gpio26::new() }).ok(); } self.pin26.as_mut().map(|p| p as _) }
            27 => { if self.pin27.is_none() { self.pin27 = PinDriver::input_output(unsafe { Gpio27::new() }).ok(); } self.pin27.as_mut().map(|p| p as _) }
            32 => { if self.pin32.is_none() { self.pin32 = PinDriver::input_output(unsafe { Gpio32::new() }).ok(); } self.pin32.as_mut().map(|p| p as _) }
            33 => { if self.pin33.is_none() { self.pin33 = PinDriver::input_output(unsafe { Gpio33::new() }).ok(); } self.pin33.as_mut().map(|p| p as _) }
            _ => None,
        }
    }

    /// Claim a mutable reference to an input pin by number.
    pub fn claim_input(
        &mut self,
        pin: u8,
    ) -> Option<&mut dyn embedded_hal::digital::InputPin<Error = GpioError>> {
        // If pin was previously used for PWM, drop the LEDC driver first
        self.release_pwm(pin);
        match pin {
            2 => { if self.pin2.is_none() { self.pin2 = PinDriver::input_output(unsafe { Gpio2::new() }).ok(); } self.pin2.as_mut().map(|p| p as _) }
            4 => { if self.pin4.is_none() { self.pin4 = PinDriver::input_output(unsafe { Gpio4::new() }).ok(); } self.pin4.as_mut().map(|p| p as _) }
            5 => { if self.pin5.is_none() { self.pin5 = PinDriver::input_output(unsafe { Gpio5::new() }).ok(); } self.pin5.as_mut().map(|p| p as _) }
            12 => { if self.pin12.is_none() { self.pin12 = PinDriver::input_output(unsafe { Gpio12::new() }).ok(); } self.pin12.as_mut().map(|p| p as _) }
            13 => { if self.pin13.is_none() { self.pin13 = PinDriver::input_output(unsafe { Gpio13::new() }).ok(); } self.pin13.as_mut().map(|p| p as _) }
            14 => { if self.pin14.is_none() { self.pin14 = PinDriver::input_output(unsafe { Gpio14::new() }).ok(); } self.pin14.as_mut().map(|p| p as _) }
            15 => { if self.pin15.is_none() { self.pin15 = PinDriver::input_output(unsafe { Gpio15::new() }).ok(); } self.pin15.as_mut().map(|p| p as _) }
            16 => { if self.pin16.is_none() { self.pin16 = PinDriver::input_output(unsafe { Gpio16::new() }).ok(); } self.pin16.as_mut().map(|p| p as _) }
            17 => { if self.pin17.is_none() { self.pin17 = PinDriver::input_output(unsafe { Gpio17::new() }).ok(); } self.pin17.as_mut().map(|p| p as _) }
            18 => { if self.pin18.is_none() { self.pin18 = PinDriver::input_output(unsafe { Gpio18::new() }).ok(); } self.pin18.as_mut().map(|p| p as _) }
            19 => { if self.pin19.is_none() { self.pin19 = PinDriver::input_output(unsafe { Gpio19::new() }).ok(); } self.pin19.as_mut().map(|p| p as _) }
            21 => { if self.pin21.is_none() { self.pin21 = PinDriver::input_output(unsafe { Gpio21::new() }).ok(); } self.pin21.as_mut().map(|p| p as _) }
            22 => { if self.pin22.is_none() { self.pin22 = PinDriver::input_output(unsafe { Gpio22::new() }).ok(); } self.pin22.as_mut().map(|p| p as _) }
            23 => { if self.pin23.is_none() { self.pin23 = PinDriver::input_output(unsafe { Gpio23::new() }).ok(); } self.pin23.as_mut().map(|p| p as _) }
            25 => { if self.pin25.is_none() { self.pin25 = PinDriver::input_output(unsafe { Gpio25::new() }).ok(); } self.pin25.as_mut().map(|p| p as _) }
            26 => { if self.pin26.is_none() { self.pin26 = PinDriver::input_output(unsafe { Gpio26::new() }).ok(); } self.pin26.as_mut().map(|p| p as _) }
            27 => { if self.pin27.is_none() { self.pin27 = PinDriver::input_output(unsafe { Gpio27::new() }).ok(); } self.pin27.as_mut().map(|p| p as _) }
            32 => { if self.pin32.is_none() { self.pin32 = PinDriver::input_output(unsafe { Gpio32::new() }).ok(); } self.pin32.as_mut().map(|p| p as _) }
            33 => { if self.pin33.is_none() { self.pin33 = PinDriver::input_output(unsafe { Gpio33::new() }).ok(); } self.pin33.as_mut().map(|p| p as _) }
            34 => self.ensure_input_34().map(|p| p as _),
            35 => self.ensure_input_35().map(|p| p as _),
            36 => self.ensure_input_36().map(|p| p as _),
            39 => self.ensure_input_39().map(|p| p as _),
            _ => None,
        }
    }

    fn ensure_input_34(&mut self) -> Option<&mut PinDriver<'static, Gpio34, Input>> {
        if self.pin34.is_none() {
            self.pin34 = PinDriver::input(unsafe { Gpio34::new() }).ok();
        }
        self.pin34.as_mut()
    }

    fn ensure_input_35(&mut self) -> Option<&mut PinDriver<'static, Gpio35, Input>> {
        if self.pin35.is_none() {
            self.pin35 = PinDriver::input(unsafe { Gpio35::new() }).ok();
        }
        self.pin35.as_mut()
    }

    fn ensure_input_36(&mut self) -> Option<&mut PinDriver<'static, Gpio36, Input>> {
        if self.pin36.is_none() {
            self.pin36 = PinDriver::input(unsafe { Gpio36::new() }).ok();
        }
        self.pin36.as_mut()
    }

    fn ensure_input_39(&mut self) -> Option<&mut PinDriver<'static, Gpio39, Input>> {
        if self.pin39.is_none() {
            self.pin39 = PinDriver::input(unsafe { Gpio39::new() }).ok();
        }
        self.pin39.as_mut()
    }

    /// Read ADC value from an ADC-capable pin (GPIO32-39).
    /// Returns 0-4095 for 12-bit resolution.
    pub fn read_adc(&mut self, pin: u8) -> Option<u16> {
        let channel = match pin {
            32 => esp_idf_sys::adc1_channel_t_ADC1_CHANNEL_4,
            33 => esp_idf_sys::adc1_channel_t_ADC1_CHANNEL_5,
            34 => esp_idf_sys::adc1_channel_t_ADC1_CHANNEL_6,
            35 => esp_idf_sys::adc1_channel_t_ADC1_CHANNEL_7,
            36 => esp_idf_sys::adc1_channel_t_ADC1_CHANNEL_0,
            39 => esp_idf_sys::adc1_channel_t_ADC1_CHANNEL_3,
            _ => {
                warn!("[GPIO] Pin {} is not ADC-capable", pin);
                return None;
            }
        };

        unsafe {
            if adc1_config_channel_atten(channel, adc_atten_t_ADC_ATTEN_DB_11) != 0 {
                warn!("[GPIO] adc1_config_channel_atten failed for pin {}", pin);
                return None;
            }
            let raw = adc1_get_raw(channel);
            if raw < 0 {
                warn!("[GPIO] adc1_get_raw failed for pin {}", pin);
                return None;
            }
            Some(raw as u16)
        }
    }

    /// Configure a pin for PWM output using LEDC.
    pub fn config_pwm(&mut self, pin: u8) -> Result<(), &'static str> {
        // Already configured?
        if self.pwm0.as_ref().map(|(p, _)| *p == pin).unwrap_or(false)
            || self.pwm1.as_ref().map(|(p, _)| *p == pin).unwrap_or(false)
            || self.pwm2.as_ref().map(|(p, _)| *p == pin).unwrap_or(false)
            || self.pwm3.as_ref().map(|(p, _)| *p == pin).unwrap_or(false)
        {
            return Ok(());
        }

        if !registry::lookup(pin).map(|s| s.can_output).unwrap_or(false) {
            warn!("[GPIO] Pin {} is not output-capable, cannot use PWM", pin);
            return Err("Pin is not output-capable");
        }

        // Free the pin from GPIO use so LEDC can take over
        self.release_pin(pin);

        let timer = self
            .pwm_timer
            .as_ref()
            .ok_or("PWM timer not initialized")?;

        let raw_pin = unsafe { AnyOutputPin::new(pin as i32) };

        macro_rules! try_slot {
            ($slot:expr, $ch:expr) => {
                if $slot.is_none() && $ch.is_some() {
                    let ch = $ch.take().unwrap();
                    match LedcDriver::new(ch, timer, raw_pin) {
                        Ok(driver) => {
                            // Sound: LedcDriver does not actually borrow the timer
                            // (fields are copied, PhantomData only).
                            let driver: LedcDriver<'static> =
                                unsafe { core::mem::transmute(driver) };
                            $slot = Some((pin, driver));
                            info!("[GPIO] PWM configured on GPIO{}", pin);
                            return Ok(());
                        }
                        Err(e) => {
                            warn!(
                                "[GPIO] LedcDriver::new failed for GPIO{}: {:?}",
                                pin, e
                            );
                            return Err("LEDC driver creation failed");
                        }
                    }
                }
            };
        }

        try_slot!(self.pwm0, self.hchannel0);
        try_slot!(self.pwm1, self.hchannel1);
        try_slot!(self.pwm2, self.hchannel2);
        try_slot!(self.pwm3, self.hchannel3);

        warn!("[GPIO] No PWM channels available for GPIO{}", pin);
        Err("No PWM channels available")
    }

    /// Set PWM duty cycle for a configured pin (0-255).
    pub fn set_pwm(&mut self, pin: u8, duty: u8) {
        let driver_opt = self
            .pwm0
            .as_mut()
            .filter(|(p, _)| *p == pin)
            .map(|(_, d)| d)
            .or_else(|| {
                self.pwm1
                    .as_mut()
                    .filter(|(p, _)| *p == pin)
                    .map(|(_, d)| d)
            })
            .or_else(|| {
                self.pwm2
                    .as_mut()
                    .filter(|(p, _)| *p == pin)
                    .map(|(_, d)| d)
            })
            .or_else(|| {
                self.pwm3
                    .as_mut()
                    .filter(|(p, _)| *p == pin)
                    .map(|(_, d)| d)
            });

        if let Some(driver) = driver_opt {
            let _ = driver.set_duty(duty as u32);
        } else {
            warn!("[GPIO] set_pwm called on unconfigured pin {}", pin);
        }
    }

    /// Release a PWM driver for a given pin.
    /// Note: the LEDC channel peripheral is consumed and cannot be recovered
    /// until a future enhancement adds channel factory storage.
    fn release_pwm(&mut self, pin: u8) {
        macro_rules! release_slot {
            ($slot:expr) => {
                if $slot.as_ref().map(|(p, _)| *p == pin).unwrap_or(false) {
                    let (_, driver) = $slot.take().unwrap();
                    core::mem::drop(driver);
                }
            };
        }
        release_slot!(self.pwm0);
        release_slot!(self.pwm1);
        release_slot!(self.pwm2);
        release_slot!(self.pwm3);
    }

    /// Release a pin from GPIO use (drop its PinDriver) so it can be reused for PWM.
    fn release_pin(&mut self, pin: u8) {
        match pin {
            2 => self.pin2 = None,
            4 => self.pin4 = None,
            5 => self.pin5 = None,
            12 => self.pin12 = None,
            13 => self.pin13 = None,
            14 => self.pin14 = None,
            15 => self.pin15 = None,
            16 => self.pin16 = None,
            17 => self.pin17 = None,
            18 => self.pin18 = None,
            19 => self.pin19 = None,
            21 => self.pin21 = None,
            22 => self.pin22 = None,
            23 => self.pin23 = None,
            25 => self.pin25 = None,
            26 => self.pin26 = None,
            27 => self.pin27 = None,
            32 => self.pin32 = None,
            33 => self.pin33 = None,
            _ => {}
        }
    }
}
