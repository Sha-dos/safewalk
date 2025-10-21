use crate::bbox;
use anyhow::{Error, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::write;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct OverpassResponse {
    pub version: f64,
    pub generator: String,
    pub osm3s: Osm3s,
    pub elements: Vec<Element>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Osm3s {
    timestamp_osm_base: String,
    copyright: String,
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

    let bbox = bbox(33.475, -111.875, 0.0355);

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
