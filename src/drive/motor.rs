use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;

pub struct Motor {
    fwd_pin: OutputPin,
    bwd_pin: OutputPin,
    pwm_pin: OutputPin,
}

impl Motor {
    pub fn new(gpio: &Gpio, fwd_pin_number: u8, bwd_pin_number: u8, pwm_pin_number: u8) -> Self {
        Self {
            fwd_pin: gpio
                .get(fwd_pin_number)
                .expect("Error: Couldn't get gpio")
                .into_output_low(),
            bwd_pin: gpio
                .get(bwd_pin_number)
                .expect("Error: Couldn't get gpio")
                .into_output_low(),
            pwm_pin: gpio
                .get(pwm_pin_number)
                .expect("Error: Couldn't get gpio")
                .into_output_low(),
        }
    }

    pub fn enable_fwd(&mut self, pwm_frequency: f64, duty_cycle: f64) {
        self.bwd_pin.set_low();
        self.pwm_pin.set_high();
        self.fwd_pin.set_high();
        self.pwm_pin
            .set_pwm_frequency(pwm_frequency, duty_cycle)
            .expect("Counldn't set PWM frequency");
    }

    pub fn enable_bwd(&mut self, pwm_frequency: f64, duty_cycle: f64) {
        self.fwd_pin.set_low();
        self.pwm_pin.set_high();
        self.bwd_pin.set_high();
        self.pwm_pin
            .set_pwm_frequency(pwm_frequency, duty_cycle)
            .expect("Counldn't set PWM frequency");
    }

    pub fn stop(&mut self) {
        self.pwm_pin.set_low();
        self.fwd_pin.set_low();
        self.bwd_pin.set_low();
    }
}