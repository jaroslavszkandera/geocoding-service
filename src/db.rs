use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub test_before_acquire: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 16,
            min_connections: 4,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            test_before_acquire: true,
        }
    }
}

impl PoolConfig {
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("DB_MAX_CONNECTIONS") {
            if let Ok(n) = v.parse() {
                cfg.max_connections = n;
            }
        }
        if let Ok(v) = std::env::var("DB_MIN_CONNECTIONS") {
            if let Ok(n) = v.parse() {
                cfg.min_connections = n;
            }
        }
        if let Ok(v) = std::env::var("DB_ACQUIRE_TIMEOUT_SECS") {
            if let Ok(n) = v.parse() {
                cfg.acquire_timeout = Duration::from_secs(n);
            }
        }
        if let Ok(v) = std::env::var("DB_IDLE_TIMEOUT_SECS") {
            if let Ok(n) = v.parse() {
                cfg.idle_timeout = Duration::from_secs(n);
            }
        }
        if let Ok(v) = std::env::var("DB_MAX_LIFETIME_SECS") {
            if let Ok(n) = v.parse() {
                cfg.max_lifetime = Duration::from_secs(n);
            }
        }
        if let Ok(v) = std::env::var("DB_TEST_BEFORE_ACQUIRE") {
            cfg.test_before_acquire = matches!(v.as_str(), "1" | "true" | "TRUE" | "yes");
        }
        cfg
    }
}

pub async fn init_pool(database_url: &str) -> PgPool {
    init_pool_with_config(database_url, PoolConfig::from_env()).await
}

pub async fn init_pool_with_config(database_url: &str, cfg: PoolConfig) -> PgPool {
    log::info!(
        "Connecting to DB: {} (max={}, min={}, acquire_timeout={}s, idle_timeout={}s, max_lifetime={}s, test_before_acquire={})",
        database_url,
        cfg.max_connections,
        cfg.min_connections,
        cfg.acquire_timeout.as_secs(),
        cfg.idle_timeout.as_secs(),
        cfg.max_lifetime.as_secs(),
        cfg.test_before_acquire,
    );

    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .min_connections(cfg.min_connections)
        .acquire_timeout(cfg.acquire_timeout)
        .idle_timeout(cfg.idle_timeout)
        .max_lifetime(cfg.max_lifetime)
        .test_before_acquire(cfg.test_before_acquire)
        .connect(database_url)
        .await
        .expect("failed to connect to database")
}
