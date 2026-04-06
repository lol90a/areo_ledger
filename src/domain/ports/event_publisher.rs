use crate::domain::events::DomainEvent;
use crate::shared::errors::DomainError;

/// Port for publishing domain events to an event bus (Kafka, NATS, Redis Streams, etc.).
/// The domain declares this need — infrastructure provides the implementation.
#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish<E: DomainEvent + serde::Serialize>(
        &self,
        event: &E,
    ) -> Result<(), DomainError>;
}
