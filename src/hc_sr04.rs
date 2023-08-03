use actix::prelude::*;

use crate::movement_calibration::Calibrator;
use crate::{server::WebSocket, Device};

use std::f32::INFINITY;
use std::thread;
use std::time::{Duration, Instant};

use rppal::gpio::{Error, Gpio, InputPin, Level, OutputPin, Trigger};

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

    pub fn new(
        gpio: &Gpio,
        trig_pin: u8,
        echo_pin: u8,
        temperature: f32,
    ) -> Result<Self, Error> {
        let (sound_speed, timeout) = Self::calculate_parameters(temperature);
        let mut echo = gpio.get(echo_pin)?.into_input_pulldown();
        echo.set_interrupt(Trigger::Both)?;

        Ok(Self {
            trig: gpio.get(trig_pin)?.into_output_low(),
            echo,
            sound_speed,
            timeout,
        })
    }

    pub fn measure_distance(&mut self) -> Result<f32, Error> {
        self.trig.set_high();
        thread::sleep(Duration::from_micros(10));
        self.trig.set_low();

        while self.echo.poll_interrupt(false, None)? != Some(Level::High) {}
        let instant = Instant::now();

        if self.echo.poll_interrupt(false, Some(self.timeout))? != Some(Level::Low) {
            return Ok(INFINITY);
        }

        Ok(self.sound_speed * instant.elapsed().as_secs_f32() * 0.5)
    }

    /// Perform `amount` measurements, discard the minimum and maximum, and return the mean
    pub fn precise_distance_measurement(&mut self, amount: usize) -> Result<f32, Error> {
        let mut measurements = Vec::new();
        let mut max = 0.0;
        let mut min = INFINITY;

        for _ in 0..amount {
            let val = self.measure_distance()?;
            measurements.push(val);
            if val < min {
                min = val;
            }
            if val > max {
                max = val;
            }
        }
        Ok(measurements
            .into_iter()
            .filter(|x| *x != min && *x != max)
            .sum::<f32>()
            / ((amount - 2) as f32))
    }
}

// Actor communication

impl Actor for HcSr04 {
    type Context = Context<Self>;
}

pub enum Recipient {
    WebSocket(Addr<WebSocket>),
    Calibrator(Addr<Calibrator>),
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct HcSr04Message(pub Recipient);

impl Handler<HcSr04Message> for HcSr04 {
    type Result = ();

    fn handle(&mut self, msg: HcSr04Message, _ctx: &mut Self::Context) -> Self::Result {
        let response = match self.measure_distance() {
            Ok(dist) => HcSr04Response::Ok(dist),
            Err(e) => HcSr04Response::Err(e),
        };
        match msg.0 {
            Recipient::WebSocket(addr) => addr.do_send(response),
            Recipient::Calibrator(addr) => addr.do_send(response),
        };
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum HcSr04Response {
    Ok(f32),
    Err(Error),
}
