use actix_web::HttpRequest;
use uuid::Uuid;

use crate::domain::repositories::booking_repository::BookingRepository;
use crate::interfaces::http::auth::{claims_from_request, user_id_from_claims};
use crate::shared::errors::DomainError;

use super::repositories::Repositories;

/// Authorizes the caller against a booking and returns the id of the user the
/// booking belongs to.
///
/// Every payment endpoint is scoped to one booking, so this is the single gate
/// in front of all of them. It returns the *owner's* id rather than the caller's
/// because an admin acting on someone else's booking must still have the use
/// case operate on the owner -- returning the caller's id there would quietly
/// re-attribute the payment.
pub(super) async fn authorize_booking_access(
    req: &HttpRequest,
    repos: &Repositories,
    booking_id: &Uuid,
) -> Result<Uuid, DomainError> {
    let claims = claims_from_request(req)?;
    let booking = repos.bookings().find_by_id(booking_id).await?;
    if claims.role == "admin" {
        return Ok(booking.user_id);
    }

    let actor_id = user_id_from_claims(&claims)?;
    if booking.user_id != actor_id {
        return Err(DomainError::Unauthorized);
    }
    Ok(actor_id)
}
