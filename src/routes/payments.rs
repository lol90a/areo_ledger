use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::services::PaymentService;

#[derive(Deserialize)]
pub struct InitPaymentRequest {
    pub booking_id: Uuid,
    pub method: String,
}

#[derive(Deserialize)]
pub struct ConfirmPaymentRequest {
    pub booking_id: Uuid,
    pub tx_hash: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payments")
            .route("/init", web::post().to(init_payment))
            .route("/confirm", web::post().to(confirm_payment))
    );
}

async fn init_payment(
    pool: web::Data<DbPool>,
    payload: web::Json<InitPaymentRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate payment method
    let allowed = ["usdt", "usdc", "eth", "sol", "btc", "binance"];
    if !allowed.contains(&payload.method.as_str()) {
        return Err(AppError::ValidationError("Unsupported payment method".to_string()));
    }

    let (wallet_address, amount) = PaymentService::init_payment(
        &pool,
        payload.booking_id,
        &payload.method,
    )?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "wallet_address": wallet_address,
        "amount": amount
    })))
}

async fn confirm_payment(
    pool: web::Data<DbPool>,
    payload: web::Json<ConfirmPaymentRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate tx hash
    if payload.tx_hash.len() < 10 {
        return Err(AppError::ValidationError("Invalid tx hash".to_string()));
    }

    PaymentService::confirm_payment(
        &pool,
        payload.booking_id,
        &payload.tx_hash,
    )?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Payment confirmed successfully"
    })))
}
