use crate::models::{FeatureCollection, PlaceRow, SearchParams};
use sqlx::PgPool;
use unicode_general_category::{GeneralCategory, get_general_category};
use unicode_normalization::UnicodeNormalization;

const EXACT_SQL: &str = r#"
    SELECT osm_id, name, feature_type, ST_X(geom) AS lon, ST_Y(geom) AS lat,
           bbox_west, bbox_south, bbox_east, bbox_north,
           housenumber, street, city, state, country_code, postcode,
           1.0::float8 AS score
    FROM places
    WHERE lower(name_unaccent) = lower(immutable_unaccent($1))
      AND ($2::boolean = false OR geom && ST_MakeEnvelope($3, $4, $5, $6, 4326))
      AND ($7::text[] IS NULL OR (($8::boolean = false AND feature_type = ANY($7)) OR ($8::boolean = true AND feature_type != ALL($7))))
      AND ($9::bigint[] IS NULL OR osm_id != ALL($9))
    LIMIT $10
"#;

const FTS_SQL: &str = r#"
    SELECT osm_id, name, feature_type, ST_X(geom) AS lon, ST_Y(geom) AS lat,
           bbox_west, bbox_south, bbox_east, bbox_north,
           housenumber, street, city, state, country_code, postcode,
           ts_rank(search_vector, to_tsquery('simple', $1))::float8 AS score
    FROM places
    WHERE search_vector @@ to_tsquery('simple', $1)
      AND ($2::boolean = false OR geom && ST_MakeEnvelope($3, $4, $5, $6, 4326))
      AND ($7::text[] IS NULL OR (($8::boolean = false AND feature_type = ANY($7)) OR ($8::boolean = true AND feature_type != ALL($7))))
      AND ($9::bigint[] IS NULL OR osm_id != ALL($9))
    ORDER BY score DESC
    LIMIT $10
"#;

const TRGM_SQL: &str = r#"
    SELECT osm_id, name, feature_type, ST_X(geom) AS lon, ST_Y(geom) AS lat,
           bbox_west, bbox_south, bbox_east, bbox_north,
           housenumber, street, city, state, country_code, postcode,
           similarity(name_unaccent, $1)::float8 AS score
    FROM places
    WHERE name_unaccent % $1
      AND similarity(name_unaccent, $1) >= 0.5
      AND ($2::boolean = false OR geom && ST_MakeEnvelope($3, $4, $5, $6, 4326))
      AND ($7::text[] IS NULL OR (($8::boolean = false AND feature_type = ANY($7)) OR ($8::boolean = true AND feature_type != ALL($7))))
      AND ($9::bigint[] IS NULL OR osm_id != ALL($9))
    ORDER BY name_unaccent <-> $1
    LIMIT $10
"#;

fn build_prefix_tsquery(query: &str) -> String {
    let mut tokens: Vec<String> = query
        .split_whitespace()
        .map(|t| format!("'{}'", t.replace('\'', "''")))
        .collect();
    if let Some(last) = tokens.last_mut() {
        last.push_str(":*");
    }
    tokens.join(" & ")
}

/// Strip diacritics
fn unaccent(query: &str) -> String {
    query
        .nfd()
        .filter(|c| get_general_category(*c) != GeneralCategory::NonspacingMark)
        .collect()
}

pub async fn search(
    pool: &PgPool,
    params: &SearchParams,
) -> Result<FeatureCollection, crate::models::GeocodeError> {
    params.validate()?;
    params.require_api_key()?;

    let limit = params.limit.unwrap_or(5).min(10) as i64;
    let bbox = params.bbox.unwrap_or([-180.0, -90.0, 180.0, 90.0]);
    let use_bbox = params.bbox.is_some();
    let query_unaccent = unaccent(&params.query);
    let ts_query = build_prefix_tsquery(&query_unaccent);

    let mut rows: Vec<PlaceRow> = sqlx::query_as::<_, PlaceRow>(EXACT_SQL)
        .bind(&params.query)
        .bind(use_bbox)
        .bind(bbox[0])
        .bind(bbox[1])
        .bind(bbox[2])
        .bind(bbox[3])
        .bind(params.types.as_deref())
        .bind(params.exclude_types)
        .bind(None::<Vec<i64>>)
        .bind(limit)
        .fetch_all(pool)
        .await?;
    log::debug!(
        "forward search [exact] query='{}' rows={}",
        params.query,
        rows.len()
    );

    if (rows.len() as i64) < limit {
        let seen: Vec<i64> = rows.iter().map(|r| r.osm_id).collect();
        let remaining = limit - rows.len() as i64;

        let fts_rows = sqlx::query_as::<_, PlaceRow>(FTS_SQL)
            .bind(&ts_query)
            .bind(use_bbox)
            .bind(bbox[0])
            .bind(bbox[1])
            .bind(bbox[2])
            .bind(bbox[3])
            .bind(params.types.as_deref())
            .bind(params.exclude_types)
            .bind(Some(&seen))
            .bind(remaining)
            .fetch_all(pool)
            .await?;
        log::debug!(
            "forward search [fts] query='{}' tsquery='{}' rows={}",
            params.query,
            ts_query,
            fts_rows.len()
        );
        rows.extend(fts_rows);
    }

    if (rows.len() as i64) < limit {
        let seen: Vec<i64> = rows.iter().map(|r| r.osm_id).collect();
        let remaining = limit - rows.len() as i64;

        let trgm_rows = sqlx::query_as::<_, PlaceRow>(TRGM_SQL)
            .bind(&query_unaccent)
            .bind(use_bbox)
            .bind(bbox[0])
            .bind(bbox[1])
            .bind(bbox[2])
            .bind(bbox[3])
            .bind(params.types.as_deref())
            .bind(params.exclude_types)
            .bind(Some(&seen))
            .bind(remaining)
            .fetch_all(pool)
            .await?;
        log::debug!(
            "forward search [trgm] query='{}' unaccent='{}' rows={}",
            params.query,
            query_unaccent,
            trgm_rows.len()
        );
        rows.extend(trgm_rows);
    }

    if let Some(prox) = params.proximity {
        for row in &mut rows {
            let dist_deg = ((row.lon - prox[0]).powi(2) + (row.lat - prox[1]).powi(2)).sqrt();
            let dist_km = dist_deg * 111.0;
            row.score = (row.score * 0.7) + ((1.0 / (1.0 + dist_km)) * 0.3);
        }
        rows.sort_by(|a, b| b.score.total_cmp(&a.score));
        log::debug!(
            "forward search [proximity-rerank] proximity={:?} rows={}",
            prox,
            rows.len()
        );
    }

    let features: Vec<_> = rows.into_iter().map(PlaceRow::into_feature).collect();
    Ok(FeatureCollection {
        r#type: "FeatureCollection",
        features,
        query: vec![params.query.clone()],
        attribution: crate::models::attribution(),
    })
}
