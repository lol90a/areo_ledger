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
        let mk = |cents: i64| {
            Money::from_cents(cents, FiatCurrency::Usd)
                .map_err(|e| DomainError::InternalError(e.to_string()))
        };

        let base_f: f64 = row.get(4);
        let markup_f: f64 = row.get(5);
        let fee_f: f64 = row.get(6);
        let total_f: f64 = row.get(7);

        let pricing = PriceBreakdown {
            base: mk((base_f * 100.0) as i64)?,
            markup: mk((markup_f * 100.0) as i64)?,
            service_fee: mk((fee_f * 100.0) as i64)?,
            total: mk((total_f * 100.0) as i64)?,
        };

        let tx_raw: Option<String> = row.get(9);
        let tx = tx_raw.and_then(|h| TxHash::new(h).ok());
        let created: chrono::NaiveDateTime = row.get(10);
        let updated: chrono::NaiveDateTime = row.get(11);

        Ok(Booking::from_db(
            Self::uuid_from_row(row, 0, "booking.id")?,
            Self::uuid_from_row(row, 1, "booking.user_id")?,
            Self::optional_uuid_from_row(row, 2, "booking.flight_id")?,
            BookingStatus::from_str(row.get::<_, String>(3).as_str()),
            pricing,
            row.get(8),
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
             CAST(base_price AS FLOAT8), CAST(base_price * 0.10 AS FLOAT8),
             CAST(base_price * 0.05 AS FLOAT8), CAST(total_price AS FLOAT8),
             payment_method, tx_hash, created_at, updated_at
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
             CAST(base_price AS FLOAT8), CAST(base_price * 0.10 AS FLOAT8),
             CAST(base_price * 0.05 AS FLOAT8), CAST(total_price AS FLOAT8),
             payment_method, tx_hash, created_at, updated_at
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
             CAST(base_price AS FLOAT8), CAST(base_price * 0.10 AS FLOAT8),
             CAST(base_price * 0.05 AS FLOAT8), CAST(total_price AS FLOAT8),
             payment_method, tx_hash, created_at, updated_at
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
        client.execute(
            "INSERT INTO bookings
             (id, user_id, flight_id, status, base_price, markup_percentage, total_price, payment_method)
             VALUES ($1, $2, $3, $4,
                     ($5::TEXT)::DECIMAL(15,2),
                     ($6::TEXT)::DECIMAL(5,2),
                     ($7::TEXT)::DECIMAL(15,2),
                     $8)",
            &[&b.id, &b.user_id, &b.flight_id, &status, &base_price, &markup_pct, &total_price, &method],
        ).await.map_err(|e| {
            log::error!("DB save booking error: {}", e);
            InfraError::Database(e.to_string())
        })?;
        Ok(())
    }

    async fn update(&self, b: &Booking) -> Result<(), DomainError> {
        let client = self.client().await?;
        let status = b.status.as_str().to_string();
        let tx_hash = b.tx_hash.as_ref().map(|h| h.as_str().to_string());
        client
            .execute(
                "UPDATE bookings SET status = $1, tx_hash = $2, updated_at = NOW() WHERE id = $3",
                &[&status, &tx_hash, &b.id],
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
