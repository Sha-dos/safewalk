use std::time::Duration;
use tokio::time::sleep;
use crate::motor::Motor;
use anyhow::Result;

pub struct SafeWalk {
    motor: Motor,
}

impl SafeWalk {
    pub fn new() -> Self {
        Self {
            motor: Motor::new(17).unwrap(),
        }
    }
    
    pub async fn stop(&mut self) {
        self.motor.off().await;
    }
    
    pub async fn main(&mut self) -> Result<()> {
        loop {
            self.motor.set(0.25).await;
            sleep(Duration::from_millis(1000)).await;

            self.motor.set(0.5).await;
            sleep(Duration::from_millis(1000)).await;

            self.motor.set(0.75).await;
            sleep(Duration::from_millis(1000)).await;

            self.motor.set(1.).await;
            sleep(Duration::from_millis(1000)).await;
        }
    }
}