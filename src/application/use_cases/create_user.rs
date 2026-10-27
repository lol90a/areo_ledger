use bcrypt::{hash, DEFAULT_COST};
use uuid::Uuid;

use crate::application::dto::{CreateUserInput, UserOutput};
use crate::domain::entities::user::User;
use crate::domain::repositories::user_repository::UserRepository;
use crate::shared::errors::DomainError;

const MAX_EMAIL_LEN: usize = 254;
const MAX_NAME_LEN: usize = 120;
const MAX_PASSWORD_BYTES: usize = 72; // bcrypt silently truncates beyond this.

pub struct CreateUser<R: UserRepository> {
    user_repo: R,
}

impl<R: UserRepository> CreateUser<R> {
    pub fn new(user_repo: R) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, input: CreateUserInput) -> Result<UserOutput, DomainError> {
        // Normalize so 'John@Example.com' and 'john@example.com' are the same account.
        let email = input.email.trim().to_ascii_lowercase();
        let name = input.name.trim().to_string();

        if !is_valid_email(&email) {
            return Err(DomainError::ValidationError(
                "Enter a valid email address".to_string(),
            ));
        }
        if name.is_empty() || name.chars().count() > MAX_NAME_LEN {
            return Err(DomainError::ValidationError(format!(
                "Name must be between 1 and {} characters",
                MAX_NAME_LEN
            )));
        }
        if input.password.len() < 8 || input.password.len() > MAX_PASSWORD_BYTES {
            return Err(DomainError::ValidationError(format!(
                "Password must be between 8 and {} characters",
                MAX_PASSWORD_BYTES
            )));
        }

        // Hash off the async executor so a burst of signups can't stall other requests.
        let password = input.password.clone();
        let password_hash = tokio::task::spawn_blocking(move || hash(&password, DEFAULT_COST))
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        // The DB unique constraint is the source of truth for duplicate emails; the
        // repository maps a unique violation to Conflict (no TOCTOU pre-check).
        let user = User::new(Uuid::new_v4(), email, name, password_hash);
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

fn is_valid_email(email: &str) -> bool {
    if email.is_empty() || email.len() > MAX_EMAIL_LEN || email.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    let mut parts = email.split('@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return false; // more than one '@'
    }
    !local.is_empty()
        && domain.len() >= 3
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.starts_with('-')
}
