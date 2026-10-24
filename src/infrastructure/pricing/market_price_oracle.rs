use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;

use chrono::{DateTime, Utc};

use reqwest::Client;
use rust_decimal::Decimal;

use crate::domain::entities::quote::MAX_RATE_AGE_SECONDS;
use crate::domain::ports::price_oracle::{PriceOracle, TokenRate};
use crate::domain::value_objects::CryptoToken;
use crate::infrastructure::db::DbPool;
use crate::shared::errors::DomainError;

/// How long a fetched rate may be served from the in-memory cache. Kept well under
/// `MAX_RATE_AGE_SECONDS` in the quote entity so a cached rate can never be the
/// reason a quote is built from a stale price.
const CACHE_TTL_SECONDS: i64 = 60;

/// Production price oracle with layered sources, tried in order:
///   1. in-memory cache (60s TTL),
///   2. CoinGecko live price (only in `live` mode),
///   3. the seeded `crypto_prices` table (offline fallback / dev),
///   4. a conservative static floor (last resort so quoting never hard-fails).
///
/// Stablecoins are pinned to exactly 1 USD and never hit the network.
pub struct MarketPriceOracle {
    pool: DbPool,
    http: Client,
    live: bool,
    cache: Mutex<HashMap<CryptoToken, TokenRate>>,
}

impl MarketPriceOracle {
    pub fn new(pool: DbPool, live: bool) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            pool,
            http,
            live,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// A cached rate keeps its ORIGINAL `observed_at`, so caching can never make a
    /// stale price look fresh to the quote layer.
    fn cached(&self, token: CryptoToken, now: DateTime<Utc>) -> Option<TokenRate> {
        let cache = self.cache.lock().ok()?;
        let rate = cache.get(&token)?;
        let age = now.signed_duration_since(rate.observed_at).num_seconds();
        if age < CACHE_TTL_SECONDS {
            Some(rate.clone())
        } else {
            None
        }
    }

    fn store(&self, token: CryptoToken, rate: &TokenRate) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(token, rate.clone());
        }
    }

    async fn fetch_coingecko(&self, token: CryptoToken) -> Option<Decimal> {
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            token.coingecko_id()
        );
        let resp = self.http.get(&url).send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let json: serde_json::Value = resp.json().await.ok()?;
        let raw = json.get(token.coingecko_id()).and_then(|v| v.get("usd"))?;
        // Parse via the JSON number's textual form to avoid f64 rounding error.
        let price = Decimal::from_str(&raw.to_string()).ok()?;
        if price > Decimal::ZERO {
            Some(price)
        } else {
            None
        }
    }

    /// Reads the seeded price table, returning the row's own `last_updated` so the
    /// caller can judge how old the number is.
    async fn fetch_db(&self, token: CryptoToken) -> Option<(Decimal, DateTime<Utc>)> {
        let client = self.pool.get().await.ok()?;
        let row = client
            .query_opt(
                "SELECT CAST(price AS TEXT), last_updated FROM crypto_prices WHERE symbol = $1",
                &[&token.as_str()],
            )
            .await
            .ok()??;
        let raw: String = row.get(0);
        let price = Decimal::from_str(raw.trim()).ok()?;
        let observed: chrono::NaiveDateTime = row.try_get(1).ok()?;
        if price > Decimal::ZERO {
            Some((price, DateTime::from_naive_utc_and_offset(observed, Utc)))
        } else {
            None
        }
    }

    /// Whether a row from `crypto_prices` is worth returning.
    ///
    /// In **live** mode it always is: the quote layer will refuse a stale rate, and
    /// failing the sale is the right outcome when a real customer is being priced.
    ///
    /// Outside live mode the seeded row is a development convenience rather than a
    /// price feed. Once it ages past what a quote will accept, returning it makes
    /// local checkout fail permanently with a stale-rate error, while the static
    /// floor below would have produced a usable (clearly-labelled) price. So a stale
    /// row is skipped and the floor is used instead.
    fn db_rate_is_usable(live: bool, age_seconds: i64) -> bool {
        live || age_seconds <= MAX_RATE_AGE_SECONDS
    }

    /// Conservative last-resort prices. Only ever used OUTSIDE live mode (local
    /// development): quoting a real customer from an invented rate would charge them
    /// the wrong amount, so live mode fails closed instead.
    fn static_floor(token: CryptoToken) -> Decimal {
        let dollars: i64 = match token {
            CryptoToken::Btc => 60_000,
            CryptoToken::Eth => 3_000,
            CryptoToken::Sol => 150,
            CryptoToken::Bnb => 600,
            CryptoToken::Usdt | CryptoToken::Usdc => 1,
        };
        Decimal::from(dollars)
    }
}

#[async_trait::async_trait]
impl PriceOracle for MarketPriceOracle {
    async fn rate(&self, token: CryptoToken) -> Result<TokenRate, DomainError> {
        let now = Utc::now();

        if token.is_stablecoin() {
            // USDT/USDC are quoted at their peg by policy. The residual risk (a
            // de-peg between quote and settlement) is documented in SECURITY_AUDIT.md.
            return Ok(TokenRate {
                unit_price_usd: Decimal::ONE,
                source: "pegged".to_string(),
                observed_at: now,
            });
        }

        if let Some(rate) = self.cached(token, now) {
            return Ok(rate);
        }

        if self.live {
            if let Some(price) = self.fetch_coingecko(token).await {
                let rate = TokenRate {
                    unit_price_usd: price,
                    source: "coingecko".to_string(),
                    observed_at: now,
                };
                self.store(token, &rate);
                return Ok(rate);
            }
            log::warn!(
                "price oracle: CoinGecko unavailable for {}, falling back to the crypto_prices table",
                token.as_str()
            );
        }

        if let Some((price, observed_at)) = self.fetch_db(token).await {
            let age = now.signed_duration_since(observed_at).num_seconds();
            if Self::db_rate_is_usable(self.live, age) {
                let rate = TokenRate {
                    unit_price_usd: price,
                    source: "database".to_string(),
                    observed_at,
                };
                // Only cache a DB rate that is actually fresh; a stale row must keep
                // looking stale to the quote layer.
                if age < CACHE_TTL_SECONDS {
                    self.store(token, &rate);
                }
                return Ok(rate);
            }
            log::warn!(
                "price oracle: the crypto_prices row for {} is {}s old (limit {}s); using the development static floor",
                token.as_str(),
                age,
                MAX_RATE_AGE_SECONDS
            );
        }

        if self.live {
            // Fail closed. A fabricated rate here would mean charging a real customer
            // an amount derived from a number nobody quoted.
            return Err(DomainError::InternalError(format!(
                "No trustworthy {} price available (CoinGecko and crypto_prices both failed); refusing to quote",
                token.as_str()
            )));
        }

        let floor = Self::static_floor(token);
        log::warn!(
            "price oracle: no live/DB price for {}, using the development static floor {}",
            token.as_str(),
            floor
        );
        Ok(TokenRate {
            unit_price_usd: floor,
            source: "static-floor-dev".to_string(),
            observed_at: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_db_row_is_used_in_either_mode() {
        assert!(MarketPriceOracle::db_rate_is_usable(true, 10));
        assert!(MarketPriceOracle::db_rate_is_usable(false, 10));
    }

    #[test]
    fn a_row_exactly_at_the_limit_is_still_usable() {
        assert!(MarketPriceOracle::db_rate_is_usable(
            false,
            MAX_RATE_AGE_SECONDS
        ));
    }

    #[test]
    fn live_mode_keeps_a_stale_row_so_the_quote_layer_refuses_the_sale() {
        // Returning it is the point: `PaymentQuote::issue` rejects it and the
        // customer is not priced from a day-old number.
        assert!(MarketPriceOracle::db_rate_is_usable(
            true,
            MAX_RATE_AGE_SECONDS + 1
        ));
        assert!(MarketPriceOracle::db_rate_is_usable(true, 200_000));
    }

    #[test]
    fn dev_mode_skips_a_stale_row_so_local_checkout_falls_back_to_the_floor() {
        assert!(!MarketPriceOracle::db_rate_is_usable(
            false,
            MAX_RATE_AGE_SECONDS + 1
        ));
        assert!(!MarketPriceOracle::db_rate_is_usable(false, 200_000));
    }

    #[test]
    fn the_development_floor_is_positive_for_every_token() {
        for token in [
            CryptoToken::Btc,
            CryptoToken::Eth,
            CryptoToken::Sol,
            CryptoToken::Bnb,
            CryptoToken::Usdt,
            CryptoToken::Usdc,
        ] {
            assert!(MarketPriceOracle::static_floor(token) > Decimal::ZERO);
        }
    }
}
