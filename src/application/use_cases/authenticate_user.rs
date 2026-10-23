use bcrypt::{hash, verify, DEFAULT_COST};

use crate::application::dto::{AuthenticateUserInput, UserOutput};
use crate::domain::repositories::user_repository::UserRepository;
use crate::shared::errors::DomainError;

pub struct AuthenticateUser<R: UserRepository> {
    user_repo: R,
}

impl<R: UserRepository> AuthenticateUser<R> {
    pub fn new(user_repo: R) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, input: AuthenticateUserInput) -> Result<UserOutput, DomainError> {
        let email = input.email.trim().to_ascii_lowercase();
        let password = input.password.clone();

        let user = self.user_repo.find_by_email(&email).await.ok();

        // Constant-time-ish: always spend a bcrypt cycle so a missing email and a wrong
        // password take the same wall-clock time (defeats user enumeration by timing).
        let valid = match &user {
            Some(u) => {
                let stored = u.password_hash.clone();
                tokio::task::spawn_blocking(move || verify(&password, &stored).unwrap_or(false))
                    .await
                    .map_err(|e| DomainError::InternalError(e.to_string()))?
            }
            None => {
                let _ = tokio::task::spawn_blocking(move || hash(&password, DEFAULT_COST))
                    .await
                    .map_err(|e| DomainError::InternalError(e.to_string()))?;
                false
            }
        };

        match user {
            Some(u) if valid => Ok(UserOutput {
                id: u.id,
                email: u.email,
                name: u.name,
                role: u.role.as_str().to_string(),
                wallet_address: u.wallet_address,
            }),
            _ => Err(DomainError::Unauthorized),
        }
    }
}
