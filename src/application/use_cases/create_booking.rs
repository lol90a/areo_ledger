use uuid::Uuid;

use crate::application::dto::{CreateBookingInput, CreateBookingOutput};
use crate::domain::entities::booking::Booking;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::repositories::catalog_repository::CatalogRepository;
use crate::shared::errors::DomainError;

pub struct CreateBooking<R: BookingRepository, C: CatalogRepository> {
    booking_repo: R,
    catalog: C,
}

impl<R: BookingRepository, C: CatalogRepository> CreateBooking<R, C> {
    pub fn new(booking_repo: R, catalog: C) -> Self {
        Self {
            booking_repo,
            catalog,
        }
    }

    pub async fn execute(
        &self,
        input: CreateBookingInput,
    ) -> Result<CreateBookingOutput, DomainError> {
        // The price is ALWAYS resolved server-side from the catalog. The client only
        // identifies WHAT it is buying, never how much it costs.
        let base_money = match (input.flight_id, input.asset_id) {
            (Some(flight_id), None) => self.catalog.flight_price(&flight_id).await?,
            (None, Some(asset_id)) => self.catalog.asset_price(&asset_id).await?,
            (Some(_), Some(_)) => {
                return Err(DomainError::ValidationError(
                    "Provide either a flight_id or an asset_id, not both".to_string(),
                ))
            }
            (None, None) => {
                return Err(DomainError::ValidationError(
                    "A flight_id or asset_id is required to create a booking".to_string(),
                ))
            }
        };

        let booking = Booking::create(
            Uuid::new_v4(),
            input.user_id,
            input.flight_id,
            input.asset_id,
            base_money,
            input.payment_method,
        )?;

        self.booking_repo.save(&booking).await?;

        // The price here is indicative only. It is re-derived from the catalogue
        // when a quote is issued at checkout, which is where the user is asked to
        // approve a figure and where the price lock actually happens.
        Ok(CreateBookingOutput {
            booking_id: booking.id,
            total_cents: booking.pricing.total.cents(),
            currency: booking.pricing.total.currency().as_str().to_string(),
            total_price: booking.pricing.total.cents() as f64 / 100.0,
        })
    }
}
