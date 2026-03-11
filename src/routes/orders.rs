use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::services::{OrderService, order_service::OrderItem};

#[derive(Deserialize)]
pub struct UpdateOrderRequest {
    pub sender_wallet: String,
    pub public_key: Option<String>,
    pub tx_hash: String,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub total: f64,
    pub crypto_method: String,
    pub sender_wallet: String,
    pub tx_hash: Option<String>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orders")
            .route("", web::post().to(create_order))
            .route("/{id}", web::get().to(get_order))
            .route("/{id}", web::put().to(update_order))
            .route("/user/{user_id}", web::get().to(get_user_orders))
    );
}

async fn create_order(
    pool: web::Data<DbPool>,
    payload: web::Json<CreateOrderRequest>,
) -> Result<HttpResponse, AppError> {
    let order = OrderService::create_order(
        &pool,
        &payload.user_id,
        payload.items.clone(),
        payload.total,
        &payload.crypto_method,
        &payload.sender_wallet,
        payload.tx_hash.as_deref(),
    )?;

    Ok(HttpResponse::Ok().json(order))
}

async fn update_order(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateOrderRequest>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();
    
    OrderService::update_order(
        &pool,
        &order_id,
        &payload.sender_wallet,
        &payload.tx_hash,
    )?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Order updated successfully",
        "order_id": order_id,
    })))
}

async fn get_order(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();
    
    let order = OrderService::get_order_by_id(&pool, &order_id)?;
    
    Ok(HttpResponse::Ok().json(order))
}

async fn get_user_orders(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    
    let orders = OrderService::get_user_orders(&pool, &user_id)?;
    
    Ok(HttpResponse::Ok().json(orders))
}
