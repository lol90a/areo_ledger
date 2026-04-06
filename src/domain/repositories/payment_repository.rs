use crate::domain::entities::payment::Payment;
use crate::shared::errors::DomainError;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn find_by_booking_id(&self, booking_id: &Uuid) -> Result<Payment, DomainError>;
    async fn find_all(&self) -> Result<Vec<Payment>, DomainError>;
    async fn save(&self, payment: &Payment) -> Result<(), DomainError>;
    async fn update(&self, payment: &Payment) -> Result<(), DomainError>;
}
