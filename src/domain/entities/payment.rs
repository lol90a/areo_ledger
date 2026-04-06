use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::value_objects::{resolve_chain_token, Money, TxHash};
use crate::shared::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Failed,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Confirmed => "confirmed",
            Self::Failed => "failed",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "confirmed" => Self::Confirmed,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Payment {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub chain: String,
    pub token: String,
    pub amount: Money,
    pub sender_address: Option<String>,
    pub receiver_address: String,
    pub tx_hash: Option<TxHash>,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
}

impl Payment {
    pub fn new(
        id: Uuid,
        booking_id: Uuid,
        method: &str,
        amount: Money,
        receiver_address: String,
    ) -> Result<Self, DomainError> {
        let (chain, token) = resolve_chain_token(method).ok_or_else(|| {
            DomainError::ValidationError(format!("Unsupported payment method: '{}'", method))
        })?;
        Ok(Self {
            id,
            booking_id,
            chain: chain.as_str().to_string(),
            token: token.as_str().to_string(),
            amount,
            sender_address: None,
            receiver_address,
            tx_hash: None,
            status: PaymentStatus::Pending,
            created_at: Utc::now(),
        })
    }

    pub fn confirm(&mut self, tx_hash: TxHash) -> Result<(), DomainError> {
        if self.status != PaymentStatus::Pending {
            return Err(DomainError::ValidationError(format!(
                "Cannot confirm payment in status '{}'",
                self.status.as_str()
            )));
        }
        self.tx_hash = Some(tx_hash);
        self.status = PaymentStatus::Confirmed;
        Ok(())
    }
}
