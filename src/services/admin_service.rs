use crate::db::DbPool;
use crate::errors::AppError;
use crate::services::{UserService, BookingService};

pub struct AdminService;

impl AdminService {
    pub fn get_stats(pool: &DbPool) -> Result<serde_json::Value, AppError> {
        //TODO: check onion architecture pattern and implement it in the code
        let users = UserService::count_users(pool)?;
        let bookings = BookingService::count_bookings(pool)?;
        let revenue = BookingService::get_total_revenue(pool)?;
        let flights = 6; // Hardcoded for now

        Ok(serde_json::json!({
            "users": users,
            "bookings": bookings,
            "revenue": revenue,
            "flights": flights
        }))
    }
}
