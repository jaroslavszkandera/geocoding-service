use sqlx::{PgPool, query_builder::QueryBuilder};

use crate::models::{Feature, FeatureCollection, FeatureProperties, Geometry, SearchParams};

pub async fn search(
    pool: &PgPool,
    params: &SearchParams,
) -> Result<FeatureCollection, crate::models::GeocodeError> {
    log::info!(
        "Forward geocoding query: '{}', params: {:?}",
        params.query,
        params
    );
    params.validate()?;
    params.require_api_key()?;

    let limit = params.limit.unwrap_or(5).min(10) as i64;
    let query = &params.query;

    let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r#"
        SELECT osm_id, name, feature_type,
               ST_X(geom) AS lon, ST_Y(geom) AS lat,
               similarity(name, "#,
    );

    qb.push_bind(query);
    qb.push(") AS score FROM places WHERE name % ");
    qb.push_bind(query);

    if let Some(bbox) = params.bbox {
        qb.push(" AND geom && ST_MakeEnvelope(");
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
            qb.push(" AND feature_type ");
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

    qb.push(" ORDER BY score DESC LIMIT ");
    qb.push_bind(limit);

    let rows = qb
        .build_query_as::<(i64, String, String, f64, f64, f32)>()
        .fetch_all(pool)
        .await?;

    let features: Vec<_> = rows
        .into_iter()
        .map(|(osm_id, name, feature_type, lon, lat, score)| {
            let epsilon = 0.001;
            let bbox = [lon - epsilon, lat - epsilon, lon + epsilon, lat + epsilon];

            Feature {
                id: format!("{}.{}", feature_type, osm_id),
                text: name.clone(),
                r#type: "Feature",
                geometry: Geometry {
                    r#type: "Point",
                    coordinates: [lon, lat],
                },
                bbox,
                center: [lon, lat],
                place_name: name.clone(),
                place_type: vec![feature_type.clone()],
                place_type_name: vec![feature_type],
                relevance: score as f64,
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

    log::info!("Forward geocoding returned {} results", features.len());
    Ok(FeatureCollection {
        r#type: "FeatureCollection",
        features,
        query: vec![query.to_string()],
        attribution: crate::models::attribution(),
    })
}
