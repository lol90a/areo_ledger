use reqwest::Client;
use std::collections::HashMap;

use crate::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification};
use crate::shared::errors::{DomainError, InfraError};

#[derive(Clone)]
pub struct SolanaGateway {
    client: Client,
    rpc_url: String,
    wallets: HashMap<String, String>,
}

impl SolanaGateway {
    pub fn new(rpc_url: String, wallets: HashMap<String, String>) -> Self {
        Self {
            client: Client::new(),
            rpc_url,
            wallets,
        }
    }

    async fn fetch_json(&self, body: &serde_json::Value) -> Result<serde_json::Value, InfraError> {
        let response = self
            .client
            .post(&self.rpc_url)
            .json(body)
            .send()
            .await
            .map_err(|e| InfraError::Blockchain(e.to_string()))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| InfraError::Blockchain(e.to_string()))?;

        if !status.is_success() {
            return Err(InfraError::Blockchain(format!(
                "Provider returned HTTP {}: {}",
                status,
                text.chars().take(200).collect::<String>()
            )));
        }

        serde_json::from_str(&text).map_err(|_| {
            InfraError::Blockchain(format!(
                "Provider returned a non-JSON response: {}",
                text.chars().take(200).collect::<String>()
            ))
        })
    }
}

#[async_trait::async_trait]
impl BlockchainGateway for SolanaGateway {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError> {
        self.wallets.get(token).cloned().ok_or_else(|| {
            DomainError::ValidationError(format!("No wallet configured for {}", token))
        })
    }

    async fn verify_transaction(
        &self,
        _chain: &str,
        tx_hash: &str,
        expected_to: &str,
        expected_amount_cents: i64,
    ) -> Result<TxVerification, DomainError> {
        let body = serde_json::json!({
            "jsonrpc": "2.0", "id": 1,
            "method": "getTransaction",
            "params": [tx_hash, { "encoding": "json", "maxSupportedTransactionVersion": 0 }]
        });

        let data = self.fetch_json(&body).await.map_err(DomainError::from)?;

        let result = data
            .get("result")
            .filter(|r| !r.is_null())
            .ok_or_else(|| InfraError::Blockchain("Transaction not found".to_string()))?;

        let meta = &result["meta"];
        let pre = meta["preBalances"]
            .as_array()
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let post = meta["postBalances"]
            .as_array()
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let lamports = post.saturating_sub(pre);
        let amount_cents = (lamports as f64 / 1e9 * 100.0) as i64;
        let confirmations = if result["slot"].is_number() { 32u32 } else { 0 };
        let tolerance = (expected_amount_cents / 100).max(1);

        Ok(TxVerification {
            verified: amount_cents >= expected_amount_cents - tolerance && confirmations >= 32,
            amount_cents,
            confirmations,
            from_address: String::new(),
            to_address: expected_to.to_string(),
        })
    }
}
