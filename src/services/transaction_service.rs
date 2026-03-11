use crate::db::DbPool;
use crate::errors::AppError;
use serde::Serialize;

#[derive(Serialize)]
pub struct Transaction {
    pub id: String,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub asset: String,
    pub amount: f64,
    pub tokens: i32,
    pub status: String,
    pub date: String,
    pub tx_hash: String,
}

pub struct TransactionService;

impl TransactionService {
    pub fn get_user_transactions(pool: &DbPool, user_id: &str) -> Result<Vec<Transaction>, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let rows = conn.query(
            "SELECT id, grand_total, status, created_at, COALESCE(tx_hash, '') FROM orders WHERE user_id = $1 ORDER BY created_at DESC",
            &[&user_id],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let transactions: Vec<Transaction> = rows.iter().map(|row| {
            let id: uuid::Uuid = row.get(0);
            let id_str = id.to_string();
            let amount: rust_decimal::Decimal = row.get(1);
            let status: String = row.get(2);
            let date: chrono::NaiveDateTime = row.get(3);
            let tx_hash: String = row.get(4);

            Transaction {
                id: id_str.clone(),
                tx_type: "buy".to_string(),
                asset: "Luxury Asset".to_string(),
                amount: amount.to_string().parse().unwrap_or(0.0),
                tokens: 0,
                status,
                date: date.to_string(),
                tx_hash: if tx_hash.is_empty() { 
                    format!("0x{}...{}", &id_str[0..4], &id_str[id_str.len()-4..]) 
                } else { 
                    tx_hash 
                },
            }
        }).collect();

        Ok(transactions)
    }
}
