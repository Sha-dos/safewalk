use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use crate::motor::Motor;
use anyhow::Result;
use crate::gps::Gps;
use crate::hazard_analyzer::HazardAnalyzer;
use crate::overpass::OverpassResponse;

pub struct SafeWalk {
    motor: Motor,
    gps: Gps,
}

impl SafeWalk {
    pub async fn new() -> Self {
        let mut gps = Gps::new();
        gps.init().await;
        
        Self {
            motor: Motor::new(17).unwrap(),
            gps,
        }
    }
    
    pub async fn stop(&mut self) {
        self.motor.off().await;
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