use actix::prelude::*;
use actix_web::web::Data;

use crate::drive::{Drive, DriveMessage, Motion, Speed};
use crate::hc_sr04::{HcSr04, HcSr04Message, HcSr04Response};

use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

pub struct Calibrator {
    drive_data: Data<Mutex<Addr<Drive>>>,
    hc_sr04_data: Data<Mutex<Addr<HcSr04>>>,
    min_duty_cycle: f64,
    max_duty_cycle: f64,
    step: f64,
    measurements_per_repetition: usize,
    repetitions: u16,
}

impl Calibrator {
    pub fn new(
        drive_data: Data<Mutex<Addr<Drive>>>,
        hc_sr04_data: Data<Mutex<Addr<HcSr04>>>,
        min_duty_cycle: f64,
        max_duty_cycle: f64,
        step: f64,
        measurements_per_repetition: usize,
        repetitions: u16,
    ) -> Self {
        Self {
            drive_data,
            hc_sr04_data,
            min_duty_cycle,
            max_duty_cycle,
            step,
            measurements_per_repetition,
            repetitions,
        }
    }
}

impl Actor for Calibrator {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Calibrator actor started");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum CalibratorMessage {
    Start,
    Stop,
}

impl Handler<CalibratorMessage> for Calibrator {
    type Result = ();

    fn handle(&mut self, msg: CalibratorMessage, ctx: &mut Self::Context) -> Self::Result {
        let drive_addr = self.drive_data.lock().unwrap();
        let hc_sr04_addr = self.hc_sr04_data.lock().unwrap();
        match msg {
            CalibratorMessage::Start => {
                drive_addr.do_send(DriveMessage {
                    motion: Motion::Forward,
                    speed: Speed::Low,
                });
                hc_sr04_addr.do_send(HcSr04Message::Multiple(
                    self.measurements_per_repetition,
                    crate::hc_sr04::Recipient::Calibrator(ctx.address()),
                ));
            }
            CalibratorMessage::Stop => {
                drive_addr.do_send(DriveMessage {
                    motion: Motion::Stop,
                    speed: Speed::Low,
                });
            }
        };
    }
}

impl Handler<HcSr04Response> for Calibrator {
    type Result = ();

    fn handle(&mut self, msg: HcSr04Response, _ctx: &mut Self::Context) -> Self::Result {
        let drive_addr = self.drive_data.lock().unwrap();
        drive_addr.do_send(DriveMessage {
            motion: Motion::Stop,
            speed: Speed::Low,
        });
        let result = match msg {
            HcSr04Response::Ok(d) => format!("{:?}", d),
            HcSr04Response::Err(e) => format!("{}", e),
        };
        println!("{}", result);
    }
}
