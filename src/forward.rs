use sqlx::PgPool;

use crate::models::FeatureCollection;

pub async fn search(_pool: &PgPool, query: &str) -> FeatureCollection {
    FeatureCollection::empty(query)
}
