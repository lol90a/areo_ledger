use actix_web::{web, HttpResponse, Result};
use crate::errors::AppError;
use crate::services::ProductService;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .route("", web::get().to(get_all))
            .route("/{id}", web::get().to(get_by_id))
    );
}

async fn get_all() -> Result<HttpResponse, AppError> {
    let products = ProductService::get_all_products();
    Ok(HttpResponse::Ok().json(products))
}

async fn get_by_id(path: web::Path<i32>) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    
    let product = ProductService::get_product_by_id(id)
        .ok_or(AppError::NotFound)?;
    
    Ok(HttpResponse::Ok().json(product))
}
