use sqlx::{Row};
use uuid::Uuid;
use crate::errors::AppError;

pub struct PaymentInitResponse {
    pub booking_id: Uuid,
    pub amount: f64,
    pub receiver_address: String,
    pub chain: String,
    pub token: String,
}

pub async fn initiate_payment(
    pool: &sqlx::PgPool,
    booking_id: Uuid,
    method: String,
) -> Result<PaymentInitResponse, AppError> {
    let row = sqlx::query("SELECT total_price FROM bookings WHERE id = ?")
        .bind(booking_id)
        .fetch_one(pool)
        .await?;

    let total_price: f64 = row.try_get("total_price")?;

    let (chain, token, receiver_address) = match method.as_str() {
        // EVM
        "usdt" | "usdc" | "eth" => (
            "evm".to_string(),
            method.clone(),
            "0xYOUR_EVM_WALLET".to_string(),
        ),

        // Solana
        "sol" => (
            "solana".to_string(),
            "sol".to_string(),
            "YOUR_SOLANA_WALLET".to_string(),
        ),

        // Bitcoin
        "btc" => (
            "btc".to_string(),
            "btc".to_string(),
            "YOUR_BTC_ADDRESS".to_string(),
        ),

        // Binance Pay
        "binance" => (
            "binance".to_string(),
            "usdt".to_string(),
            "BINANCE_INVOICE".to_string(),
        ),

        _ => ("unknown".to_string(), "".to_string(), "".to_string()),
    };

    let payment_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO payments (
            id,
            booking_id,
            chain,
            token,
            amount,
            receiver_address,
            status
        )
        VALUES (?,?,?,?,?,?,'pending')
        "#,
    )
    .bind(payment_id)
    .bind(booking_id)
    .bind(&chain)
    .bind(&token)
    .bind(total_price)
    .bind(&receiver_address)
    .execute(pool)
    .await?;

    Ok(PaymentInitResponse {
        booking_id,
        amount: total_price,
        receiver_address,
        chain,
        token,
    })
}

/// تأكيد الدفع بعد ما TX يتأكد على الشبكة
pub async fn confirm_payment(
    pool: &sqlx::PgPool,
    booking_id: Uuid,
    tx_hash: String,
) -> Result<(), AppError> {
    // تحديث حالة الدفع
    sqlx::query(
        r#"
        UPDATE payments
        SET status = 'confirmed', tx_hash = ?
        WHERE booking_id = ?
        "#,
    )
    .bind(tx_hash.clone())
    .bind(booking_id)
    .execute(pool)
    .await?;

    // تحديث حالة الحجز
    sqlx::query(
        r#"
        UPDATE bookings
        SET status = 'confirmed', tx_hash = ?
        WHERE id = ?
        "#,
    )
    .bind(tx_hash)
    .bind(booking_id)
    .execute(pool)
    .await?;

    Ok(())
}
