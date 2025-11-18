use std::process::Stdio;
use tokio::process::Command;

pub struct Espeak;

impl Espeak {
    pub async fn speak(text: &str) {
        Command::new("espeak")
            .arg(text)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start espeak command")
            .wait()
            .await
            .unwrap();
    }
}
