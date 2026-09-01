use crate::models::GeocodeError;

pub fn parse_csv(s: &str) -> Vec<String> {
    s.split(',').map(|p| p.trim().to_string()).collect()
}

pub fn parse_bbox_str(s: &str) -> Result<[f64; 4], GeocodeError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 {
        return Err(GeocodeError::InvalidParams("bbox must be 'W,S,E,N'".into()));
    }
    let coords = parse_f64_array(&parts)?;
    reject_non_finite(&coords, "bbox")?;
    Ok([coords[0], coords[1], coords[2], coords[3]])
}

pub fn parse_proximity_str(s: &str) -> Result<[f64; 2], GeocodeError> {
    if s.eq_ignore_ascii_case("ip") {
        return Err(GeocodeError::InvalidParams(
            "IP-based proximity not implemented".into(),
        ));
    }
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(GeocodeError::InvalidParams(
            "proximity must be 'lon,lat'".into(),
        ));
    }
    let coords = parse_f64_array(&parts)?;
    reject_non_finite(&coords, "proximity")?;
    Ok([coords[0], coords[1]])
}

pub fn parse_coords(s: &str) -> Option<(f64, f64)> {
    let (lon_str, lat_str) = s.split_once(',')?;
    let lon: f64 = lon_str.trim().parse().ok()?;
    let lat: f64 = lat_str.trim().parse().ok()?;

    if !lon.is_finite() || !lat.is_finite() {
        return None;
    }

    if (-180.0..=180.0).contains(&lon) && (-90.0..=90.0).contains(&lat) {
        Some((lon, lat))
    } else {
        log::debug!(
            "longitude ({}) or latitude ({}) outside of valid range",
            lon,
            lat
        );
        None
    }
}

fn parse_f64_array(parts: &[&str]) -> Result<Vec<f64>, GeocodeError> {
    let coords: Result<Vec<f64>, _> = parts.iter().map(|p| p.trim().parse()).collect();
    coords.map_err(|_| GeocodeError::InvalidParams("coordinates must be numbers".into()))
}

fn reject_non_finite(coords: &[f64], name: &str) -> Result<(), GeocodeError> {
    for c in coords {
        if !c.is_finite() {
            return Err(GeocodeError::InvalidParams(format!(
                "{} coordinates must be finite numbers",
                name
            )));
        }
    }
    Ok(())
}
