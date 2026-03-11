use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::booking::Booking;
use uuid::Uuid;
use chrono::NaiveDateTime;

pub struct BookingService;

impl BookingService {
    pub fn create_booking(
        pool: &DbPool,
        user_id: Uuid,
        flight_id: Uuid,
        base_price: f64,
        payment_method: &str,
    ) -> Result<(Uuid, f64), AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let booking_id = Uuid::new_v4();
        let markup = base_price * 0.10;
        let service_fee = base_price * 0.05;
        let total_price = base_price + markup + service_fee;

        conn.execute(
            "INSERT INTO bookings (id, user_id, flight_id, status, base_price, markup, service_fee, total_price, payment_method) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[&booking_id, &user_id, &flight_id, &"pending", &base_price, &markup, &service_fee, &total_price, &payment_method],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok((booking_id, total_price))
    }

    pub fn get_all_bookings(pool: &DbPool) -> Result<Vec<Booking>, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = conn.query(
            "SELECT id, user_id, flight_id, status, base_price, markup, service_fee, total_price, payment_method, created_at 
             FROM bookings ORDER BY created_at DESC",
            &[],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let bookings = rows.iter().map(|row| Booking {
            id: row.get(0),
            user_id: row.get(1),
            flight_id: row.get(2),
            status: row.get(3),
            base_price: row.get::<_, rust_decimal::Decimal>(4).to_string().parse().unwrap_or(0.0),
            markup: row.get::<_, rust_decimal::Decimal>(5).to_string().parse().unwrap_or(0.0),
            service_fee: row.get::<_, rust_decimal::Decimal>(6).to_string().parse().unwrap_or(0.0),
            total_price: row.get::<_, rust_decimal::Decimal>(7).to_string().parse().unwrap_or(0.0),
            payment_method: row.get(8),
            created_at: row.get::<_, NaiveDateTime>(9).to_string(),
        }).collect();

        Ok(bookings)
    }

    pub fn count_bookings(pool: &DbPool) -> Result<i64, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let row = conn.query_one("SELECT COUNT(*) FROM bookings", &[])
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.get(0))
    }

    pub fn get_total_revenue(pool: &DbPool) -> Result<f64, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let row = conn.query_one("SELECT COALESCE(SUM(total_price), 0) FROM bookings WHERE status = 'confirmed'", &[])
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let revenue: rust_decimal::Decimal = row.get(0);
        Ok(revenue.to_string().parse().unwrap_or(0.0))
    }
}
