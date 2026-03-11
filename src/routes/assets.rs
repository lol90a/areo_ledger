use actix_web::{web, HttpResponse, Result};
use crate::errors::AppError;
use crate::services::AssetService;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/assets")
            .route("", web::get().to(get_all))
            .route("/{id}", web::get().to(get_by_id))
    );
}

async fn get_all() -> Result<HttpResponse, AppError> {
    let assets = AssetService::get_all_assets();
    Ok(HttpResponse::Ok().json(assets))
}

async fn get_by_id(path: web::Path<String>) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    
    let asset = AssetService::get_asset_by_id(&id)
        .ok_or(AppError::NotFound)?;
    
    Ok(HttpResponse::Ok().json(asset))
}
