mod overpass;

use std::path::PathBuf;
use reqwest;
use serde_json::Value;
use anyhow::Result;
use tokio::fs::write;
use crate::overpass::{OverpassResponse, Point};

#[tokio::main]
async fn main() -> Result<()> {
    let overpass_url = "https://overpass-api.de/api/interpreter";

    // let bbox = "33.42,-111.98,33.53,-111.97";

    let bbox = bbox(33.475, -111.875, 0.0355);

    let bbox_str = bbox.iter().map(|p| format!("{},{}", p.lat, p.lon)).collect::<Vec<String>>().join(",");

    let overpass_query = format!(r#"
        [out:json][timeout:25][bbox:{}];
        (
          // --- 1. Crossings with signals but NO audible/vibration aids ---
          node["highway"="traffic_signals"];
          (._;)->.crossings_with_signals;
          node.crossings_with_signals["traffic_signals:sound"="no"];
          node.crossings_with_signals["traffic_signals:vibration"="no"];
          node.crossings_with_signals[!"traffic_signals:sound"][!"traffic_signals:vibration"];

          // --- 2. Uncontrolled or unmarked crossings ---
          node["highway"="crossing"]["crossing"~"uncontrolled|unmarked"];

          // --- 3. Raised kerbs (potential trip hazards) ---
          node["kerb"="raised"];

          // --- 4. Uneven or unpaved footpaths/sidewalks ---
          way["highway"~"footway|sidewalk|path"]["surface"~"unpaved|gravel|dirt|sand|ground"];

          // --- 5. Generic hazards ---
          node["hazard"];
          way["hazard"];

          // --- 6. Lack of or incorrect tactile paving ---
          node["tactile_paving"="no"];
          node["tactile_paving"="incorrect"];
          way["tactile_paving"="no"];
          way["tactile_paving"="incorrect"];
        );
        out geom;
    "#, bbox_str);

    let client = reqwest::Client::builder()
        .user_agent("safewalk/0.1.0")
        .build()?;

    let response = client
        .post(overpass_url)
        .body(overpass_query)
        .send()
        .await?;

    if response.status().is_success() {
        let data: OverpassResponse = response.json().await?;
        
        let out_path = PathBuf::from("out.json");
        let out_data = serde_json::to_string_pretty(&data)?;
        write(out_path, out_data).await?;

        println!("Fetched {} elements", data.elements.len());

        // println!("{:?}", data.ways.first().unwrap().tags);
    } else {
        eprintln!("Error: Query failed with status: {}", response.status());
        eprintln!("Response body: {}", response.text().await?);
    }

    Ok(())
}

fn bbox(lat: f64, lon: f64, delta: f64) -> [Point; 2] {
    [
        Point { lat: lat - delta, lon: lon - delta },
        Point { lat: lat + delta, lon: lon + delta }
    ]
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use crate::overpass::OverpassResponse;

    #[test]
    fn parse_response() {
        let data = fs::read_to_string(PathBuf::from("out.json")).unwrap();
        
        serde_json::from_str::<OverpassResponse>(&data).unwrap();
    }
}
