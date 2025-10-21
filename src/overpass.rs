use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OverpassResponse {
    pub nodes: Vec<OverpassNode>,
    pub ways: Vec<OverpassWay>,
}

#[derive(Debug, Deserialize)]
pub struct OverpassNode {
    id: u64,
    lat: f64,
    lon: f64,
    tags: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct OverpassWay {
    bounds: OverpassBounds,
    geometry: Vec<Point>,
    id: u64,
    nodes: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct Point {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Deserialize)]
pub struct OverpassBounds {
    #[serde(rename = "maxlat")]
    max_lat: f64,
    #[serde(rename = "maxlon")]
    max_lon: f64,
    #[serde(rename = "minlat")]
    min_lat: f64,
    #[serde(rename = "minlon")]
    min_lon: f64,
}