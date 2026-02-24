use actix_web::{web, Result};
use uuid::Uuid;
use crate::routes::dto::CreateBookingRequest;
use crate::errors::AppError;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/bookings")
            .route("", web::post().to(create))
    );
}

// Creates a new booking with automatic markup calculation
async fn create(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<CreateBookingRequest>,
) -> Result<web::Json<serde_json::Value>, AppError> {
    // Validate payment method and price
    payload.validate().map_err(|e: String| AppError::ValidationError(e))?;

    let booking_id = Uuid::new_v4().to_string();
    // Apply 15% markup: 10% profit + 5% service fee
    let total_price = payload.base_price * 1.15;

    sqlx::query(
        "INSERT INTO bookings (id, user_id, flight_id, status, base_price, markup, service_fee, total_price, payment_method) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(&booking_id)
    .bind(payload.user_id.to_string())
    .bind(&payload.flight_id)
    .bind("pending")
    .bind(payload.base_price)
    .bind(payload.base_price * 0.10)  // 10% markup
    .bind(payload.base_price * 0.05)  // 5% service fee
    .bind(total_price)
    .bind(&payload.payment_method)
    .execute(&**pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(web::Json(serde_json::json!({
        "booking_id": booking_id,
        "total_price": total_price
    })))
}
