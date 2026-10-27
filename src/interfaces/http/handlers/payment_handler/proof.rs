use actix_multipart::Multipart;
use actix_web::{mime, web, HttpRequest, HttpResponse};
use futures_util::TryStreamExt;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::domain::entities::payment::PaymentStatus;
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::infrastructure::db::DbPool;
use crate::interfaces::http::error_response::into_response;
use crate::shared::errors::DomainError;

use super::access::authorize_booking_access;
use super::audit;
use super::repositories::Repositories;

const MAX_PAYMENT_PROOF_BYTES: usize = 5 * 1024 * 1024;

/// Image types a payment proof may be stored as.
///
/// This doubles as the extension whitelist: the stored filename is built from
/// this list and never from the name the client sent, so an upload cannot choose
/// its own extension and land something executable in the uploads directory.
const ALLOWED_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp"];

#[derive(Serialize)]
struct PaymentProofUploadResponse {
    message: String,
    booking_id: String,
    proof_path: String,
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

/// Decides the extension a proof will be stored under.
///
/// The client's filename is consulted first, but only as a hint: an extension is
/// accepted solely when it is already on the whitelist. Anything else falls
/// through to the declared content type, and an unrecognised type is rejected
/// outright rather than guessed at.
fn image_extension(content_type: Option<&mime::Mime>, filename: Option<&str>) -> Option<String> {
    if let Some(name) = filename {
        if let Some(ext) = Path::new(name).extension().and_then(|value| value.to_str()) {
            let ext = ext.to_ascii_lowercase();
            if ALLOWED_IMAGE_EXTENSIONS.contains(&ext.as_str()) {
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

/// Manual settlement path: the user uploads a screenshot of a transfer that
/// cannot be verified on-chain automatically (Bitcoin, in this deployment).
pub(super) async fn upload_payment_proof(
    req: HttpRequest,
    path: web::Path<Uuid>,
    pool: web::Data<DbPool>,
    mut payload: Multipart,
) -> HttpResponse {
    let booking_id = path.into_inner();
    let repos = Repositories::new(&pool);
    let actor_user_id = match authorize_booking_access(&req, &repos, &booking_id).await {
        Ok(id) => id,
        Err(e) => return into_response(e),
    };

    let payment_repo = repos.payments();
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

        // Server-generated name: nothing the client sent reaches the filesystem.
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

        // Counted as it streams rather than trusting Content-Length, and the
        // partial file is removed on every failure path so a rejected upload
        // never leaves a stub behind.
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

        audit::record(
            &repos,
            &req,
            actor_user_id,
            "payment.proof_uploaded",
            "payment",
            &booking_id.to_string(),
            serde_json::json!({
                "booking_id": booking_id,
                "proof_path": relative_path_string,
                "content_type": content_type_string,
            }),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn mime_of(value: &str) -> mime::Mime {
        value.parse().expect("test mime must parse")
    }

    #[test]
    fn uses_the_filename_extension_when_it_is_allowed() {
        let ext = image_extension(Some(&mime_of("image/png")), Some("receipt.webp"));
        assert_eq!(ext.as_deref(), Some("webp"));
    }

    #[test]
    fn normalises_the_case_of_the_filename_extension() {
        let ext = image_extension(Some(&mime_of("image/jpeg")), Some("RECEIPT.JPG"));
        assert_eq!(ext.as_deref(), Some("jpg"));
    }

    /// The decisive one: a client naming its upload `.php`, `.exe` or `.svg`
    /// must not get that extension written to disk. The name is only ever a
    /// hint, so an off-whitelist one falls back to the declared content type.
    #[test]
    fn an_extension_outside_the_whitelist_never_reaches_the_filesystem() {
        for name in ["shell.php", "payload.exe", "icon.svg", "notes.txt"] {
            let ext = image_extension(Some(&mime_of("image/png")), Some(name));
            assert_eq!(
                ext.as_deref(),
                Some("png"),
                "'{}' should have fallen back to the content type",
                name
            );
        }
    }

    #[test]
    fn falls_back_to_the_content_type_when_there_is_no_filename() {
        assert_eq!(
            image_extension(Some(&mime_of("image/gif")), None).as_deref(),
            Some("gif")
        );
    }

    /// A content type with parameters ("image/jpeg; charset=binary") still has
    /// to match, which is why the essence is compared rather than the whole
    /// header value.
    #[test]
    fn matches_a_content_type_that_carries_parameters() {
        let ext = image_extension(Some(&mime_of("image/jpeg; charset=binary")), None);
        assert_eq!(ext.as_deref(), Some("jpg"));
    }

    #[test]
    fn rejects_an_unrecognised_type_instead_of_guessing() {
        assert_eq!(image_extension(Some(&mime_of("image/tiff")), None), None);
        assert_eq!(image_extension(None, Some("receipt")), None);
        assert_eq!(image_extension(None, None), None);
    }

    /// Proofs are filed per booking, so one booking's upload can never overwrite
    /// or read another's.
    #[test]
    fn proofs_are_stored_under_a_per_booking_directory() {
        let booking_id = Uuid::new_v4();
        let dir = proof_directory(&booking_id).expect("working directory must resolve");

        assert!(dir.ends_with(booking_id.to_string()));
        assert!(dir
            .to_string_lossy()
            .replace('\\', "/")
            .contains("uploads/payment_proofs/"));
    }

    #[test]
    fn two_bookings_never_share_a_proof_directory() {
        let a = proof_directory(&Uuid::new_v4()).unwrap();
        let b = proof_directory(&Uuid::new_v4()).unwrap();
        assert_ne!(a, b);
    }
}
