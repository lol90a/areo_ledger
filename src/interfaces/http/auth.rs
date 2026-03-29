use actix_web::{HttpMessage, HttpRequest};
use uuid::Uuid;

use crate::infrastructure::auth::jwt_service::Claims;
use crate::shared::errors::DomainError;

pub fn claims_from_request(req: &HttpRequest) -> Result<Claims, DomainError> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(DomainError::Unauthorized)
}

pub fn user_id_from_claims(claims: &Claims) -> Result<Uuid, DomainError> {
    Uuid::parse_str(&claims.sub).map_err(|_| DomainError::Unauthorized)
}

pub fn require_same_user_or_admin(req: &HttpRequest, target_user_id: &Uuid) -> Result<Claims, DomainError> {
    let claims = claims_from_request(req)?;
    let actor_id = user_id_from_claims(&claims)?;
    if claims.role == "admin" || actor_id == *target_user_id {
        Ok(claims)
    } else {
        Err(DomainError::Unauthorized)
    }
}

