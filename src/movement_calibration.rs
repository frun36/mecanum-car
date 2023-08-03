use actix::prelude::*;

use crate::drive::{Drive, Motion};
use crate::hc_sr04::{HcSr04, HcSr04Message, HcSr04Response};

use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

pub struct Calibrator {
    drive_addr: Addr<Drive>,
    hc_sr04_addr: Addr<HcSr04>,
    min_duty_cycle: f64,
    max_duty_cycle: f64,
    step: f64,
    measurements_per_repetition: usize,
    repetitions: u16,
}

impl Calibrator {
    pub fn new(
        drive_addr: Addr<Drive>,
        hc_sr04_addr: Addr<HcSr04>,
        min_duty_cycle: f64,
        max_duty_cycle: f64,
        step: f64,
        measurements_per_repetition: usize,
        repetitions: u16,
    ) -> Self {
        Self {
            drive_addr,
            hc_sr04_addr,
            min_duty_cycle,
            max_duty_cycle,
            step,
            measurements_per_repetition,
            repetitions,
        }
    }

    pub fn calibrate(&mut self) {
        // let mut duty_cycle = self.min_duty_cycle;
        // // Goes through all specified duty cycle values
        // while duty_cycle <= self.max_duty_cycle {
        //     // Performs `repetition` measurements (forward-backward cycles)
        //     for i in 0..self.repetitions {
        //         // Forward calibration
        //         let mut f = File::create(format!("measurements/{}_{}.csv", duty_cycle, i)).unwrap();
        //         let fwd = self.single_calibration(Motion::Forward, duty_cycle);
        //         fwd.into_iter().for_each(|(dur, dist)| {
        //             println!("{} {}", dur.as_millis(), dist);
        //             writeln!(f, "{},{}", dur.as_millis(), dist).unwrap();
        //         });

        //         thread::sleep(Duration::from_millis(1000));

        //         // Backward calibration
        //         let mut f =
        //             File::create(format!("measurements/{}_{}.csv", -duty_cycle, i)).unwrap();
        //         let bwd = self.single_calibration(Motion::Backward, duty_cycle);
        //         bwd.into_iter().for_each(|(dur, dist)| {
        //             println!("{} {}", dur.as_millis(), dist);
        //             writeln!(f, "{},{}", dur.as_millis(), dist).unwrap();
        //         });

        //         thread::sleep(Duration::from_millis(1000));
        //     }
        //     duty_cycle += self.step;
        // }
    }

    fn single_calibration(&mut self, motion: Motion, duty_cycle: f64) -> Vec<(Duration, f32)> {
        // // Initialize vector for measurements
        // let initial_distance = self
        //     .distance_sensor
        //     .precise_distance_measurement(7)
        //     .unwrap();

        // let mut measurements = vec![(Duration::from_millis(0), initial_distance)];

        // // Measurements
        // let start_time = Instant::now();
        // self.drive.move_robot_pwm_speed(motion, duty_cycle).unwrap();
        // for _ in 0..self.measurements_per_repetition {
        //     measurements.push((
        //         start_time.elapsed(),
        //         self.distance_sensor.measure_distance().unwrap(),
        //     ));
        // }
        // self.drive
        //     .move_robot_pwm_speed(Motion::Stop, duty_cycle)
        //     .unwrap();

        // measurements
        vec![]
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

    fn handle(&mut self, msg: HcSr04Response, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}