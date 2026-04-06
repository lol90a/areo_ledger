use crate::domain::entities::user::User;
use crate::shared::errors::DomainError;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<User, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<User, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn count(&self) -> Result<i64, DomainError>;
}
