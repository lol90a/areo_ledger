use uuid::Uuid;
use crate::domain::entities::booking::Booking;
use crate::shared::errors::DomainError;

#[async_trait::async_trait]
pub trait BookingRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Booking, DomainError>;
    async fn find_all(&self) -> Result<Vec<Booking>, DomainError>;
    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Vec<Booking>, DomainError>;
    async fn save(&self, booking: &Booking) -> Result<(), DomainError>;
    async fn update(&self, booking: &Booking) -> Result<(), DomainError>;
    async fn count(&self) -> Result<i64, DomainError>;
    async fn total_revenue(&self) -> Result<i64, DomainError>;
}
