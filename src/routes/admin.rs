use actix_web::{web, HttpResponse, Result};
use crate::db::DbPool;
use crate::errors::AppError;
use crate::services::{AdminService, BookingService, PaymentService};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/stats", web::get().to(get_stats))
            .route("/bookings", web::get().to(get_bookings))
            .route("/payments", web::get().to(get_payments))
    );
}

async fn get_stats(pool: web::Data<DbPool>) -> Result<HttpResponse, AppError> {
    let stats = AdminService::get_stats(&pool)?;
    Ok(HttpResponse::Ok().json(stats))
}

async fn get_bookings(pool: web::Data<DbPool>) -> Result<HttpResponse, AppError> {
    let bookings = BookingService::get_all_bookings(&pool)?;
    Ok(HttpResponse::Ok().json(bookings))
}

async fn get_payments(pool: web::Data<DbPool>) -> Result<HttpResponse, AppError> {
    let payments = PaymentService::get_all_payments(&pool)?;
    Ok(HttpResponse::Ok().json(payments))
}
