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
    /// The caller supplied something an external service rejected as malformed
    /// -- a transaction hash that is not a valid signature, say.
    ///
    /// Distinct from `Blockchain` because every other variant means "we failed",
    /// and `From<InfraError> for DomainError` turns those into a 500. Without
    /// this, a user pasting a mistyped hash got "Internal server error", which
    /// blames the server for the user's typo and buries a real fault under
    /// noise if one ever happens.
    InvalidRequest(String),
}

impl fmt::Display for InfraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfraError::Database(msg) => write!(f, "Database error: {}", msg),
            InfraError::Blockchain(msg) => write!(f, "Blockchain error: {}", msg),
            InfraError::Auth(msg) => write!(f, "Auth error: {}", msg),
            // No prefix: this one is shown to the user as-is.
            InfraError::InvalidRequest(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for InfraError {}

impl From<InfraError> for DomainError {
    fn from(e: InfraError) -> Self {
        match e {
            // Bad input is the caller's to fix, so it must not surface as a 500.
            InfraError::InvalidRequest(msg) => DomainError::ValidationError(msg),
            other => DomainError::InternalError(other.to_string()),
        }
    }
}
