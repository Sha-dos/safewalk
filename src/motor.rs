use std::error::Error;
use std::time::Duration;
use rppal::gpio::{Gpio, OutputPin};
use tokio::time::sleep;
use tokio::task::JoinHandle;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Motor {
    state: Arc<Mutex<MotorState>>,
    _handle: Arc<JoinHandle<()>>,
}

#[derive(Copy, Clone)]
struct MotorState {
    mode: MotorMode,
    power: f64,
}

#[derive(Copy, Clone)]
enum MotorMode {
    Off,
    On,
    Pwm,
}

impl Motor {
    pub fn new(pin: u8) -> Result<Self, Box<dyn Error>> {
        let state = Arc::new(Mutex::new(MotorState {
            mode: MotorMode::Off,
            power: 0.0,
        }));

        let state_clone = state.clone();
        let gpio = Gpio::new()?;
        let mut output_pin = gpio.get(pin)?.into_output();

        let handle = tokio::spawn(async move {
            loop {
                let current_state = {
                    state_clone.lock().await
                };

                match current_state.mode {
                    MotorMode::Off => {
                        output_pin.set_high();
                        sleep(Duration::from_millis(10)).await;
                    }
                    MotorMode::On => {
                        output_pin.set_low();
                        sleep(Duration::from_millis(10)).await;
                    }
                    MotorMode::Pwm => {
                        let on_duration = Duration::from_millis((current_state.power * 10.0) as u64);
                        let off_duration = Duration::from_millis(((1.0 - current_state.power) * 10.0) as u64);

                        output_pin.set_high();
                        sleep(on_duration).await;
                        output_pin.set_low();
                        sleep(off_duration).await;
                    }
                }
            }
        });

        Ok(Motor {
            state,
            _handle: Arc::new(handle),
        })
    }

    pub async fn on(&self) {
        let mut state = self.state.lock().await;
        state.mode = MotorMode::On;
    }

    pub async fn off(&self) {
        let mut state = self.state.lock().await;
        state.mode = MotorMode::Off;
    }

    pub async fn set(&self, power: f64) {
        let mut state = self.state.lock().await;
        state.power = power;
        state.mode = MotorMode::Pwm;
    }
}
