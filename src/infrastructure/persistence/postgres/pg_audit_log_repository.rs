use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::infrastructure::db::DbPool;
use crate::shared::errors::{DomainError, InfraError};

pub struct PgAuditLogRepository {
    pool: DbPool,
}

impl PgAuditLogRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    async fn client(&self) -> Result<deadpool_postgres::Client, DomainError> {
        self.pool
            .get()
            .await
            .map_err(|e| InfraError::Database(e.to_string()).into())
    }

    pub async fn record(
        &self,
        user_id: Option<Uuid>,
        action: &str,
        entity_type: &str,
        entity_id: &str,
        new_value: Value,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), DomainError> {
        let client = self.client().await?;
        client.execute(
            "INSERT INTO audit_log
             (id, user_id, action, entity_type, entity_id, new_value, ip_address, user_agent, created_at)
             VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9)",
            &[
                &Uuid::new_v4(),
                &user_id,
                &action,
                &entity_type,
                &entity_id,
                &new_value.to_string(),
                &ip_address,
                &user_agent,
                &Utc::now().naive_utc(),
            ],
        ).await.map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(())
    }
}
