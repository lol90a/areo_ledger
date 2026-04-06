use actix_web::HttpResponse;
use serde::Serialize;
use uuid::Uuid;

use crate::shared::errors::DomainError;

/// Every API response is wrapped in this envelope.
/// Clients always get a consistent shape regardless of success or failure.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub request_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            request_id: Uuid::new_v4(),
        }
    }
}

impl ApiResponse<()> {
    pub fn err(code: &'static str, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError { code, message }),
            request_id: Uuid::new_v4(),
        }
    }
}

/// Maps DomainError → HTTP status + standardized error body.
/// This is the ONLY place in the codebase that knows both DomainError and HTTP.
pub fn domain_error_to_response(err: DomainError) -> HttpResponse {
    match err {
        DomainError::NotFound(msg) => {
            HttpResponse::NotFound().json(ApiResponse::<()>::err("NOT_FOUND", msg))
        }

        DomainError::ValidationError(msg) => {
            HttpResponse::BadRequest().json(ApiResponse::<()>::err("VALIDATION_ERROR", msg))
        }

        DomainError::Unauthorized => HttpResponse::Unauthorized().json(ApiResponse::<()>::err(
            "UNAUTHORIZED",
            "Authentication required".to_string(),
        )),

        DomainError::Conflict(msg) => {
            HttpResponse::Conflict().json(ApiResponse::<()>::err("CONFLICT", msg))
        }

        DomainError::InternalError(msg) => {
            // Never leak internal details to the client
            tracing::error!(internal_message = %msg, "Internal server error");
            HttpResponse::InternalServerError().json(ApiResponse::<()>::err(
                "INTERNAL_ERROR",
                "An unexpected error occurred".to_string(),
            ))
        }
    }
}

/// Convenience macro for handler return sites.
#[macro_export]
macro_rules! ok_response {
    ($data:expr) => {
        HttpResponse::Ok().json(ApiResponse::ok($data))
    };
}
