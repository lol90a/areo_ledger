use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::value_objects::Money;
use crate::shared::errors::DomainError;

/// Account types in the double-entry system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountType {
    /// Funds received from customers.
    Revenue,
    /// Platform service fees collected.
    ServiceFee,
    /// Funds held pending settlement.
    Escrow,
    /// Funds returned to customers.
    Refund,
    /// Platform profit after fees.
    Profit,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Revenue    => "revenue",
            Self::ServiceFee => "service_fee",
            Self::Escrow     => "escrow",
            Self::Refund     => "refund",
            Self::Profit     => "profit",
        }
    }
}

/// A single side of a double-entry ledger entry.
#[derive(Debug, Clone)]
pub struct LedgerEntry {
    pub id:           Uuid,
    pub account_type: AccountType,
    pub amount:       Money,
    pub is_debit:     bool,   // true = debit, false = credit
    pub reference_id: Uuid,   // booking_id, payment_id, etc.
    pub description:  String,
    pub created_at:   DateTime<Utc>,
}

/// A balanced double-entry journal entry.
/// Invariant: debit total MUST equal credit total.
#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub id:          Uuid,
    pub entries:     Vec<LedgerEntry>,
    pub created_at:  DateTime<Utc>,
}

impl JournalEntry {
    /// Create a balanced journal entry for a confirmed payment.
    /// Debit: Escrow (funds leaving escrow)
    /// Credit: Revenue + ServiceFee (funds allocated to accounts)
    pub fn for_payment_settlement(
        payment_id: Uuid,
        booking_id: Uuid,
        total: Money,
        service_fee: Money,
    ) -> Result<Self, DomainError> {
        let revenue = total.checked_sub(service_fee)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let now = Utc::now();
        let entries = vec![
            // Debit escrow — funds leaving the hold account
            LedgerEntry {
                id: Uuid::new_v4(),
                account_type: AccountType::Escrow,
                amount: total,
                is_debit: true,
                reference_id: payment_id,
                description: format!("Escrow release for booking {}", booking_id),
                created_at: now,
            },
            // Credit revenue
            LedgerEntry {
                id: Uuid::new_v4(),
                account_type: AccountType::Revenue,
                amount: revenue,
                is_debit: false,
                reference_id: booking_id,
                description: format!("Revenue from booking {}", booking_id),
                created_at: now,
            },
            // Credit service fee
            LedgerEntry {
                id: Uuid::new_v4(),
                account_type: AccountType::ServiceFee,
                amount: service_fee,
                is_debit: false,
                reference_id: booking_id,
                description: format!("Service fee from booking {}", booking_id),
                created_at: now,
            },
        ];

        // Enforce balance invariant
        Self::assert_balanced(&entries)?;

        Ok(Self { id: Uuid::new_v4(), entries, created_at: now })
    }

    fn assert_balanced(entries: &[LedgerEntry]) -> Result<(), DomainError> {
        let mut debit_total: i64 = 0;
        let mut credit_total: i64 = 0;
        for e in entries {
            if e.is_debit { debit_total += e.amount.cents(); }
            else          { credit_total += e.amount.cents(); }
        }
        if debit_total != credit_total {
            return Err(DomainError::InternalError(format!(
                "Unbalanced journal entry: debits={} credits={}", debit_total, credit_total
            )));
        }
        Ok(())
    }
}
