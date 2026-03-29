use deadpool_postgres::{Config, Pool, Runtime, PoolError};
use tokio_postgres::NoTls;

pub type DbPool = Pool;
pub type DbError = PoolError;

pub fn init_pool(database_url: &str) -> DbPool {
    if database_url.trim().is_empty() {
        log::error!("DATABASE_URL is empty");
        std::process::exit(1);
    }

    let mut cfg = Config::new();

    if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        cfg.url = Some(database_url.to_string());
    } else {
        for part in database_url.split_whitespace() {
            if let Some((key, val)) = part.split_once('=') {
                match key {
                    "host"     => cfg.host     = Some(val.to_string()),
                    "port"     => cfg.port     = val.parse().ok(),
                    "dbname"   => cfg.dbname   = Some(val.to_string()),
                    "user"     => cfg.user     = Some(val.to_string()),
                    "password" => cfg.password = Some(val.to_string()),
                    _ => {}
                }
            }
        }
    }

    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .unwrap_or_else(|e| {
            log::error!("Failed to create connection pool: {}", e);
            std::process::exit(1);
        })
}

pub async fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = pool.get().await?;
    conn.batch_execute(include_str!("../../migrations/complete_schema.sql")).await?;
    log::info!("Database migrations applied successfully.");
    Ok(())
}


