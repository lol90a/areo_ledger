use actix_web::{web, Result};
use crate::routes::dto::{InitPaymentRequest, ConfirmPaymentRequest};
use crate::errors::AppError;
use uuid::Uuid;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payments")
            .route("/init", web::post().to(init))
            .route("/confirm", web::post().to(confirm))
    );
}

// Initialize crypto payment - generates wallet address and amount
async fn init(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<InitPaymentRequest>,
) -> Result<web::Json<serde_json::Value>, AppError> {
    payload.validate().map_err(|e: String| AppError::ValidationError(e))?;

    // Fetch booking total price
    let booking: (f64,) = sqlx::query_as(
        "SELECT total_price FROM bookings WHERE id = $1"
    )
    .bind(&payload.booking_id.to_string())
    .fetch_one(&**pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Map payment method to blockchain network and token
    let (chain, token) = match payload.method.as_str() {
        "usdt" | "usdc" | "eth" => ("Ethereum", payload.method.to_uppercase()),
        "btc" => ("Bitcoin", "BTC".to_string()),
        "sol" => ("Solana", "SOL".to_string()),
        "binance" => ("BSC", "BNB".to_string()),
        _ => ("Ethereum", "USDT".to_string()),
    };

    let payment_id = Uuid::new_v4().to_string();
    // TODO: Replace with actual wallet address from environment or config
    let receiver_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";

    // Store payment record
    sqlx::query(
        "INSERT INTO payments (id, booking_id, chain, token, amount, receiver_address, status) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&payment_id)
    .bind(&payload.booking_id.to_string())
    .bind(chain)
    .bind(&token)
    .bind(booking.0)
    .bind(receiver_address)
    .bind("pending")
    .execute(&**pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(web::Json(serde_json::json!({
        "payment_id": payment_id,
        "receiver_address": receiver_address,
        "amount": booking.0,
        "chain": chain,
        "token": token
    })))
}

// Confirm payment with transaction hash from blockchain
async fn confirm(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<ConfirmPaymentRequest>,
) -> Result<web::Json<serde_json::Value>, AppError> {
    payload.validate().map_err(|e: String| AppError::ValidationError(e))?;

    // Update payment status with transaction hash
    sqlx::query(
        "UPDATE payments SET tx_hash = $1, status = $2 WHERE booking_id = $3"
    )
    .bind(&payload.tx_hash)
    .bind("confirmed")
    .bind(&payload.booking_id.to_string())
    .execute(&**pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Mark booking as confirmed
    sqlx::query(
        "UPDATE bookings SET status = $1 WHERE id = $2"
    )
    .bind("confirmed")
    .bind(&payload.booking_id.to_string())
    .execute(&**pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(web::Json(serde_json::json!({
        "status": "confirmed",
        "message": "Payment confirmed successfully"
    })))
}
