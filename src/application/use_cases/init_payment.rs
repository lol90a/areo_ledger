//! Turn an **accepted** quote into a pending payment.
//!
//! This use case deliberately carries no pricing logic of its own. Every number it
//! records -- the fiat total, the token, the exact base units, the receiving address
//! -- is copied from a quote the user already approved. That is what makes the price
//! the user saw and the amount the chain is checked against the same number, and it
//! is why a client can no longer influence either: there is no amount in the request.

use uuid::Uuid;

use crate::application::dto::{InitPaymentInput, InitPaymentOutput};
use crate::domain::entities::payment::Payment;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::repositories::quote_repository::QuoteRepository;
use crate::shared::errors::DomainError;

pub struct InitPayment<BR, PR, QR>
where
    BR: BookingRepository,
    PR: PaymentRepository,
    QR: QuoteRepository,
{
    booking_repo: BR,
    payment_repo: PR,
    quotes: QR,
}

impl<BR, PR, QR> InitPayment<BR, PR, QR>
where
    BR: BookingRepository,
    PR: PaymentRepository,
    QR: QuoteRepository,
{
    pub fn new(booking_repo: BR, payment_repo: PR, quotes: QR) -> Self {
        Self {
            booking_repo,
            payment_repo,
            quotes,
        }
    }

    pub async fn execute(&self, input: InitPaymentInput) -> Result<InitPaymentOutput, DomainError> {
        let booking = self.booking_repo.find_by_id(&input.booking_id).await?;

        // One payment per booking. The DB also enforces this with a unique index;
        // checking here turns the race into a clean 409 for the common case.
        if self
            .payment_repo
            .find_by_booking_id(&input.booking_id)
            .await
            .is_ok()
        {
            return Err(DomainError::Conflict(
                "Payment already initialized for this booking".to_string(),
            ));
        }

        let quote = self
            .quotes
            .find_accepted_for_booking(&input.booking_id)
            .await?
            .ok_or_else(|| {
                DomainError::Conflict(
                    "No accepted price quote for this booking. Request a quote and approve it before paying."
                        .to_string(),
                )
            })?;

        if !quote.is_settleable() {
            return Err(DomainError::Conflict(
                "The quote for this booking is no longer settleable; request a fresh price"
                    .to_string(),
            ));
        }
        if quote.user_id != booking.user_id {
            return Err(DomainError::Unauthorized);
        }
        // The approved fiat figure and the booking must still agree. If they do not,
        // something re-priced the booking after acceptance and settling would charge
        // an amount the user never approved.
        if quote.pricing.total != booking.pricing.total {
            return Err(DomainError::Conflict(
                "The booking was re-priced after this quote was accepted. Request a new quote."
                    .to_string(),
            ));
        }

        let payment = Payment::from_quote(Uuid::new_v4(), &quote)?;
        self.payment_repo.save(&payment).await?;

        Ok(InitPaymentOutput {
            wallet_address: quote.receiver_address.clone(),
            token: quote.token.as_str().to_string(),
            chain: quote.chain.as_str().to_string(),
            crypto_amount: quote.crypto_amount.normalize().to_string(),
            crypto_base_units: quote.crypto_base_units.to_string(),
            total_cents: quote.pricing.total.cents(),
            currency: quote.pricing.total.currency().as_str().to_string(),
            quote_id: quote.id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::booking::{Booking, BookingStatus};
    use crate::domain::entities::quote::{PaymentQuote, QuoteStatus};
    use crate::domain::value_objects::money::{FiatCurrency, Money, PriceBreakdown};
    use crate::domain::value_objects::{Chain, CryptoToken};
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};

    fn booking_with(base_cents: i64) -> Booking {
        let now = Utc::now();
        let pricing =
            PriceBreakdown::calculate(Money::from_cents(base_cents, FiatCurrency::Usd).unwrap())
                .unwrap();
        Booking::from_db(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            None,
            BookingStatus::Pending,
            pricing,
            "eth".to_string(),
            None,
            now,
            now,
        )
    }

    fn quote_for(booking: &Booking, status: QuoteStatus) -> PaymentQuote {
        let now = Utc::now();
        let mut q = PaymentQuote::issue(
            Uuid::new_v4(),
            booking.id,
            booking.user_id,
            "eth",
            Chain::Ethereum,
            CryptoToken::Eth,
            booking.pricing,
            Decimal::from(2300),
            "test".to_string(),
            now,
            Decimal::from_str("0.05").unwrap(),
            50_000_000_000_000_000,
            "0xReceiver0000000000000000000000000000000000".to_string(),
            now,
        )
        .unwrap();
        q.status = status;
        if status == QuoteStatus::Accepted {
            q.accepted_at = Some(now);
        }
        q
    }

    #[derive(Clone)]
    struct BookingRepo(Booking);

    #[async_trait]
    impl BookingRepository for BookingRepo {
        async fn find_by_id(&self, _: &Uuid) -> Result<Booking, DomainError> {
            Ok(self.0.clone())
        }
        async fn find_all(&self) -> Result<Vec<Booking>, DomainError> {
            Ok(vec![self.0.clone()])
        }
        async fn find_by_user_id(&self, _: &Uuid) -> Result<Vec<Booking>, DomainError> {
            Ok(vec![self.0.clone()])
        }
        async fn save(&self, _: &Booking) -> Result<(), DomainError> {
            Ok(())
        }
        async fn update(&self, _: &Booking) -> Result<(), DomainError> {
            Ok(())
        }
        async fn count(&self) -> Result<i64, DomainError> {
            Ok(1)
        }
        async fn total_revenue(&self) -> Result<i64, DomainError> {
            Ok(0)
        }
    }

    #[derive(Clone, Default)]
    struct PaymentRepo {
        saved: Arc<Mutex<Vec<Payment>>>,
        existing: Option<Payment>,
    }

    #[async_trait]
    impl PaymentRepository for PaymentRepo {
        async fn find_by_booking_id(&self, _: &Uuid) -> Result<Payment, DomainError> {
            self.existing
                .clone()
                .ok_or_else(|| DomainError::NotFound("missing".to_string()))
        }
        async fn find_all(&self) -> Result<Vec<Payment>, DomainError> {
            Ok(vec![])
        }
        async fn find_pending_with_tx(&self) -> Result<Vec<Payment>, DomainError> {
            Ok(vec![])
        }
        async fn find_by_tx_hash(&self, _: &str) -> Result<Option<Payment>, DomainError> {
            Ok(None)
        }
        async fn save(&self, p: &Payment) -> Result<(), DomainError> {
            self.saved.lock().unwrap().push(p.clone());
            Ok(())
        }
        async fn update(&self, _: &Payment) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct QuoteRepo(Option<PaymentQuote>);

    #[async_trait]
    impl QuoteRepository for QuoteRepo {
        async fn issue(&self, _: &PaymentQuote) -> Result<(), DomainError> {
            Ok(())
        }
        async fn find_by_id(&self, _: &Uuid) -> Result<PaymentQuote, DomainError> {
            self.0
                .clone()
                .ok_or_else(|| DomainError::NotFound("quote".to_string()))
        }
        async fn find_active_for_booking(
            &self,
            _: &Uuid,
        ) -> Result<Option<PaymentQuote>, DomainError> {
            Ok(None)
        }
        async fn find_accepted_for_booking(
            &self,
            _: &Uuid,
        ) -> Result<Option<PaymentQuote>, DomainError> {
            Ok(self.0.clone())
        }
        async fn mark_accepted(&self, _: &PaymentQuote) -> Result<bool, DomainError> {
            Ok(true)
        }
        async fn expire_stale(&self) -> Result<u64, DomainError> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn refuses_to_initialise_without_an_accepted_quote() {
        let booking = booking_with(100_00);
        let use_case = InitPayment::new(
            BookingRepo(booking.clone()),
            PaymentRepo::default(),
            QuoteRepo(None),
        );
        let err = use_case
            .execute(InitPaymentInput {
                booking_id: booking.id,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn copies_the_accepted_quote_verbatim_into_the_payment() {
        let booking = booking_with(100_00);
        let quote = quote_for(&booking, QuoteStatus::Accepted);
        let saved = Arc::new(Mutex::new(vec![]));
        let use_case = InitPayment::new(
            BookingRepo(booking.clone()),
            PaymentRepo {
                saved: saved.clone(),
                existing: None,
            },
            QuoteRepo(Some(quote.clone())),
        );

        let out = use_case
            .execute(InitPaymentInput {
                booking_id: booking.id,
            })
            .await
            .unwrap();

        assert_eq!(out.token, "ETH");
        assert_eq!(out.chain, "Ethereum");
        assert_eq!(out.total_cents, booking.pricing.total.cents());
        assert_eq!(out.crypto_base_units, quote.crypto_base_units.to_string());
        assert_eq!(out.quote_id, quote.id);

        let stored = saved.lock().unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].crypto_base_units, quote.crypto_base_units);
        assert_eq!(stored[0].receiver_address, quote.receiver_address);
    }

    #[tokio::test]
    async fn refuses_an_unaccepted_quote() {
        let booking = booking_with(100_00);
        let quote = quote_for(&booking, QuoteStatus::Active);
        let use_case = InitPayment::new(
            BookingRepo(booking.clone()),
            PaymentRepo::default(),
            QuoteRepo(Some(quote)),
        );
        let err = use_case
            .execute(InitPaymentInput {
                booking_id: booking.id,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn refuses_when_the_booking_was_repriced_after_acceptance() {
        let booking = booking_with(100_00);
        let quote = quote_for(&booking, QuoteStatus::Accepted);
        // The catalogue moved after the user approved the quote.
        let repriced = booking_with(150_00);
        let mut stale_booking = repriced.clone();
        stale_booking.id = booking.id;
        stale_booking.user_id = booking.user_id;

        let use_case = InitPayment::new(
            BookingRepo(stale_booking.clone()),
            PaymentRepo::default(),
            QuoteRepo(Some(quote)),
        );
        let err = use_case
            .execute(InitPaymentInput {
                booking_id: stale_booking.id,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn rejects_duplicate_payment_initialization() {
        let booking = booking_with(100_00);
        let quote = quote_for(&booking, QuoteStatus::Accepted);
        let existing = Payment::from_quote(Uuid::new_v4(), &quote).unwrap();
        let use_case = InitPayment::new(
            BookingRepo(booking.clone()),
            PaymentRepo {
                saved: Arc::new(Mutex::new(vec![])),
                existing: Some(existing),
            },
            QuoteRepo(Some(quote)),
        );
        let err = use_case
            .execute(InitPaymentInput {
                booking_id: booking.id,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }
}
