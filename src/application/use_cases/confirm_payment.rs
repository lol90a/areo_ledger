use crate::application::dto::ConfirmPaymentInput;
use crate::domain::entities::payment::PaymentStatus;
use crate::domain::ports::blockchain_gateway::{BlockchainGateway, VerifyRequest};
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::repositories::settlement_repository::SettlementRepository;
use crate::domain::value_objects::blockchain::TxHash;
use crate::domain::value_objects::CryptoToken;
use crate::shared::errors::DomainError;

/// Render a base-unit amount as a human token figure, e.g. 97486785679 lamports
/// at 9 decimals becomes "97.486785679".
///
/// Integer-only: the amounts involved exceed f64's exact range, and a payment
/// message that misreports what someone sent is worse than no message.
fn format_base_units(units: u128, decimals: u8) -> String {
    let scale = 10u128.pow(decimals as u32);
    let whole = units / scale;
    if decimals == 0 {
        return whole.to_string();
    }
    let frac = format!("{:0width$}", units % scale, width = decimals as usize);
    let frac = frac.trim_end_matches('0');
    if frac.is_empty() {
        whole.to_string()
    } else {
        format!("{}.{}", whole, frac)
    }
}

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
            // Record the submitted hash but keep the payment pending: the buyer may
            // still send a correct transaction, which arrives under a different hash.
            let mut pending = payment.clone();
            if !verification.from_address.trim().is_empty() {
                pending.sender_address = Some(verification.from_address.clone());
            }
            pending.tx_hash = Some(tx_hash.clone());
            self.payment_repo.update(&pending).await?;

            // Say which of the three it is. These used to share one message --
            // "not verified yet ... it will settle automatically once valid" --
            // which told someone who had underpaid, or paid the wrong address
            // entirely, to sit and wait for a settlement that could never come.
            let message = if !verification.observed {
                format!(
                    "Transaction not confirmed yet (confirmations: {}). \
                     It will settle automatically once it is final on-chain.",
                    verification.confirmations
                )
            } else if !receiver_ok {
                format!(
                    "This transaction paid {}, but this order must be paid to {}. \
                     Funds sent elsewhere cannot be recovered by this platform, and \
                     this transaction will never settle the order.",
                    if verification.to_address.trim().is_empty() {
                        "a different address"
                    } else {
                        verification.to_address.trim()
                    },
                    payment.receiver_address
                )
            } else {
                format!(
                    "This transaction paid {} {}, short of the {} this order requires, \
                     so it will never settle it. Send the full amount in a single \
                     transaction.",
                    format_base_units(verification.base_units, decimals),
                    token.as_str(),
                    format_base_units(expected_base_units, decimals),
                )
            };
            return Err(DomainError::ValidationError(message));
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
        /// Defaults to matching `verified`; set independently to model a
        /// transaction that IS final but paid the wrong amount or address.
        observed: Option<bool>,
        /// Overrides the recipient the gateway reports seeing.
        to_address: Option<String>,
    }

    impl Gateway {
        fn new(verified: bool, base_units: u128) -> Self {
            Self {
                verified,
                base_units,
                observed: None,
                to_address: None,
            }
        }
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
            let observed = self.observed.unwrap_or(self.verified);
            Ok(TxVerification {
                verified: self.verified,
                observed,
                base_units: self.base_units,
                confirmations: if observed { 20 } else { 1 },
                from_address: "0xSender".to_string(),
                to_address: self
                    .to_address
                    .clone()
                    .unwrap_or_else(|| req.expected_to.clone()),
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
            Gateway::new(true, expected),
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
            Gateway::new(false, 0),
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
            Gateway::new(true, payment.crypto_base_units),
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
            Gateway::new(true, payment.crypto_base_units - 1),
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

        // The message must not promise this will settle on its own. It cannot:
        // the transaction is final and short.
        let text = format!("{}", err);
        assert!(text.contains("never settle"), "{}", text);
        assert!(
            !text.contains("automatically"),
            "a short payment was described as pending: {}",
            text
        );
    }

    #[tokio::test]
    async fn a_payment_to_the_wrong_address_says_so_rather_than_pending() {
        let payment = eth_payment();
        let settlement = Settlement::default();
        let mut gateway = Gateway::new(false, payment.crypto_base_units);
        gateway.observed = Some(true); // final on-chain, just to the wrong place
        gateway.to_address = Some("0xSomebodyElse".to_string());

        let use_case = ConfirmPayment::new(
            PaymentRepo {
                payment: Arc::new(Mutex::new(payment.clone())),
                by_hash: None,
            },
            settlement.clone(),
            gateway,
        );

        let err = use_case
            .execute(ConfirmPaymentInput {
                booking_id: payment.booking_id,
                tx_hash: TX.to_string(),
            })
            .await
            .unwrap_err();

        let text = format!("{}", err);
        assert!(text.contains("0xSomebodyElse"), "{}", text);
        assert!(text.contains("never settle"), "{}", text);
        assert_eq!(*settlement.calls.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn a_transaction_that_is_merely_unconfirmed_is_reported_as_pending() {
        // The trap in classifying failures: an unconfirmed transaction has no
        // amount yet, so it must NOT be reported as an underpayment.
        let payment = eth_payment();
        let settlement = Settlement::default();
        let use_case = ConfirmPayment::new(
            PaymentRepo {
                payment: Arc::new(Mutex::new(payment.clone())),
                by_hash: None,
            },
            settlement.clone(),
            Gateway::new(false, 0),
        );

        let err = use_case
            .execute(ConfirmPaymentInput {
                booking_id: payment.booking_id,
                tx_hash: TX.to_string(),
            })
            .await
            .unwrap_err();

        let text = format!("{}", err);
        assert!(text.contains("not confirmed yet"), "{}", text);
        assert!(
            !text.contains("never settle"),
            "a pending transaction was called permanently failed: {}",
            text
        );
        assert_eq!(*settlement.calls.lock().unwrap(), 0);
    }

    #[test]
    fn base_units_render_as_token_amounts() {
        assert_eq!(format_base_units(97_486_785_679, 9), "97.486785679");
        assert_eq!(format_base_units(1_000_000_000, 9), "1");
        assert_eq!(format_base_units(0, 9), "0");
        assert_eq!(format_base_units(1, 9), "0.000000001");
        assert_eq!(format_base_units(12_345, 0), "12345");
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
            Gateway::new(true, payment.crypto_base_units + 1_000),
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
