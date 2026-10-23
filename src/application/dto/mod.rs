use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateUserInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserOutput {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingInput {
    pub user_id: Uuid,
    /// Flight to purchase. Price is resolved server-side from the flights catalog.
    pub flight_id: Option<Uuid>,
    /// Marketplace asset to purchase. Price is resolved server-side from the assets catalog.
    pub asset_id: Option<Uuid>,
    pub payment_method: String,
}

#[derive(Debug, Serialize)]
pub struct CreateBookingOutput {
    pub booking_id: Uuid,
    /// Fiat total in integer cents. The float `total_price` is kept alongside it for
    /// the existing UI, but cents is the number any further logic must use.
    pub total_cents: i64,
    pub currency: String,
    pub total_price: f64,
}

#[derive(Debug, Deserialize)]
pub struct InitPaymentInput {
    pub booking_id: Uuid,
}

// ── Price quotes ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CreateQuoteInput {
    pub booking_id: Uuid,
    /// Resolved server-side from the authenticated session, never from the body.
    pub user_id: Uuid,
    pub method: String,
}

#[derive(Debug)]
pub struct AcceptQuoteInput {
    pub quote_id: Uuid,
    pub booking_id: Uuid,
    pub user_id: Uuid,
    /// The fiat total the client displayed, in integer cents.
    pub approved_total_cents: i64,
    /// The token amount the client displayed, in the token's smallest unit, as a
    /// decimal string (u128 does not survive JSON number precision).
    pub approved_crypto_base_units: String,
}

/// What the API returns for a quote. Deliberately includes every number the user is
/// asked to approve, so the client can echo them back on acceptance.
#[derive(Debug, Serialize)]
pub struct QuoteView {
    pub quote_id: Uuid,
    pub booking_id: Uuid,
    pub status: String,
    pub method: String,
    pub chain: String,
    pub token: String,
    pub token_decimals: u8,
    pub base_cents: i64,
    pub markup_cents: i64,
    pub service_fee_cents: i64,
    pub total_cents: i64,
    pub currency: String,
    /// Human-readable total, e.g. "5750.00".
    pub total_display: String,
    pub unit_price_usd: String,
    pub rate_source: String,
    pub rate_observed_at: String,
    /// Amount to send, in whole token units (display only).
    pub crypto_amount: String,
    /// Amount to send, in the token's smallest unit. This is the authoritative number.
    pub crypto_base_units: String,
    pub receiver_address: String,
    pub expires_at: String,
    pub seconds_remaining: i64,
    /// True when the catalogue price moved since this booking was created. The UI
    /// must show both figures and make the user approve the new one.
    pub price_changed: bool,
    /// The total the user was previously shown, when `price_changed` is true.
    pub previous_total_cents: Option<i64>,
}

impl QuoteView {
    pub fn from_quote(
        quote: &crate::domain::entities::quote::PaymentQuote,
        now: chrono::DateTime<chrono::Utc>,
        previous_total_cents: Option<i64>,
    ) -> Self {
        Self {
            quote_id: quote.id,
            booking_id: quote.booking_id,
            status: quote.status.as_str().to_string(),
            method: quote.method.clone(),
            chain: quote.chain.as_str().to_string(),
            token: quote.token.as_str().to_string(),
            token_decimals: quote.token_decimals,
            base_cents: quote.pricing.base.cents(),
            markup_cents: quote.pricing.markup.cents(),
            service_fee_cents: quote.pricing.service_fee.cents(),
            total_cents: quote.pricing.total.cents(),
            currency: quote.pricing.total.currency().as_str().to_string(),
            total_display: quote.pricing.total.to_decimal_str(),
            unit_price_usd: quote.unit_price_usd.normalize().to_string(),
            rate_source: quote.rate_source.clone(),
            rate_observed_at: quote.rate_observed_at.to_rfc3339(),
            crypto_amount: quote.crypto_amount.normalize().to_string(),
            crypto_base_units: quote.crypto_base_units.to_string(),
            receiver_address: quote.receiver_address.clone(),
            expires_at: quote.expires_at.to_rfc3339(),
            seconds_remaining: quote.seconds_remaining(now),
            price_changed: previous_total_cents.is_some(),
            previous_total_cents,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InitPaymentOutput {
    pub wallet_address: String,
    pub token: String,
    pub chain: String,
    /// Exact amount to send, in whole token units (decimal string).
    pub crypto_amount: String,
    /// The authoritative amount, in the token's smallest on-chain unit.
    pub crypto_base_units: String,
    /// Fiat total in integer cents (never a float).
    pub total_cents: i64,
    pub currency: String,
    /// The accepted quote this payment settles.
    pub quote_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmPaymentInput {
    pub booking_id: Uuid,
    pub tx_hash: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionOutput {
    pub id: String,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub asset: String,
    pub amount: f64,
    pub status: String,
    pub date: String,
    pub tx_hash: String,
}
