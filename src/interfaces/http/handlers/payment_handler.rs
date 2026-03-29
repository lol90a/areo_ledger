use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::application::dto::{ConfirmPaymentInput, InitPaymentInput};
use crate::application::use_cases::confirm_payment::ConfirmPayment;
use crate::application::use_cases::init_payment::InitPayment;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::infrastructure::blockchain::router_gateway::MultiChainGateway;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::persistence::postgres::pg_audit_log_repository::PgAuditLogRepository;
use crate::infrastructure::persistence::postgres::pg_booking_repository::PgBookingRepository;
use crate::infrastructure::persistence::postgres::pg_payment_repository::PgPaymentRepository;
use crate::infrastructure::persistence::postgres::pg_transaction_repository::PgTransactionRepository;
use crate::interfaces::http::auth::{claims_from_request, user_id_from_claims};
use crate::interfaces::http::error_response::{into_response, request_metadata};
use crate::shared::errors::DomainError;

#[derive(Deserialize)]
pub struct InitPaymentRequest {
    pub booking_id: Uuid,
    pub method: String,
}

#[derive(Deserialize)]
pub struct ConfirmPaymentRequest {
    pub booking_id: Uuid,
    pub tx_hash: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payments")
            .route("/init", web::post().to(init_payment))
            .route("/confirm", web::post().to(confirm_payment)),
    );
}

async fn authorize_booking_access(
    req: &HttpRequest,
    repo: &PgBookingRepository,
    booking_id: &Uuid,
) -> Result<Uuid, DomainError> {
    let claims = claims_from_request(req)?;
    let booking = repo.find_by_id(booking_id).await?;
    if claims.role == "admin" {
        return Ok(booking.user_id);
    }

    let actor_id = user_id_from_claims(&claims)?;
    if booking.user_id != actor_id {
        return Err(DomainError::Unauthorized);
    }
    Ok(actor_id)
}

async fn init_payment(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    gateway: web::Data<MultiChainGateway>,
    body: web::Json<InitPaymentRequest>,
) -> HttpResponse {
    let booking_repo = PgBookingRepository::new(pool.get_ref().clone());
    let actor_user_id = match authorize_booking_access(&req, &booking_repo, &body.booking_id).await {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = InitPayment::new(
        booking_repo,
        PgPaymentRepository::new(pool.get_ref().clone()),
        gateway.get_ref().clone(),
    );
    match use_case.execute(InitPaymentInput {
        booking_id: body.booking_id,
        method: body.method.clone(),
    }).await {
        Ok(out) => {
            let (ip, ua) = request_metadata(&req);
            let _ = PgAuditLogRepository::new(pool.get_ref().clone()).record(
                Some(actor_user_id),
                "payment.initialized",
                "payment",
                &body.booking_id.to_string(),
                serde_json::json!({
                    "booking_id": body.booking_id,
                    "method": body.method,
                    "amount": out.amount,
                    "wallet_address": out.wallet_address,
                }),
                ip,
                ua,
            ).await;
            HttpResponse::Ok().json(serde_json::json!({
                "wallet_address": out.wallet_address,
                "amount": out.amount,
            }))
        }
        Err(e) => into_response(e),
    }
}

async fn confirm_payment(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    gateway: web::Data<MultiChainGateway>,
    body: web::Json<ConfirmPaymentRequest>,
) -> HttpResponse {
    let booking_repo = PgBookingRepository::new(pool.get_ref().clone());
    let actor_user_id = match authorize_booking_access(&req, &booking_repo, &body.booking_id).await {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = ConfirmPayment::new(
        booking_repo,
        PgPaymentRepository::new(pool.get_ref().clone()),
        PgTransactionRepository::new(pool.get_ref().clone()),
        gateway.get_ref().clone(),
    );
    match use_case.execute(ConfirmPaymentInput {
        booking_id: body.booking_id,
        tx_hash: body.tx_hash.clone(),
    }).await {
        Ok(_) => {
            let (ip, ua) = request_metadata(&req);
            let _ = PgAuditLogRepository::new(pool.get_ref().clone()).record(
                Some(actor_user_id),
                "payment.confirmed",
                "payment",
                &body.booking_id.to_string(),
                serde_json::json!({
                    "booking_id": body.booking_id,
                    "tx_hash": body.tx_hash,
                }),
                ip,
                ua,
            ).await;
            HttpResponse::Ok().json(serde_json::json!({ "message": "Payment confirmed successfully" }))
        }
        Err(e) => into_response(e),
    }
}
