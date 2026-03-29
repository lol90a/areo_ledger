use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::domain::entities::transaction::{Transaction, TransactionType};
use crate::domain::repositories::transaction_repository::TransactionRepository;
use crate::infrastructure::db::DbPool;
use crate::shared::errors::{DomainError, InfraError};

pub struct PgTransactionRepository {
    pool: DbPool,
}

impl PgTransactionRepository {
    pub fn new(pool: DbPool) -> Self { Self { pool } }

    async fn client(&self) -> Result<deadpool_postgres::Client, DomainError> {
        self.pool.get().await.map_err(|e| InfraError::Database(e.to_string()).into())
    }
}

#[async_trait::async_trait]
impl TransactionRepository for PgTransactionRepository {
    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Vec<Transaction>, DomainError> {
        let client = self.client().await?;
        let rows = client.query(
            "SELECT id, user_id, type, asset, CAST(amount AS FLOAT8), status, COALESCE(tx_hash, ''), date
             FROM transactions WHERE user_id = $1 ORDER BY date DESC",
            &[user_id],
        ).await.map_err(|e| InfraError::Database(e.to_string()))?;

        Ok(rows.iter().map(|row| {
            let created: NaiveDateTime = row.get(7);
            let tx_type = match row.get::<_, String>(2).as_str() {
                "sell" => TransactionType::Sell,
                "transfer" => TransactionType::Transfer,
                _ => TransactionType::Buy,
            };
            Transaction::new(
                row.get(0),
                row.get(1),
                tx_type,
                row.get(3),
                row.get(4),
                row.get(5),
                row.get(6),
                chrono::DateTime::from_naive_utc_and_offset(created, chrono::Utc),
            )
        }).collect())
    }

    async fn save(&self, t: &Transaction) -> Result<(), DomainError> {
        let client = self.client().await?;
        let tx_type = t.tx_type.as_str().to_string();
        client.execute(
            "INSERT INTO transactions (id, user_id, type, asset, amount, status, tx_hash, date)
             VALUES ($1, $2, $3, $4, ($5::TEXT)::DECIMAL(15,2), $6, $7, $8)",
            &[&t.id, &t.user_id, &tx_type, &t.asset, &format!("{:.2}", t.amount), &t.status, &t.tx_hash, &t.created_at.naive_utc()],
        ).await.map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(())
    }
}
