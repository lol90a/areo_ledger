mod routes;
mod db;
mod config;
mod errors;
mod auth;

use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment configuration
    let config = config::AppConfig::from_env();

    // Initialize PostgreSQL database connection pool with optimizations
    let pool = db::init_db(&config.database_url).await;

    // Run database migrations on startup
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    println!("🚀 Server starting on http://127.0.0.1:8080");

    // Configure rate limiting: 100 requests per minute
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(100)
        .finish()
        .unwrap();

    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS for frontend access
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Governor::new(&governor_conf))  // Rate limiting
            .wrap(middleware::Logger::default())   // Request logging
            .app_data(web::Data::new(pool.clone()))
            .configure(routes::config)  // Mount all API routes
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
