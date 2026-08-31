use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::{GeocodeError, SearchParams};
use crate::parsers;
use crate::{forward, reverse};

#[derive(Deserialize, Debug, Default)]
pub struct GeocodeQueryParams {
    #[serde(default, rename = "key")]
    pub api_key: Option<String>,
    #[serde(default)]
    pub bbox: Option<String>,
    #[serde(default)]
    pub proximity: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub types: Option<String>,
    #[serde(default)]
    pub exclude_types: Option<bool>,
}

pub async fn geocode(
    Path(raw_query): Path<String>,
    Query(params): Query<GeocodeQueryParams>,
    State(pool): State<PgPool>,
) -> Response {
    let query = strip_json_suffix(&raw_query);
    log::info!("Geocoding request: query='{}', params={:?}", query, params);

    let search_params = match build_search_params(query, params) {
        Ok(p) => p,
        Err(e) => return error_response(e),
    };

    let result = match parsers::parse_coords(query) {
        Some((lon, lat)) => reverse::search(&pool, lon, lat, &search_params).await,
        None => forward::search(&pool, &search_params).await,
    };

    match result {
        Ok(fc) => (StatusCode::OK, Json(fc)).into_response(),
        Err(e) => error_response(e),
    }
}

fn strip_json_suffix(s: &str) -> &str {
    if s.len() >= 5 && s[s.len() - 5..].eq_ignore_ascii_case(".json") {
        &s[..s.len() - 5]
    } else {
        s
    }
}

fn build_search_params(
    query: &str,
    params: GeocodeQueryParams,
) -> Result<SearchParams, GeocodeError> {
    let bbox = params
        .bbox
        .as_deref()
        .map(parsers::parse_bbox_str)
        .transpose()?;
    let proximity = params
        .proximity
        .as_deref()
        .map(parsers::parse_proximity_str)
        .transpose()?;
    let types = params.types.as_deref().map(parsers::parse_csv);

    let default_search_params = SearchParams::default();
    let search_params = SearchParams {
        query: query.to_string(),
        api_key: params.api_key,
        bbox,
        proximity,
        limit: params.limit,
        types,
        exclude_types: params
            .exclude_types
            .unwrap_or(default_search_params.exclude_types),
    };

    search_params.validate()?;
    Ok(search_params)
}

fn error_response(err: GeocodeError) -> Response {
    let (status, message) = match &err {
        GeocodeError::QueryTooLong => (StatusCode::BAD_REQUEST, err.to_string()),
        GeocodeError::InvalidParams(_) => (StatusCode::BAD_REQUEST, err.to_string()),
        GeocodeError::InvalidApiKey => (StatusCode::FORBIDDEN, err.to_string()),
        GeocodeError::DatabaseError(_) => {
            log::error!("{}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            )
        }
    };
    log::warn!("Geocoding error: {} - {}", status, message);
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}
