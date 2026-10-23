use uuid::Uuid;

use crate::domain::entities::quote::PaymentQuote;
use crate::shared::errors::DomainError;

/// Persistence for time-limited price quotes.
///
/// The implementation is responsible for the invariant the entity cannot enforce on
/// its own: **at most one `active` quote per booking at any instant**. Issuing a new
/// quote must supersede the previous one in the same database transaction, otherwise
/// two concurrent checkout tabs could each hold a live quote and the cheaper one
/// could be accepted after the price moved.
#[async_trait::async_trait]
pub trait QuoteRepository: Send + Sync {
    /// Persist `quote` as the booking's only active quote, marking any previous
    /// active quote `superseded`. Atomic.
    async fn issue(&self, quote: &PaymentQuote) -> Result<(), DomainError>;

    /// Fetch by id. `NotFound` if it does not exist.
    async fn find_by_id(&self, quote_id: &Uuid) -> Result<PaymentQuote, DomainError>;

    /// The booking's current active (not superseded, not expired-by-status) quote.
    async fn find_active_for_booking(
        &self,
        booking_id: &Uuid,
    ) -> Result<Option<PaymentQuote>, DomainError>;

    /// The booking's accepted quote, if the user has approved one.
    async fn find_accepted_for_booking(
        &self,
        booking_id: &Uuid,
    ) -> Result<Option<PaymentQuote>, DomainError>;

    /// Transition `active` -> `accepted` for exactly this quote.
    ///
    /// Must be conditional on the row still being `active` and must return
    /// `Ok(false)` when it is not, so two concurrent acceptances cannot both win.
    async fn mark_accepted(&self, quote: &PaymentQuote) -> Result<bool, DomainError>;

    /// Transition `active` -> `expired` (housekeeping; acceptance also checks the
    /// clock, so this is an optimisation rather than a security control).
    async fn expire_stale(&self) -> Result<u64, DomainError>;
}
