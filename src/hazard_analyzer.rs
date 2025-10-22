use crate::overpass::Element;

pub struct HazardAnalyzer {
    lat: f64,
    lon: f64,
    elements: Vec<Element>,
}

pub enum HazardSeverity {
    Low,
    Medium,
    High,
}

pub struct HazardReport {
    pub hazard: Element,
    pub distance: f64,
    pub severity: HazardSeverity,
}

impl HazardAnalyzer {
    pub fn new(lat: f64, lon: f64, elements: Vec<Element>) -> Self {
        Self { lat, lon, elements }
    }

    pub fn update_location(&mut self, lat: f64, lon: f64) {
        self.lat = lat;
        self.lon = lon;
    }

    pub fn update_elements(&mut self, elements: Vec<Element>) {
        self.elements = elements;
    }

    pub fn analyze(&self) -> Option<Vec<HazardReport>> {
        None
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
