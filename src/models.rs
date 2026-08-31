use serde::Serialize;

#[derive(Serialize)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub r#type: &'static str,
    pub features: Vec<Feature>,
    pub query: Vec<String>,
    pub attribution: String,
}

#[derive(Serialize)]
pub struct Feature {
    pub id: String,
    pub text: String,
    #[serde(rename = "type")]
    pub r#type: &'static str,
    pub geometry: Geometry,
    pub bbox: [f64; 4],
    pub center: [f64; 2],
    pub place_name: String,
    pub place_type: Vec<String>,
    pub place_type_name: Vec<String>,
    pub relevance: f64,
    pub properties: FeatureProperties,
    pub context: Vec<ContextItem>,
    pub address: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct FeatureProperties {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub kind: Option<String>,
    pub categories: Vec<String>,
    pub feature_tags: serde_json::Value,
    pub place_designation: Option<String>,
    #[serde(flatten)]
    pub additional: serde_json::Value,
}

#[derive(Serialize, Debug, Clone)]
pub struct ContextItem {
    pub id: String,
    pub text: String,
    #[serde(rename = "ref")]
    pub ref_: String,
    pub kind: Option<String>,
    pub categories: Vec<String>,
    pub feature_tags: serde_json::Value,
    pub place_designation: Option<String>,
    #[serde(flatten)]
    pub additional: serde_json::Value,
}

#[derive(Serialize, Debug, Clone)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub r#type: &'static str,
    pub coordinates: [f64; 2],
}

pub fn attribution() -> String {
    r#"<a href="https://www.openstreetmap.org/copyright" target="_blank">&copy; OpenStreetMap contributors</a>"#.to_string()
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

#[derive(Debug, Clone)]
pub struct SearchParams {
    pub query: String,
    pub api_key: Option<String>,
    pub bbox: Option<[f64; 4]>,
    pub proximity: Option<[f64; 2]>,
    pub limit: Option<u32>,
    pub types: Option<Vec<String>>,
    pub exclude_types: bool,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            api_key: None,
            bbox: None,
            proximity: None,
            limit: None,
            types: None,
            exclude_types: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GeocodeError {
    #[error("Query too long (max 256 chars)")]
    QueryTooLong,
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    #[error("API key is missing, invalid or restricted")]
    InvalidApiKey,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

impl SearchParams {
    pub fn validate(&self) -> Result<(), GeocodeError> {
        if self.query.len() > 256 {
            return Err(GeocodeError::QueryTooLong);
        }
        if self.query.is_empty() {
            return Err(GeocodeError::InvalidParams("query cannot be empty".into()));
        }
        if let Some(limit) = self.limit {
            if !(1..=10).contains(&limit) {
                return Err(GeocodeError::InvalidParams(
                    "limit must be between 1 and 10".into(),
                ));
            }
        }
        if let Some(bbox) = self.bbox {
            if bbox[0] >= bbox[2] || bbox[1] >= bbox[3] {
                return Err(GeocodeError::InvalidParams(
                    "invalid bbox: w < e and s < n required".into(),
                ));
            }
            if !(-180.0..=180.0).contains(&bbox[0])
                || !(-180.0..=180.0).contains(&bbox[2])
                || !(-90.0..=90.0).contains(&bbox[1])
                || !(-90.0..=90.0).contains(&bbox[3])
            {
                return Err(GeocodeError::InvalidParams(
                    "bbox coordinates out of range".into(),
                ));
            }
        }
        if let Some(proximity) = self.proximity {
            if !(-180.0..=180.0).contains(&proximity[0]) || !(-90.0..=90.0).contains(&proximity[1])
            {
                return Err(GeocodeError::InvalidParams(
                    "proximity coordinates out of range".into(),
                ));
            }
        }
        Ok(())
    }

    pub fn require_api_key(&self) -> Result<(), GeocodeError> {
        if self.api_key.is_none() {
            return Err(GeocodeError::InvalidApiKey);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_query() {
        let p = SearchParams::default();
        assert!(p.validate().is_err());
    }

    #[test]
    fn test_validate_query_too_long() {
        let p = SearchParams {
            query: "a".repeat(257),
            ..Default::default()
        };
        assert!(matches!(p.validate(), Err(GeocodeError::QueryTooLong)));
    }

    #[test]
    fn test_validate_query_max_length() {
        let p = SearchParams {
            query: "a".repeat(256),
            ..Default::default()
        };
        assert!(p.validate().is_ok());
    }

    #[test]
    fn test_validate_limit_min() {
        let p = SearchParams {
            query: "x".into(),
            limit: Some(0),
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn test_validate_limit_max() {
        let p = SearchParams {
            query: "x".into(),
            limit: Some(11),
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn test_validate_limit_in_range() {
        let p = SearchParams {
            query: "x".into(),
            limit: Some(5),
            ..Default::default()
        };
        assert!(p.validate().is_ok());
    }

    #[test]
    fn test_validate_bbox_west_ge_east() {
        let p = SearchParams {
            query: "x".into(),
            bbox: Some([15.0, 49.0, 14.0, 50.0]),
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn test_validate_bbox_south_ge_north() {
        let p = SearchParams {
            query: "x".into(),
            bbox: Some([14.0, 51.0, 15.0, 50.0]),
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn test_validate_bbox_out_of_range_lon() {
        let p = SearchParams {
            query: "x".into(),
            bbox: Some([200.0, 49.0, 15.0, 50.0]),
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn test_validate_bbox_valid() {
        let p = SearchParams {
            query: "x".into(),
            bbox: Some([14.0, 49.0, 15.0, 50.0]),
            ..Default::default()
        };
        assert!(p.validate().is_ok());
    }

    #[test]
    fn test_validate_proximity_lon_out_of_range() {
        let p = SearchParams {
            query: "x".into(),
            proximity: Some([200.0, 50.0]),
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn test_validate_proximity_lat_out_of_range() {
        let p = SearchParams {
            query: "x".into(),
            proximity: Some([14.0, 95.0]),
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }
}
