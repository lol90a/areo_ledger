use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(String),
    NotFound,
    ValidationError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::NotFound => write!(f, "Not found"),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::DatabaseError(_) => HttpResponse::InternalServerError().json("Database error"),
            AppError::NotFound => HttpResponse::NotFound().json("Not found"),
            AppError::ValidationError(msg) => HttpResponse::BadRequest().json(msg),
        }
    }
}
