use reqwest::Client;
use crate::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification};
use crate::shared::errors::{DomainError, InfraError};

#[derive(Clone)]
pub struct BitcoinGateway {
    client: Client,
    base_url: String,
    wallet_btc: String,
    min_confirmations: u32,
}

impl BitcoinGateway {
    pub fn new(base_url: String, wallet_btc: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            wallet_btc,
            min_confirmations: 3,
        }
    }
}

#[async_trait::async_trait]
impl BlockchainGateway for BitcoinGateway {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError> {
        match token {
            "BTC" => Ok(self.wallet_btc.clone()),
            other => Err(DomainError::ValidationError(format!("No Bitcoin wallet configured for {}", other))),
        }
    }

    async fn verify_transaction(
        &self,
        _chain: &str,
        tx_hash: &str,
        expected_to: &str,
        expected_amount_cents: i64,
    ) -> Result<TxVerification, DomainError> {
        let tx_url = format!("{}/tx/{}", self.base_url, tx_hash);
        let tx: serde_json::Value = self.client.get(&tx_url).send().await
            .map_err(|e| InfraError::Blockchain(e.to_string()))?
            .json().await
            .map_err(|e| InfraError::Blockchain(e.to_string()))?;

        let outputs = tx.get("vout")
            .and_then(|v| v.as_array())
            .ok_or_else(|| DomainError::InternalError("Malformed Bitcoin transaction payload".to_string()))?;

        let mut matched_sats: u64 = 0;
        for output in outputs {
            let address = output.get("scriptpubkey_address").and_then(|v| v.as_str()).unwrap_or("");
            if address == expected_to {
                matched_sats = matched_sats.saturating_add(output.get("value").and_then(|v| v.as_u64()).unwrap_or(0));
            }
        }

        let amount_cents = ((matched_sats as f64 / 100_000_000.0) * 100.0).round() as i64;
        let status = tx.get("status").cloned().unwrap_or_default();
        let confirmations = if status.get("confirmed").and_then(|v| v.as_bool()).unwrap_or(false) {
            let tip: u64 = self.client.get(format!("{}/blocks/tip/height", self.base_url)).send().await
                .map_err(|e| InfraError::Blockchain(e.to_string()))?
                .text().await
                .map_err(|e| InfraError::Blockchain(e.to_string()))?
                .trim().parse().map_err(|e: std::num::ParseIntError| InfraError::Blockchain(e.to_string()))?;
            let block_height = status.get("block_height").and_then(|v| v.as_u64()).unwrap_or(tip);
            tip.saturating_sub(block_height) as u32 + 1
        } else {
            0
        };

        let tolerance = (expected_amount_cents / 100).max(1);
        let verified = matched_sats > 0
            && amount_cents >= expected_amount_cents - tolerance
            && confirmations >= self.min_confirmations;

        Ok(TxVerification {
            verified,
            amount_cents,
            confirmations,
            from_address: tx.get("vin")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|vin| vin.get("prevout"))
                .and_then(|prev| prev.get("scriptpubkey_address"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            to_address: expected_to.to_string(),
        })
    }
}
