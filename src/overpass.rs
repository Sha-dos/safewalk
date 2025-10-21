use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct OverpassResponse {
    version: f64,
    generator: String,
    osm3s: Osm3s,
    elements: Vec<Element>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Osm3s {
    timestamp_osm_base: String,
    copyright: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum Element {
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