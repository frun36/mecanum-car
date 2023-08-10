use actix::prelude::*;
use actix_web::web::Data;

use crate::drive::{Drive, DriveMessage, Motion, Speed};
use crate::hc_sr04::{HcSr04, HcSr04Measurement, HcSr04Message, HcSr04Response};

use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct CalibratorParams {
    min_duty_cycle: f64,
    max_duty_cycle: f64,
    step: f64,
    measurements_per_repetition: usize,
    repetitions: usize,
    curr_motion: Motion,
    curr_duty_cycle: f64,
    curr_repetition: usize,
}

impl Default for CalibratorParams {
    fn default() -> Self {
        Self {
            min_duty_cycle: 0.5,
            max_duty_cycle: 1.0,
            step: 0.1,
            measurements_per_repetition: 500,
            repetitions: 1,
            curr_motion: Motion::Forward,
            curr_duty_cycle: 0.5,
            curr_repetition: 0,
        }
    }
}

impl CalibratorParams {
    pub fn new(
        min_duty_cycle: f64,
        max_duty_cycle: f64,
        step: f64,
        measurements_per_repetition: usize,
        repetitions: usize,
    ) -> CalibratorParams {
        Self {
            min_duty_cycle,
            max_duty_cycle,
            step,
            measurements_per_repetition,
            repetitions,
            curr_duty_cycle: min_duty_cycle,
            ..Default::default()
        }
    }
}

pub struct Calibrator {
    drive_data: Data<Mutex<Addr<Drive>>>,
    hc_sr04_data: Data<Mutex<Addr<HcSr04>>>,
    params: CalibratorParams,
}

impl Calibrator {
    pub fn new(
        drive_data: Data<Mutex<Addr<Drive>>>,
        hc_sr04_data: Data<Mutex<Addr<HcSr04>>>,
    ) -> Self {
        Self {
            drive_data,
            hc_sr04_data,
            params: CalibratorParams::default(),
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
    Start(CalibratorParams),
    Stop,
}

impl Handler<CalibratorMessage> for Calibrator {
    type Result = ();

    fn handle(&mut self, msg: CalibratorMessage, ctx: &mut Self::Context) -> Self::Result {
        let drive_addr = self.drive_data.lock().unwrap();
        let hc_sr04_addr = self.hc_sr04_data.lock().unwrap();
        match msg {
            CalibratorMessage::Start(params) => {
                self.params = params;
                println!("Performing calibration: {:?}", self.params);
                // Move robot
                drive_addr.do_send(DriveMessage {
                    motion: self.params.curr_motion,
                    speed: Speed::Manual(self.params.curr_duty_cycle),
                });
                // Start measurement
                hc_sr04_addr.do_send(HcSr04Message::Multiple(
                    self.params.measurements_per_repetition,
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

    fn handle(&mut self, msg: HcSr04Response, ctx: &mut Self::Context) -> Self::Result {
        let drive_addr = self.drive_data.lock().unwrap();
        // Stop the robot
        drive_addr.do_send(DriveMessage {
            motion: Motion::Stop,
            speed: Speed::Low,
        });
        // Process the result
        let result = match msg {
            HcSr04Response::Ok(measurement) => match measurement {
                HcSr04Measurement::Single(d) => format!("{},{}", d.time.as_millis(), d.distance),
                HcSr04Measurement::Multiple(d_vec) => d_vec
                    .iter()
                    .map(|x| format!("{},{}", x.time.as_millis(), x.distance))
                    .collect::<Vec<String>>()
                    .join("\n"),
            },
            HcSr04Response::Err(e) => format!("{}", e),
        };
        // Save the result to file
        let mut file = File::create(format!(
            "measurements/{}_{:.2}_{:02}.csv",
            self.params.curr_motion, self.params.curr_duty_cycle, self.params.curr_repetition
        ))
        .unwrap();
        write!(file, "{}", result).unwrap();
        // Update params for next measurement
        self.params.curr_motion = match self.params.curr_motion {
            Motion::Forward => Motion::Backward,
            Motion::Backward => {
                self.params.curr_repetition += 1;
                Motion::Forward
            }
            _ => panic!("Invalid calibration motion"),
        };
        // Check if all repetitions were done
        if self.params.curr_repetition >= self.params.repetitions {
            self.params.curr_repetition = 0;
            self.params.curr_duty_cycle += self.params.step;
        }
        // Return if measurement is completed
        if self.params.curr_duty_cycle > self.params.max_duty_cycle {
            return;
        }
        // Send message for next measurement if not
        ctx.address().do_send(CalibratorMessage::Start(self.params));
    }
}
