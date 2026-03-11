use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::services::BookingService;

#[derive(Deserialize)]
pub struct CreateBookingRequest {
    pub user_id: Uuid,
    pub flight_id: Uuid,
    pub base_price: f64,
    pub payment_method: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/bookings")
            .route("", web::post().to(create))
    );
}

async fn create(
    pool: web::Data<DbPool>,
    payload: web::Json<CreateBookingRequest>,
) -> Result<HttpResponse, AppError> {
    //TODO: separate concern of routes , and implement controller layer that handles calling the service, also implement validation generic middleware for handling validations in the future with objects
    // Validate payment method
    let allowed = ["usdt", "usdc", "eth", "sol", "btc", "binance"];
    if !allowed.contains(&payload.payment_method.as_str()) {
        return Err(AppError::ValidationError("Unsupported payment method".to_string()));
    }

    // Validate base price
    if payload.base_price <= 0.0 {
        return Err(AppError::ValidationError("Base price must be positive".to_string()));
    }

    let (booking_id, total_price) = BookingService::create_booking(
        &pool,
        payload.user_id,
        payload.flight_id,
        payload.base_price,
        &payload.payment_method,
    )?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "booking_id": booking_id,
        "total_price": total_price
    })))
}
