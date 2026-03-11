use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::PaymentService;

#[derive(Deserialize)]
pub struct InitPaymentRequest {
    pub booking_id: Uuid,
    pub payment_method: String,
}

#[derive(Deserialize)]
pub struct ConfirmPaymentRequest {
    pub booking_id: Uuid,
    pub tx_hash: String,
    pub sender_address: Option<String>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payments")
            .route("/init", web::post().to(init_payment))
            .route("/confirm", web::post().to(confirm_payment))
            .route("/{payment_id}", web::get().to(get_payment_status))
            .route("/booking/{booking_id}", web::get().to(get_booking_payments))
    );
}

/// Initialize a new payment
async fn init_payment(
    pool: web::Data<PgPool>,
    payload: web::Json<InitPaymentRequest>,
) -> impl Responder {
    let payment_service = PaymentService::new();

    match payment_service
        .init_payment(&pool, payload.booking_id, &payload.payment_method)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// Confirm payment with transaction hash
async fn confirm_payment(
    pool: web::Data<PgPool>,
    payload: web::Json<ConfirmPaymentRequest>,
) -> impl Responder {
    let payment_service = PaymentService::new();

    match payment_service
        .confirm_payment(
            &pool,
            payload.booking_id,
            &payload.tx_hash,
            payload.sender_address.clone(),
        )
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// Get payment status by payment ID
async fn get_payment_status(
    pool: web::Data<PgPool>,
    payment_id: web::Path<Uuid>,
) -> impl Responder {
    let payment_service = PaymentService::new();

    match payment_service
        .get_payment_status(&pool, payment_id.into_inner())
        .await
    {
        Ok(payment) => HttpResponse::Ok().json(payment),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "error": e
        })),
    }
}

/// Get all payments for a booking
async fn get_booking_payments(
    pool: web::Data<PgPool>,
    booking_id: web::Path<Uuid>,
) -> impl Responder {
    let payment_service = PaymentService::new();

    match payment_service
        .get_booking_payments(&pool, booking_id.into_inner())
        .await
    {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}
