use uuid::Uuid;

use crate::domain::entities::payment::{Payment, PaymentStatus};
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::value_objects::blockchain::TxHash;
use crate::domain::value_objects::money::{FiatCurrency, Money};
use crate::infrastructure::db::DbPool;
use crate::shared::errors::{DomainError, InfraError};

pub struct PgPaymentRepository {
    pool: DbPool,
}

impl PgPaymentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    async fn client(&self) -> Result<deadpool_postgres::Client, DomainError> {
        self.pool
            .get()
            .await
            .map_err(|e| InfraError::Database(e.to_string()).into())
    }

    fn row_to_payment(row: &tokio_postgres::Row) -> Result<Payment, DomainError> {
        let amount_f: f64 = row.get(4);
        let amount = Money::from_cents((amount_f * 100.0) as i64, FiatCurrency::Usd)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let tx_raw: Option<String> = row.get(7);
        let tx_hash = tx_raw.and_then(|h| TxHash::new(h).ok());
        let created = row.get::<_, chrono::NaiveDateTime>(9);

        Ok(Payment {
            id: row.get(0),
            booking_id: row.get(1),
            chain: row.get(2),
            token: row.get(3),
            amount,
            sender_address: row.get(5),
            receiver_address: row.get(6),
            tx_hash,
            status: PaymentStatus::from_str(row.get::<_, &str>(8)),
            created_at: chrono::DateTime::from_naive_utc_and_offset(created, chrono::Utc),
        })
    }

    pub async fn attach_proof(
        &self,
        booking_id: &Uuid,
        proof_path: &str,
        content_type: Option<&str>,
    ) -> Result<(), DomainError> {
        let client = self.client().await?;
        let updated = client
            .execute(
                "UPDATE payments
                 SET payment_proof_path = $1,
                     payment_proof_content_type = $2,
                     payment_proof_uploaded_at = NOW(),
                     updated_at = NOW()
                 WHERE booking_id = $3",
                &[&proof_path, &content_type, booking_id],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;

        if updated == 0 {
            return Err(DomainError::NotFound(format!(
                "Payment for booking {}",
                booking_id
            )));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl PaymentRepository for PgPaymentRepository {
    async fn find_by_booking_id(&self, booking_id: &Uuid) -> Result<Payment, DomainError> {
        let client = self.client().await?;
        let row = client
            .query_one(
                "SELECT id, booking_id, chain, token, CAST(amount AS FLOAT8),
                 sender_wallet, wallet_address, tx_hash, status, created_at
                 FROM payments WHERE booking_id = $1",
                &[booking_id],
            )
            .await
            .map_err(|_| DomainError::NotFound(format!("Payment for booking {}", booking_id)))?;
        Self::row_to_payment(&row)
    }

    async fn find_all(&self) -> Result<Vec<Payment>, DomainError> {
        let client = self.client().await?;
        let rows = client
            .query(
                "SELECT id, booking_id, chain, token, CAST(amount AS FLOAT8),
                 sender_wallet, wallet_address, tx_hash, status, created_at
                 FROM payments ORDER BY created_at DESC",
                &[],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        rows.iter().map(Self::row_to_payment).collect()
    }

    async fn save(&self, p: &Payment) -> Result<(), DomainError> {
        let client = self.client().await?;
        let amount = p.amount.to_decimal_str();
        let status = p.status.as_str().to_string();
        let result = client.execute(
            "INSERT INTO payments (id, booking_id, chain, token, amount, amount_usd, wallet_address, status)
             VALUES ($1, $2, $3, $4,
                     ($5::TEXT)::DECIMAL(20,8),
                     ($5::TEXT)::DECIMAL(15,2),
                     $6, $7)",
            &[&p.id, &p.booking_id, &p.chain, &p.token, &amount, &p.receiver_address, &status],
        ).await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.to_string().contains("duplicate key") {
                    Err(DomainError::Conflict(
                        "Payment already exists for this booking".to_string(),
                    ))
                } else {
                    log::error!("DB save payment error: {}", e);
                    Err(InfraError::Database(e.to_string()).into())
                }
            }
        }
    }

    async fn update(&self, p: &Payment) -> Result<(), DomainError> {
        let client = self.client().await?;
        let tx = p.tx_hash.as_ref().map(|h| h.as_str().to_string());
        let status = p.status.as_str().to_string();
        client.execute(
            "UPDATE payments
             SET sender_wallet = $1, tx_hash = $2, status = $3, verified = $4, verified_at = NOW(), updated_at = NOW()
             WHERE booking_id = $5",
            &[&p.sender_address, &tx, &status, &(p.status == PaymentStatus::Confirmed), &p.booking_id],
        ).await.map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(())
    }
}
