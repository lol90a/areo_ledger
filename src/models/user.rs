use serde::{Serialize, Deserialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub name: String,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub wallet_address: Option<String>,
}
