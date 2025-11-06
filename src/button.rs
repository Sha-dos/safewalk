use rppal::gpio::{Gpio, InputPin};
use std::time::Duration;
use tokio::time::sleep;

pub struct Button {
    pin: InputPin,
}

impl Button {
    pub fn new(pin: u8) -> Self {
        let gpio = Gpio::new().unwrap();
        let pin = gpio.get(pin).unwrap().into_input();

        Self { pin }
    }

    pub fn is_pressed(&self) -> bool {
        self.pin.is_low()
    }

    pub async fn wait(&self) {
        while !self.is_pressed() {
            sleep(Duration::from_millis(50)).await;
        }
    }
}
