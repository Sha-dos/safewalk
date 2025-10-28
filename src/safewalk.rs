use std::time::Duration;
use tokio::time::sleep;
use crate::motor::Motor;
use anyhow::Result;
use crate::gps::Gps;

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
            
            let response = self.gps.get().await;
            println!("{:?}", response);
            
            sleep(Duration::from_secs(1)).await;
        }
    }
}