use serde::Serialize;

#[derive(Serialize)]
pub struct FeatureCollection {
    pub r#type: &'static str,
    pub features: Vec<Feature>,
    pub query: Vec<String>,
    pub attribution: String,
}

#[derive(Serialize)]
pub struct Feature {
    pub id: String,
    pub text: String,
    pub r#type: &'static str,
    pub geometry: Geometry,
    pub bbox: [f64; 4],
    pub center: [f64; 2],
    pub place_name: String,
    pub place_type: Vec<String>,
    pub place_type_name: Vec<String>,
    pub relevance: f64,
}

#[derive(Serialize)]
pub struct Geometry {
    pub r#type: &'static str,
    pub coordinates: [f64; 2],
}

impl FeatureCollection {
    pub fn empty(query: &str) -> Self {
        FeatureCollection {
            r#type: "FeatureCollection",
            features: vec![],
            query: vec![query.to_string()],
            attribution: attribution(),
        }
    }
}

pub fn attribution() -> String {
    r#"<a href="https://www.openstreetmap.org/copyright" target="_blank">&copy; OpenStreetMap contributors</a>"#.to_string()
}
