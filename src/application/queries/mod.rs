use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::repositories::transaction_repository::TransactionRepository;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::shared::errors::DomainError;

// ── View Models (read-side DTOs) ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TransactionView {
    pub id:         String,
    #[serde(rename = "type")]
    pub tx_type:    String,
    pub asset:      String,
    /// Decimal string — never f64 in API responses for financial amounts.
    pub amount:     String,
    pub currency:   String,
    pub status:     String,
    pub date:       DateTime<Utc>,
    pub tx_hash:    Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BookingSummaryView {
    pub id:             String,
    pub flight_id:      String,
    pub status:         String,
    pub base_price:     String,
    pub total_price:    String,
    pub currency:       String,
    pub payment_method: String,
    pub created_at:     DateTime<Utc>,
}

// ── GetUserTransactions Query ─────────────────────────────────────────────────

pub struct GetUserTransactionsQuery {
    pub user_id: Uuid,
    pub limit:   Option<u32>,
    pub offset:  Option<u32>,
}

pub struct GetUserTransactionsHandler<R: TransactionRepository> {
    repo: R,
}

impl<R: TransactionRepository> GetUserTransactionsHandler<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn handle(&self, query: GetUserTransactionsQuery) -> Result<Vec<TransactionView>, DomainError> {
        let transactions = self.repo
            .find_by_user_id_paginated(&query.user_id, query.limit.unwrap_or(50), query.offset.unwrap_or(0))
            .await?;

        Ok(transactions.into_iter().map(|t| TransactionView {
            id:       t.id().to_string(),
            tx_type:  t.tx_type().as_str().to_string(),
            asset:    t.asset().to_string(),
            amount:   t.amount().to_decimal_str(),
            currency: t.amount().currency().as_str().to_string(),
            status:   t.status().as_str().to_string(),
            date:     t.created_at(),
            tx_hash:  t.tx_hash().map(|h| h.as_str().to_string()),
        }).collect())
    }
}

// ── GetUserBookings Query ─────────────────────────────────────────────────────

pub struct GetUserBookingsQuery {
    pub user_id: Uuid,
}

pub struct GetUserBookingsHandler<R: BookingRepository> {
    repo: R,
}

impl<R: BookingRepository> GetUserBookingsHandler<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn handle(&self, query: GetUserBookingsQuery) -> Result<Vec<BookingSummaryView>, DomainError> {
        let bookings = self.repo.find_by_user_id(&query.user_id).await?;

        Ok(bookings.into_iter().map(|b| BookingSummaryView {
            id:             b.id().to_string(),
            flight_id:      b.flight_id().to_string(),
            status:         b.status().as_str().to_string(),
            base_price:     b.pricing().base.to_decimal_str(),
            total_price:    b.pricing().total.to_decimal_str(),
            currency:       b.pricing().total.currency().as_str().to_string(),
            payment_method: b.payment_method().to_string(),
            created_at:     b.created_at(),
        }).collect())
    }
}
