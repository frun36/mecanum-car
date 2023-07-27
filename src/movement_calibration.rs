use crate::drive::{Drive, Motion};
use crate::hc_sr04::HcSr04;

use std::time::{Duration, Instant};

pub struct Calibrator<'a> {
    drive: &'a mut Drive,
    distance_sensor: &'a mut HcSr04,
    min_duty_cycle: f64,
    max_duty_cycle: f64,
    step: f64,
    measurement_duration: Duration,
    repetitions: u16,
}

impl<'a> Calibrator<'a> {
    pub fn new(
        drive: &'a mut Drive,
        distance_sensor: &'a mut HcSr04,
        min_duty_cycle: f64,
        max_duty_cycle: f64,
        step: f64,
        measurement_duration: Duration,
        repetitions: u16,
    ) -> Self {
        Self {
            drive,
            distance_sensor,
            min_duty_cycle,
            max_duty_cycle,
            step,
            measurement_duration,
            repetitions,
        }
    }

    pub fn calibrate(&mut self) {
        for _ in 0..self.repetitions {
            let mut duty_cycle = self.min_duty_cycle;
            while duty_cycle <= self.max_duty_cycle {
                println!("{}", duty_cycle);
                let fwd = self.single_calibration(&Motion::Forward, duty_cycle);
                fwd.into_iter().for_each(|(dur, d)| {
                    println!("{} {}", dur.as_millis(), d);
                });

                println!("{}", -duty_cycle);
                let bwd = self.single_calibration(&Motion::Backward, duty_cycle);
                bwd.into_iter().for_each(|(dur, d)| {
                    println!("{} {}", dur.as_millis(), d);
                });

                duty_cycle += self.step;
            }
        }
    }

    fn single_calibration(&mut self, motion: &Motion, duty_cycle: f64) -> Vec<(Duration, f32)> {
        // Initialize vector for measurements
        let initial_distance = self.distance_sensor.precise_distance_measurement(7);

        let mut measurements = vec![(Duration::from_millis(0), initial_distance)];

        // Measurements
        let start_time = Instant::now();
        let mut elapsed = start_time.elapsed();
        self.drive.move_robot_pwm_speed(motion, duty_cycle).unwrap();
        while elapsed < self.measurement_duration {
            elapsed = start_time.elapsed();
            measurements.push((elapsed, self.distance_sensor.measure_distance()));
            // println!("{:?} {:?}", elapsed, self.measurement_duration);
        }
        self.drive.move_robot_pwm_speed(&Motion::Stop, duty_cycle).unwrap();

        measurements
    }
}
