use actix_web::{web, HttpResponse};

use crate::infrastructure::db::DbPool;
use crate::infrastructure::persistence::postgres::pg_catalog_repository::PgCatalogRepository;
use crate::interfaces::http::error_response::into_response;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/flights").route("", web::get().to(list_flights)))
        .service(web::scope("/assets").route("", web::get().to(list_assets)));
}

async fn list_flights(pool: web::Data<DbPool>) -> HttpResponse {
    let repo = PgCatalogRepository::new(pool.get_ref().clone());
    match repo.list_flights().await {
        Ok(flights) => HttpResponse::Ok().json(serde_json::json!({ "flights": flights })),
        Err(e) => into_response(e),
    }
}

async fn list_assets(pool: web::Data<DbPool>) -> HttpResponse {
    let repo = PgCatalogRepository::new(pool.get_ref().clone());
    match repo.list_assets().await {
        Ok(assets) => HttpResponse::Ok().json(serde_json::json!({ "assets": assets })),
        Err(e) => into_response(e),
    }
}
