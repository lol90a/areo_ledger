use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::payment::Payment;
use uuid::Uuid;
use chrono::NaiveDateTime;

pub struct PaymentService;

impl PaymentService {
    pub fn init_payment(
        pool: &DbPool,
        booking_id: Uuid,
        method: &str,
    ) -> Result<(String, f64), AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get booking details
        let row = conn.query_one(
            "SELECT total_price FROM bookings WHERE id = $1",
            &[&booking_id],
        ).map_err(|_| AppError::NotFound)?;

        let total_price: rust_decimal::Decimal = row.get(0);
        let total_price_f64: f64 = total_price.to_string().parse().unwrap_or(0.0);

        // Hardcoded wallet address (should be from env in production)
        let wallet_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string();

        // Create payment record
        let payment_id = Uuid::new_v4();
        let (chain, token) = Self::get_chain_and_token(method);

        conn.execute(
            "INSERT INTO payments (id, booking_id, chain, token, amount, receiver_address, status) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[&payment_id, &booking_id, &chain, &token, &total_price_f64, &wallet_address, &"pending"],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok((wallet_address, total_price_f64))
    }

    pub fn confirm_payment(
        pool: &DbPool,
        booking_id: Uuid,
        tx_hash: &str,
    ) -> Result<(), AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Update payment
        conn.execute(
            "UPDATE payments SET tx_hash = $1, status = $2 WHERE booking_id = $3",
            &[&tx_hash, &"confirmed", &booking_id],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Update booking
        conn.execute(
            "UPDATE bookings SET status = $1, tx_hash = $2 WHERE id = $3",
            &[&"confirmed", &tx_hash, &booking_id],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn get_all_payments(pool: &DbPool) -> Result<Vec<Payment>, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = conn.query(
            "SELECT id, booking_id, chain, token, amount, sender_address, receiver_address, tx_hash, status, created_at 
             FROM payments ORDER BY created_at DESC",
            &[],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let payments = rows.iter().map(|row| Payment {
            id: row.get(0),
            booking_id: row.get(1),
            chain: row.get(2),
            token: row.get(3),
            amount: row.get::<_, rust_decimal::Decimal>(4).to_string().parse().unwrap_or(0.0),
            sender_address: row.get(5),
            receiver_address: row.get(6),
            tx_hash: row.get(7),
            status: row.get(8),
            created_at: row.get::<_, NaiveDateTime>(9).to_string(),
        }).collect();

        Ok(payments)
    }

    fn get_chain_and_token(method: &str) -> (String, String) {
        match method {
            "btc" => ("Bitcoin".to_string(), "BTC".to_string()),
            "eth" => ("Ethereum".to_string(), "ETH".to_string()),
            "usdt" => ("Ethereum".to_string(), "USDT".to_string()),
            "usdc" => ("Ethereum".to_string(), "USDC".to_string()),
            "sol" => ("Solana".to_string(), "SOL".to_string()),
            "binance" => ("BSC".to_string(), "BNB".to_string()),
            _ => ("Unknown".to_string(), "Unknown".to_string()),
        }
    }
}
