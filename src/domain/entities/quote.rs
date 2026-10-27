//! Time-limited, server-authoritative price quotes.
//!
//! A quote is the object that answers "what happens if the price changes while the
//! user is checking out?". The rules it encodes:
//!
//! 1. **The server owns the price.** A quote is built from the catalogue price and
//!    the oracle rate at the moment it is issued. The client never supplies an
//!    amount; it only names what it is buying.
//! 2. **A quote expires.** An unaccepted quote is valid for [`QUOTE_TTL_SECONDS`].
//!    After that it cannot be accepted and a fresh one must be issued.
//! 3. **Acceptance is explicit and echoes the numbers.** The client must send back
//!    the exact fiat total and the exact token base units it displayed. If either
//!    differs from the server's quote -- because the flight re-priced or the token
//!    rate moved -- acceptance is refused and the user is shown the new figures and
//!    asked to approve again.
//! 4. **Acceptance locks the amount.** Once accepted, the token base units are the
//!    settlement target forever: later FX movement does not change what the buyer
//!    owes, and the buyer cannot be charged more than they approved.
//! 5. **One live quote per booking.** Issuing a new quote supersedes the previous
//!    one, so an old (cheaper) quote can never be replayed.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::domain::value_objects::{Chain, CryptoToken, Money, PriceBreakdown};
use crate::shared::errors::DomainError;

/// How long an unaccepted quote stays usable. Short enough that the platform's FX
/// exposure between quoting and acceptance is small, long enough for a human to
/// read a checkout page and connect a wallet.
pub const QUOTE_TTL_SECONDS: i64 = 600;

/// Oldest exchange rate that may be used to build a quote. A rate older than this
/// is treated as stale and the quote is refused rather than silently mispriced.
pub const MAX_RATE_AGE_SECONDS: i64 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteStatus {
    /// Issued, not yet accepted, not yet expired.
    Active,
    /// The user approved these exact figures. The amount is now locked.
    Accepted,
    /// Replaced by a newer quote for the same booking.
    Superseded,
    /// Passed its expiry without being accepted.
    Expired,
}

impl QuoteStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Accepted => "accepted",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "accepted" => Self::Accepted,
            "superseded" => Self::Superseded,
            "expired" => Self::Expired,
            _ => Self::Active,
        }
    }
}

/// A priced, expiring offer to settle one booking in one token on one chain.
#[derive(Debug, Clone)]
pub struct PaymentQuote {
    pub id: Uuid,
    pub booking_id: Uuid,
    /// Owner of the booking. A quote is bound to a user so another account cannot
    /// accept or settle against it.
    pub user_id: Uuid,
    pub method: String,
    pub chain: Chain,
    pub token: CryptoToken,
    pub token_decimals: u8,
    /// Server-side fiat breakdown at issue time.
    pub pricing: PriceBreakdown,
    /// USD price of one whole token unit used for the conversion.
    pub unit_price_usd: Decimal,
    pub rate_source: String,
    pub rate_observed_at: DateTime<Utc>,
    /// Amount owed in whole token units (display).
    pub crypto_amount: Decimal,
    /// The same amount in the token's smallest on-chain unit. This is the number
    /// that on-chain verification compares against -- never the decimal, never USD.
    pub crypto_base_units: u128,
    pub receiver_address: String,
    pub status: QuoteStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
}

impl PaymentQuote {
    #[allow(clippy::too_many_arguments)]
    pub fn issue(
        id: Uuid,
        booking_id: Uuid,
        user_id: Uuid,
        method: &str,
        chain: Chain,
        token: CryptoToken,
        pricing: PriceBreakdown,
        unit_price_usd: Decimal,
        rate_source: String,
        rate_observed_at: DateTime<Utc>,
        crypto_amount: Decimal,
        crypto_base_units: u128,
        receiver_address: String,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if pricing.total.is_zero() {
            return Err(DomainError::ValidationError(
                "Cannot quote a zero-value booking".to_string(),
            ));
        }
        if crypto_base_units == 0 {
            return Err(DomainError::ValidationError(
                "Quoted token amount rounds to zero at this token's precision".to_string(),
            ));
        }
        if receiver_address.trim().is_empty() {
            return Err(DomainError::InternalError(format!(
                "No receiving wallet is configured for {}",
                token.as_str()
            )));
        }
        // Fail closed on a stale rate rather than quoting from a price that may be
        // minutes or hours out of date.
        let rate_age = now.signed_duration_since(rate_observed_at).num_seconds();
        if rate_age > MAX_RATE_AGE_SECONDS {
            return Err(DomainError::InternalError(format!(
                "Exchange rate for {} is {}s old (limit {}s); refusing to quote",
                token.as_str(),
                rate_age,
                MAX_RATE_AGE_SECONDS
            )));
        }

        Ok(Self {
            id,
            booking_id,
            user_id,
            method: method.to_string(),
            chain,
            token,
            token_decimals: token.decimals(),
            pricing,
            unit_price_usd,
            rate_source,
            rate_observed_at,
            crypto_amount,
            crypto_base_units,
            receiver_address,
            status: QuoteStatus::Active,
            created_at: now,
            expires_at: now + Duration::seconds(QUOTE_TTL_SECONDS),
            accepted_at: None,
        })
    }

    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now >= self.expires_at
    }

    /// Seconds left before this quote can no longer be accepted (0 once expired).
    pub fn seconds_remaining(&self, now: DateTime<Utc>) -> i64 {
        self.expires_at
            .signed_duration_since(now)
            .num_seconds()
            .max(0)
    }

    /// The user approves this quote, echoing back the exact numbers their client
    /// displayed.
    ///
    /// Re-checking both figures is what makes "the price changed while I was paying"
    /// safe: if the browser is showing stale numbers, acceptance fails loudly instead
    /// of charging an amount the user never saw.
    pub fn accept(
        &mut self,
        approved_total_cents: i64,
        approved_base_units: u128,
        now: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        match self.status {
            QuoteStatus::Accepted => {
                // Idempotent: re-accepting the same figures is a no-op, anything else
                // is a conflict.
                if approved_total_cents == self.pricing.total.cents()
                    && approved_base_units == self.crypto_base_units
                {
                    return Ok(());
                }
                return Err(DomainError::Conflict(
                    "This quote was already accepted at different figures".to_string(),
                ));
            }
            QuoteStatus::Superseded => {
                return Err(DomainError::Conflict(
                    "This quote has been replaced by a newer one; refresh the price".to_string(),
                ))
            }
            QuoteStatus::Expired => {
                return Err(DomainError::Conflict(
                    "This quote has expired; request a new one".to_string(),
                ))
            }
            QuoteStatus::Active => {}
        }

        if self.is_expired(now) {
            self.status = QuoteStatus::Expired;
            return Err(DomainError::Conflict(
                "This quote has expired; request a new one".to_string(),
            ));
        }

        if approved_total_cents != self.pricing.total.cents() {
            return Err(DomainError::Conflict(format!(
                "Price changed: you approved {} but the current quote is {}. Review and confirm the new price.",
                Money::from_cents(approved_total_cents.max(0), self.pricing.total.currency())
                    .map(|m| m.to_string())
                    .unwrap_or_else(|_| approved_total_cents.to_string()),
                self.pricing.total
            )));
        }
        if approved_base_units != self.crypto_base_units {
            return Err(DomainError::Conflict(format!(
                "Token amount changed: you approved {} base units but the current quote is {}. Review and confirm the new amount.",
                approved_base_units, self.crypto_base_units
            )));
        }

        self.status = QuoteStatus::Accepted;
        self.accepted_at = Some(now);
        Ok(())
    }

    /// True when this quote may still be turned into a payment.
    pub fn is_settleable(&self) -> bool {
        self.status == QuoteStatus::Accepted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::FiatCurrency;
    use std::str::FromStr;

    fn pricing(base_cents: i64) -> PriceBreakdown {
        PriceBreakdown::calculate(Money::from_cents(base_cents, FiatCurrency::Usd).unwrap())
            .unwrap()
    }

    fn quote_at(now: DateTime<Utc>, base_cents: i64) -> PaymentQuote {
        PaymentQuote::issue(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "eth",
            Chain::Ethereum,
            CryptoToken::Eth,
            pricing(base_cents),
            Decimal::from(2000),
            "test".to_string(),
            now,
            Decimal::from_str("0.05").unwrap(),
            50_000_000_000_000_000,
            "0xReceiver0000000000000000000000000000000000".to_string(),
            now,
        )
        .unwrap()
    }

    #[test]
    fn a_fresh_quote_is_active_and_not_yet_settleable() {
        let now = Utc::now();
        let q = quote_at(now, 100_00);
        assert_eq!(q.status, QuoteStatus::Active);
        assert!(!q.is_settleable());
        assert!(!q.is_expired(now));
        assert_eq!(q.seconds_remaining(now), QUOTE_TTL_SECONDS);
    }

    #[test]
    fn rejects_a_quote_built_from_a_stale_rate() {
        let now = Utc::now();
        let err = PaymentQuote::issue(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "eth",
            Chain::Ethereum,
            CryptoToken::Eth,
            pricing(100_00),
            Decimal::from(2000),
            "test".to_string(),
            now - Duration::seconds(MAX_RATE_AGE_SECONDS + 1),
            Decimal::from_str("0.05").unwrap(),
            50_000_000_000_000_000,
            "0xReceiver0000000000000000000000000000000000".to_string(),
            now,
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::InternalError(_)));
    }

    #[test]
    fn rejects_a_quote_that_rounds_to_zero_base_units() {
        let now = Utc::now();
        let err = PaymentQuote::issue(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "eth",
            Chain::Ethereum,
            CryptoToken::Eth,
            pricing(100_00),
            Decimal::from(2000),
            "test".to_string(),
            now,
            Decimal::ZERO,
            0,
            "0xReceiver0000000000000000000000000000000000".to_string(),
            now,
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::ValidationError(_)));
    }

    #[test]
    fn accepts_when_the_user_echoes_the_exact_figures() {
        let now = Utc::now();
        let mut q = quote_at(now, 100_00);
        q.accept(q.pricing.total.cents(), q.crypto_base_units, now)
            .unwrap();
        assert_eq!(q.status, QuoteStatus::Accepted);
        assert!(q.is_settleable());
        assert!(q.accepted_at.is_some());
    }

    #[test]
    fn rejects_acceptance_of_a_stale_fiat_total() {
        // The classic attack / UX failure: the browser still shows $5,000 while the
        // server has re-priced the flight to $5,500.
        let now = Utc::now();
        let mut q = quote_at(now, 5_500_00);
        let err = q.accept(5_000_00, q.crypto_base_units, now).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
        assert_eq!(
            q.status,
            QuoteStatus::Active,
            "a refused acceptance must not settle"
        );
    }

    #[test]
    fn rejects_acceptance_of_a_stale_token_amount() {
        let now = Utc::now();
        let mut q = quote_at(now, 100_00);
        let err = q
            .accept(q.pricing.total.cents(), q.crypto_base_units - 1, now)
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
        assert_eq!(q.status, QuoteStatus::Active);
    }

    #[test]
    fn rejects_acceptance_after_expiry_and_marks_it_expired() {
        let now = Utc::now();
        let mut q = quote_at(now, 100_00);
        let later = now + Duration::seconds(QUOTE_TTL_SECONDS + 1);
        assert!(q.is_expired(later));
        let err = q
            .accept(q.pricing.total.cents(), q.crypto_base_units, later)
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
        assert_eq!(q.status, QuoteStatus::Expired);
        assert!(!q.is_settleable());
    }

    #[test]
    fn re_accepting_identical_figures_is_idempotent() {
        let now = Utc::now();
        let mut q = quote_at(now, 100_00);
        q.accept(q.pricing.total.cents(), q.crypto_base_units, now)
            .unwrap();
        q.accept(q.pricing.total.cents(), q.crypto_base_units, now)
            .unwrap();
        assert_eq!(q.status, QuoteStatus::Accepted);
    }

    #[test]
    fn re_accepting_an_accepted_quote_at_other_figures_is_a_conflict() {
        let now = Utc::now();
        let mut q = quote_at(now, 100_00);
        q.accept(q.pricing.total.cents(), q.crypto_base_units, now)
            .unwrap();
        let err = q.accept(1, q.crypto_base_units, now).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn a_superseded_quote_can_never_be_accepted() {
        let now = Utc::now();
        let mut q = quote_at(now, 100_00);
        q.status = QuoteStatus::Superseded;
        let err = q
            .accept(q.pricing.total.cents(), q.crypto_base_units, now)
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn an_accepted_amount_stays_locked_regardless_of_later_rate_moves() {
        // Acceptance fixes `crypto_base_units`; nothing in the entity can change it
        // afterwards, so a later FX move cannot alter what the buyer owes.
        let now = Utc::now();
        let mut q = quote_at(now, 100_00);
        let locked = q.crypto_base_units;
        q.accept(q.pricing.total.cents(), locked, now).unwrap();
        assert_eq!(q.crypto_base_units, locked);
    }
}
