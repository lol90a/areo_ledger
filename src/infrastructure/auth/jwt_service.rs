use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::shared::errors::{DomainError, InfraError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
}

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn create_token(
        &self,
        user_id: &str,
        email: &str,
        role: &str,
    ) -> Result<String, DomainError> {
        let exp = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        encode(
            &Header::default(),
            &Claims {
                sub: user_id.to_owned(),
                email: email.to_owned(),
                role: role.to_owned(),
                exp,
            },
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .map_err(|e| InfraError::Auth(e.to_string()).into())
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, DomainError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default(),
        )
        .map(|d| d.claims)
        .map_err(|_| DomainError::Unauthorized)
    }
}
