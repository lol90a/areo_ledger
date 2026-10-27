use chrono::{DateTime, Utc};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::entities::booking::{Booking, BookingStatus};
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::value_objects::blockchain::TxHash;
use crate::domain::value_objects::money::{FiatCurrency, Money, PriceBreakdown};
use crate::infrastructure::db::DbPool;
use crate::shared::errors::{DomainError, InfraError};

pub struct PgBookingRepository {
    pool: DbPool,
}

impl PgBookingRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    async fn client(&self) -> Result<deadpool_postgres::Client, DomainError> {
        self.pool
            .get()
            .await
            .map_err(|e| InfraError::Database(e.to_string()).into())
    }

    fn uuid_from_row(
        row: &tokio_postgres::Row,
        idx: usize,
        field: &str,
    ) -> Result<Uuid, DomainError> {
        let raw: String = row.try_get(idx).map_err(|e| {
            DomainError::InternalError(format!("error retrieving {}: {}", field, e))
        })?;
        Uuid::from_str(raw.trim())
            .map_err(|e| DomainError::InternalError(format!("invalid UUID in {}: {}", field, e)))
    }

    fn optional_uuid_from_row(
        row: &tokio_postgres::Row,
        idx: usize,
        field: &str,
    ) -> Result<Option<Uuid>, DomainError> {
        let raw: Option<String> = row.try_get(idx).map_err(|e| {
            DomainError::InternalError(format!("error retrieving {}: {}", field, e))
        })?;

        raw.map(|value| {
            Uuid::from_str(value.trim()).map_err(|e| {
                DomainError::InternalError(format!("invalid UUID in {}: {}", field, e))
            })
        })
        .transpose()
    }

    fn row_to_booking(row: &tokio_postgres::Row) -> Result<Booking, DomainError> {
        // Read the base price exactly as a decimal string and recompute the breakdown
        // through the canonical integer-cents Money type (no f64 truncation).
        let base_raw: String = row.get(4);
        let base = Money::from_decimal_str(base_raw.trim(), FiatCurrency::Usd)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let pricing = PriceBreakdown::calculate(base)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let tx_raw: Option<String> = row.get(6);
        let tx = tx_raw.and_then(|h| TxHash::new(h).ok());
        let created: chrono::NaiveDateTime = row.get(7);
        let updated: chrono::NaiveDateTime = row.get(8);

        Ok(Booking::from_db(
            Self::uuid_from_row(row, 0, "booking.id")?,
            Self::uuid_from_row(row, 1, "booking.user_id")?,
            Self::optional_uuid_from_row(row, 2, "booking.flight_id")?,
            Self::optional_uuid_from_row(row, 9, "booking.asset_id")?,
            BookingStatus::from_str(row.get::<_, String>(3).as_str()),
            pricing,
            row.get(5),
            tx,
            DateTime::from_naive_utc_and_offset(created, Utc),
            DateTime::from_naive_utc_and_offset(updated, Utc),
        ))
    }
}

#[async_trait::async_trait]
impl BookingRepository for PgBookingRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<Booking, DomainError> {
        let client = self.client().await?;
        let row = client
            .query_one(
                "SELECT id::TEXT, user_id::TEXT, flight_id::TEXT, status::TEXT,
             CAST(base_price AS TEXT), payment_method, tx_hash, created_at, updated_at,
             asset_id::TEXT
             FROM bookings WHERE id = $1",
                &[id],
            )
            .await
            .map_err(|_| DomainError::NotFound(format!("Booking {}", id)))?;
        Self::row_to_booking(&row)
    }

    async fn find_all(&self) -> Result<Vec<Booking>, DomainError> {
        let client = self.client().await?;
        let rows = client
            .query(
                "SELECT id::TEXT, user_id::TEXT, flight_id::TEXT, status::TEXT,
             CAST(base_price AS TEXT), payment_method, tx_hash, created_at, updated_at,
             asset_id::TEXT
             FROM bookings ORDER BY created_at DESC",
                &[],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        rows.iter().map(Self::row_to_booking).collect()
    }

    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Vec<Booking>, DomainError> {
        let client = self.client().await?;
        let rows = client
            .query(
                "SELECT id::TEXT, user_id::TEXT, flight_id::TEXT, status::TEXT,
             CAST(base_price AS TEXT), payment_method, tx_hash, created_at, updated_at,
             asset_id::TEXT
             FROM bookings WHERE user_id = $1 ORDER BY created_at DESC",
                &[user_id],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        rows.iter().map(Self::row_to_booking).collect()
    }

    async fn save(&self, b: &Booking) -> Result<(), DomainError> {
        let client = self.client().await?;
        let base_price = b.pricing.base.to_decimal_str();
        let markup_pct = format!("{:.2}", 10.00_f64);
        let total_price = b.pricing.total.to_decimal_str();
        let status = b.status.as_str().to_string();
        let method = b.payment_method.clone();
        client
            .execute(
                "INSERT INTO bookings
             (id, user_id, flight_id, asset_id, status, base_price, markup_percentage,
              total_price, payment_method)
             VALUES ($1, $2, $3, $4, $5,
                     ($6::TEXT)::DECIMAL(15,2),
                     ($7::TEXT)::DECIMAL(5,2),
                     ($8::TEXT)::DECIMAL(15,2),
                     $9)",
                &[
                    &b.id,
                    &b.user_id,
                    &b.flight_id,
                    &b.asset_id,
                    &status,
                    &base_price,
                    &markup_pct,
                    &total_price,
                    &method,
                ],
            )
            .await
            .map_err(|e| {
                log::error!("DB save booking error: {}", e);
                InfraError::Database(e.to_string())
            })?;
        Ok(())
    }

    /// Persists status, settlement hash AND pricing.
    ///
    /// Pricing is included because re-quoting at checkout re-reads the authoritative
    /// catalogue price; if that were not written back, the stored booking and the
    /// quote the user approved would disagree about what was owed.
    async fn update(&self, b: &Booking) -> Result<(), DomainError> {
        let client = self.client().await?;
        let status = b.status.as_str().to_string();
        let tx_hash = b.tx_hash.as_ref().map(|h| h.as_str().to_string());
        let base_price = b.pricing.base.to_decimal_str();
        let total_price = b.pricing.total.to_decimal_str();
        client
            .execute(
                "UPDATE bookings
                 SET status = $1,
                     tx_hash = $2,
                     base_price = ($3::TEXT)::DECIMAL(15,2),
                     total_price = ($4::TEXT)::DECIMAL(15,2),
                     updated_at = NOW()
                 WHERE id = $5",
                &[&status, &tx_hash, &base_price, &total_price, &b.id],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, DomainError> {
        let client = self.client().await?;
        let row = client
            .query_one("SELECT COUNT(*) FROM bookings", &[])
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(row.get(0))
    }

    async fn total_revenue(&self) -> Result<i64, DomainError> {
        let client = self.client().await?;
        let row = client.query_one(
            "SELECT COALESCE(SUM(CAST(total_price AS FLOAT8)), 0.0) FROM bookings WHERE status = 'confirmed'",
            &[],
        ).await.map_err(|e| InfraError::Database(e.to_string()))?;
        let val: f64 = row.get(0);
        Ok((val * 100.0) as i64)
    }
}
