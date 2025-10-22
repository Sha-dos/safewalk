use crate::bbox;
use anyhow::{Error, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::write;

#[derive(Debug, Deserialize, Serialize)]
pub struct OverpassResponse {
    pub version: f64,
    pub generator: String,
    pub osm3s: Osm3s,
    pub elements: Vec<Element>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Osm3s {
    pub timestamp_osm_base: String,
    pub copyright: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Node {
        id: u64,
        lat: f64,
        lon: f64,
        #[serde(default)]
        tags: HashMap<String, String>,
    },
    Way {
        bounds: OverpassBounds,
        geometry: Vec<Point>,
        id: u64,
        nodes: Option<Vec<u64>>,
        #[serde(default)]
        tags: HashMap<String, String>,
    },
    Relation {
        id: u64,
        members: Vec<Member>,
        #[serde(default)]
        tags: HashMap<String, String>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct Member {
    #[serde(rename = "type")]
    member_type: String,
    #[serde(rename = "ref")]
    reference: u64,
    role: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OverpassBounds {
    #[serde(rename = "maxlat")]
    pub max_lat: f64,
    #[serde(rename = "maxlon")]
    pub max_lon: f64,
    #[serde(rename = "minlat")]
    pub min_lat: f64,
    #[serde(rename = "minlon")]
    pub min_lon: f64,
}

pub async fn fetch() -> Result<OverpassResponse, Error> {
    let overpass_url = "https://overpass-api.de/api/interpreter";
    
    let bbox = bbox(33.423322, -111.932648, 0.015);

    let bbox_str = bbox
        .iter()
        .map(|p| format!("{},{}", p.lat, p.lon))
        .collect::<Vec<String>>()
        .join(",");

    let overpass_query = format!(
        r#"
        [out:json][timeout:25][bbox:{}];

(
  // --- 1. Crossings with signals but NO audible/vibration aids ---
  node["highway"="traffic_signals"]
    (if:!t["traffic_signals:sound"] || t["traffic_signals:sound"] == "no")
    (if:!t["traffic_signals:vibration"] || t["traffic_signals:vibration"] == "no");

  // --- 2. Crossings missing tactile paving or with incorrect tactile paving ---
  node["highway"="crossing"]["tactile_paving"~"no|incorrect"];
  way["highway"="crossing"]["tactile_paving"~"no|incorrect"];

  // --- 3. Uncontrolled or unmarked crossings ---
  node["highway"="crossing"]["crossing"~"uncontrolled|unmarked"];
  node["highway"="crossing"][!"crossing"]; // crossings with no type defined

  // --- 4. Raised kerbs ---
  node["kerb"="raised"];
  way["kerb"="raised"];

  // --- 5. Missing kerb ramps ---
  node["kerb"="no"];
  node["kerb"="unknown"];
  node["highway"="crossing"][!"kerb"];

  // --- 6. Uneven or unpaved sidewalks / footpaths ---
  way["highway"~"footway|sidewalk|path|pedestrian"]["surface"~"unpaved|gravel|dirt|sand|ground|cobblestone|pebblestone|grass"];

  // --- 7. Missing sidewalk ---
  way["highway"~"primary|secondary|tertiary|residential"]["sidewalk"="no"];
  way["highway"~"primary|secondary|tertiary|residential"][!"sidewalk"];

  // --- 8. Generic hazards ---
  node["hazard"];
  way["hazard"];

  // --- 9. Steps or stairs without tactile or handrail info ---
  way["highway"="steps"][!"tactile_paving"];
  way["highway"="steps"][!"handrail"];
);

out geom;
    "#,
        bbox_str
    );

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

        Ok(data)
    } else {
        Err(anyhow!(
            "Query failed with status: {}\n Response body: {}",
            response.status(),
            response.text().await?
        ))
    }
}
