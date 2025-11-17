use tokio::process::Command;

pub struct Espeak;

impl Espeak {
    pub async fn speak(text: &str) {
        Command::new("espeak")
            .arg(text)
            .spawn()
            .expect("Failed to start espeak command")
            .await
            .expect("Espeak command failed to execute");
    }
}