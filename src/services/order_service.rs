use crate::db::DbPool;
use crate::errors::AppError;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone)]
pub struct OrderItem {
    pub product_id: i32,
    pub name: String,
    pub price: f64,
    pub quantity: i32,
}

#[derive(Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub total: f64,
    pub service_fee: f64,
    pub grand_total: f64,
    pub status: String,
    pub wallet_address: String,
    pub crypto_amount: f64,
}

pub struct OrderService;

impl OrderService {
    pub fn create_order(
        pool: &DbPool,
        user_id: &str,
        items: Vec<OrderItem>,
        total: f64,
        crypto_method: &str,
        sender_wallet: &str,
        tx_hash: Option<&str>,
    ) -> Result<OrderResponse, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let order_id = Uuid::new_v4();
        let service_fee = total * 0.05;
        let grand_total = total + service_fee;
        
        let crypto_prices = Self::get_crypto_prices();
        let crypto_amount = grand_total / crypto_prices.get(crypto_method).unwrap_or(&1.0);
        
        let wallet_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
        let status = if tx_hash.is_some() { "confirmed" } else { "pending" };

        conn.execute(
            "INSERT INTO orders (id, user_id, total, service_fee, grand_total, crypto_method, sender_wallet, tx_hash, wallet_address, crypto_amount, status) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            &[&order_id, &user_id, &total, &service_fee, &grand_total, &crypto_method, &sender_wallet, &tx_hash, &wallet_address, &crypto_amount, &status],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        for item in &items {
            conn.execute(
                "INSERT INTO order_items (order_id, product_id, name, price, quantity) VALUES ($1, $2, $3, $4, $5)",
                &[&order_id, &item.product_id, &item.name, &item.price, &item.quantity],
            ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        Ok(OrderResponse {
            order_id: order_id.to_string(),
            total,
            service_fee,
            grand_total,
            status: status.to_string(),
            wallet_address: wallet_address.to_string(),
            crypto_amount,
        })
    }

    pub fn update_order(
        pool: &DbPool,
        order_id: &str,
        sender_wallet: &str,
        tx_hash: &str,
    ) -> Result<(), AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let order_uuid = Uuid::parse_str(order_id)
            .map_err(|_| AppError::ValidationError("Invalid order ID".to_string()))?;

        conn.execute(
            "UPDATE orders SET sender_wallet = $1, tx_hash = $2, status = 'confirmed' WHERE id = $3",
            &[&sender_wallet, &tx_hash, &order_uuid],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn get_order_by_id(pool: &DbPool, order_id: &str) -> Result<serde_json::Value, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let order_uuid = Uuid::parse_str(order_id)
            .map_err(|_| AppError::ValidationError("Invalid order ID".to_string()))?;

        let row = conn.query_one(
            "SELECT id, user_id, total, service_fee, grand_total, crypto_method, status, created_at FROM orders WHERE id = $1",
            &[&order_uuid],
        ).map_err(|_| AppError::NotFound)?;

        Ok(serde_json::json!({
            "id": row.get::<_, Uuid>(0).to_string(),
            "user_id": row.get::<_, String>(1),
            "total": row.get::<_, rust_decimal::Decimal>(2).to_string().parse::<f64>().unwrap_or(0.0),
            "service_fee": row.get::<_, rust_decimal::Decimal>(3).to_string().parse::<f64>().unwrap_or(0.0),
            "grand_total": row.get::<_, rust_decimal::Decimal>(4).to_string().parse::<f64>().unwrap_or(0.0),
            "crypto_method": row.get::<_, String>(5),
            "status": row.get::<_, String>(6),
            "created_at": row.get::<_, chrono::NaiveDateTime>(7).to_string(),
        }))
    }

    pub fn get_user_orders(pool: &DbPool, user_id: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = conn.query(
            "SELECT id, grand_total, crypto_amount, crypto_method, status FROM orders WHERE user_id = $1 ORDER BY created_at DESC",
            &[&user_id],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let orders = rows.iter().map(|row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0).to_string(),
                "grand_total": row.get::<_, rust_decimal::Decimal>(1).to_string().parse::<f64>().unwrap_or(0.0),
                "crypto_amount": row.get::<_, rust_decimal::Decimal>(2).to_string().parse::<f64>().unwrap_or(0.0),
                "crypto_method": row.get::<_, String>(3),
                "status": row.get::<_, String>(4),
            })
        }).collect();

        Ok(orders)
    }

    fn get_crypto_prices() -> HashMap<&'static str, f64> {
        let mut prices = HashMap::new();
        prices.insert("BTC", 45000.0);
        prices.insert("ETH", 2500.0);
        prices.insert("USDT", 1.0);
        prices.insert("USDC", 1.0);
        prices.insert("SOL", 100.0);
        prices.insert("BNB", 320.0);
        prices
    }
}
