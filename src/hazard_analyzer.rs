use serde::{Deserialize, Serialize};
use crate::gps::Vector;
use crate::overpass::{Element, Point};

pub struct HazardAnalyzer {
    lat: f64,
    lon: f64,
    elements: Vec<Element>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum HazardSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HazardReport {
    pub hazard: Element,
    pub distance: f64,
    pub severity: HazardSeverity,
    pub vector: Vector,
}

impl HazardAnalyzer {
    pub fn new(lat: f64, lon: f64, elements: Vec<Element>) -> Self {
        Self { lat, lon, elements }
    }

    pub fn update_location(&mut self, point: Point) {
        self.lat = point.lat;
        self.lon = point.lon;
    }

    pub fn update_elements(&mut self, elements: Vec<Element>) {
        self.elements = elements;
    }

    pub fn analyze(&self) -> Option<Vec<HazardReport>> {
        let hazards = self.nearby_hazards(0.001);

        if hazards.is_empty() {
            None
        } else {
            let reports = hazards.into_iter().map(|hazard| {
                let locations = hazard.location().unwrap();

                let mut min_distance = f64::MAX;
                let mut x_diff = 0.0;
                let mut y_diff = 0.0;

                for point in locations {
                    let distance = ((point.lat - self.lat).powi(2) + (point.lon - self.lon).powi(2)).sqrt();
                    if distance < min_distance {
                        min_distance = distance;
                        x_diff = point.lon - self.lon;
                        y_diff = point.lat - self.lat;
                    }
                }

                let severity = if min_distance < 0.0003 {
                    HazardSeverity::High
                } else if min_distance < 0.0006 {
                    HazardSeverity::Medium
                } else {
                    HazardSeverity::Low
                };

                let vector = Vector::new(f64::atan2(y_diff, x_diff), min_distance);

                HazardReport {
                    hazard: hazard.clone(),
                    distance: min_distance,
                    severity,
                    vector,
                }
            }).collect();

            Some(reports)
        }
    }

    pub fn nearby_hazards(&self, radius: f64) -> Vec<&Element> {
        self.elements.iter().filter(|element| {
            if let Some(locations) = element.location() {
                for point in locations {
                    let distance = ((point.lat - self.lat).powi(2) + (point.lon - self.lon).powi(2)).sqrt();
                    if distance <= radius {
                        return true;
                    }
                }
            }
            false
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use crate::hazard_analyzer::HazardAnalyzer;
    use crate::overpass::OverpassResponse;

    #[test]
    fn test_nearby_hazards() {
        let data = fs::read_to_string(PathBuf::from("out.json")).unwrap();

        let response = serde_json::from_str::<OverpassResponse>(&data).unwrap();

        let analyzer = HazardAnalyzer::new(33.423322, -111.932648, response.elements);

        let hazards = analyzer.nearby_hazards(0.0003);

        for hazard in &hazards {
            println!("{:?}", hazard);
        }

        assert!(!hazards.is_empty());
    }
}
