use chrono::{DateTime, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::entities::quote::{PaymentQuote, QuoteStatus};
use crate::domain::repositories::quote_repository::QuoteRepository;
use crate::domain::value_objects::money::{FiatCurrency, Money, PriceBreakdown};
use crate::domain::value_objects::{Chain, CryptoToken};
use crate::infrastructure::db::DbPool;
use crate::shared::errors::{DomainError, InfraError};

const SELECT_COLUMNS: &str = "id, booking_id, user_id, method, chain, token, token_decimals,
     base_cents, markup_cents, fee_cents, total_cents, currency,
     CAST(unit_price_usd AS TEXT), rate_source, rate_observed_at,
     CAST(crypto_amount AS TEXT), CAST(crypto_base_units AS TEXT), receiver_address,
     status, created_at, expires_at, accepted_at";

pub struct PgQuoteRepository {
    pool: DbPool,
}

impl PgQuoteRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    async fn client(&self) -> Result<deadpool_postgres::Client, DomainError> {
        self.pool
            .get()
            .await
            .map_err(|e| InfraError::Database(e.to_string()).into())
    }

    fn row_to_quote(row: &tokio_postgres::Row) -> Result<PaymentQuote, DomainError> {
        let bad = |what: &str| DomainError::InternalError(format!("corrupt quote row: {}", what));

        let currency_raw: String = row.get(11);
        let currency = FiatCurrency::from_str(&currency_raw).ok_or_else(|| bad("currency"))?;

        let base = Money::from_cents(row.get::<_, i64>(7), currency)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let markup = Money::from_cents(row.get::<_, i64>(8), currency)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let service_fee = Money::from_cents(row.get::<_, i64>(9), currency)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let total = Money::from_cents(row.get::<_, i64>(10), currency)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let token = CryptoToken::from_str(row.get::<_, &str>(5)).ok_or_else(|| bad("token"))?;
        let chain = Chain::from_str(row.get::<_, &str>(4)).ok_or_else(|| bad("chain"))?;

        let unit_price: String = row.get(12);
        let crypto_amount: String = row.get(15);
        // base units are stored as NUMERIC(40,0) and read as text: u128 covers every
        // supported token's supply, and parsing it here keeps the whole pipeline in
        // exact integers.
        let base_units_raw: String = row.get(16);

        let rate_observed: NaiveDateTime = row.get(14);
        let created: NaiveDateTime = row.get(19);
        let expires: NaiveDateTime = row.get(20);
        let accepted: Option<NaiveDateTime> = row.get(21);

        Ok(PaymentQuote {
            id: row.get(0),
            booking_id: row.get(1),
            user_id: row.get(2),
            method: row.get(3),
            chain,
            token,
            token_decimals: row.get::<_, i16>(6) as u8,
            pricing: PriceBreakdown {
                base,
                markup,
                service_fee,
                total,
            },
            unit_price_usd: Decimal::from_str(unit_price.trim())
                .map_err(|_| bad("unit_price_usd"))?,
            rate_source: row.get(13),
            rate_observed_at: DateTime::from_naive_utc_and_offset(rate_observed, Utc),
            crypto_amount: Decimal::from_str(crypto_amount.trim())
                .map_err(|_| bad("crypto_amount"))?,
            crypto_base_units: u128::from_str(base_units_raw.trim())
                .map_err(|_| bad("crypto_base_units"))?,
            receiver_address: row.get(17),
            status: QuoteStatus::from_str(row.get::<_, &str>(18)),
            created_at: DateTime::from_naive_utc_and_offset(created, Utc),
            expires_at: DateTime::from_naive_utc_and_offset(expires, Utc),
            accepted_at: accepted.map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
        })
    }
}

#[async_trait::async_trait]
impl QuoteRepository for PgQuoteRepository {
    async fn issue(&self, quote: &PaymentQuote) -> Result<(), DomainError> {
        let mut client = self.client().await?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;

        // Supersede any previous live quote for this booking FIRST, inside the same
        // transaction, so there is never a window with two active quotes and an old,
        // cheaper one can never be accepted after a re-price.
        tx.execute(
            "UPDATE payment_quotes SET status = 'superseded'
             WHERE booking_id = $1 AND status = 'active'",
            &[&quote.booking_id],
        )
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

        tx.execute(
            "INSERT INTO payment_quotes (
                 id, booking_id, user_id, method, chain, token, token_decimals,
                 base_cents, markup_cents, fee_cents, total_cents, currency,
                 unit_price_usd, rate_source, rate_observed_at,
                 crypto_amount, crypto_base_units, receiver_address,
                 status, created_at, expires_at
             ) VALUES (
                 $1, $2, $3, $4, $5, $6, $7,
                 $8, $9, $10, $11, $12,
                 ($13::TEXT)::DECIMAL(30,10), $14, $15,
                 ($16::TEXT)::DECIMAL(40,18), ($17::TEXT)::NUMERIC(40,0), $18,
                 'active', $19, $20
             )",
            &[
                &quote.id,
                &quote.booking_id,
                &quote.user_id,
                &quote.method,
                &quote.chain.as_str(),
                &quote.token.as_str(),
                &(quote.token_decimals as i16),
                &quote.pricing.base.cents(),
                &quote.pricing.markup.cents(),
                &quote.pricing.service_fee.cents(),
                &quote.pricing.total.cents(),
                &quote.pricing.total.currency().as_str(),
                &quote.unit_price_usd.to_string(),
                &quote.rate_source,
                &quote.rate_observed_at.naive_utc(),
                &quote.crypto_amount.normalize().to_string(),
                &quote.crypto_base_units.to_string(),
                &quote.receiver_address,
                &quote.created_at.naive_utc(),
                &quote.expires_at.naive_utc(),
            ],
        )
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(())
    }

    async fn find_by_id(&self, quote_id: &Uuid) -> Result<PaymentQuote, DomainError> {
        let client = self.client().await?;
        let row = client
            .query_opt(
                &format!(
                    "SELECT {} FROM payment_quotes WHERE id = $1",
                    SELECT_COLUMNS
                ),
                &[quote_id],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound(format!("Quote {}", quote_id)))?;
        Self::row_to_quote(&row)
    }

    async fn find_active_for_booking(
        &self,
        booking_id: &Uuid,
    ) -> Result<Option<PaymentQuote>, DomainError> {
        let client = self.client().await?;
        let row = client
            .query_opt(
                &format!(
                    "SELECT {} FROM payment_quotes
                     WHERE booking_id = $1 AND status = 'active'
                     ORDER BY created_at DESC LIMIT 1",
                    SELECT_COLUMNS
                ),
                &[booking_id],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        row.as_ref().map(Self::row_to_quote).transpose()
    }

    async fn find_accepted_for_booking(
        &self,
        booking_id: &Uuid,
    ) -> Result<Option<PaymentQuote>, DomainError> {
        let client = self.client().await?;
        let row = client
            .query_opt(
                &format!(
                    "SELECT {} FROM payment_quotes
                     WHERE booking_id = $1 AND status = 'accepted'
                     ORDER BY accepted_at DESC LIMIT 1",
                    SELECT_COLUMNS
                ),
                &[booking_id],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        row.as_ref().map(Self::row_to_quote).transpose()
    }

    async fn mark_accepted(&self, quote: &PaymentQuote) -> Result<bool, DomainError> {
        let client = self.client().await?;
        // Conditional on `status = 'active'`: whichever concurrent request gets here
        // first wins, the other sees 0 rows and is told to re-quote.
        let updated = client
            .execute(
                "UPDATE payment_quotes
                 SET status = 'accepted', accepted_at = $2
                 WHERE id = $1 AND status = 'active'",
                &[
                    &quote.id,
                    &quote.accepted_at.unwrap_or_else(Utc::now).naive_utc(),
                ],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(updated == 1)
    }

    async fn expire_stale(&self) -> Result<u64, DomainError> {
        let client = self.client().await?;
        client
            .execute(
                "UPDATE payment_quotes SET status = 'expired'
                 WHERE status = 'active' AND expires_at <= NOW()",
                &[],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()).into())
    }
}
