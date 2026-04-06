use crate::shared::errors::DomainError;
use std::fmt;

/// A validated, canonical transaction hash.
/// Enforces minimum length and hex format at construction time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TxHash(String);

impl TxHash {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let s = raw.into();
        let trimmed = s.trim_start_matches("0x");
        if trimmed.len() < 10 {
            return Err(DomainError::ValidationError(format!(
                "Transaction hash too short: '{}'",
                s
            )));
        }
        if !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DomainError::ValidationError(format!(
                "Transaction hash contains non-hex characters: '{}'",
                s
            )));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validated wallet address (EVM, Solana, or Bitcoin).
/// Enforces non-empty and minimum length — chain-specific validation
/// is delegated to the blockchain gateway (infrastructure concern).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WalletAddress(String);

impl WalletAddress {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let s = raw.into();
        if s.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Wallet address cannot be empty".to_string(),
            ));
        }
        if s.len() < 26 {
            return Err(DomainError::ValidationError(format!(
                "Wallet address too short: '{}'",
                s
            )));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Idempotency key — a client-supplied UUID that prevents duplicate payment processing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(uuid::Uuid);

impl IdempotencyKey {
    pub fn new(id: uuid::Uuid) -> Self {
        Self(id)
    }
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    pub fn value(&self) -> uuid::Uuid {
        self.0
    }
}

impl fmt::Display for IdempotencyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
