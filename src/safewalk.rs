use std::f64::consts::PI;
use crate::gps::{Gps, GpsSimulator, Vector};
use crate::hazard_analyzer::HazardAnalyzer;
use crate::motor::Motor;
use crate::overpass::{OverpassResponse, Point};
use anyhow::Result;
use log::{info, warn};
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;
use tokio::time::{Instant, sleep};

pub struct SafeWalk {
    vibration_system: VibrationSystem,
    gps: Gps,
    motor: Motor,
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
        let max_detection_distance = 0.0001;

        let length = (max_detection_distance - vector.length).max(0.0) / max_detection_distance;

        // Coordinate system:
        // - Relative angle 0° = straight ahead
        // - Negative angles = RIGHT (clockwise rotation)
        // - Positive angles = LEFT (counter-clockwise rotation)
        //
        // cos(angle) gives forward/back component:
        //   cos(0°) = 1 (straight ahead)
        //   cos(±180°) = -1 (behind)
        //
        // sin(angle) gives left/right component:
        //   sin(negative angle) = negative value = RIGHT
        //   sin(positive angle) = positive value = LEFT

        let forward_component = vector.rotation.cos() * length;
        let side_component = vector.rotation.sin() * length;

        let front = forward_component.max(0.0);
        let back = (-forward_component).max(0.0);
        let left = side_component.max(0.0);      // positive sin = left
        let right = (-side_component).max(0.0);  // negative sin = right

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
            motor: Motor::new(29).unwrap(),
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
                lon: -111.932611,
            },
            Point {
                lat: 33.423528,
                lon: -111.932806,
            }
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

            // Check if simulation ended
            if location.1.is_none() {
                println!("Simulation complete - reached destination");
                exit(0);
            }

            let current_pos = location.0.unwrap();
            analyzer.update_location(current_pos);

            println!("Current Location: {}, {}", current_pos.lat, current_pos.lon);

            let reports = analyzer.analyze();

            if let Some(mut reports) = reports {
                reports.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

                let hazard_vector = reports.first().unwrap().vector;
                let user_heading = location.1.unwrap();

                let mut relative_angle = hazard_vector.rotation - user_heading;

                // Normalize to [-π, π]
                while relative_angle > PI {
                    relative_angle -= 2.0 * PI;
                }
                while relative_angle < -PI {
                    relative_angle += 2.0 * PI;
                }

                // In the relative coordinate system:
                // 0° = straight ahead
                // NEGATIVE angles (0° to -180°) = to the RIGHT (clockwise)
                // POSITIVE angles (0° to 180°) = to the LEFT (counter-clockwise)
                // ±180° = directly behind

                let relative_vector = Vector::new(relative_angle, hazard_vector.length);

                println!("Hazard Detected: {:?}", reports.first().unwrap().hazard.location().unwrap().first().unwrap());
                println!("User heading (radians): {:.4} ({:.1}°)", user_heading, user_heading.to_degrees());
                println!("Hazard absolute angle (radians): {:.4} ({:.1}°)", hazard_vector.rotation, hazard_vector.rotation.to_degrees());
                println!("Relative angle: {:.4} rad ({:.1}°) - Negative=RIGHT, Positive=LEFT", relative_angle, relative_angle.to_degrees());
                println!("Relative Vector: {:?}", relative_vector);

                let speeds = VibrationSystem::get_speeds(relative_vector);
                println!("Vibration - Front: {:.2}, Back: {:.2}, Left: {:.2}, Right: {:.2}",
                    speeds.front, speeds.back, speeds.left, speeds.right);
            } else {
                info!("No hazards found");
            }

            prev_location = Some(current_pos);

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
