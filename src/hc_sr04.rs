use actix::prelude::*;

use crate::distance_scan::Scanner;
use crate::movement_calibration::Calibrator;
use crate::server::WebSocket;

use std::f32::INFINITY;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rppal::gpio::{Error, Gpio, InputPin, Level, OutputPin, Trigger};

use serde::Serialize;

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

    pub fn new(gpio: &Gpio, trig_pin: u8, echo_pin: u8, temperature: f32) -> Result<Self, Error> {
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

    /// Perform a single distance measurement
    pub fn measure_distance(&mut self) -> Result<HcSr04Result, Error> {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        self.trig.set_high();
        thread::sleep(Duration::from_micros(10));
        self.trig.set_low();

        while self.echo.poll_interrupt(false, None)? != Some(Level::High) {
            // println!("Mysterious first loop")
        }
        let instant = Instant::now();

        if self.echo.poll_interrupt(false, Some(self.timeout))? != Some(Level::Low) {
            return Ok(HcSr04Result {
                time,
                distance: INFINITY,
            });
        }

        Ok(HcSr04Result {
            time,
            distance: self.sound_speed * instant.elapsed().as_secs_f32() * 0.5,
        })
    }

    /// Perform `n` distance measurements, return a vector containing them
    pub fn measure_distance_n(&mut self, n: usize) -> Result<Vec<HcSr04Result>, Error> {
        let mut measurements = Vec::new();

        for _ in 0..n {
            measurements.push(self.measure_distance()?);
        }
        Ok(measurements)
    }
}

// Actor communication

impl Actor for HcSr04 {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(1024);
        println!("HcSr04 actor started");
    }
}

pub enum Recipient {
    WebSocket(Addr<WebSocket>),
    Calibrator(Addr<Calibrator>),
    Scanner(Addr<Scanner>),
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum HcSr04Message {
    Single(Recipient),
    Multiple(usize, Recipient),
}

impl Handler<HcSr04Message> for HcSr04 {
    type Result = ();

    fn handle(&mut self, msg: HcSr04Message, _ctx: &mut Self::Context) -> Self::Result {
        let response;
        let recipient;
        match msg {
            HcSr04Message::Single(m_recipient) => {
                recipient = m_recipient;
                response = match self.measure_distance() {
                    Ok(dist) => HcSr04Response::Ok(HcSr04Measurement::Single(dist)),
                    Err(e) => HcSr04Response::Err(e),
                };
            }
            HcSr04Message::Multiple(n, m_recipient) => {
                recipient = m_recipient;
                response = match self.measure_distance_n(n) {
                    Ok(dist) => HcSr04Response::Ok(HcSr04Measurement::Multiple(dist)),
                    Err(e) => HcSr04Response::Err(e),
                };
            }
        };

        match recipient {
            Recipient::WebSocket(addr) => addr.do_send(response),
            Recipient::Calibrator(addr) => addr.do_send(response),
            Recipient::Scanner(addr) => addr.do_send(response),
        };
    }
}

#[derive(Debug, Serialize)]
pub struct HcSr04Result {
    pub time: Duration,
    pub distance: f32,
}

#[derive(Debug, Serialize)]
pub enum HcSr04Measurement {
    Single(HcSr04Result),
    Multiple(Vec<HcSr04Result>),
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum HcSr04Response {
    Ok(HcSr04Measurement),
    Err(Error),
}
