use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::application::dto::{ConfirmPaymentInput, InitPaymentInput};
use crate::application::use_cases::confirm_payment::ConfirmPayment;
use crate::application::use_cases::init_payment::InitPayment;
use crate::infrastructure::blockchain::router_gateway::MultiChainGateway;
use crate::infrastructure::db::DbPool;
use crate::interfaces::http::error_response::into_response;

use super::access::authorize_booking_access;
use super::audit;
use super::repositories::Repositories;

#[derive(Deserialize)]
pub struct InitPaymentRequest {
    pub booking_id: Uuid,
}

#[derive(Deserialize)]
pub struct ConfirmPaymentRequest {
    pub booking_id: Uuid,
    pub tx_hash: String,
}

/// Hands the client the address and exact amount to pay against a locked quote.
pub(super) async fn init_payment(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    body: web::Json<InitPaymentRequest>,
) -> HttpResponse {
    let repos = Repositories::new(&pool);
    let actor_user_id = match authorize_booking_access(&req, &repos, &body.booking_id).await {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = InitPayment::new(repos.bookings(), repos.payments(), repos.quotes());
    match use_case
        .execute(InitPaymentInput {
            booking_id: body.booking_id,
        })
        .await
    {
        Ok(out) => {
            audit::record(
                &repos,
                &req,
                actor_user_id,
                "payment.initialized",
                "payment",
                &body.booking_id.to_string(),
                serde_json::json!({
                    "booking_id": body.booking_id,
                    "quote_id": out.quote_id,
                    "token": out.token,
                    "crypto_base_units": out.crypto_base_units,
                    "total_cents": out.total_cents,
                    "wallet_address": out.wallet_address,
                }),
            )
            .await;
            HttpResponse::Ok().json(out)
        }
        Err(e) => into_response(e),
    }
}

/// Verifies a client-supplied transaction hash on-chain and settles the booking.
///
/// The hash is the only thing the client contributes; the amount, the recipient
/// and finality are all read back from the chain by the use case.
pub(super) async fn confirm_payment(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    gateway: web::Data<MultiChainGateway>,
    body: web::Json<ConfirmPaymentRequest>,
) -> HttpResponse {
    let repos = Repositories::new(&pool);
    let actor_user_id = match authorize_booking_access(&req, &repos, &body.booking_id).await {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = ConfirmPayment::new(
        repos.payments(),
        repos.settlements(),
        gateway.get_ref().clone(),
    );
    match use_case
        .execute(ConfirmPaymentInput {
            booking_id: body.booking_id,
            tx_hash: body.tx_hash.clone(),
        })
        .await
    {
        Ok(_) => {
            audit::record(
                &repos,
                &req,
                actor_user_id,
                "payment.confirmed",
                "payment",
                &body.booking_id.to_string(),
                serde_json::json!({
                    "booking_id": body.booking_id,
                    "tx_hash": body.tx_hash,
                }),
            )
            .await;
            HttpResponse::Ok()
                .json(serde_json::json!({ "message": "Payment confirmed successfully" }))
        }
        Err(e) => into_response(e),
    }
}
