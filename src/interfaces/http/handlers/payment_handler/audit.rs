use actix_web::HttpRequest;
use serde_json::Value;
use uuid::Uuid;

use crate::interfaces::http::error_response::request_metadata;

use super::repositories::Repositories;

/// Writes one audit row for a payment event.
///
/// The result is deliberately discarded. An audit write is a side record of a
/// payment that has *already* succeeded: failing the response because the log
/// insert failed would turn a bookkeeping problem into a user-facing error and,
/// worse, invite the client to retry an operation that already went through.
/// The repository logs its own database errors.
pub(super) async fn record(
    repos: &Repositories,
    req: &HttpRequest,
    actor_id: Uuid,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    details: Value,
) {
    let (ip, ua) = request_metadata(req);
    let _ = repos
        .audit_log()
        .record(
            Some(actor_id),
            action,
            entity_type,
            entity_id,
            details,
            ip,
            ua,
        )
        .await;
}
