use uuid::Uuid;

use crate::domain::entities::payment::Payment;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::ports::blockchain_gateway::BlockchainGateway;
use crate::application::dto::{InitPaymentInput, InitPaymentOutput};
use crate::shared::errors::DomainError;

pub struct InitPayment<BR, PR, BG>
where BR: BookingRepository, PR: PaymentRepository, BG: BlockchainGateway,
{
    booking_repo: BR,
    payment_repo: PR,
    blockchain:   BG,
}

impl<BR, PR, BG> InitPayment<BR, PR, BG>
where BR: BookingRepository, PR: PaymentRepository, BG: BlockchainGateway,
{
    pub fn new(booking_repo: BR, payment_repo: PR, blockchain: BG) -> Self {
        Self { booking_repo, payment_repo, blockchain }
    }

    pub async fn execute(&self, input: InitPaymentInput) -> Result<InitPaymentOutput, DomainError> {
        let token = match input.method.as_str() {
            "btc" => "BTC",
            "eth" => "ETH",
            "usdt" => "USDT",
            "usdc" => "USDC",
            "sol" => "SOL",
            "binance" => "BNB",
            other => return Err(DomainError::ValidationError(
                format!("Unsupported payment method: '{}'", other)
            )),
        };

        let booking = self.booking_repo.find_by_id(&input.booking_id).await?;
        if booking.payment_method != input.method {
            return Err(DomainError::ValidationError("Payment method does not match the booking".to_string()));
        }
        if self.payment_repo.find_by_booking_id(&input.booking_id).await.is_ok() {
            return Err(DomainError::Conflict("Payment already initialized for this booking".to_string()));
        }

        let wallet_address = self.blockchain.receiving_address(token)?;
        let payment = Payment::new(
            Uuid::new_v4(), booking.id,
            &input.method, booking.pricing.total,
            wallet_address.clone(),
        )?;

        let amount = payment.amount.cents() as f64 / 100.0;
        self.payment_repo.save(&payment).await?;

        Ok(InitPaymentOutput { wallet_address, amount })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::booking::Booking;
    use crate::domain::entities::payment::Payment;
    use crate::domain::entities::booking::BookingStatus;
    use crate::domain::repositories::booking_repository::BookingRepository;
    use crate::domain::repositories::payment_repository::PaymentRepository;
    use crate::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification};
    use crate::domain::value_objects::money::{Money, FiatCurrency, PriceBreakdown};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct BookingRepo { booking: Booking }

    #[async_trait]
    impl BookingRepository for BookingRepo {
        async fn find_by_id(&self, _: &Uuid) -> Result<Booking, DomainError> { Ok(self.booking.clone()) }
        async fn find_all(&self) -> Result<Vec<Booking>, DomainError> { Ok(vec![self.booking.clone()]) }
        async fn find_by_user_id(&self, _: &Uuid) -> Result<Vec<Booking>, DomainError> { Ok(vec![self.booking.clone()]) }
        async fn save(&self, _: &Booking) -> Result<(), DomainError> { Ok(()) }
        async fn update(&self, _: &Booking) -> Result<(), DomainError> { Ok(()) }
        async fn count(&self) -> Result<i64, DomainError> { Ok(1) }
        async fn total_revenue(&self) -> Result<i64, DomainError> { Ok(0) }
    }

    #[derive(Clone, Default)]
    struct PaymentRepo { saved: Arc<Mutex<Vec<Payment>>>, existing: Option<Payment> }

    #[async_trait]
    impl PaymentRepository for PaymentRepo {
        async fn find_by_booking_id(&self, _: &Uuid) -> Result<Payment, DomainError> {
            self.existing.clone().ok_or_else(|| DomainError::NotFound("missing".to_string()))
        }
        async fn find_all(&self) -> Result<Vec<Payment>, DomainError> { Ok(vec![]) }
        async fn save(&self, payment: &Payment) -> Result<(), DomainError> {
            self.saved.lock().unwrap().push(payment.clone());
            Ok(())
        }
        async fn update(&self, _: &Payment) -> Result<(), DomainError> { Ok(()) }
    }

    #[derive(Clone)]
    struct Gateway;

    #[async_trait]
    impl BlockchainGateway for Gateway {
        fn receiving_address(&self, token: &str) -> Result<String, DomainError> { Ok(format!("wallet-{}", token)) }
        async fn verify_transaction(&self, _: &str, _: &str, _: &str, _: i64) -> Result<TxVerification, DomainError> {
            unreachable!()
        }
    }

    fn sample_booking(method: &str) -> Booking {
        let now = Utc::now();
        let base = Money::from_cents(100_00, FiatCurrency::Usd).unwrap();
        let pricing = PriceBreakdown::calculate(base).unwrap();
        Booking::from_db(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            BookingStatus::Pending,
            pricing,
            method.to_string(),
            None,
            now,
            now,
        )
    }

    #[tokio::test]
    async fn rejects_duplicate_payment_initialization() {
        let booking = sample_booking("eth");
        let existing = Payment::new(Uuid::new_v4(), booking.id, "eth", booking.pricing.total, "0xreceiver".to_string()).unwrap();
        let use_case = InitPayment::new(
            BookingRepo { booking: booking.clone() },
            PaymentRepo { saved: Arc::new(Mutex::new(vec![])), existing: Some(existing) },
            Gateway,
        );

        let err = use_case.execute(InitPaymentInput { booking_id: booking.id, method: "eth".to_string() }).await.unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }
}
