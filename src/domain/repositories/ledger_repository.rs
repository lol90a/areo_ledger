use crate::domain::entities::ledger::JournalEntry;
use crate::shared::errors::DomainError;

#[async_trait::async_trait]
pub trait LedgerRepository: Send + Sync {
    async fn save_journal(&self, journal: &JournalEntry) -> Result<(), DomainError>;
}
