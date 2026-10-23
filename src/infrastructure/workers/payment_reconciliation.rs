use tokio::time::{sleep, Duration};

use crate::domain::ports::blockchain_gateway::{BlockchainGateway, VerifyRequest};
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::repositories::quote_repository::QuoteRepository;
use crate::domain::repositories::settlement_repository::SettlementRepository;
use crate::domain::value_objects::CryptoToken;
use crate::infrastructure::blockchain::router_gateway::MultiChainGateway;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::persistence::postgres::pg_audit_log_repository::PgAuditLogRepository;
use crate::infrastructure::persistence::postgres::pg_payment_repository::PgPaymentRepository;
use crate::infrastructure::persistence::postgres::pg_quote_repository::PgQuoteRepository;
use crate::infrastructure::persistence::postgres::pg_settlement_repository::PgSettlementRepository;
use crate::shared::errors::DomainError;

#[derive(Clone)]
pub struct PaymentReconciliationWorker {
    pool: DbPool,
    gateway: MultiChainGateway,
    interval_seconds: u64,
}

impl PaymentReconciliationWorker {
    pub fn new(pool: DbPool, gateway: MultiChainGateway, interval_seconds: u64) -> Self {
        Self {
            pool,
            gateway,
            interval_seconds,
        }
    }

    pub fn spawn(self) {
        // Manual verification mode has no on-chain source of truth, so auto-confirming
        // pending payments would rubber-stamp unpaid bookings. Only reconcile in live mode.
        if self.gateway.is_manual() {
            log::info!("payment reconciliation disabled (manual verification mode)");
            return;
        }
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.run_once().await {
                    log::error!("payment reconciliation cycle failed: {}", e);
                }
                sleep(Duration::from_secs(self.interval_seconds)).await;
            }
        });
    }

    pub async fn run_once(&self) -> Result<(), DomainError> {
        let payment_repo = PgPaymentRepository::new(self.pool.clone());
        let settlement = PgSettlementRepository::new(self.pool.clone());
        let audit_repo = PgAuditLogRepository::new(self.pool.clone());

        // Housekeeping: move timed-out quotes out of `active`. This is tidiness, not
        // a security control -- acceptance checks the clock itself, so an un-swept
        // row can never be accepted late.
        match PgQuoteRepository::new(self.pool.clone())
            .expire_stale()
            .await
        {
            Ok(n) if n > 0 => log::info!("expired {} stale payment quote(s)", n),
            Ok(_) => {}
            Err(e) => log::warn!("quote expiry sweep failed: {}", e),
        }

        let payments = payment_repo.find_pending_with_tx().await?;
        for payment in payments {
            // Isolate per-payment failures so one flaky provider call can't starve the batch.
            if let Err(e) = self.reconcile_one(&payment, &settlement, &audit_repo).await {
                log::warn!(
                    "reconciliation skipped payment for booking {}: {}",
                    payment.booking_id,
                    e
                );
            }
        }

        Ok(())
    }

    async fn reconcile_one(
        &self,
        payment: &crate::domain::entities::payment::Payment,
        settlement: &PgSettlementRepository,
        audit_repo: &PgAuditLogRepository,
    ) -> Result<(), DomainError> {
        let tx_hash = match payment.tx_hash.as_ref() {
            Some(h) => h.as_str().to_string(),
            None => return Ok(()),
        };

        let token = CryptoToken::from_str(&payment.token).ok_or_else(|| {
            DomainError::InternalError(format!("Unknown payment token '{}'", payment.token))
        })?;
        // Same authoritative source as the synchronous confirm path: the base-unit
        // amount locked at quote acceptance.
        let decimals = payment.token_decimals;
        let expected_base_units = payment.crypto_base_units;

        let request = VerifyRequest {
            chain: payment.chain.clone(),
            token: payment.token.clone(),
            tx_hash: tx_hash.clone(),
            expected_to: payment.receiver_address.clone(),
            expected_base_units,
            decimals,
            token_contract: token.erc20_contract().map(|s| s.to_string()),
        };

        let verification = self.gateway.verify_transaction(&request).await?;
        let receiver_ok = verification
            .to_address
            .eq_ignore_ascii_case(&payment.receiver_address);
        let amount_ok = verification.base_units >= expected_base_units;

        if !(verification.verified && receiver_ok && amount_ok) {
            return Ok(());
        }

        let sender = (!verification.from_address.trim().is_empty())
            .then(|| verification.from_address.clone());
        let outcome = settlement
            .confirm_payment(&payment.booking_id, &tx_hash, sender.as_deref())
            .await?;

        if matches!(
            outcome,
            crate::domain::repositories::settlement_repository::ConfirmOutcome::Confirmed
        ) {
            let _ = audit_repo
                .record(
                    None,
                    "payment.reconciled",
                    "payment",
                    &payment.id.to_string(),
                    serde_json::json!({
                        "booking_id": payment.booking_id,
                        "tx_hash": tx_hash,
                        "confirmations": verification.confirmations,
                    }),
                    None,
                    Some("payment_reconciliation_worker".to_string()),
                )
                .await;
        }

        Ok(())
    }
}
