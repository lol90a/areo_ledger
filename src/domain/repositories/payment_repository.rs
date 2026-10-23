use crate::domain::entities::payment::Payment;
use crate::shared::errors::DomainError;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn find_by_booking_id(&self, booking_id: &Uuid) -> Result<Payment, DomainError>;
    async fn find_all(&self) -> Result<Vec<Payment>, DomainError>;
    /// Pending payments that carry a submitted tx hash (reconciliation candidates).
    async fn find_pending_with_tx(&self) -> Result<Vec<Payment>, DomainError>;
    /// The payment already associated with `tx_hash`, if any (replay detection).
    async fn find_by_tx_hash(&self, tx_hash: &str) -> Result<Option<Payment>, DomainError>;
    async fn save(&self, payment: &Payment) -> Result<(), DomainError>;
    async fn update(&self, payment: &Payment) -> Result<(), DomainError>;
}
