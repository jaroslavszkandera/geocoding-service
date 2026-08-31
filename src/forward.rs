use sqlx::{PgPool, query_builder::QueryBuilder};

use crate::models::{BBOX_GEOM_EXPR, FeatureCollection, PlaceRow, SearchParams};

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
    let prox_lon = params.proximity.map(|p| p[0]);
    let prox_lat = params.proximity.map(|p| p[1]);

    let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        "WITH candidates AS (SELECT osm_id, name, feature_type, geom, housenumber, street, \
         city, state, country_code, postcode FROM places WHERE name % ",
    );
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

    qb.push(" ORDER BY name <-> ");
    qb.push_bind(query);
    qb.push(
        " LIMIT 50) SELECT osm_id, name, feature_type, ST_X(geom) AS lon, ST_Y(geom) AS lat, \
         ST_XMin(bbox_geom) AS bbox_west, ST_YMin(bbox_geom) AS bbox_south, \
         ST_XMax(bbox_geom) AS bbox_east, ST_YMax(bbox_geom) AS bbox_north, \
         housenumber, street, city, state, country_code, postcode, \
         (similarity(name, ",
    );
    qb.push_bind(query);
    qb.push(
        ") * 0.7 + COALESCE(1.0 / (1.0 + ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint(",
    );
    qb.push_bind(prox_lon);
    qb.push(", ");
    qb.push_bind(prox_lat);
    qb.push("), 4326)::geography) / 1000.0), 0) * 0.3) AS score FROM (SELECT *, ");
    qb.push(BBOX_GEOM_EXPR);
    qb.push(" AS bbox_geom FROM candidates) sub ORDER BY score DESC LIMIT ");
    qb.push_bind(limit);

    let rows = qb.build_query_as::<PlaceRow>().fetch_all(pool).await?;
    let features: Vec<_> = rows.into_iter().map(PlaceRow::into_feature).collect();

    log::info!("Forward geocoding returned {} results", features.len());
    Ok(FeatureCollection {
        r#type: "FeatureCollection",
        features,
        query: vec![query.to_string()],
        attribution: crate::models::attribution(),
    })
}
