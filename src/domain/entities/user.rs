use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    User,
    Admin,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "admin" => Self::Admin,
            _ => Self::User,
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub password_hash: String,
    pub wallet_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Used by application layer — role defaults to User, never inferred from email.
    pub fn new(id: Uuid, email: String, name: String, password_hash: String) -> Self {
        Self {
            id,
            email,
            name,
            role: UserRole::User,
            password_hash,
            wallet_address: None,
            created_at: Utc::now(),
        }
    }

    /// Used by infrastructure layer when reconstructing from DB.
    pub fn from_db(
        id: Uuid,
        email: String,
        name: String,
        role: UserRole,
        password_hash: String,
        wallet_address: Option<String>,
    ) -> Self {
        Self {
            id,
            email,
            name,
            role,
            password_hash,
            wallet_address,
            created_at: Utc::now(),
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }
}
