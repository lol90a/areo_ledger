use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Flight {
    pub id: Uuid,
    pub provider: String,
    pub from_airport: String,
    pub to_airport: String,
    pub departure_time: NaiveDateTime,
    pub arrival_time: NaiveDateTime,
    pub base_price: f64,
    pub currency: String,
}
