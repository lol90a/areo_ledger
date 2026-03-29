use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::value_objects::{Money, TxHash, PriceBreakdown, MoneyError, resolve_chain_token};
use crate::shared::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookingStatus {
    Pending,
    Confirmed,
    Cancelled,
}

impl BookingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending   => "pending",
            Self::Confirmed => "confirmed",
            Self::Cancelled => "cancelled",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "confirmed" => Self::Confirmed,
            "cancelled" => Self::Cancelled,
            _           => Self::Pending,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Booking {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub flight_id:      Uuid,
    pub status:         BookingStatus,
    pub pricing:        PriceBreakdown,
    pub payment_method: String,
    pub tx_hash:        Option<TxHash>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

impl Booking {
    /// Application factory — validates invariants, returns the new booking.
    pub fn create(
        id: Uuid,
        user_id: Uuid,
        flight_id: Uuid,
        base_price: Money,
        payment_method: String,
    ) -> Result<Self, DomainError> {
        if resolve_chain_token(&payment_method).is_none() {
            return Err(DomainError::ValidationError(
                format!("Unsupported payment method: '{}'", payment_method),
            ));
        }
        if base_price.is_zero() {
            return Err(DomainError::ValidationError("Base price cannot be zero".to_string()));
        }

        let pricing = PriceBreakdown::calculate(base_price)
            .map_err(|e: MoneyError| DomainError::ValidationError(e.to_string()))?;

        let now = Utc::now();
        Ok(Self {
            id,
            user_id,
            flight_id,
            status: BookingStatus::Pending,
            pricing,
            payment_method,
            tx_hash: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Infrastructure factory — reconstructs from DB row, no validation.
    pub fn from_db(
        id: Uuid,
        user_id: Uuid,
        flight_id: Uuid,
        status: BookingStatus,
        pricing: PriceBreakdown,
        payment_method: String,
        tx_hash: Option<TxHash>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self { id, user_id, flight_id, status, pricing, payment_method, tx_hash, created_at, updated_at }
    }

    pub fn confirm(&mut self, tx_hash: TxHash) -> Result<(), DomainError> {
        if self.status != BookingStatus::Pending {
            return Err(DomainError::ValidationError(
                format!("Cannot confirm booking in status '{}'", self.status.as_str()),
            ));
        }
        self.tx_hash = Some(tx_hash);
        self.status = BookingStatus::Confirmed;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if self.status == BookingStatus::Confirmed {
            return Err(DomainError::ValidationError("Cannot cancel a confirmed booking".to_string()));
        }
        self.status = BookingStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }
}
