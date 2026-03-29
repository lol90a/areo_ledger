use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::domain::ports::blockchain_gateway::BlockchainGateway;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::repositories::transaction_repository::TransactionRepository;
use crate::domain::entities::transaction::{Transaction, TransactionType};
use crate::infrastructure::blockchain::router_gateway::MultiChainGateway;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::persistence::postgres::pg_audit_log_repository::PgAuditLogRepository;
use crate::infrastructure::persistence::postgres::pg_booking_repository::PgBookingRepository;
use crate::infrastructure::persistence::postgres::pg_payment_repository::PgPaymentRepository;
use crate::infrastructure::persistence::postgres::pg_transaction_repository::PgTransactionRepository;
use crate::shared::errors::DomainError;

#[derive(Clone)]
pub struct PaymentReconciliationWorker {
    pool: DbPool,
    gateway: MultiChainGateway,
    interval_seconds: u64,
}

impl PaymentReconciliationWorker {
    pub fn new(pool: DbPool, gateway: MultiChainGateway, interval_seconds: u64) -> Self {
        Self { pool, gateway, interval_seconds }
    }

    pub fn spawn(self) {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.run_once().await {
                    log::error!("payment reconciliation failed: {}", e);
                }
                sleep(Duration::from_secs(self.interval_seconds)).await;
            }
        });
    }

    pub async fn run_once(&self) -> Result<(), DomainError> {
        let booking_repo = PgBookingRepository::new(self.pool.clone());
        let payment_repo = PgPaymentRepository::new(self.pool.clone());
        let tx_repo = PgTransactionRepository::new(self.pool.clone());
        let audit_repo = PgAuditLogRepository::new(self.pool.clone());

        let payments = payment_repo.find_all().await?;
        for mut payment in payments.into_iter().filter(|p| p.status.as_str() == "pending" && p.tx_hash.is_some()) {
            let verification = self.gateway.verify_transaction(
                &payment.chain,
                payment.tx_hash.as_ref().map(|h| h.as_str()).unwrap_or_default(),
                &payment.receiver_address,
                payment.amount.cents(),
            ).await?;

            let tolerance = (payment.amount.cents() / 100).max(1);
            let amount_ok = verification.amount_cents >= payment.amount.cents() - tolerance;
            let receiver_ok = verification.to_address.eq_ignore_ascii_case(&payment.receiver_address) || verification.to_address == payment.receiver_address;
            if !(receiver_ok && amount_ok && verification.verified) {
                continue;
            }

            let mut booking = booking_repo.find_by_id(&payment.booking_id).await?;
            if booking.status.as_str() == "confirmed" && payment.status.as_str() == "confirmed" {
                continue;
            }

            if !verification.from_address.trim().is_empty() {
                payment.sender_address = Some(verification.from_address.clone());
            }
            let tx_hash = payment.tx_hash.clone().ok_or_else(|| DomainError::ValidationError("Missing tx hash on pending payment".to_string()))?;
            payment.confirm(tx_hash.clone())?;
            booking.confirm(tx_hash.clone())?;

            payment_repo.update(&payment).await?;
            booking_repo.update(&booking).await?;

            tx_repo.save(&Transaction::new(
                Uuid::new_v4(),
                booking.user_id,
                TransactionType::Buy,
                "Flight Booking".to_string(),
                payment.amount.cents() as f64 / 100.0,
                payment.status.as_str().to_string(),
                tx_hash.as_str().to_string(),
                chrono::Utc::now(),
            )).await?;

            let _ = audit_repo.record(
                Some(booking.user_id),
                "payment.reconciled",
                "payment",
                &payment.id.to_string(),
                serde_json::json!({
                    "booking_id": booking.id,
                    "tx_hash": tx_hash.as_str(),
                    "confirmations": verification.confirmations,
                }),
                None,
                Some("payment_reconciliation_worker".to_string()),
            ).await;
        }

        Ok(())
    }
}
