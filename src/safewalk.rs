use crate::button::Button;
use crate::espeak::Espeak;
use crate::gps::{Gps, GpsSimulator, Vector};
use crate::hazard_analyzer::HazardAnalyzer;
use crate::motor::Motor;
use crate::networking::Telemetry;
use crate::overpass::{OverpassResponse, Point};
use anyhow::Result;
use log::{info, warn};
use std::f64::consts::PI;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;
use tokio::task::AbortHandle;
use tokio::time::{Instant, sleep};

pub struct SafeWalk {
    vibration_system: VibrationSystem,
    gps: Gps,
    button: Button,
    speak_handle: Option<AbortHandle>,
}

#[derive(Clone)]
struct VibrationSystemSpeeds {
    front: f64,
    back: f64,
    left: f64,
    right: f64,
}

impl VibrationSystemSpeeds {
    pub fn vec(&self) -> Vec<f64> {
        vec![self.front, self.right, self.back, self.left]
    }
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

    pub async fn test(&self) {
        println!("Front motor ON");
        self.front.set(1.0).await;
        sleep(Duration::from_millis(2000)).await;
        self.front.off().await;

        println!("Back motor ON");
        self.back.set(1.0).await;
        sleep(Duration::from_millis(2000)).await;
        self.back.off().await;

        println!("Left motor ON");
        self.left.set(1.0).await;
        sleep(Duration::from_millis(2000)).await;
        self.left.off().await;

        println!("Right motor ON");
        self.right.set(1.0).await;
        sleep(Duration::from_millis(2000)).await;
        self.right.off().await;
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
        let left = side_component.max(0.0); // positive sin = left
        let right = (-side_component).max(0.0); // negative sin = right

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
            vibration_system: VibrationSystem::new(26, 27, 7, 5),
            gps,
            button: Button::new(4),
            speak_handle: None,
        }
    }

    pub async fn stop(&mut self) {
        self.vibration_system.stop().await;
    }

    pub async fn main(&mut self) -> Result<()> {
        let data = fs::read_to_string(PathBuf::from("out.json"))?;

        let response = serde_json::from_str::<OverpassResponse>(&data)?;

        let mut analyzer = HazardAnalyzer::new(33.423528, -111.932806, response.elements);

        let mut prev_location = self.gps.get().await.google_coordinates();
        sleep(Duration::from_millis(25)).await;

        // let mut gps = GpsSimulator::new(
        //     Point {
        //         lat: 33.423528,
        //         lon: -111.932806,
        //     },
        //     Point {
        //         lat: 33.423528,
        //         lon: -111.932611,
        //     }
        // );

        // let mut prev_location = gps.get();

        let mut last_loop = Instant::now();

        loop {
            // println!("{}", "=".repeat(50));
            // let response = self.gps.get().await;
            // println!("{:?}", response);

            let location = self.gps.get_with_direction(Some(prev_location)).await;

            // Check if simulation ended
            // if location.1.is_none() {
            //     println!("Simulation complete - reached destination");
            //     return Ok(());
            // }

            // Only update location if GPS has a valid fix (status = 1 = 'A')
            let current_pos = if location.0.status == 1 {
                let pos = location.0.google_coordinates();
                prev_location = pos; // Update previous location for next bearing calculation
                pos
            } else {
                info!(
                    "GPS has no valid fix (status={}), using previous location",
                    location.0.status
                );
                prev_location
            };

            analyzer.update_location(current_pos);

            info!("Current Location: {}, {}", current_pos.lat, current_pos.lon);
            Telemetry::put_number("latitude", current_pos.lat).await;
            Telemetry::put_number("longitude", current_pos.lon).await;
            Telemetry::put_number("heading", location.1.unwrap_or(0.0)).await;

            let reports = analyzer.analyze();

            if self.button.is_pressed() && self.speak_handle.is_none() {
                let reports_clone = reports.clone();
                let handle = tokio::spawn(async move {
                    if let Some(r) = reports_clone {
                        let nearest = r.first().unwrap();
                        if nearest.hazard.tags().get("highway") == Some(&"crossing".to_string()) {
                            Espeak::speak("Hazard ahead pedestrian crossing").await;
                        } else {
                            Espeak::speak("Hazard ahead").await;
                        }
                    } else {
                        Espeak::speak("No hazards detected").await;
                    }
                })
                .abort_handle();

                self.speak_handle = Some(handle);
            } else if !self.button.is_pressed() {
                if let Some(handle) = &self.speak_handle {
                    handle.abort(); // Doesnt actually kill the process
                    self.speak_handle = None;
                }
            }

            if let Some(mut reports) = reports {
                reports.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                Telemetry::put_vec("hazards", reports.clone()).await;

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

                info!(
                    "Hazard Detected: {:?}",
                    reports
                        .first()
                        .unwrap()
                        .hazard
                        .location()
                        .unwrap()
                        .first()
                        .unwrap()
                );
                info!("Hazard tags: {:?}", reports.first().unwrap().hazard.tags());
                info!(
                    "User heading (radians): {:.4} ({:.1}°)",
                    user_heading,
                    user_heading.to_degrees()
                );
                info!(
                    "Hazard absolute angle (radians): {:.4} ({:.1}°)",
                    hazard_vector.rotation,
                    hazard_vector.rotation.to_degrees()
                );
                info!(
                    "Relative angle: {:.4} rad ({:.1}°) - Negative=RIGHT, Positive=LEFT",
                    relative_angle,
                    relative_angle.to_degrees()
                );
                info!("Relative Vector: {:?}", relative_vector);

                let speeds = VibrationSystem::get_speeds(relative_vector);
                // println!("Vibration - Front: {:.2}, Back: {:.2}, Left: {:.2}, Right: {:.2}",
                //     speeds.front, speeds.back, speeds.left, speeds.right);

                self.vibration_system.set_speeds(speeds.clone()).await;
                Telemetry::put_vec("speeds", speeds.vec()).await;
            } else {
                // info!("No hazards found");
            }

            // println!("{}", "=".repeat(50));
            // print!("\n\n\n");

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
