use serde::{Serialize, Deserialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub chain: String,
    pub token: String,
    pub amount: f64,
    pub sender_address: Option<String>,
    pub receiver_address: String,
    pub tx_hash: Option<String>,
    pub status: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct InitPaymentRequest {
    pub booking_id: Uuid,
    #[validate(length(min = 1))]
    pub method: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmPaymentRequest {
    pub booking_id: Uuid,
    #[validate(length(min = 10))]
    pub tx_hash: String,
}
