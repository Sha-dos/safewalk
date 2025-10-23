mod overpass;
mod hazard_analyzer;
mod motor;
mod button;

use crate::overpass::{OverpassResponse, Point, fetch};
use anyhow::Result;
use std::env;
use std::path::PathBuf;
use tokio::time::sleep;
use std::time::Duration;
use tokio::fs::read_to_string;
use tokio::signal;
use crate::button::Button;
use crate::motor::Motor;

#[tokio::main]
async fn main() -> Result<()> {
    // let args = env::args().collect::<Vec<String>>();
    // let data = if args.contains(&"--cache".to_string()) {
    //     println!("Using cached data");
    // 
    //     let s = read_to_string(PathBuf::from("out.json")).await?;
    //     serde_json::from_str::<OverpassResponse>(&*s)?
    // } else {
    //     println!("Fetching data");
    //     
    //     let bbox = bbox(33.423322, -111.932648, 0.015);
    //     
    //     fetch(bbox).await?
    // };
    // 
    // println!("Fetched {} elements", data.elements.len());
    
    let mut motor = Motor::new(17).unwrap();
    
    let motor_clone = motor.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
        motor_clone.off().await;
        std::process::exit(0);
    });

    let button = Button::new(21);
    
    loop {
        if button.is_pressed() {
            motor.on().await;
        } else {
            motor.off().await;
        }
        
        sleep(Duration::from_millis(50)).await;
    }

    Ok(())
}

fn bbox(lat: f64, lon: f64, delta: f64) -> [Point; 2] {
    [
        Point {
            lat: lat - delta,
            lon: lon - delta,
        },
        Point {
            lat: lat + delta,
            lon: lon + delta,
        },
    ]
}

#[cfg(test)]
mod tests {
    use crate::overpass::OverpassResponse;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn parse_response() {
        let data = fs::read_to_string(PathBuf::from("out.json")).unwrap();

        serde_json::from_str::<OverpassResponse>(&data).unwrap();
    }
}
