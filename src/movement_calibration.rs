use actix::prelude::*;
use actix_web::web::Data;

use crate::drive::{Drive, DriveMessage, Motion};
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

impl Handler<HcSr04Response> for Calibrator {
    type Result = ();

    fn handle(&mut self, msg: HcSr04Response, _ctx: &mut Self::Context) -> Self::Result {
        let result = match msg {
            HcSr04Response::Ok(d) => format!("{:?}", d),
            HcSr04Response::Err(e) => format!("{}", e),
        };
        println!("{}", result);
    }
}
