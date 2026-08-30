use sqlx::PgPool;

use crate::models::FeatureCollection;

pub async fn search(_pool: &PgPool, lon: f64, lat: f64) -> FeatureCollection {
    FeatureCollection::empty(&format!("{lon},{lat}"))
}
