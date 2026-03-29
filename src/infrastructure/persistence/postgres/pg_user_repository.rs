use uuid::Uuid;

use crate::domain::entities::user::{User, UserRole};
use crate::domain::repositories::user_repository::UserRepository;
use crate::infrastructure::db::DbPool;
use crate::shared::errors::{DomainError, InfraError};

pub struct PgUserRepository {
    pool: DbPool,
}

impl PgUserRepository {
    pub fn new(pool: DbPool) -> Self { Self { pool } }

    async fn client(&self) -> Result<deadpool_postgres::Client, DomainError> {
        self.pool.get().await.map_err(|e| InfraError::Database(e.to_string()).into())
    }

    fn row_to_user(row: &tokio_postgres::Row) -> User {
        User::from_db(
            row.get(0),
            row.get(1),
            row.get(2),
            UserRole::from_str(row.get::<_, &str>(3)),
            row.get(4),
            row.get(5),
        )
    }
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<User, DomainError> {
        let client = self.client().await?;
        let row = client
            .query_one(
                "SELECT id, email, name, role, password_hash, wallet_address_eth \
                 FROM users WHERE id = $1",
                &[id],
            ).await
            .map_err(|_| DomainError::NotFound(format!("User {}", id)))?;
        Ok(Self::row_to_user(&row))
    }

    async fn find_by_email(&self, email: &str) -> Result<User, DomainError> {
        let client = self.client().await?;
        let row = client
            .query_one(
                "SELECT id, email, name, role, password_hash, wallet_address_eth \
                 FROM users WHERE email = $1",
                &[&email],
            ).await
            .map_err(|_| DomainError::NotFound(format!("User with email {}", email)))?;
        Ok(Self::row_to_user(&row))
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let client = self.client().await?;
        let role = user.role.as_str().to_string();
        client.execute(
            "INSERT INTO users (id, email, name, role, password_hash) \
             VALUES ($1, $2, $3, $4, $5)",
            &[&user.id, &user.email, &user.name, &role, &user.password_hash],
        ).await.map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, DomainError> {
        let client = self.client().await?;
        let row = client.query_one("SELECT COUNT(*) FROM users", &[]).await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(row.get(0))
    }
}
