use actix_multipart::Multipart;
use actix_web::{mime, web, HttpRequest, HttpResponse};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::application::dto::{
    AcceptQuoteInput, ConfirmPaymentInput, CreateQuoteInput, InitPaymentInput,
};
use crate::application::use_cases::accept_quote::AcceptQuote;
use crate::application::use_cases::confirm_payment::ConfirmPayment;
use crate::application::use_cases::create_quote::CreateQuote;
use crate::application::use_cases::init_payment::InitPayment;
use crate::domain::entities::payment::PaymentStatus;
use crate::domain::ports::price_oracle::PriceOracle;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::value_objects::{resolve_chain_token, Chain};
use crate::infrastructure::blockchain::router_gateway::MultiChainGateway;
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::persistence::postgres::pg_audit_log_repository::PgAuditLogRepository;
use crate::infrastructure::persistence::postgres::pg_booking_repository::PgBookingRepository;
use crate::infrastructure::persistence::postgres::pg_catalog_repository::PgCatalogRepository;
use crate::infrastructure::persistence::postgres::pg_payment_repository::PgPaymentRepository;
use crate::infrastructure::persistence::postgres::pg_quote_repository::PgQuoteRepository;
use crate::infrastructure::persistence::postgres::pg_settlement_repository::PgSettlementRepository;
use crate::interfaces::http::auth::{claims_from_request, user_id_from_claims};
use crate::interfaces::http::error_response::{into_response, request_metadata};
use crate::shared::errors::DomainError;

const MAX_PAYMENT_PROOF_BYTES: usize = 5 * 1024 * 1024;

#[derive(Deserialize)]
pub struct InitPaymentRequest {
    pub booking_id: Uuid,
}

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

#[derive(Deserialize)]
pub struct ConfirmPaymentRequest {
    pub booking_id: Uuid,
    pub tx_hash: String,
}

/// A payment route the client may select.
///
/// `chain_id` and `token_contract` come from server configuration rather than
/// being hard-coded in the browser, so the wallet connects to the chain and token
/// the backend will actually verify against. A mismatch here is how funds get sent
/// to the right address on the wrong network.
#[derive(Serialize)]
struct PaymentOptionResponse {
    method: String,
    token: String,
    chain: String,
    display_name: String,
    wallet_address: Option<String>,
    available: bool,
    /// Wallet family the client should use: "evm", "solana" or "manual".
    wallet_kind: &'static str,
    /// EIP-155 chain id for EVM routes.
    chain_id: Option<u64>,
    /// ERC-20 contract address when the token is not the chain's native asset.
    token_contract: Option<String>,
    /// Decimals of the token's smallest unit.
    decimals: u8,
}

#[derive(Serialize)]
struct PaymentProofUploadResponse {
    message: String,
    booking_id: String,
    proof_path: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payments")
            .route("/options", web::get().to(payment_options))
            .route("/quote", web::post().to(create_quote))
            .route("/quote/accept", web::post().to(accept_quote))
            .route("/init", web::post().to(init_payment))
            .route("/confirm", web::post().to(confirm_payment))
            .route("/proof/{booking_id}", web::post().to(upload_payment_proof)),
    );
}

fn build_payment_options(config: &AppConfig) -> Vec<PaymentOptionResponse> {
    [
        ("btc", "Bitcoin (BTC)", config.wallet_btc.as_str()),
        ("eth", "Ethereum (ETH)", config.wallet_eth.as_str()),
        ("usdt", "Tether USD (USDT)", config.wallet_usdt.as_str()),
        ("usdc", "USD Coin (USDC)", config.wallet_usdc.as_str()),
        ("bsc", "BNB on BSC", config.wallet_bnb.as_str()),
        ("sol", "Solana (SOL)", config.wallet_sol.as_str()),
    ]
    .into_iter()
    .filter_map(|(method, display_name, wallet_address)| {
        // Chain and token come from the single source of truth in the domain layer,
        // so the API can never advertise a route the verifier does not understand.
        let (chain, token) = resolve_chain_token(method)?;
        let chain_id = match chain {
            Chain::Ethereum => Some(config.ethereum_chain_id),
            Chain::Bsc => Some(config.bsc_chain_id),
            _ => None,
        };
        let wallet_kind = match chain {
            Chain::Ethereum | Chain::Bsc => "evm",
            Chain::Solana => "solana",
            Chain::Bitcoin => "manual",
        };
        Some(PaymentOptionResponse {
            method: method.to_string(),
            token: token.as_str().to_string(),
            chain: chain.as_str().to_string(),
            display_name: display_name.to_string(),
            wallet_address: (!wallet_address.trim().is_empty()).then(|| wallet_address.to_string()),
            available: !wallet_address.trim().is_empty(),
            wallet_kind,
            chain_id,
            token_contract: token.erc20_contract().map(|s| s.to_string()),
            decimals: token.decimals(),
        })
    })
    .collect()
}

async fn payment_options(config: web::Data<AppConfig>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "options": build_payment_options(config.get_ref())
    }))
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

fn proof_directory(booking_id: &Uuid) -> Result<PathBuf, DomainError> {
    let mut dir = std::env::current_dir().map_err(|e| {
        DomainError::InternalError(format!("Failed to resolve working directory: {}", e))
    })?;
    dir.push("uploads");
    dir.push("payment_proofs");
    dir.push(booking_id.to_string());
    Ok(dir)
}

fn image_extension(content_type: Option<&mime::Mime>, filename: Option<&str>) -> Option<String> {
    if let Some(name) = filename {
        if let Some(ext) = Path::new(name).extension().and_then(|value| value.to_str()) {
            let ext = ext.to_ascii_lowercase();
            if matches!(
                ext.as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp"
            ) {
                return Some(ext);
            }
        }
    }

    match content_type.map(|value| value.essence_str()) {
        Some("image/png") => Some("png".to_string()),
        Some("image/jpeg") => Some("jpg".to_string()),
        Some("image/webp") => Some("webp".to_string()),
        Some("image/gif") => Some("gif".to_string()),
        Some("image/bmp") => Some("bmp".to_string()),
        _ => None,
    }
}

/// Issue a fresh, expiring quote for a booking.
///
/// Re-reads the authoritative catalogue price, so this is where a price change
/// between browsing and paying is detected and surfaced to the user.
async fn create_quote(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    gateway: web::Data<MultiChainGateway>,
    oracle: web::Data<Arc<dyn PriceOracle>>,
    body: web::Json<CreateQuoteRequest>,
) -> HttpResponse {
    let booking_repo = PgBookingRepository::new(pool.get_ref().clone());
    let owner_id = match authorize_booking_access(&req, &booking_repo, &body.booking_id).await {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = CreateQuote::new(
        booking_repo,
        PgQuoteRepository::new(pool.get_ref().clone()),
        PgPaymentRepository::new(pool.get_ref().clone()),
        PgCatalogRepository::new(pool.get_ref().clone()),
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
            let (ip, ua) = request_metadata(&req);
            let _ = PgAuditLogRepository::new(pool.get_ref().clone())
                .record(
                    Some(owner_id),
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
                    ip,
                    ua,
                )
                .await;
            HttpResponse::Ok().json(view)
        }
        Err(e) => into_response(e),
    }
}

/// The user explicitly approves the figures they were shown. This is the moment the
/// price is locked.
async fn accept_quote(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    body: web::Json<AcceptQuoteRequest>,
) -> HttpResponse {
    let booking_repo = PgBookingRepository::new(pool.get_ref().clone());
    let owner_id = match authorize_booking_access(&req, &booking_repo, &body.booking_id).await {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = AcceptQuote::new(PgQuoteRepository::new(pool.get_ref().clone()));
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
            let (ip, ua) = request_metadata(&req);
            let _ = PgAuditLogRepository::new(pool.get_ref().clone())
                .record(
                    Some(owner_id),
                    "payment.quote_accepted",
                    "quote",
                    &view.quote_id.to_string(),
                    serde_json::json!({
                        "booking_id": view.booking_id,
                        "total_cents": view.total_cents,
                        "crypto_base_units": view.crypto_base_units,
                    }),
                    ip,
                    ua,
                )
                .await;
            HttpResponse::Ok().json(view)
        }
        Err(e) => into_response(e),
    }
}

async fn init_payment(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    body: web::Json<InitPaymentRequest>,
) -> HttpResponse {
    let booking_repo = PgBookingRepository::new(pool.get_ref().clone());
    let actor_user_id = match authorize_booking_access(&req, &booking_repo, &body.booking_id).await
    {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = InitPayment::new(
        booking_repo,
        PgPaymentRepository::new(pool.get_ref().clone()),
        PgQuoteRepository::new(pool.get_ref().clone()),
    );
    match use_case
        .execute(InitPaymentInput {
            booking_id: body.booking_id,
        })
        .await
    {
        Ok(out) => {
            let (ip, ua) = request_metadata(&req);
            let _ = PgAuditLogRepository::new(pool.get_ref().clone())
                .record(
                    Some(actor_user_id),
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
                    ip,
                    ua,
                )
                .await;
            HttpResponse::Ok().json(out)
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
    let actor_user_id = match authorize_booking_access(&req, &booking_repo, &body.booking_id).await
    {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = ConfirmPayment::new(
        PgPaymentRepository::new(pool.get_ref().clone()),
        PgSettlementRepository::new(pool.get_ref().clone()),
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
            let (ip, ua) = request_metadata(&req);
            let _ = PgAuditLogRepository::new(pool.get_ref().clone())
                .record(
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
                )
                .await;
            HttpResponse::Ok()
                .json(serde_json::json!({ "message": "Payment confirmed successfully" }))
        }
        Err(e) => into_response(e),
    }
}

async fn upload_payment_proof(
    req: HttpRequest,
    path: web::Path<Uuid>,
    pool: web::Data<DbPool>,
    mut payload: Multipart,
) -> HttpResponse {
    let booking_id = path.into_inner();
    let booking_repo = PgBookingRepository::new(pool.get_ref().clone());
    let actor_user_id = match authorize_booking_access(&req, &booking_repo, &booking_id).await {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let payment_repo = PgPaymentRepository::new(pool.get_ref().clone());
    match payment_repo.find_by_booking_id(&booking_id).await {
        Ok(payment) => {
            if payment.status != PaymentStatus::Pending {
                return into_response(DomainError::ValidationError(
                    "This payment is already settled; proof can no longer be changed".to_string(),
                ));
            }
        }
        Err(e) => return into_response(e),
    }

    let storage_dir = match proof_directory(&booking_id) {
        Ok(dir) => dir,
        Err(e) => return into_response(e),
    };

    if let Err(e) = tokio::fs::create_dir_all(&storage_dir).await {
        return into_response(DomainError::InternalError(format!(
            "Failed to create proof storage directory: {}",
            e
        )));
    }

    while let Ok(Some(mut field)) = payload.try_next().await {
        if field.name() != Some("proof") {
            continue;
        }

        let content_type = field.content_type().cloned();
        if !matches!(
            content_type.as_ref().map(|value| value.type_().as_str()),
            Some("image")
        ) {
            return into_response(DomainError::ValidationError(
                "Payment proof must be an image file".to_string(),
            ));
        }

        let original_name = field
            .content_disposition()
            .and_then(|value| value.get_filename())
            .map(str::to_string);
        let extension = match image_extension(content_type.as_ref(), original_name.as_deref()) {
            Some(ext) => ext,
            None => {
                return into_response(DomainError::ValidationError(
                    "Unsupported image type. Use PNG, JPG, WEBP, GIF, or BMP".to_string(),
                ))
            }
        };

        let filename = format!("{}-proof.{}", Uuid::new_v4(), extension);
        let absolute_path = storage_dir.join(&filename);
        let relative_path = Path::new("uploads")
            .join("payment_proofs")
            .join(booking_id.to_string())
            .join(&filename);

        let mut file = match tokio::fs::File::create(&absolute_path).await {
            Ok(file) => file,
            Err(e) => {
                return into_response(DomainError::InternalError(format!(
                    "Failed to create payment proof file: {}",
                    e
                )))
            }
        };

        let mut size = 0usize;
        while let Ok(Some(chunk)) = field.try_next().await {
            size += chunk.len();
            if size > MAX_PAYMENT_PROOF_BYTES {
                let _ = tokio::fs::remove_file(&absolute_path).await;
                return into_response(DomainError::ValidationError(
                    "Payment proof image must be 5 MB or smaller".to_string(),
                ));
            }

            if let Err(e) = file.write_all(&chunk).await {
                let _ = tokio::fs::remove_file(&absolute_path).await;
                return into_response(DomainError::InternalError(format!(
                    "Failed to write payment proof file: {}",
                    e
                )));
            }
        }

        let relative_path_string = relative_path.to_string_lossy().replace('\\', "/");
        let content_type_string = content_type.as_ref().map(|value| value.to_string());
        if let Err(e) = payment_repo
            .attach_proof(
                &booking_id,
                &relative_path_string,
                content_type_string.as_deref(),
            )
            .await
        {
            let _ = tokio::fs::remove_file(&absolute_path).await;
            return into_response(e);
        }

        let (ip, ua) = request_metadata(&req);
        let _ = PgAuditLogRepository::new(pool.get_ref().clone())
            .record(
                Some(actor_user_id),
                "payment.proof_uploaded",
                "payment",
                &booking_id.to_string(),
                serde_json::json!({
                    "booking_id": booking_id,
                    "proof_path": relative_path_string,
                    "content_type": content_type_string,
                }),
                ip,
                ua,
            )
            .await;

        return HttpResponse::Ok().json(PaymentProofUploadResponse {
            message: "Payment proof uploaded successfully".to_string(),
            booking_id: booking_id.to_string(),
            proof_path: relative_path_string,
        });
    }

    into_response(DomainError::ValidationError(
        "Attach an image in the 'proof' field before uploading".to_string(),
    ))
}
