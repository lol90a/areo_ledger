use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::domain::entities::quote::PaymentQuote;
use crate::domain::value_objects::{Money, TxHash};
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

/// A settlement obligation derived from an accepted quote.
///
/// `crypto_base_units` is the authoritative figure: it is what on-chain verification
/// compares against, in the token's smallest unit. `crypto_amount` and `amount` exist
/// for display and accounting respectively and are never used for the comparison --
/// mixing USD cents with token units is exactly the class of bug this separation
/// exists to prevent.
#[derive(Debug, Clone)]
pub struct Payment {
    pub id: Uuid,
    pub booking_id: Uuid,
    /// The accepted quote that fixed this amount.
    pub quote_id: Option<Uuid>,
    pub chain: String,
    pub token: String,
    pub token_decimals: u8,
    /// Fiat value of the booking this payment settles.
    pub amount: Money,
    /// Expected on-chain amount in whole token units (display only).
    pub crypto_amount: Decimal,
    /// Expected on-chain amount in the token's smallest unit. Authoritative.
    pub crypto_base_units: u128,
    pub sender_address: Option<String>,
    pub receiver_address: String,
    pub tx_hash: Option<TxHash>,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
}

impl Payment {
    /// The only supported construction path: copy an accepted quote verbatim.
    ///
    /// There is deliberately no constructor that takes a caller-supplied amount, so
    /// no code path can create a payment for a price nobody approved.
    pub fn from_quote(id: Uuid, quote: &PaymentQuote) -> Result<Self, DomainError> {
        if !quote.is_settleable() {
            return Err(DomainError::Conflict(
                "Only an accepted quote can be turned into a payment".to_string(),
            ));
        }
        Ok(Self {
            id,
            booking_id: quote.booking_id,
            quote_id: Some(quote.id),
            chain: quote.chain.as_str().to_string(),
            token: quote.token.as_str().to_string(),
            token_decimals: quote.token_decimals,
            amount: quote.pricing.total,
            crypto_amount: quote.crypto_amount,
            crypto_base_units: quote.crypto_base_units,
            sender_address: None,
            receiver_address: quote.receiver_address.clone(),
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
