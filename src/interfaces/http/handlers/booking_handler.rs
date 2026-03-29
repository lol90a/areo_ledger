use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::dto::CreateBookingInput;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::application::use_cases::create_booking::CreateBooking;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::persistence::postgres::pg_audit_log_repository::PgAuditLogRepository;
use crate::infrastructure::persistence::postgres::pg_booking_repository::PgBookingRepository;
use crate::interfaces::http::auth::{claims_from_request, user_id_from_claims};
use crate::interfaces::http::error_response::{into_response, request_metadata};
use crate::shared::errors::DomainError;

#[derive(Deserialize)]
pub struct CreateBookingRequest {
    pub user_id: Uuid,
    pub flight_id: Uuid,
    pub base_price: f64,
    pub payment_method: String,
}

#[derive(Serialize)]
pub struct BookingSummaryResponse {
    pub booking_id: String,
    pub flight_id: String,
    pub status: String,
    pub base_price: f64,
    pub total_price: f64,
    pub payment_method: String,
    pub created_at: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/bookings")
            .route("", web::post().to(create))
            .route("/", web::post().to(create))
            .route("/latest", web::get().to(latest_for_current_user)),
    );
}

async fn create(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    body: web::Json<CreateBookingRequest>,
) -> HttpResponse {
    let claims = match claims_from_request(&req) {
        Ok(c) => c,
        Err(e) => return into_response(e),
    };
    let actor_id = match user_id_from_claims(&claims) {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    if claims.role != "admin" && body.user_id != actor_id {
        return into_response(crate::shared::errors::DomainError::Unauthorized);
    }

    let effective_user_id = if claims.role == "admin" { body.user_id } else { actor_id };
    let use_case = CreateBooking::new(PgBookingRepository::new(pool.get_ref().clone()));
    match use_case.execute(CreateBookingInput {
        user_id: effective_user_id,
        flight_id: body.flight_id,
        base_price: body.base_price,
        payment_method: body.payment_method.clone(),
    }).await {
        Ok(out) => {
            let (ip, ua) = request_metadata(&req);
            let _ = PgAuditLogRepository::new(pool.get_ref().clone()).record(
                Some(effective_user_id),
                "booking.created",
                "booking",
                &out.booking_id.to_string(),
                serde_json::json!({
                    "flight_id": body.flight_id,
                    "payment_method": body.payment_method,
                    "total_price": out.total_price,
                }),
                ip,
                ua,
            ).await;
            HttpResponse::Ok().json(out)
        }
        Err(e) => into_response(e),
    }
}

async fn latest_for_current_user(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> HttpResponse {
    let claims = match claims_from_request(&req) {
        Ok(c) => c,
        Err(e) => return into_response(e),
    };
    let actor_id = match user_id_from_claims(&claims) {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let repo = PgBookingRepository::new(pool.get_ref().clone());
    match repo.find_by_user_id(&actor_id).await {
        Ok(bookings) => {
            let maybe_booking = bookings
                .into_iter()
                .find(|booking| booking.status.as_str() == "pending");

            match maybe_booking {
                Some(booking) => HttpResponse::Ok().json(BookingSummaryResponse {
                    booking_id: booking.id.to_string(),
                    flight_id: booking.flight_id.to_string(),
                    status: booking.status.as_str().to_string(),
                    base_price: booking.pricing.base.cents() as f64 / 100.0,
                    total_price: booking.pricing.total.cents() as f64 / 100.0,
                    payment_method: booking.payment_method,
                    created_at: booking.created_at.to_rfc3339(),
                }),
                None => into_response(DomainError::NotFound("No pending booking found".to_string())),
            }
        }
        Err(e) => into_response(e),
    }
}

