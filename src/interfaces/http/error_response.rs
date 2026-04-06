use crate::shared::errors::DomainError;
use actix_web::{HttpRequest, HttpResponse};

pub fn into_response(err: DomainError) -> HttpResponse {
    match err {
        DomainError::NotFound(msg) => {
            HttpResponse::NotFound().json(serde_json::json!({ "error": msg }))
        }
        DomainError::ValidationError(msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        DomainError::Unauthorized => {
            HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }))
        }
        DomainError::Conflict(msg) => {
            HttpResponse::Conflict().json(serde_json::json!({ "error": msg }))
        }
        DomainError::InternalError(msg) => {
            log::error!("Internal error: {}", msg);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Internal server error" }))
        }
    }
}

pub fn request_metadata(req: &HttpRequest) -> (Option<String>, Option<String>) {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .map(str::to_string);
    let ua = req
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    (ip, ua)
}
