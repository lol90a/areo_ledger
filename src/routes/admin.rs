use actix_web::{web, Result};
use serde::Serialize;

#[derive(Serialize)]
struct AdminStats {
    bookings: i64,
    revenue: f64,
    users: i64,
    flights: i64,
}

#[derive(Serialize)]
struct BookingRecord {
    id: String,
    status: String,
    total_price: f64,
}

#[derive(Serialize)]
struct PaymentRecord {
    id: String,
    chain: String,
    token: String,
    amount: f64,
    status: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/stats", web::get().to(get_stats))
            .route("/bookings", web::get().to(get_bookings))
            .route("/payments", web::get().to(get_payments))
    );
}

async fn get_stats(pool: web::Data<sqlx::PgPool>) -> Result<web::Json<AdminStats>> {
    let bookings: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bookings")
        .fetch_one(&**pool)
        .await
        .unwrap_or((0,));

    let revenue: (f64,) = sqlx::query_as("SELECT COALESCE(SUM(total_price), 0.0) FROM bookings WHERE status = 'confirmed'")
        .fetch_one(&**pool)
        .await
        .unwrap_or((0.0,));

    let users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&**pool)
        .await
        .unwrap_or((0,));

    Ok(web::Json(AdminStats {
        bookings: bookings.0,
        revenue: revenue.0,
        users: users.0,
        flights: 45,
    }))
}

async fn get_bookings(pool: web::Data<sqlx::PgPool>) -> Result<web::Json<Vec<BookingRecord>>> {
    let bookings: Vec<(String, String, f64)> = sqlx::query_as(
        "SELECT id, status, total_price FROM bookings LIMIT 10"
    )
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    let records: Vec<BookingRecord> = bookings.into_iter()
        .map(|(id, status, total_price)| BookingRecord { id, status, total_price })
        .collect();

    Ok(web::Json(records))
}

async fn get_payments(pool: web::Data<sqlx::PgPool>) -> Result<web::Json<Vec<PaymentRecord>>> {
    let payments: Vec<(String, String, String, f64, String)> = sqlx::query_as(
        "SELECT id, chain, token, amount, status FROM payments LIMIT 10"
    )
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    let records: Vec<PaymentRecord> = payments.into_iter()
        .map(|(id, chain, token, amount, status)| PaymentRecord { id, chain, token, amount, status })
        .collect();

    Ok(web::Json(records))
}
