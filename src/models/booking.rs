use serde::{Serialize, Deserialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct Booking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub flight_id: Uuid,
    pub status: String,
    pub base_price: f64,
    pub markup: f64,
    pub service_fee: f64,
    pub total_price: f64,
    pub payment_method: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBookingRequest {
    pub user_id: Uuid,
    pub flight_id: Uuid,
    pub base_price: f64,
    #[validate(length(min = 1))]
    pub payment_method: String,
}

impl From<CreateBookingRequest> for crate::services::booking_service::BookingInput {
    fn from(req: CreateBookingRequest) -> Self {
        Self {
            user_id: req.user_id,
            flight_id: req.flight_id,
            base_price: req.base_price,
            payment_method: req.payment_method,
        }
    }
}
