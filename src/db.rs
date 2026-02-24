use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

// Initialize PostgreSQL connection pool with optimizations
pub async fn init_db(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(20)           // Connection pool size
        .min_connections(5)            // Minimum idle connections
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .connect(database_url)
        .await
        .expect("Failed to connect to PostgreSQL")
}
