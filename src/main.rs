mod routes;
mod db;
mod config;
mod errors;
mod auth;
mod models;
mod services;
mod middleware;

use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //build repository layer that services call with DTOs or (Business model) that handles connections with database
    env_logger::init();

    let config = config::AppConfig::from_env();
    let pool = db::init_db(&config.database_url);

    if let Err(e) = db::create_tables(&pool) {
        eprintln!("Failed to create tables: {}", e);
        std::process::exit(1);
    }

    println!("🚀 Server starting on http://127.0.0.1:8080");

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(100)
        .finish()
        .unwrap();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:3001")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Governor::new(&governor_conf))
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .configure(routes::config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
