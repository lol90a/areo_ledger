use uuid::Uuid;

use crate::shared::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmOutcome {
    /// The payment/booking were transitioned to confirmed by this call.
    Confirmed,
    /// The payment was already confirmed — a safe, idempotent no-op.
    AlreadyConfirmed,
}

/// Atomically settles a verified payment across the payment, booking, and
/// transaction tables in a single database transaction.
#[async_trait::async_trait]
pub trait SettlementRepository: Send + Sync {
    /// Confirm the payment for `booking_id`, guarded so that:
    ///   * a `tx_hash` already used by a *different* booking is rejected (replay),
    ///   * concurrent confirms are serialized (row lock) and only settle once,
    ///   * the booking is confirmed and a single Buy transaction is recorded,
    /// all in one transaction. Returns [`ConfirmOutcome::AlreadyConfirmed`] if the
    /// payment was already settled.
    async fn confirm_payment(
        &self,
        booking_id: &Uuid,
        tx_hash: &str,
        sender_address: Option<&str>,
    ) -> Result<ConfirmOutcome, DomainError>;
}
