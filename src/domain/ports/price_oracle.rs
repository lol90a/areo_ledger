use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::domain::value_objects::CryptoToken;
use crate::shared::errors::DomainError;

/// An exchange rate together with its provenance.
///
/// A bare number is not enough to quote a payment safely: the quote has to record
/// *where* the rate came from and *when* it was observed, so a stale or fabricated
/// rate can be detected and refused instead of silently mispricing an order.
#[derive(Debug, Clone)]
pub struct TokenRate {
    /// USD price of one whole unit of the token (e.g. 1 ETH -> 2500.00).
    pub unit_price_usd: Decimal,
    /// Where the rate came from: `coingecko`, `database`, `pegged`, `static-floor-dev`.
    pub source: String,
    /// When this rate was observed upstream (not when it was read from cache).
    pub observed_at: DateTime<Utc>,
}

/// Converts a crypto asset to its current USD price so a fiat-denominated booking
/// can be quoted (and later verified) as an exact crypto amount.
#[async_trait::async_trait]
pub trait PriceOracle: Send + Sync {
    /// Current rate for `token`, or an error if no trustworthy rate is available.
    ///
    /// Implementations MUST fail rather than invent a price in production: quoting
    /// from a made-up rate charges the customer the wrong amount.
    async fn rate(&self, token: CryptoToken) -> Result<TokenRate, DomainError>;
}
