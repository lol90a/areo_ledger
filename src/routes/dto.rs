use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateBookingRequest {
    pub user_id: Uuid,
    pub flight_id: Uuid,
    pub base_price: f64,
    pub payment_method: String,
}

impl CreateBookingRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.base_price <= 0.0 {
            return Err("Base price must be positive".into());
        }

        let allowed = ["usdt", "usdc", "eth", "sol", "btc", "binance"];
        if !allowed.contains(&self.payment_method.as_str()) {
            return Err("Unsupported payment method".into());
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct InitPaymentRequest {
    pub booking_id: Uuid,
    pub method: String,
}

impl InitPaymentRequest {
    pub fn validate(&self) -> Result<(), String> {
        let allowed = ["usdt", "usdc", "eth", "sol", "btc", "binance"];
        if !allowed.contains(&self.method.as_str()) {
            return Err("Unsupported payment method".into());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct ConfirmPaymentRequest {
    pub booking_id: Uuid,
    pub tx_hash: String,
}

impl ConfirmPaymentRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.tx_hash.len() < 10 {
            return Err("Invalid tx hash".into());
        }
        Ok(())
    }
}
