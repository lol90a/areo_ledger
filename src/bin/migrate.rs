use areo_ledger::infrastructure::db::{init_pool, run_migrations};

#[actix_web::main]
async fn main() {
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_default();
    let pool = init_pool(&database_url);

    if let Err(e) = run_migrations(&pool).await {
        log::error!("Migration failed: {}", e);
        let mut source = std::error::Error::source(e.as_ref());
        while let Some(err) = source {
            log::error!("Caused by: {}", err);
            source = err.source();
        }
        std::process::exit(1);
    }
}
