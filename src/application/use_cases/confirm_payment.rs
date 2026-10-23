use crate::application::dto::ConfirmPaymentInput;
use crate::domain::entities::payment::PaymentStatus;
use crate::domain::ports::blockchain_gateway::{BlockchainGateway, VerifyRequest};
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::repositories::settlement_repository::SettlementRepository;
use crate::domain::value_objects::blockchain::TxHash;
use crate::domain::value_objects::CryptoToken;
use crate::shared::errors::DomainError;

pub struct ConfirmPayment<PR, SR, BG>
where
    PR: PaymentRepository,
    SR: SettlementRepository,
    BG: BlockchainGateway,
{
    payment_repo: PR,
    settlement: SR,
    blockchain: BG,
}

impl<PR, SR, BG> ConfirmPayment<PR, SR, BG>
where
    PR: PaymentRepository,
    SR: SettlementRepository,
    BG: BlockchainGateway,
{
    pub fn new(payment_repo: PR, settlement: SR, blockchain: BG) -> Self {
        Self {
            payment_repo,
            settlement,
            blockchain,
        }
    }

    pub async fn execute(&self, input: ConfirmPaymentInput) -> Result<(), DomainError> {
        let tx_hash = TxHash::new(&input.tx_hash)?;
        let payment = self
            .payment_repo
            .find_by_booking_id(&input.booking_id)
            .await?;

        // Already settled — idempotent success.
        if payment.status == PaymentStatus::Confirmed {
            return Ok(());
        }

        // Replay guard: this hash must not already belong to a different booking.
        if let Some(existing) = self.payment_repo.find_by_tx_hash(tx_hash.as_str()).await? {
            if existing.booking_id != payment.booking_id {
                return Err(DomainError::Conflict(
                    "This transaction hash has already been used to settle another booking"
                        .to_string(),
                ));
            }
        }

        let token = CryptoToken::from_str(&payment.token).ok_or_else(|| {
            DomainError::InternalError(format!("Unknown payment token '{}'", payment.token))
        })?;
        // The authoritative figure is the integer base-unit amount locked when the
        // user accepted the quote. It is never re-derived from a decimal here, and
        // never compared against a fiat value.
        let decimals = payment.token_decimals;
        let expected_base_units = payment.crypto_base_units;

        let request = VerifyRequest {
            chain: payment.chain.clone(),
            token: payment.token.clone(),
            tx_hash: tx_hash.as_str().to_string(),
            expected_to: payment.receiver_address.clone(),
            expected_base_units,
            decimals,
            token_contract: token.erc20_contract().map(|s| s.to_string()),
        };

        let verification = self.blockchain.verify_transaction(&request).await?;

        let receiver_ok = verification
            .to_address
            .eq_ignore_ascii_case(&payment.receiver_address);
        // Exact-or-more. The quoted amount was already rounded UP to the token's
        // precision when the quote was issued, so the payer knows the precise figure
        // and there is no rounding slack to concede. Accepting even a fraction under
        // the quote is a direct revenue leak on every order.
        let amount_ok = verification.base_units >= expected_base_units;

        if !verification.verified || !receiver_ok || !amount_ok {
            // Record the submitted hash but keep the payment pending for retry/reconciliation.
            let mut pending = payment;
            if !verification.from_address.trim().is_empty() {
                pending.sender_address = Some(verification.from_address.clone());
            }
            pending.tx_hash = Some(tx_hash.clone());
            self.payment_repo.update(&pending).await?;
            return Err(DomainError::ValidationError(format!(
                "Transaction not verified yet (confirmations: {}). It will settle automatically once valid.",
                verification.confirmations
            )));
        }

        let sender = (!verification.from_address.trim().is_empty())
            .then(|| verification.from_address.clone());
        self.settlement
            .confirm_payment(&payment.booking_id, tx_hash.as_str(), sender.as_deref())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::payment::Payment;
    use crate::domain::entities::quote::{PaymentQuote, QuoteStatus};
    use crate::domain::ports::blockchain_gateway::TxVerification;
    use crate::domain::repositories::settlement_repository::ConfirmOutcome;
    use crate::domain::value_objects::money::{FiatCurrency, Money, PriceBreakdown};
    use crate::domain::value_objects::Chain;
    use async_trait::async_trait;
    use rust_decimal::Decimal;
    use std::str::FromStr as _;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    /// 0.05 ETH, i.e. 50_000_000_000_000_000 wei.
    const EXPECTED_WEI: u128 = 50_000_000_000_000_000;

    fn eth_payment() -> Payment {
        let now = chrono::Utc::now();
        let pricing =
            PriceBreakdown::calculate(Money::from_cents(100_00, FiatCurrency::Usd).unwrap())
                .unwrap();
        let mut quote = PaymentQuote::issue(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "eth",
            Chain::Ethereum,
            CryptoToken::Eth,
            pricing,
            Decimal::from(2300),
            "test".to_string(),
            now,
            Decimal::from_str("0.05").unwrap(),
            EXPECTED_WEI,
            "0xReceiver".to_string(),
            now,
        )
        .unwrap();
        quote.status = QuoteStatus::Accepted;
        quote.accepted_at = Some(now);
        Payment::from_quote(Uuid::new_v4(), &quote).unwrap()
    }

    #[derive(Clone)]
    struct PaymentRepo {
        payment: Arc<Mutex<Payment>>,
        by_hash: Option<Payment>,
    }

    #[async_trait]
    impl PaymentRepository for PaymentRepo {
        async fn find_by_booking_id(&self, _: &Uuid) -> Result<Payment, DomainError> {
            Ok(self.payment.lock().unwrap().clone())
        }
        async fn find_all(&self) -> Result<Vec<Payment>, DomainError> {
            Ok(vec![])
        }
        async fn find_pending_with_tx(&self) -> Result<Vec<Payment>, DomainError> {
            Ok(vec![])
        }
        async fn find_by_tx_hash(&self, _: &str) -> Result<Option<Payment>, DomainError> {
            Ok(self.by_hash.clone())
        }
        async fn save(&self, _: &Payment) -> Result<(), DomainError> {
            Ok(())
        }
        async fn update(&self, p: &Payment) -> Result<(), DomainError> {
            *self.payment.lock().unwrap() = p.clone();
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct Settlement {
        calls: Arc<Mutex<u32>>,
    }

    #[async_trait]
    impl SettlementRepository for Settlement {
        async fn confirm_payment(
            &self,
            _: &Uuid,
            _: &str,
            _: Option<&str>,
        ) -> Result<ConfirmOutcome, DomainError> {
            *self.calls.lock().unwrap() += 1;
            Ok(ConfirmOutcome::Confirmed)
        }
    }

    #[derive(Clone)]
    struct Gateway {
        verified: bool,
        base_units: u128,
    }

    #[async_trait]
    impl BlockchainGateway for Gateway {
        fn receiving_address(&self, _: &str) -> Result<String, DomainError> {
            Ok("0xReceiver".to_string())
        }
        async fn verify_transaction(
            &self,
            req: &VerifyRequest,
        ) -> Result<TxVerification, DomainError> {
            Ok(TxVerification {
                verified: self.verified,
                base_units: self.base_units,
                confirmations: if self.verified { 20 } else { 1 },
                from_address: "0xSender".to_string(),
                to_address: req.expected_to.clone(),
            })
        }
    }

    const TX: &str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

    #[tokio::test]
    async fn settles_a_verified_payment() {
        let payment = eth_payment();
        let settlement = Settlement::default();
        let expected = payment.crypto_base_units;
        let use_case = ConfirmPayment::new(
            PaymentRepo {
                payment: Arc::new(Mutex::new(payment.clone())),
                by_hash: None,
            },
            settlement.clone(),
            Gateway {
                verified: true,
                base_units: expected,
            },
        );

        use_case
            .execute(ConfirmPaymentInput {
                booking_id: payment.booking_id,
                tx_hash: TX.to_string(),
            })
            .await
            .unwrap();
        assert_eq!(*settlement.calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn rejects_unverified_transactions() {
        let payment = eth_payment();
        let settlement = Settlement::default();
        let use_case = ConfirmPayment::new(
            PaymentRepo {
                payment: Arc::new(Mutex::new(payment.clone())),
                by_hash: None,
            },
            settlement.clone(),
            Gateway {
                verified: false,
                base_units: 0,
            },
        );

        let err = use_case
            .execute(ConfirmPaymentInput {
                booking_id: payment.booking_id,
                tx_hash: TX.to_string(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::ValidationError(_)));
        assert_eq!(*settlement.calls.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn rejects_reused_tx_hash_from_another_booking() {
        let payment = eth_payment();
        let mut other = eth_payment();
        other.booking_id = Uuid::new_v4();
        let settlement = Settlement::default();
        let use_case = ConfirmPayment::new(
            PaymentRepo {
                payment: Arc::new(Mutex::new(payment.clone())),
                by_hash: Some(other),
            },
            settlement.clone(),
            Gateway {
                verified: true,
                base_units: payment.crypto_base_units,
            },
        );

        let err = use_case
            .execute(ConfirmPaymentInput {
                booking_id: payment.booking_id,
                tx_hash: TX.to_string(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
        assert_eq!(*settlement.calls.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn rejects_a_payment_one_base_unit_short_of_the_quote() {
        // The gateway reports a successful transfer, but for 1 wei less than the
        // amount the user accepted. Settling it would leak revenue on every order.
        let payment = eth_payment();
        let settlement = Settlement::default();
        let use_case = ConfirmPayment::new(
            PaymentRepo {
                payment: Arc::new(Mutex::new(payment.clone())),
                by_hash: None,
            },
            settlement.clone(),
            Gateway {
                verified: true,
                base_units: payment.crypto_base_units - 1,
            },
        );

        let err = use_case
            .execute(ConfirmPaymentInput {
                booking_id: payment.booking_id,
                tx_hash: TX.to_string(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::ValidationError(_)));
        assert_eq!(*settlement.calls.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn accepts_an_overpayment() {
        let payment = eth_payment();
        let settlement = Settlement::default();
        let use_case = ConfirmPayment::new(
            PaymentRepo {
                payment: Arc::new(Mutex::new(payment.clone())),
                by_hash: None,
            },
            settlement.clone(),
            Gateway {
                verified: true,
                base_units: payment.crypto_base_units + 1_000,
            },
        );

        use_case
            .execute(ConfirmPaymentInput {
                booking_id: payment.booking_id,
                tx_hash: TX.to_string(),
            })
            .await
            .unwrap();
        assert_eq!(*settlement.calls.lock().unwrap(), 1);
    }
}
