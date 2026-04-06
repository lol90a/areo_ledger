use crate::domain::entities::transaction::Transaction;
use crate::shared::errors::DomainError;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Vec<Transaction>, DomainError>;
    async fn save(&self, transaction: &Transaction) -> Result<(), DomainError>;
}
