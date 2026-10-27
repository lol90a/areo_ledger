use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{AcceptQuoteInput, CreateQuoteInput};
use crate::application::use_cases::accept_quote::AcceptQuote;
use crate::application::use_cases::create_quote::CreateQuote;
use crate::domain::ports::price_oracle::PriceOracle;
use crate::infrastructure::blockchain::router_gateway::MultiChainGateway;
use crate::infrastructure::db::DbPool;
use crate::interfaces::http::error_response::into_response;

use super::access::authorize_booking_access;
use super::audit;
use super::repositories::Repositories;

#[derive(Deserialize)]
pub struct CreateQuoteRequest {
    pub booking_id: Uuid,
    pub method: String,
}

/// Acceptance echoes back the exact figures the client displayed. If either has
/// moved, the server refuses and returns the new quote instead of charging an
/// amount the user never saw.
#[derive(Deserialize)]
pub struct AcceptQuoteRequest {
    pub booking_id: Uuid,
    pub quote_id: Uuid,
    pub approved_total_cents: i64,
    /// Decimal string: a u128 does not survive a JSON number.
    pub approved_crypto_base_units: String,
}

/// Issue a fresh, expiring quote for a booking.
///
/// Re-reads the authoritative catalogue price, so this is where a price change
/// between browsing and paying is detected and surfaced to the user.
pub(super) async fn create_quote(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    gateway: web::Data<MultiChainGateway>,
    oracle: web::Data<Arc<dyn PriceOracle>>,
    body: web::Json<CreateQuoteRequest>,
) -> HttpResponse {
    let repos = Repositories::new(&pool);
    let owner_id = match authorize_booking_access(&req, &repos, &body.booking_id).await {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = CreateQuote::new(
        repos.bookings(),
        repos.quotes(),
        repos.payments(),
        repos.catalog(),
        gateway.get_ref().clone(),
        oracle.get_ref().clone(),
    );

    match use_case
        .execute(CreateQuoteInput {
            booking_id: body.booking_id,
            user_id: owner_id,
            method: body.method.clone(),
        })
        .await
    {
        Ok(view) => {
            audit::record(
                &repos,
                &req,
                owner_id,
                "payment.quoted",
                "quote",
                &view.quote_id.to_string(),
                serde_json::json!({
                    "booking_id": view.booking_id,
                    "token": view.token,
                    "chain": view.chain,
                    "total_cents": view.total_cents,
                    "crypto_base_units": view.crypto_base_units,
                    "rate_source": view.rate_source,
                    "price_changed": view.price_changed,
                    "previous_total_cents": view.previous_total_cents,
                }),
            )
            .await;
            HttpResponse::Ok().json(view)
        }
        Err(e) => into_response(e),
    }
}

/// The user explicitly approves the figures they were shown. This is the moment the
/// price is locked.
pub(super) async fn accept_quote(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    body: web::Json<AcceptQuoteRequest>,
) -> HttpResponse {
    let repos = Repositories::new(&pool);
    let owner_id = match authorize_booking_access(&req, &repos, &body.booking_id).await {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = AcceptQuote::new(repos.quotes());
    match use_case
        .execute(AcceptQuoteInput {
            quote_id: body.quote_id,
            booking_id: body.booking_id,
            user_id: owner_id,
            approved_total_cents: body.approved_total_cents,
            approved_crypto_base_units: body.approved_crypto_base_units.clone(),
        })
        .await
    {
        Ok(view) => {
            audit::record(
                &repos,
                &req,
                owner_id,
                "payment.quote_accepted",
                "quote",
                &view.quote_id.to_string(),
                serde_json::json!({
                    "booking_id": view.booking_id,
                    "total_cents": view.total_cents,
                    "crypto_base_units": view.crypto_base_units,
                }),
            )
            .await;
            HttpResponse::Ok().json(view)
        }
        Err(e) => into_response(e),
    }
}
