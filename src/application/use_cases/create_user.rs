use bcrypt::{hash, DEFAULT_COST};
use uuid::Uuid;

use crate::application::dto::{CreateUserInput, UserOutput};
use crate::domain::entities::user::User;
use crate::domain::repositories::user_repository::UserRepository;
use crate::shared::errors::DomainError;

pub struct CreateUser<R: UserRepository> {
    user_repo: R,
}

impl<R: UserRepository> CreateUser<R> {
    pub fn new(user_repo: R) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, input: CreateUserInput) -> Result<UserOutput, DomainError> {
        if input.email.is_empty() || !input.email.contains('@') {
            return Err(DomainError::ValidationError(
                "Invalid email address".to_string(),
            ));
        }
        if input.name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }
        if input.password.len() < 8 {
            return Err(DomainError::ValidationError(
                "Password must be at least 8 characters".to_string(),
            ));
        }
        if self.user_repo.find_by_email(&input.email).await.is_ok() {
            return Err(DomainError::Conflict(
                "Email already registered".to_string(),
            ));
        }

        let password_hash = hash(&input.password, DEFAULT_COST)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let user = User::new(Uuid::new_v4(), input.email, input.name, password_hash);
        self.user_repo.save(&user).await?;

        Ok(UserOutput {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role.as_str().to_string(),
            wallet_address: user.wallet_address,
        })
    }
}
