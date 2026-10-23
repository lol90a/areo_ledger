use uuid::Uuid;

use crate::domain::repositories::settlement_repository::{ConfirmOutcome, SettlementRepository};
use crate::infrastructure::db::DbPool;
use crate::shared::errors::{DomainError, InfraError};

pub struct PgSettlementRepository {
    pool: DbPool,
}

impl PgSettlementRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SettlementRepository for PgSettlementRepository {
    async fn confirm_payment(
        &self,
        booking_id: &Uuid,
        tx_hash: &str,
        sender_address: Option<&str>,
    ) -> Result<ConfirmOutcome, DomainError> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;

        // Lock the payment + booking rows for this booking so concurrent confirms
        // (and the reconciliation worker) serialize on the same row.
        let locked = tx
            .query_opt(
                "SELECT b.user_id, b.flight_id, p.status,
                        CAST(p.amount_usd AS FLOAT8)
                 FROM payments p
                 JOIN bookings b ON b.id = p.booking_id
                 WHERE p.booking_id = $1
                 FOR UPDATE OF p, b",
                &[booking_id],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;

        let Some(row) = locked else {
            return Err(DomainError::NotFound(format!(
                "Payment for booking {}",
                booking_id
            )));
        };

        let user_id: Uuid = row.get(0);
        let flight_id: Option<Uuid> = row.get(1);
        let current_status: String = row.get(2);
        let amount_usd: f64 = row.get(3);

        match current_status.as_str() {
            "confirmed" => return Ok(ConfirmOutcome::AlreadyConfirmed),
            "pending" => {}
            other => {
                return Err(DomainError::ValidationError(format!(
                    "Cannot confirm a payment in status '{}'",
                    other
                )))
            }
        }

        // Reject a tx hash already attached to a *different* booking's payment.
        let dup = tx
            .query_opt(
                "SELECT 1 FROM payments WHERE tx_hash = $1 AND booking_id <> $2 LIMIT 1",
                &[&tx_hash, booking_id],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        if dup.is_some() {
            return Err(DomainError::Conflict(
                "This transaction hash has already been used to settle another booking".to_string(),
            ));
        }

        let updated = tx
            .execute(
                "UPDATE payments
                 SET status = 'confirmed', verified = TRUE, verified_at = NOW(),
                     tx_hash = $1, sender_wallet = COALESCE($2, sender_wallet), updated_at = NOW()
                 WHERE booking_id = $3 AND status = 'pending'",
                &[&tx_hash, &sender_address, booking_id],
            )
            .await
            .map_err(|e| {
                // Unique index on tx_hash is the final backstop against replay.
                if e.to_string().contains("duplicate key") {
                    DomainError::Conflict("This transaction hash has already been used".to_string())
                } else {
                    InfraError::Database(e.to_string()).into()
                }
            })?;

        // Someone else won the race between our SELECT and UPDATE.
        if updated == 0 {
            return Ok(ConfirmOutcome::AlreadyConfirmed);
        }

        tx.execute(
            "UPDATE bookings SET status = 'confirmed', tx_hash = $1, updated_at = NOW()
             WHERE id = $2 AND status = 'pending'",
            &[&tx_hash, booking_id],
        )
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

        let asset = if flight_id.is_some() {
            "Flight Booking"
        } else {
            "Marketplace Asset"
        };

        tx.execute(
            "INSERT INTO transactions (id, user_id, type, asset, amount, status, tx_hash, date)
             VALUES ($1, $2, 'buy', $3, ($4::TEXT)::DECIMAL(15,2), 'confirmed', $5, NOW())",
            &[
                &Uuid::new_v4(),
                &user_id,
                &asset,
                &format!("{:.2}", amount_usd),
                &tx_hash,
            ],
        )
        .await
        .map_err(|e| InfraError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;

        Ok(ConfirmOutcome::Confirmed)
    }
}
