use axum::Router;
use axum::routing::get;
use geocoding_service::{db, handlers};

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::init_pool(&database_url).await;

    let app = Router::new()
        .route("/geocoding/{query}", get(handlers::geocode))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    log::info!("listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
