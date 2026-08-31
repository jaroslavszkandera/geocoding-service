use sqlx::{PgPool, query_builder::QueryBuilder};

use crate::models::{Feature, FeatureCollection, FeatureProperties, Geometry, SearchParams};

pub async fn search(
    pool: &PgPool,
    lon: f64,
    lat: f64,
    params: &SearchParams,
) -> Result<FeatureCollection, crate::models::GeocodeError> {
    log::info!(
        "Reverse geocoding query: ({}, {}), params: {:?}",
        lon,
        lat,
        params
    );
    params.validate()?;
    params.require_api_key()?;

    let limit = params.limit.unwrap_or(1).min(10) as i64;
    let query_str = format!("{},{}", lon, lat);

    let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r#"
        SELECT osm_id, name, feature_type,
               ST_X(geom) AS out_lon, ST_Y(geom) AS out_lat,
               ST_Distance(geom::geography, ST_MakePoint("#,
    );

    qb.push_bind(lon);
    qb.push(", ");
    qb.push_bind(lat);
    qb.push(")::geography) AS dist_m FROM places ");

    if let Some(bbox) = params.bbox {
        qb.push("WHERE geom && ST_MakeEnvelope(");
        qb.push_bind(bbox[0]);
        qb.push(", ");
        qb.push_bind(bbox[1]);
        qb.push(", ");
        qb.push_bind(bbox[2]);
        qb.push(", ");
        qb.push_bind(bbox[3]);
        qb.push(", 4326) ");
    }

    if let Some(types) = &params.types {
        if !types.is_empty() {
            if params.bbox.is_some() {
                qb.push(" AND ");
            } else {
                qb.push(" WHERE ");
            }
            qb.push("feature_type ");
            if params.exclude_types {
                qb.push("NOT ");
            }
            qb.push("IN (");
            let mut separated = qb.separated(", ");
            for t in types {
                separated.push_bind(t);
            }
            separated.push_unseparated(") ");
        }
    }

    if params.limit.is_some() && params.types.as_ref().map(|t| t.len()).unwrap_or(0) == 1 {
        qb.push(" ORDER BY geom <-> ST_MakePoint(");
        qb.push_bind(lon);
        qb.push(", ");
        qb.push_bind(lat);
        qb.push(") ");
    } else {
        qb.push(" ORDER BY dist_m ASC ");
    }

    qb.push(" LIMIT ");
    qb.push_bind(limit);

    let rows = qb
        .build_query_as::<(i64, String, String, f64, f64, f64)>()
        .fetch_all(pool)
        .await?;

    let features: Vec<_> = rows
        .into_iter()
        .map(|(osm_id, name, feature_type, out_lon, out_lat, dist_m)| {
            let relevance = if dist_m == 0.0 {
                1.0
            } else {
                1.0 / (1.0 + dist_m / 1000.0)
            };

            let epsilon = 0.001;
            let bbox = [
                out_lon - epsilon,
                out_lat - epsilon,
                out_lon + epsilon,
                out_lat + epsilon,
            ];

            Feature {
                id: format!("{}.{}", feature_type, osm_id),
                text: name.clone(),
                r#type: "Feature",
                geometry: Geometry {
                    r#type: "Point",
                    coordinates: [out_lon, out_lat],
                },
                bbox,
                center: [out_lon, out_lat],
                place_name: name.clone(),
                place_type: vec![feature_type.clone()],
                place_type_name: vec![feature_type],
                relevance,
                properties: FeatureProperties {
                    ref_: format!("osm:{}", osm_id),
                    kind: None,
                    categories: vec![],
                    feature_tags: serde_json::json!({}),
                    place_designation: None,
                    additional: serde_json::json!({}),
                },
                context: vec![],
                address: None,
            }
        })
        .collect();

    log::info!("Reverse geocoding returned {} results", features.len());
    Ok(FeatureCollection {
        r#type: "FeatureCollection",
        features,
        query: vec![query_str],
        attribution: crate::models::attribution(),
    })
}

