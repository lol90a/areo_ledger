use actix_multipart::Multipart;
use actix_web::{mime, web, HttpRequest, HttpResponse};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::application::dto::{ConfirmPaymentInput, InitPaymentInput};
use crate::application::use_cases::confirm_payment::ConfirmPayment;
use crate::application::use_cases::init_payment::InitPayment;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::infrastructure::blockchain::router_gateway::MultiChainGateway;
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::persistence::postgres::pg_audit_log_repository::PgAuditLogRepository;
use crate::infrastructure::persistence::postgres::pg_booking_repository::PgBookingRepository;
use crate::infrastructure::persistence::postgres::pg_payment_repository::PgPaymentRepository;
use crate::infrastructure::persistence::postgres::pg_transaction_repository::PgTransactionRepository;
use crate::interfaces::http::auth::{claims_from_request, user_id_from_claims};
use crate::interfaces::http::error_response::{into_response, request_metadata};
use crate::shared::errors::DomainError;

const MAX_PAYMENT_PROOF_BYTES: usize = 5 * 1024 * 1024;

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

#[derive(Serialize)]
struct PaymentOptionResponse {
    method: String,
    token: String,
    chain: String,
    display_name: String,
    wallet_address: Option<String>,
    available: bool,
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
            .route("/init", web::post().to(init_payment))
            .route("/confirm", web::post().to(confirm_payment))
            .route("/proof/{booking_id}", web::post().to(upload_payment_proof)),
    );
}

fn build_payment_options(config: &AppConfig) -> Vec<PaymentOptionResponse> {
    [
        (
            "btc",
            "BTC",
            "Bitcoin",
            "Bitcoin (BTC)",
            config.wallet_btc.as_str(),
        ),
        (
            "eth",
            "ETH",
            "Ethereum",
            "Ethereum (ETH)",
            config.wallet_eth.as_str(),
        ),
        (
            "usdt",
            "USDT",
            "Ethereum",
            "Tether USD (USDT)",
            config.wallet_usdt.as_str(),
        ),
        (
            "usdc",
            "USDC",
            "Ethereum",
            "USD Coin (USDC)",
            config.wallet_usdc.as_str(),
        ),
        (
            "binance",
            "BNB",
            "BSC",
            "BNB on BSC",
            config.wallet_bnb.as_str(),
        ),
        (
            "sol",
            "SOL",
            "Solana",
            "Solana (SOL)",
            config.wallet_sol.as_str(),
        ),
    ]
    .into_iter()
    .map(
        |(method, token, chain, display_name, wallet_address)| PaymentOptionResponse {
            method: method.to_string(),
            token: token.to_string(),
            chain: chain.to_string(),
            display_name: display_name.to_string(),
            wallet_address: (!wallet_address.trim().is_empty()).then(|| wallet_address.to_string()),
            available: !wallet_address.trim().is_empty(),
        },
    )
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
    let mut dir = std::env::current_dir()
        .map_err(|e| DomainError::InternalError(format!("Failed to resolve working directory: {}", e)))?;
    dir.push("uploads");
    dir.push("payment_proofs");
    dir.push(booking_id.to_string());
    Ok(dir)
}

fn image_extension(content_type: Option<&mime::Mime>, filename: Option<&str>) -> Option<String> {
    if let Some(name) = filename {
        if let Some(ext) = Path::new(name).extension().and_then(|value| value.to_str()) {
            let ext = ext.to_ascii_lowercase();
            if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp") {
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

async fn init_payment(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    gateway: web::Data<MultiChainGateway>,
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
        gateway.get_ref().clone(),
    );
    match use_case
        .execute(InitPaymentInput {
            booking_id: body.booking_id,
            method: body.method.clone(),
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
                        "method": body.method,
                        "amount": out.amount,
                        "wallet_address": out.wallet_address,
                    }),
                    ip,
                    ua,
                )
                .await;
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
    let actor_user_id = match authorize_booking_access(&req, &booking_repo, &body.booking_id).await
    {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let use_case = ConfirmPayment::new(
        booking_repo,
        PgPaymentRepository::new(pool.get_ref().clone()),
        PgTransactionRepository::new(pool.get_ref().clone()),
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
    if let Err(e) = payment_repo.find_by_booking_id(&booking_id).await {
        return into_response(e);
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
        if !matches!(content_type.as_ref().map(|value| value.type_().as_str()), Some("image")) {
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
            .attach_proof(&booking_id, &relative_path_string, content_type_string.as_deref())
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
