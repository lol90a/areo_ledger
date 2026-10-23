use uuid::Uuid;

use crate::domain::value_objects::Money;
use crate::shared::errors::DomainError;

/// Authoritative product catalog. Prices for bookings come from here — never from
/// the client — so a buyer cannot dictate what they pay.
#[async_trait::async_trait]
pub trait CatalogRepository: Send + Sync {
    /// Server-side price of a flight. `NotFound` if the flight does not exist.
    async fn flight_price(&self, flight_id: &Uuid) -> Result<Money, DomainError>;
    /// Server-side price of a marketplace asset. `NotFound` if it does not exist.
    async fn asset_price(&self, asset_id: &Uuid) -> Result<Money, DomainError>;
}
