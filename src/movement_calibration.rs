use crate::drive::{Drive, Motion};
use crate::hc_sr04::HcSr04;

use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

pub struct Calibrator<'a> {
    drive: &'a mut Drive,
    distance_sensor: &'a mut HcSr04,
    min_duty_cycle: f64,
    max_duty_cycle: f64,
    step: f64,
    measurements_per_repetition: usize,
    repetitions: u16,
}

impl<'a> Calibrator<'a> {
    pub fn new(
        drive: &'a mut Drive,
        distance_sensor: &'a mut HcSr04,
        min_duty_cycle: f64,
        max_duty_cycle: f64,
        step: f64,
        measurements_per_repetition: usize,
        repetitions: u16,
    ) -> Self {
        Self {
            drive,
            distance_sensor,
            min_duty_cycle,
            max_duty_cycle,
            step,
            measurements_per_repetition,
            repetitions,
        }
    }

    pub fn calibrate(&mut self) {
        let mut duty_cycle = self.min_duty_cycle;
        while duty_cycle <= self.max_duty_cycle {
            for i in 0..self.repetitions {
                let mut f = File::create(format!("measurements/{}_{}.csv", duty_cycle, i)).unwrap();
                let fwd = self.single_calibration(&Motion::Forward, duty_cycle);
                fwd.into_iter().for_each(|(dur, d)| {
                    writeln!(f, "{},{}", dur.as_millis(), d).unwrap();
                });

                std::thread::sleep(Duration::from_millis(1000));

                let mut f =
                    File::create(format!("measurements/{}_{}.csv", -duty_cycle, i)).unwrap();
                let bwd = self.single_calibration(&Motion::Backward, duty_cycle);
                bwd.into_iter().for_each(|(dur, d)| {
                    writeln!(f, "{},{}", dur.as_millis(), d).unwrap();
                });

                std::thread::sleep(Duration::from_millis(1000));
            }
            duty_cycle += self.step;
        }
    }

    fn single_calibration(&mut self, motion: &Motion, duty_cycle: f64) -> Vec<(Duration, f32)> {
        // Initialize vector for measurements
        let initial_distance = self.distance_sensor.precise_distance_measurement(7);

        let mut measurements = vec![(Duration::from_millis(0), initial_distance)];

        // Measurements
        let start_time = Instant::now();
        self.drive.move_robot_pwm_speed(motion, duty_cycle).unwrap();
        for i in 0..self.measurements_per_repetition {
            measurements.push((
                start_time.elapsed(),
                self.distance_sensor.measure_distance(),
            ));
            println!("{} {:?} {}", i, measurements[i].0, measurements[i].1);
            // println!("{:?} {:?}", elapsed, self.measurement_duration);
        }
        self.drive
            .move_robot_pwm_speed(&Motion::Stop, duty_cycle)
            .unwrap();

        measurements
    }
}
