//! The user explicitly approves a quote.
//!
//! Acceptance is the moment the price is locked, so it is deliberately strict:
//!
//! * the quote must belong to the caller,
//! * it must still be `active` and inside its TTL,
//! * the client must echo back the exact fiat total **and** the exact token base
//!   units it displayed. A mismatch means the browser was showing stale numbers --
//!   the user is shown the current figures and must approve again, instead of being
//!   charged an amount they never saw,
//! * the `active -> accepted` transition is conditional in SQL, so two concurrent
//!   acceptances cannot both succeed.

use crate::application::dto::{AcceptQuoteInput, QuoteView};
use crate::domain::repositories::quote_repository::QuoteRepository;
use crate::shared::errors::DomainError;

pub struct AcceptQuote<QR: QuoteRepository> {
    quotes: QR,
}

impl<QR: QuoteRepository> AcceptQuote<QR> {
    pub fn new(quotes: QR) -> Self {
        Self { quotes }
    }

    pub async fn execute(&self, input: AcceptQuoteInput) -> Result<QuoteView, DomainError> {
        let mut quote = self.quotes.find_by_id(&input.quote_id).await?;

        if quote.user_id != input.user_id {
            return Err(DomainError::Unauthorized);
        }
        if quote.booking_id != input.booking_id {
            return Err(DomainError::ValidationError(
                "This quote does not belong to that booking".to_string(),
            ));
        }

        let approved_base_units: u128 = input.approved_crypto_base_units.parse().map_err(|_| {
            DomainError::ValidationError(
                "approved_crypto_base_units must be a non-negative integer".to_string(),
            )
        })?;

        let now = chrono::Utc::now();
        // The entity owns the rules; it mutates `quote` to `Accepted` on success and
        // to `Expired` when the TTL has passed.
        quote.accept(input.approved_total_cents, approved_base_units, now)?;

        if !self.quotes.mark_accepted(&quote).await? {
            // Someone else moved the row out of `active` between our read and our
            // write (a second tab, or a concurrent re-quote).
            return Err(DomainError::Conflict(
                "This quote is no longer active; request a fresh price".to_string(),
            ));
        }

        Ok(QuoteView::from_quote(&quote, now, None))
    }
}
