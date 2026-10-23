//! Issue a fresh, time-limited price quote for a booking.
//!
//! This is the use case that answers the review question "what happens if the flight
//! price changes while the user is booking?". Every quote is re-derived from the
//! authoritative catalogue price at the moment it is issued, never from the price
//! stored when the booking was first created and never from anything the client sent.
//! If the catalogue has moved, the quote carries the old and new totals so the UI can
//! show the user both and ask them to approve the new one.

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{CreateQuoteInput, QuoteView};
use crate::domain::entities::booking::Booking;
use crate::domain::entities::quote::PaymentQuote;
use crate::domain::ports::blockchain_gateway::BlockchainGateway;
use crate::domain::ports::price_oracle::PriceOracle;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::repositories::catalog_repository::CatalogRepository;
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::repositories::quote_repository::QuoteRepository;
use crate::domain::value_objects::money::PriceBreakdown;
use crate::domain::value_objects::{crypto_base_units, resolve_chain_token, usd_to_crypto_amount};
use crate::shared::errors::DomainError;

pub struct CreateQuote<BR, QR, PR, CR, BG>
where
    BR: BookingRepository,
    QR: QuoteRepository,
    PR: PaymentRepository,
    CR: CatalogRepository,
    BG: BlockchainGateway,
{
    bookings: BR,
    quotes: QR,
    payments: PR,
    catalog: CR,
    blockchain: BG,
    oracle: Arc<dyn PriceOracle>,
}

impl<BR, QR, PR, CR, BG> CreateQuote<BR, QR, PR, CR, BG>
where
    BR: BookingRepository,
    QR: QuoteRepository,
    PR: PaymentRepository,
    CR: CatalogRepository,
    BG: BlockchainGateway,
{
    pub fn new(
        bookings: BR,
        quotes: QR,
        payments: PR,
        catalog: CR,
        blockchain: BG,
        oracle: Arc<dyn PriceOracle>,
    ) -> Self {
        Self {
            bookings,
            quotes,
            payments,
            catalog,
            blockchain,
            oracle,
        }
    }

    pub async fn execute(&self, input: CreateQuoteInput) -> Result<QuoteView, DomainError> {
        let (chain, token) = resolve_chain_token(&input.method).ok_or_else(|| {
            DomainError::ValidationError(format!("Unsupported payment method: '{}'", input.method))
        })?;

        let mut booking = self.bookings.find_by_id(&input.booking_id).await?;
        if booking.user_id != input.user_id {
            return Err(DomainError::Unauthorized);
        }
        if booking.status.as_str() != "pending" {
            return Err(DomainError::Conflict(format!(
                "Cannot quote a booking in status '{}'",
                booking.status.as_str()
            )));
        }
        // Never re-quote something that is already paid for: the settled amount is
        // fixed and a new quote would create a second, conflicting price.
        if self
            .payments
            .find_by_booking_id(&input.booking_id)
            .await
            .is_ok()
        {
            return Err(DomainError::Conflict(
                "Payment has already been initialised for this booking".to_string(),
            ));
        }

        // Nor re-quote one whose price the user has already locked. Acceptance IS
        // the lock; issuing a second quote afterwards would leave two prices for one
        // booking and collide with `idx_payment_quotes_one_accepted`. Refusing here
        // turns that into a clear conflict instead of a raw database error.
        if self
            .quotes
            .find_accepted_for_booking(&input.booking_id)
            .await?
            .is_some()
        {
            return Err(DomainError::Conflict(
                "You have already approved a price for this booking. Complete the payment, \
                 or start a new booking to change the amount or network."
                    .to_string(),
            ));
        }

        // --- Re-price from the authoritative catalogue -----------------------
        let current_base = self.current_base_price(&booking).await?;
        let previous_total = booking.pricing.total;
        let price_changed = current_base != booking.pricing.base;

        if price_changed {
            let repriced = PriceBreakdown::calculate(current_base)
                .map_err(|e| DomainError::InternalError(e.to_string()))?;
            booking.pricing = repriced;
            booking.updated_at = chrono::Utc::now();
            // Persist the new price so every later read of the booking agrees with
            // the quote the user is about to be shown.
            self.bookings.update(&booking).await?;
        }

        // --- Convert to the selected token, in exact base units --------------
        let rate = self.oracle.rate(token).await?;
        let crypto_amount = usd_to_crypto_amount(
            booking.pricing.total.to_decimal(),
            rate.unit_price_usd,
            token.decimals(),
        )
        .ok_or_else(|| {
            DomainError::InternalError(
                "Unable to price this booking in the selected asset".to_string(),
            )
        })?;
        let base_units = crypto_base_units(crypto_amount, token.decimals()).ok_or_else(|| {
            DomainError::InternalError("Quoted token amount is out of range".to_string())
        })?;

        let receiver = self.blockchain.receiving_address(token.as_str())?;
        let now = chrono::Utc::now();

        let quote = PaymentQuote::issue(
            Uuid::new_v4(),
            booking.id,
            booking.user_id,
            &input.method,
            chain,
            token,
            booking.pricing,
            rate.unit_price_usd,
            rate.source,
            rate.observed_at,
            crypto_amount,
            base_units,
            receiver,
            now,
        )?;

        // Issuing supersedes any previous active quote for this booking (atomically,
        // in the repository), so a stale cheaper quote cannot be accepted afterwards.
        self.quotes.issue(&quote).await?;

        Ok(QuoteView::from_quote(
            &quote,
            now,
            price_changed.then(|| previous_total.cents()),
        ))
    }

    /// The catalogue's price for whatever this booking is for, right now.
    async fn current_base_price(
        &self,
        booking: &Booking,
    ) -> Result<crate::domain::value_objects::Money, DomainError> {
        match (booking.flight_id, booking.asset_id) {
            (Some(flight_id), _) => self.catalog.flight_price(&flight_id).await,
            (None, Some(asset_id)) => self.catalog.asset_price(&asset_id).await,
            (None, None) => Err(DomainError::InternalError(
                "Booking references neither a flight nor an asset; cannot re-price".to_string(),
            )),
        }
    }
}
