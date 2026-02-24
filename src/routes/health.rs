use actix_web::{web, HttpResponse, Result};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/health")
            .route("", web::get().to(health_check))
    );
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json("OK"))
}
