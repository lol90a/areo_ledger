use uuid::Uuid;
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionType {
    Buy,
    Sell,
    Transfer,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self { Self::Buy => "buy", Self::Sell => "sell", Self::Transfer => "transfer" }
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id:         Uuid,
    pub user_id:    Uuid,
    pub tx_type:    TransactionType,
    pub asset:      String,
    pub amount:     f64,        // kept as f64 for display only — not used in arithmetic
    pub status:     String,
    pub tx_hash:    String,
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    pub fn new(
        id: Uuid,
        user_id: Uuid,
        tx_type: TransactionType,
        asset: String,
        amount: f64,
        status: String,
        tx_hash: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self { id, user_id, tx_type, asset, amount, status, tx_hash, created_at }
    }

    pub fn is_settled(&self) -> bool {
        self.status == "confirmed" && !self.tx_hash.is_empty()
    }
}
