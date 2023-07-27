use std::f32::INFINITY;
use std::thread;
use std::time::{Duration, Instant};

use rppal::gpio::{Gpio, InputPin, Level, OutputPin, Trigger};

pub struct HcSr04 {
    trig: OutputPin,
    echo: InputPin,
    sound_speed: f32,
    timeout: Duration,
}

impl HcSr04 {
    fn calculate_parameters(temperature: f32) -> (f32, Duration) {
        const SOUND_SPEED_0C: f32 = 331.3; // m/s
        const SOUND_SPEED_INCR: f32 = 0.606; // (m/s)/*C
        const MAX_RANGE: f32 = 4.; // m
        let sound_speed: f32 = SOUND_SPEED_0C + temperature * SOUND_SPEED_INCR;
        let timeout = Duration::from_secs_f32(MAX_RANGE / sound_speed * 2.);
        (sound_speed, timeout)
    }

    pub fn new(gpio: &Gpio, trig_pin: u8, echo_pin: u8, temperature: f32) -> Self {
        let (sound_speed, timeout) = Self::calculate_parameters(temperature);
        let mut echo = gpio
            .get(echo_pin)
            .expect("Error: Couldn't get gpio")
            .into_input_pulldown();
        echo.set_interrupt(Trigger::Both)
            .expect("Error: Couldn't set interrupt");
        Self {
            trig: gpio
                .get(trig_pin)
                .expect("Error: Couldn't get gpio")
                .into_output_low(),
            echo,
            sound_speed,
            timeout,
        }
    }

    pub fn measure_distance(&mut self) -> f32 {
        self.trig.set_high();
        thread::sleep(Duration::from_micros(10));
        self.trig.set_low();

        while self
            .echo
            .poll_interrupt(false, None)
            .expect("Error: Couldn't poll interrupt")
            != Some(Level::High)
        {}
        let instant = Instant::now();

        if self
            .echo
            .poll_interrupt(false, Some(self.timeout))
            .expect("Error: Couldn't poll interrupt")
            != Some(Level::Low)
        {
            return f32::INFINITY;
        }

        self.sound_speed * instant.elapsed().as_secs_f32() * 0.5
    }

    /// Perform `amount` measurements, discard the minimum and maximum, and return the mean
    pub fn precise_distance_measurement(&mut self, amount: usize) -> f32 {
        let mut measurements = Vec::new();
        let mut max = 0.0;
        let mut min = INFINITY;

        for _ in 0..amount {
            let val = self.measure_distance();
            measurements.push(val);
            if val < min {
                min = val;
            }
            if val > max {
                max = val;
            }
        }
        measurements
            .into_iter()
            .filter(|x| *x != min && *x != max)
            .sum::<f32>()
            / ((amount - 2) as f32)
    }
}
