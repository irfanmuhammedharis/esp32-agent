//! Compile-time pin capability table.

/// Capability descriptor for a single GPIO.
#[derive(Copy, Clone, Debug)]
pub struct PinSpec {
    pub number: u8,
    pub can_output: bool,
    pub can_pwm: bool,
    pub can_input: bool,
    pub can_adc: bool,
    pub strapping: bool,
}

/// All usable GPIOs on the classic ESP32 (WROOM / WROVER).
pub static PIN_TABLE: &[PinSpec] = &[
    PinSpec { number:  2, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: true  },
    PinSpec { number:  4, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number:  5, can_output: true,  can_pwm: true,  can_input: true,  can_adc: false, strapping: true  },
    PinSpec { number: 12, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: true  },
    PinSpec { number: 13, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number: 14, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number: 15, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: true  },
    PinSpec { number: 16, can_output: true,  can_pwm: true,  can_input: true,  can_adc: false, strapping: false },
    PinSpec { number: 17, can_output: true,  can_pwm: true,  can_input: true,  can_adc: false, strapping: false },
    PinSpec { number: 18, can_output: true,  can_pwm: true,  can_input: true,  can_adc: false, strapping: false },
    PinSpec { number: 19, can_output: true,  can_pwm: true,  can_input: true,  can_adc: false, strapping: false },
    PinSpec { number: 21, can_output: true,  can_pwm: true,  can_input: true,  can_adc: false, strapping: false },
    PinSpec { number: 22, can_output: true,  can_pwm: true,  can_input: true,  can_adc: false, strapping: false },
    PinSpec { number: 23, can_output: true,  can_pwm: true,  can_input: true,  can_adc: false, strapping: false },
    PinSpec { number: 25, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number: 26, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number: 27, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number: 32, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number: 33, can_output: true,  can_pwm: true,  can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number: 34, can_output: false, can_pwm: false, can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number: 35, can_output: false, can_pwm: false, can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number: 36, can_output: false, can_pwm: false, can_input: true,  can_adc: true,  strapping: false },
    PinSpec { number: 39, can_output: false, can_pwm: false, can_input: true,  can_adc: true,  strapping: false },
];

/// Look up a pin's capabilities by number.
pub fn lookup(pin: u8) -> Option<PinSpec> {
    PIN_TABLE.iter().copied().find(|p| p.number == pin)
}
