use crate::application::dto::ConfirmPaymentInput;
use crate::domain::entities::transaction::{Transaction, TransactionType};
use crate::domain::ports::blockchain_gateway::BlockchainGateway;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::repositories::transaction_repository::TransactionRepository;
use crate::domain::value_objects::blockchain::TxHash;
use crate::shared::errors::DomainError;
use uuid::Uuid;

pub struct ConfirmPayment<BR, PR, TR, BG>
where
    BR: BookingRepository,
    PR: PaymentRepository,
    TR: TransactionRepository,
    BG: BlockchainGateway,
{
    booking_repo: BR,
    payment_repo: PR,
    transaction_repo: TR,
    blockchain: BG,
}

impl<BR, PR, TR, BG> ConfirmPayment<BR, PR, TR, BG>
where
    BR: BookingRepository,
    PR: PaymentRepository,
    TR: TransactionRepository,
    BG: BlockchainGateway,
{
    pub fn new(booking_repo: BR, payment_repo: PR, transaction_repo: TR, blockchain: BG) -> Self {
        Self {
            booking_repo,
            payment_repo,
            transaction_repo,
            blockchain,
        }
    }

    pub async fn execute(&self, input: ConfirmPaymentInput) -> Result<(), DomainError> {
        let tx_hash = TxHash::new(&input.tx_hash)?;

        let mut payment = self
            .payment_repo
            .find_by_booking_id(&input.booking_id)
            .await?;
        let mut booking = self.booking_repo.find_by_id(&input.booking_id).await?;

        let verification = self
            .blockchain
            .verify_transaction(
                &payment.chain,
                tx_hash.as_str(),
                &payment.receiver_address,
                payment.amount.cents(),
            )
            .await?;

        let tolerance = (payment.amount.cents() / 100).max(1);
        let amount_ok = verification.amount_cents >= payment.amount.cents() - tolerance;
        let receiver_ok = verification
            .to_address
            .eq_ignore_ascii_case(&payment.receiver_address)
            || verification.to_address == payment.receiver_address;

        if !receiver_ok || !amount_ok {
            return Err(DomainError::ValidationError(
                "Transaction does not match the expected receiver or amount".to_string(),
            ));
        }

        if !verification.from_address.trim().is_empty() {
            payment.sender_address = Some(verification.from_address.clone());
        }
        payment.tx_hash = Some(tx_hash.clone());

        if !verification.verified {
            self.payment_repo.update(&payment).await?;
            return Err(DomainError::ValidationError(format!(
                "Transaction detected but still pending confirmations: {}",
                verification.confirmations
            )));
        }

        payment.confirm(tx_hash.clone())?;
        booking.confirm(tx_hash.clone())?;

        self.payment_repo.update(&payment).await?;
        self.booking_repo.update(&booking).await?;

        let purchased_item = if booking.flight_id.is_some() {
            "Flight Booking".to_string()
        } else {
            "Marketplace Asset".to_string()
        };

        let transaction = Transaction::new(
            Uuid::new_v4(),
            booking.user_id,
            TransactionType::Buy,
            purchased_item,
            payment.amount.cents() as f64 / 100.0,
            payment.status.as_str().to_string(),
            tx_hash.as_str().to_string(),
            chrono::Utc::now(),
        );
        self.transaction_repo.save(&transaction).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::booking::{Booking, BookingStatus};
    use crate::domain::entities::payment::Payment;
    use crate::domain::ports::blockchain_gateway::TxVerification;
    use crate::domain::repositories::booking_repository::BookingRepository;
    use crate::domain::repositories::payment_repository::PaymentRepository;
    use crate::domain::repositories::transaction_repository::TransactionRepository;
    use crate::domain::value_objects::money::{FiatCurrency, Money, PriceBreakdown};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct BookingRepo {
        booking: Arc<Mutex<Booking>>,
    }

    #[async_trait]
    impl BookingRepository for BookingRepo {
        async fn find_by_id(&self, _: &Uuid) -> Result<Booking, DomainError> {
            Ok(self.booking.lock().unwrap().clone())
        }
        async fn find_all(&self) -> Result<Vec<Booking>, DomainError> {
            Ok(vec![])
        }
        async fn find_by_user_id(&self, _: &Uuid) -> Result<Vec<Booking>, DomainError> {
            Ok(vec![])
        }
        async fn save(&self, _: &Booking) -> Result<(), DomainError> {
            Ok(())
        }
        async fn update(&self, booking: &Booking) -> Result<(), DomainError> {
            *self.booking.lock().unwrap() = booking.clone();
            Ok(())
        }
        async fn count(&self) -> Result<i64, DomainError> {
            Ok(1)
        }
        async fn total_revenue(&self) -> Result<i64, DomainError> {
            Ok(0)
        }
    }

    #[derive(Clone)]
    struct PaymentRepo {
        payment: Arc<Mutex<Payment>>,
    }

    #[async_trait]
    impl PaymentRepository for PaymentRepo {
        async fn find_by_booking_id(&self, _: &Uuid) -> Result<Payment, DomainError> {
            Ok(self.payment.lock().unwrap().clone())
        }
        async fn find_all(&self) -> Result<Vec<Payment>, DomainError> {
            Ok(vec![])
        }
        async fn save(&self, _: &Payment) -> Result<(), DomainError> {
            Ok(())
        }
        async fn update(&self, payment: &Payment) -> Result<(), DomainError> {
            *self.payment.lock().unwrap() = payment.clone();
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct TxRepo {
        saved: Arc<Mutex<Vec<Transaction>>>,
    }

    #[async_trait]
    impl TransactionRepository for TxRepo {
        async fn find_by_user_id(&self, _: &Uuid) -> Result<Vec<Transaction>, DomainError> {
            Ok(vec![])
        }
        async fn save(&self, transaction: &Transaction) -> Result<(), DomainError> {
            self.saved.lock().unwrap().push(transaction.clone());
            Ok(())
        }
    }

    #[derive(Clone)]
    struct Gateway {
        verified: bool,
        confirmations: u32,
    }

    #[async_trait]
    impl BlockchainGateway for Gateway {
        fn receiving_address(&self, _: &str) -> Result<String, DomainError> {
            Ok("0xreceiver".to_string())
        }
        async fn verify_transaction(
            &self,
            _: &str,
            _: &str,
            expected_to: &str,
            expected_amount_cents: i64,
        ) -> Result<TxVerification, DomainError> {
            Ok(TxVerification {
                verified: self.verified,
                amount_cents: expected_amount_cents,
                confirmations: self.confirmations,
                from_address: "0xsender".to_string(),
                to_address: expected_to.to_string(),
            })
        }
    }

    fn sample_booking() -> Booking {
        let now = Utc::now();
        let base = Money::from_cents(100_00, FiatCurrency::Usd).unwrap();
        let pricing = PriceBreakdown::calculate(base).unwrap();
        Booking::from_db(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            BookingStatus::Pending,
            pricing,
            "eth".to_string(),
            None,
            now,
            now,
        )
    }

    #[tokio::test]
    async fn rejects_unverified_transactions() {
        let booking = sample_booking();
        let payment = Payment::new(
            Uuid::new_v4(),
            booking.id,
            "eth",
            booking.pricing.total,
            "0xreceiver".to_string(),
        )
        .unwrap();
        let payment_arc = Arc::new(Mutex::new(payment));
        let use_case = ConfirmPayment::new(
            BookingRepo {
                booking: Arc::new(Mutex::new(booking.clone())),
            },
            PaymentRepo {
                payment: payment_arc.clone(),
            },
            TxRepo::default(),
            Gateway {
                verified: false,
                confirmations: 1,
            },
        );

        let err = use_case
            .execute(ConfirmPaymentInput {
                booking_id: booking.id,
                tx_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                    .to_string(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::ValidationError(_)));
        assert!(payment_arc.lock().unwrap().tx_hash.is_some());
    }
}
