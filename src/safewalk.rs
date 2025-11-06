use crate::gps::{Gps, GpsSimulator, Vector};
use crate::hazard_analyzer::HazardAnalyzer;
use crate::motor::Motor;
use crate::overpass::{OverpassResponse, Point};
use anyhow::Result;
use log::{info, warn};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::{Instant, sleep};

pub struct SafeWalk {
    vibration_system: VibrationSystem,
    gps: Gps,
}

struct VibrationSystemSpeeds {
    front: f64,
    back: f64,
    left: f64,
    right: f64,
}

struct VibrationSystem {
    front: Motor,
    back: Motor,
    left: Motor,
    right: Motor,
}

impl VibrationSystem {
    pub fn new(front_pin: u8, back_pin: u8, left_pin: u8, right_pin: u8) -> Self {
        Self {
            front: Motor::new(front_pin).unwrap(),
            back: Motor::new(back_pin).unwrap(),
            left: Motor::new(left_pin).unwrap(),
            right: Motor::new(right_pin).unwrap(),
        }
    }

    pub fn get_speeds(vector: Vector) -> VibrationSystemSpeeds {
        let max_detection_distance = 0.00001;

        let length = (max_detection_distance - vector.length).max(0.0) / max_detection_distance;

        let x = length * vector.rotation.cos();
        let y = length * vector.rotation.sin();

        let front = y.max(0.0);
        let back = (-y).max(0.0);
        let left = (-x).max(0.0);
        let right = x.max(0.0);

        VibrationSystemSpeeds {
            front,
            back,
            left,
            right,
        }
    }

    pub async fn set_speeds(&mut self, speeds: VibrationSystemSpeeds) {
        self.front.set(speeds.front).await;
        self.back.set(speeds.back).await;
        self.left.set(speeds.left).await;
        self.right.set(speeds.right).await;
    }

    pub async fn stop(&mut self) {
        self.front.off().await;
        self.back.off().await;
        self.left.off().await;
        self.right.off().await;
    }
}

impl SafeWalk {
    pub async fn new() -> Self {
        let mut gps = Gps::new();
        gps.init().await;

        Self {
            vibration_system: VibrationSystem::new(24, 25, 27, 28),
            gps,
        }
    }

    pub async fn stop(&mut self) {
        self.vibration_system.stop().await;
    }

    pub async fn main(&mut self) -> Result<()> {
        let data = fs::read_to_string(PathBuf::from("out.json"))?;

        let response = serde_json::from_str::<OverpassResponse>(&data)?;

        let mut analyzer = HazardAnalyzer::new(33.423528, -111.932806, response.elements);

        // let mut gps = Gps::new();
        // gps.init().await;
        //
        // let mut prev_location = gps.get().await.google_coordinates();
        // sleep(Duration::from_millis(25)).await;

        let mut gps = GpsSimulator::new(
            Point {
                lat: 33.423528,
                lon: -111.932806,
            },
            Point {
                lat: 33.423528,
                lon: -111.932611,
            },
        );

        let mut prev_location = gps.get();

        let mut last_loop = Instant::now();

        loop {
            // self.motor.set(0.25).await;
            // sleep(Duration::from_millis(1000)).await;
            //
            // self.motor.set(0.5).await;
            // sleep(Duration::from_millis(1000)).await;
            //
            // self.motor.set(0.75).await;
            // sleep(Duration::from_millis(1000)).await;
            //
            // self.motor.set(1.).await;
            // sleep(Duration::from_millis(1000)).await;

            // let response = self.gps.get().await;
            // println!("{:?}", response);

            let location = gps.get_with_direction(prev_location);
            analyzer.update_location(location.0);

            println!("Current Location: {}, {}", location.0.lat, location.0.lon);

            let mut reports = analyzer.analyze();

            if let Some(mut reports) = reports {
                reports.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

                let relative_vector = reports.first().unwrap().vector.rotate(-location.1.unwrap());
                // let speeds = VibrationSystem::get_speeds(relative_vector);
                // self.vibration_system.set_speeds(speeds).await;

                println!("Hazard Detected: {:?}", reports.first().unwrap().hazard.location().unwrap().first().unwrap());
                println!("Relative Vector: {:?}", relative_vector);
            } else {
                info!("No hazards found");
            }

            prev_location = Some(location.0);

            println!();

            let dt = last_loop.elapsed();
            let elapsed = dt.as_secs_f64();
            let left = 1. / 10. - elapsed;

            if left < 0. {
                warn!("Loop overrun: {} ms", -left * 1000.);
            }

            sleep(Duration::from_secs_f64(left.max(0.))).await;
            last_loop = Instant::now();
        }
    }
}
