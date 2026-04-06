use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

use crate::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification};
use crate::shared::errors::{DomainError, InfraError};

#[derive(Clone)]
pub struct EvmGateway {
    client: Client,
    api_key: String,
    base_url: String,
    wallets: HashMap<String, String>,
    min_confirmations: u32,
}

impl EvmGateway {
    pub fn new_ethereum(api_key: String, wallets: HashMap<String, String>) -> Self {
        Self::build(
            api_key,
            "https://api.etherscan.io/api".to_string(),
            wallets,
            12,
        )
    }

    pub fn new_bsc(api_key: String, wallets: HashMap<String, String>) -> Self {
        Self::build(
            api_key,
            "https://api.bscscan.com/api".to_string(),
            wallets,
            15,
        )
    }

    fn build(
        api_key: String,
        base_url: String,
        wallets: HashMap<String, String>,
        min_confirmations: u32,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            client,
            api_key,
            base_url,
            wallets,
            min_confirmations,
        }
    }

    async fn fetch_json(&self, url: &str) -> Result<serde_json::Value, InfraError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| InfraError::Blockchain(e.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| InfraError::Blockchain(e.to_string()))?;

        if !status.is_success() {
            return Err(InfraError::Blockchain(format!(
                "Provider returned HTTP {}: {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }

        serde_json::from_str(&body).map_err(|_| {
            InfraError::Blockchain(format!(
                "Provider returned a non-JSON response: {}",
                body.chars().take(200).collect::<String>()
            ))
        })
    }

    async fn current_block(&self) -> Result<u64, InfraError> {
        let url = format!(
            "{}?module=proxy&action=eth_blockNumber&apikey={}",
            self.base_url, self.api_key
        );
        let data = self.fetch_json(&url).await?;
        let hex = data["result"].as_str().unwrap_or("0x0");
        u64::from_str_radix(hex.trim_start_matches("0x"), 16)
            .map_err(|e| InfraError::Blockchain(e.to_string()))
    }
}

#[async_trait::async_trait]
impl BlockchainGateway for EvmGateway {
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
        if self.api_key.trim().is_empty() {
            return Err(DomainError::InternalError(
                "Missing blockchain API key".to_string(),
            ));
        }

        let url = format!(
            "{}?module=proxy&action=eth_getTransactionByHash&txhash={}&apikey= {}",
            self.base_url, tx_hash, self.api_key
        ).replace("apikey= ", "apikey=");
        let data = self.fetch_json(&url).await.map_err(DomainError::from)?;
        let result = data
            .get("result")
            .filter(|r| !r.is_null())
            .ok_or_else(|| InfraError::Blockchain(format!("Transaction not found: {}", tx_hash)))?;

        let to_address = result["to"].as_str().unwrap_or("");
        let value_hex = result["value"].as_str().unwrap_or("0x0");
        let value_wei = u128::from_str_radix(value_hex.trim_start_matches("0x"), 16).unwrap_or(0);
        let amount_cents = (value_wei as f64 / 1e18 * 100.0) as i64;

        let tx_block = result["blockNumber"]
            .as_str()
            .and_then(|h| u64::from_str_radix(h.trim_start_matches("0x"), 16).ok())
            .unwrap_or(0);
        let current = self.current_block().await.map_err(DomainError::from)?;
        let confirmations = current.saturating_sub(tx_block) as u32;

        let tolerance = (expected_amount_cents / 100).max(1);
        Ok(TxVerification {
            verified: to_address.eq_ignore_ascii_case(expected_to)
                && amount_cents >= expected_amount_cents - tolerance
                && confirmations >= self.min_confirmations,
            amount_cents,
            confirmations,
            from_address: result["from"].as_str().unwrap_or("").to_string(),
            to_address: to_address.to_string(),
        })
    }
}
