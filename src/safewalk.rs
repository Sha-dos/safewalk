use std::fs;
use std::path::PathBuf;
use crate::motor::Motor;
use anyhow::Result;
use crate::gps::{Gps, Vector};
use crate::hazard_analyzer::HazardAnalyzer;
use crate::overpass::OverpassResponse;

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
    pub fn new(front_pin: u8, back_pin: u8, left_pin: u8, right_pin: u8) -> Result<Self> {
        Ok(Self {
            front: Motor::new(front_pin)?,
            back: Motor::new(back_pin)?,
            left: Motor::new(left_pin)?,
            right: Motor::new(right_pin)?,
        })
    }
    
    fn get_speeds(vector: Vector) -> VibrationSystemSpeeds {
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
            vibration_system: VibrationSystem::new(0, 0, 0, 0).unwrap(),
            gps,
        }
    }
    
    pub async fn stop(&mut self) {
        self.vibration_system.stop().await;
    }
    
    pub async fn main(&mut self) -> Result<()> {
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

            let data = fs::read_to_string(PathBuf::from("out.json")).unwrap();

            let response = serde_json::from_str::<OverpassResponse>(&data).unwrap();

            let analyzer = HazardAnalyzer::new(33.423322, -111.932648, response.elements);

            let reports = analyzer.analyze();

            if let Some(reports) = reports {
                for report in &reports {
                    println!("{}", serde_json::to_string_pretty(report)?);
                }
                assert!(!reports.is_empty());
            } else {
                panic!("No hazards found");
            }
            
            return Ok(());
        }
    }
}