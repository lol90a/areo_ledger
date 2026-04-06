use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateUserInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserOutput {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingInput {
    pub user_id: Uuid,
    pub flight_id: Option<Uuid>,
    pub base_price: f64,
    pub payment_method: String,
}

#[derive(Debug, Serialize)]
pub struct CreateBookingOutput {
    pub booking_id: Uuid,
    pub total_price: f64,
}

#[derive(Debug, Deserialize)]
pub struct InitPaymentInput {
    pub booking_id: Uuid,
    pub method: String,
}

#[derive(Debug, Serialize)]
pub struct InitPaymentOutput {
    pub wallet_address: String,
    pub amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmPaymentInput {
    pub booking_id: Uuid,
    pub tx_hash: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionOutput {
    pub id: String,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub asset: String,
    pub amount: f64,
    pub status: String,
    pub date: String,
    pub tx_hash: String,
}
