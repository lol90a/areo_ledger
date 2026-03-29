use uuid::Uuid;

use crate::domain::entities::booking::Booking;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::value_objects::money::{Money, FiatCurrency};
use crate::application::dto::{CreateBookingInput, CreateBookingOutput};
use crate::shared::errors::DomainError;

pub struct CreateBooking<R: BookingRepository> {
    booking_repo: R,
}

impl<R: BookingRepository> CreateBooking<R> {
    pub fn new(booking_repo: R) -> Self { Self { booking_repo } }

    pub async fn execute(&self, input: CreateBookingInput) -> Result<CreateBookingOutput, DomainError> {
        if input.base_price <= 0.0 {
            return Err(DomainError::ValidationError("Base price must be positive".to_string()));
        }

        let base_money = Money::from_cents((input.base_price * 100.0) as i64, FiatCurrency::Usd)
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        let booking = Booking::create(
            Uuid::new_v4(), input.user_id, input.flight_id,
            base_money, input.payment_method,
        )?;

        let total_price = booking.pricing.total.cents() as f64 / 100.0;
        self.booking_repo.save(&booking).await?;

        Ok(CreateBookingOutput { booking_id: booking.id, total_price })
    }
}
