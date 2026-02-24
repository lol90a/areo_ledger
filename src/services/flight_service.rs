use sqlx::PgPool;
use uuid::Uuid;
use crate::models::flight::Flight;

pub async fn search_flights(
    _pool: &PgPool,
    from: String,
    to: String,
    _date: String,
) -> Vec<Flight> {
    // TODO: integrate real flight API later
    vec![
        Flight {
            id: Uuid::new_v4(),
            provider: "MockAir".to_string(),
            from_airport: from,
            to_airport: to,
            departure_time: chrono::Utc::now().naive_utc(),
            arrival_time: chrono::Utc::now().naive_utc(),
            base_price: 5000.0,
            currency: "USDT".to_string(),
        },
    ]
}
