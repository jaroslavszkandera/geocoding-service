use crate::models::{FeatureCollection, PlaceRow, SearchParams};
use sqlx::PgPool;

pub async fn search(
    pool: &PgPool,
    lon: f64,
    lat: f64,
    params: &SearchParams,
) -> Result<FeatureCollection, crate::models::GeocodeError> {
    params.validate()?;
    params.require_api_key()?;

    let limit = params.limit.unwrap_or(1).min(10) as i64;
    let knn_limit = (limit * 3).max(10);
    let bbox = params.bbox.unwrap_or([-180.0, -90.0, 180.0, 90.0]);
    let use_bbox = params.bbox.is_some();

    log::debug!(
        "reverse search [knn] lon={} lat={} knn_limit={}",
        lon,
        lat,
        knn_limit
    );

    let rows = sqlx::query_as::<_, PlaceRow>(
        r#"
        WITH nearest AS (
            SELECT osm_id, name, feature_type, geom, housenumber, street, city, state, country_code, postcode,
                   bbox_west, bbox_south, bbox_east, bbox_north
            FROM places
            WHERE ($1::boolean = false OR geom && ST_MakeEnvelope($2, $3, $4, $5, 4326))
              AND ($6::text[] IS NULL OR (($7::boolean = false AND feature_type = ANY($6)) OR ($7::boolean = true AND feature_type != ALL($6))))
            ORDER BY geom <-> ST_SetSRID(ST_MakePoint($8, $9), 4326)
            LIMIT $10
        )
        SELECT osm_id, name, feature_type, ST_X(geom) AS lon, ST_Y(geom) AS lat,
               bbox_west, bbox_south, bbox_east, bbox_north,
               housenumber, street, city, state, country_code, postcode,
               1.0 / (1.0 + ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($8, $9), 4326)::geography) / 1000.0) AS score
        FROM nearest
        ORDER BY score DESC LIMIT $11
        "#
    )
    .bind(use_bbox).bind(bbox[0]).bind(bbox[1]).bind(bbox[2]).bind(bbox[3])
    .bind(params.types.as_deref()).bind(params.exclude_types)
    .bind(lon).bind(lat).bind(knn_limit).bind(limit)
    .fetch_all(pool).await?;

    log::debug!(
        "reverse search [exact-rank] lon={} lat={} rows={}",
        lon,
        lat,
        rows.len()
    );

    let features: Vec<_> = rows.into_iter().map(PlaceRow::into_feature).collect();
    Ok(FeatureCollection {
        r#type: "FeatureCollection",
        features,
        query: vec![format!("{},{}", lon, lat)],
        attribution: crate::models::attribution(),
    })
}
