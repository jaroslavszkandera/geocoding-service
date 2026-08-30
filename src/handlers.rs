use axum::Json;
use axum::extract::{Path, State};
use sqlx::PgPool;

use crate::models::FeatureCollection;
use crate::{forward, reverse};

pub async fn geocode(
    Path(raw_query): Path<String>,
    State(pool): State<PgPool>,
) -> Json<FeatureCollection> {
    let query = raw_query.trim_end_matches(".json");

    let result = match parse_coords(query) {
        Some((lon, lat)) => reverse::search(&pool, lon, lat).await,
        None => forward::search(&pool, query).await,
    };

    Json(result)
}

fn parse_coords(query: &str) -> Option<(f64, f64)> {
    let (lon_str, lat_str) = query.split_once(',')?;
    let lon: f64 = lon_str.trim().parse().ok()?;
    let lat: f64 = lat_str.trim().parse().ok()?;

    if (-180.0..=180.0).contains(&lon) && (-90.0..=90.0).contains(&lat) {
        Some((lon, lat))
    } else {
        None
    }
}
