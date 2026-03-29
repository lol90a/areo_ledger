use bcrypt::verify;

use crate::domain::repositories::user_repository::UserRepository;
use crate::application::dto::{AuthenticateUserInput, UserOutput};
use crate::shared::errors::DomainError;

pub struct AuthenticateUser<R: UserRepository> {
    user_repo: R,
}

impl<R: UserRepository> AuthenticateUser<R> {
    pub fn new(user_repo: R) -> Self { Self { user_repo } }

    pub async fn execute(&self, input: AuthenticateUserInput) -> Result<UserOutput, DomainError> {
        let user = self.user_repo.find_by_email(&input.email).await
            .map_err(|_| DomainError::Unauthorized)?;

        let valid = verify(&input.password, &user.password_hash)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        if !valid { return Err(DomainError::Unauthorized); }

        Ok(UserOutput {
            id: user.id, email: user.email,
            name: user.name, role: user.role.as_str().to_string(),
            wallet_address: user.wallet_address,
        })
    }
}
