mod overpass;
mod hazard_analyzer;
mod motor;

use crate::overpass::{OverpassResponse, Point, fetch};
use anyhow::Result;
use std::env;
use std::path::PathBuf;
use tokio::fs::read_to_string;

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let data = if args.contains(&"--cache".to_string()) {
        println!("Using cached data");

        let s = read_to_string(PathBuf::from("out.json")).await?;
        serde_json::from_str::<OverpassResponse>(&*s)?
    } else {
        println!("Fetching data");
        fetch().await?
    };

    println!("Fetched {} elements", data.elements.len());

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
