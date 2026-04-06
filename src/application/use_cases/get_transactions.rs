use uuid::Uuid;

use crate::application::dto::TransactionOutput;
use crate::domain::repositories::transaction_repository::TransactionRepository;
use crate::shared::errors::DomainError;

pub struct GetTransactions<R: TransactionRepository> {
    transaction_repo: R,
}

impl<R: TransactionRepository> GetTransactions<R> {
    pub fn new(transaction_repo: R) -> Self {
        Self { transaction_repo }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<Vec<TransactionOutput>, DomainError> {
        let transactions = self.transaction_repo.find_by_user_id(&user_id).await?;

        Ok(transactions
            .into_iter()
            .map(|t| TransactionOutput {
                id: t.id.to_string(),
                tx_type: t.tx_type.as_str().to_string(),
                asset: t.asset,
                amount: t.amount,
                status: t.status,
                date: t.created_at.to_string(),
                tx_hash: t.tx_hash,
            })
            .collect())
    }
}
