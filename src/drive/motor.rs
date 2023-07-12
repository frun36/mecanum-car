use rppal::gpio::Error;
use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;

/// Allows control over an H bridge-connected DC motor
pub struct Motor {
    fwd_pin: OutputPin,
    bwd_pin: OutputPin,
    enable_pin: OutputPin,
}

impl Motor {
    /// Creates new `Motor` instance
    pub fn new(
        gpio: &Gpio,
        fwd_pin_number: u8,
        bwd_pin_number: u8,
        pwm_pin_number: u8,
    ) -> Result<Self, Error> {
        Ok(Self {
            fwd_pin: gpio.get(fwd_pin_number)?.into_output_low(),
            bwd_pin: gpio.get(bwd_pin_number)?.into_output_low(),
            enable_pin: gpio.get(pwm_pin_number)?.into_output_low(),
        })
    }

    /// Starts spinning the motor forward, with specified pwm parameters
    pub fn enable_fwd(&mut self, pwm_frequency: f64, duty_cycle: f64) -> Result<(), Error> {
        self.bwd_pin.set_low();
        self.enable_pin.set_high();
        self.fwd_pin.set_high();
        self.enable_pin
            .set_pwm_frequency(pwm_frequency, duty_cycle)?;
        Ok(())
    }

    /// Starts spinning the motor backward, with specified pwm parameters
    pub fn enable_bwd(&mut self, pwm_frequency: f64, duty_cycle: f64) -> Result<(), Error> {
        self.fwd_pin.set_low();
        self.enable_pin.set_high();
        self.bwd_pin.set_high();
        self.enable_pin
            .set_pwm_frequency(pwm_frequency, duty_cycle)?;
        Ok(())
    }

    /// Stops the motor
    pub fn stop(&mut self) {
        self.enable_pin.set_low();
        self.fwd_pin.set_low();
        self.bwd_pin.set_low();
    }

    /// Prints pins used by motor
    pub fn print_pins(&self) {
        println!("Enable pin: {}", self.enable_pin.pin());
        println!("Forward pin: {}", self.fwd_pin.pin());
        println!("Backward pin: {}", self.bwd_pin.pin());
    }
}
