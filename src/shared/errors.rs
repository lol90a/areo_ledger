use std::fmt;

/// Core domain error — zero external dependencies.
/// HTTP mapping lives in the interfaces layer via `From<DomainError>`.
#[derive(Debug)]
pub enum DomainError {
    NotFound(String),
    ValidationError(String),
    Unauthorized,
    Conflict(String),
    InternalError(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DomainError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            DomainError::Unauthorized => write!(f, "Unauthorized"),
            DomainError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            DomainError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

/// Infrastructure-level error (DB, network) — converted to DomainError at the boundary.
#[derive(Debug)]
pub enum InfraError {
    Database(String),
    Blockchain(String),
    Auth(String),
}

impl fmt::Display for InfraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfraError::Database(msg) => write!(f, "Database error: {}", msg),
            InfraError::Blockchain(msg) => write!(f, "Blockchain error: {}", msg),
            InfraError::Auth(msg) => write!(f, "Auth error: {}", msg),
        }
    }
}

impl std::error::Error for InfraError {}

impl From<InfraError> for DomainError {
    fn from(e: InfraError) -> Self {
        DomainError::InternalError(e.to_string())
    }
}
