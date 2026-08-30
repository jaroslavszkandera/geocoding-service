use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn init_pool(database_url: &str) -> PgPool {
    log::info!("Connecting to the DB: {}", database_url);
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("failed to connect to database")
}
