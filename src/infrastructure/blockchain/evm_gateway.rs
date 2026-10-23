use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

use crate::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification, VerifyRequest};
use crate::shared::errors::{DomainError, InfraError};

/// keccak256("Transfer(address,address,uint256)") — the ERC-20 Transfer event topic.
const ERC20_TRANSFER_TOPIC: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

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

    fn hex_to_u128(hex: &str) -> u128 {
        u128::from_str_radix(hex.trim_start_matches("0x"), 16).unwrap_or(0)
    }

    /// Block number of a MINED transaction. `None` for a pending transaction
    /// (blockNumber is null) or an unparseable value.
    ///
    /// The previous implementation defaulted a missing block to 0, which made
    /// `current_block - 0` a huge confirmation count: a pending, or never-mined,
    /// transaction looked deeply confirmed.
    fn mined_block(hex: Option<&str>) -> Option<u64> {
        let raw = hex?;
        let parsed = u64::from_str_radix(raw.trim_start_matches("0x"), 16).ok()?;
        (parsed > 0).then_some(parsed)
    }

    /// Confirmations of a transaction mined at `tx_block`, counting the mining block
    /// itself as one. Returns 0 if the node reports a tip behind the transaction.
    fn confirmations_for(current: u64, tx_block: u64) -> u32 {
        if current < tx_block {
            return 0;
        }
        (current - tx_block + 1).min(u32::MAX as u64) as u32
    }

    /// The low 20 bytes of a 32-byte topic, as a 0x-prefixed address.
    fn address_from_topic(topic: &str) -> String {
        let clean = topic.trim_start_matches("0x");
        if clean.len() >= 40 {
            format!("0x{}", &clean[clean.len() - 40..])
        } else {
            format!("0x{}", clean)
        }
    }

    /// Fetch a transaction receipt and return (mined_block, succeeded).
    ///
    /// A receipt only exists once the transaction is mined, and a status of 0x1 is the
    /// only proof that it did not revert. Both facts are required before any transfer
    /// the transaction claims to make can be believed.
    async fn receipt_status(&self, tx_hash: &str) -> Result<(u64, bool), DomainError> {
        let url = format!(
            "{}?module=proxy&action=eth_getTransactionReceipt&txhash={}&apikey={}",
            self.base_url, tx_hash, self.api_key
        );
        let data = self.fetch_json(&url).await.map_err(DomainError::from)?;
        let result = data.get("result").filter(|r| !r.is_null()).ok_or_else(|| {
            InfraError::Blockchain(format!(
                "Transaction {} is not mined yet (no receipt)",
                tx_hash
            ))
        })?;

        let block = Self::mined_block(result["blockNumber"].as_str()).ok_or_else(|| {
            InfraError::Blockchain(format!("Transaction {} has no block number", tx_hash))
        })?;
        // Pre-Byzantium receipts carry no status field; every chain this backend
        // targets is long past that, so a missing status means "not proven successful".
        let succeeded = result["status"]
            .as_str()
            .map(|v| Self::hex_to_u128(v) == 1)
            .unwrap_or(false);

        Ok((block, succeeded))
    }

    /// Verify a native-asset (ETH/BNB) transfer.
    async fn verify_native(&self, req: &VerifyRequest) -> Result<TxVerification, DomainError> {
        // The receipt comes FIRST. eth_getTransactionByHash happily reports the value
        // of a transaction that reverted or was never mined, and the previous
        // implementation treated both as a successful payment.
        let (tx_block, succeeded) = self.receipt_status(&req.tx_hash).await?;

        let url = format!(
            "{}?module=proxy&action=eth_getTransactionByHash&txhash={}&apikey={}",
            self.base_url, req.tx_hash, self.api_key
        );
        let data = self.fetch_json(&url).await.map_err(DomainError::from)?;
        let result = data.get("result").filter(|r| !r.is_null()).ok_or_else(|| {
            InfraError::Blockchain(format!("Transaction not found: {}", req.tx_hash))
        })?;

        let to_address = result["to"].as_str().unwrap_or("").to_string();
        let from_address = result["from"].as_str().unwrap_or("").to_string();
        // A reverted transaction moved no value, whatever its value field says.
        let base_units = if succeeded {
            Self::hex_to_u128(result["value"].as_str().unwrap_or("0x0"))
        } else {
            0
        };

        let current = self.current_block().await.map_err(DomainError::from)?;
        let confirmations = Self::confirmations_for(current, tx_block);

        Ok(self.decide(
            req,
            base_units,
            confirmations,
            from_address,
            to_address,
            succeeded,
        ))
    }

    /// Verify an ERC-20 (USDT/USDC) transfer by scanning the receipt's Transfer logs.
    async fn verify_erc20(
        &self,
        req: &VerifyRequest,
        token_contract: &str,
    ) -> Result<TxVerification, DomainError> {
        let url = format!(
            "{}?module=proxy&action=eth_getTransactionReceipt&txhash={}&apikey={}",
            self.base_url, req.tx_hash, self.api_key
        );
        let data = self.fetch_json(&url).await.map_err(DomainError::from)?;
        let result = data.get("result").filter(|r| !r.is_null()).ok_or_else(|| {
            InfraError::Blockchain(format!(
                "Transaction {} is not mined yet (no receipt)",
                req.tx_hash
            ))
        })?;

        // A reverted transaction still has a receipt; check the status explicitly so a
        // failed call can never settle an order.
        let succeeded = result["status"]
            .as_str()
            .map(|v| Self::hex_to_u128(v) == 1)
            .unwrap_or(false);
        let tx_block = Self::mined_block(result["blockNumber"].as_str()).ok_or_else(|| {
            InfraError::Blockchain(format!("Transaction {} has no block number", req.tx_hash))
        })?;
        let empty = vec![];
        let logs = result["logs"].as_array().unwrap_or(&empty);

        let mut base_units: u128 = 0;
        let mut from_address = String::new();
        for entry in logs {
            let log_addr = entry["address"].as_str().unwrap_or("");
            if !log_addr.eq_ignore_ascii_case(token_contract) {
                continue;
            }
            let topics = entry["topics"].as_array().cloned().unwrap_or_default();
            if topics.len() < 3 {
                continue;
            }
            let topic0 = topics[0].as_str().unwrap_or("");
            if !topic0.eq_ignore_ascii_case(ERC20_TRANSFER_TOPIC) {
                continue;
            }
            let to = Self::address_from_topic(topics[2].as_str().unwrap_or(""));
            if !to.eq_ignore_ascii_case(&req.expected_to) {
                continue;
            }
            base_units = base_units
                .saturating_add(Self::hex_to_u128(entry["data"].as_str().unwrap_or("0x0")));
            from_address = Self::address_from_topic(topics[1].as_str().unwrap_or(""));
        }

        if !succeeded {
            base_units = 0;
        }

        let current = self.current_block().await.map_err(DomainError::from)?;
        let confirmations = Self::confirmations_for(current, tx_block);
        // The receiver is proven by the matched log; report it as the expected address.
        let to_address = if base_units > 0 {
            req.expected_to.clone()
        } else {
            String::new()
        };

        Ok(self.decide(
            req,
            base_units,
            confirmations,
            from_address,
            to_address,
            succeeded,
        ))
    }

    fn decide(
        &self,
        req: &VerifyRequest,
        base_units: u128,
        confirmations: u32,
        from_address: String,
        to_address: String,
        succeeded: bool,
    ) -> TxVerification {
        // Exact-or-more: the quote was rounded up to the token precision when it was
        // issued, so there is no rounding slack left to concede.
        let amount_ok = base_units >= req.expected_base_units;
        let receiver_ok = to_address.eq_ignore_ascii_case(&req.expected_to);
        TxVerification {
            verified: succeeded
                && amount_ok
                && receiver_ok
                && confirmations >= self.min_confirmations,
            base_units,
            confirmations,
            from_address,
            to_address,
        }
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
        request: &VerifyRequest,
    ) -> Result<TxVerification, DomainError> {
        if self.api_key.trim().is_empty() {
            return Err(DomainError::InternalError(
                "Missing blockchain API key".to_string(),
            ));
        }

        match &request.token_contract {
            Some(contract) => self.verify_erc20(request, contract).await,
            None => self.verify_native(request).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_address_from_padded_topic() {
        let topic = "0x000000000000000000000000742d35cc6634c0532925a3b844bc9e7595f0beb0";
        assert_eq!(
            EvmGateway::address_from_topic(topic),
            "0x742d35cc6634c0532925a3b844bc9e7595f0beb0"
        );
    }

    #[test]
    fn hex_amount_parses() {
        // 1,000,000 USDT base units (6 decimals) = 0xF4240
        assert_eq!(EvmGateway::hex_to_u128("0xf4240"), 1_000_000);
    }

    #[test]
    fn a_pending_transaction_has_no_mined_block() {
        // eth_getTransactionByHash returns blockNumber null while pending.
        assert_eq!(EvmGateway::mined_block(None), None);
        assert_eq!(EvmGateway::mined_block(Some("0x0")), None);
        assert_eq!(EvmGateway::mined_block(Some("0x10")), Some(16));
    }

    #[test]
    fn confirmations_count_the_mining_block_and_never_underflow() {
        assert_eq!(EvmGateway::confirmations_for(100, 100), 1);
        assert_eq!(EvmGateway::confirmations_for(112, 100), 13);
        // A node whose tip lags the transaction must not report a huge number.
        assert_eq!(EvmGateway::confirmations_for(90, 100), 0);
    }

    fn request(expected: u128) -> VerifyRequest {
        VerifyRequest {
            chain: "Ethereum".to_string(),
            token: "ETH".to_string(),
            tx_hash: "0xabc".to_string(),
            expected_to: "0xReceiver".to_string(),
            expected_base_units: expected,
            decimals: 18,
            token_contract: None,
        }
    }

    fn gateway() -> EvmGateway {
        EvmGateway::new_ethereum("key".to_string(), HashMap::new())
    }

    #[test]
    fn a_reverted_transaction_is_never_verified() {
        let v = gateway().decide(
            &request(1_000),
            1_000,
            50,
            "0xSender".to_string(),
            "0xReceiver".to_string(),
            false,
        );
        assert!(!v.verified);
    }

    #[test]
    fn an_underpayment_of_one_base_unit_is_rejected() {
        let v = gateway().decide(
            &request(1_000),
            999,
            50,
            "0xSender".to_string(),
            "0xReceiver".to_string(),
            true,
        );
        assert!(!v.verified, "a payment short of the quote must not settle");
    }

    #[test]
    fn an_exact_payment_with_enough_confirmations_is_verified() {
        let v = gateway().decide(
            &request(1_000),
            1_000,
            50,
            "0xSender".to_string(),
            "0xReceiver".to_string(),
            true,
        );
        assert!(v.verified);
    }

    #[test]
    fn a_payment_to_the_wrong_receiver_is_rejected() {
        let v = gateway().decide(
            &request(1_000),
            1_000,
            50,
            "0xSender".to_string(),
            "0xAttacker".to_string(),
            true,
        );
        assert!(!v.verified);
    }

    #[test]
    fn too_few_confirmations_is_rejected() {
        let v = gateway().decide(
            &request(1_000),
            1_000,
            1,
            "0xSender".to_string(),
            "0xReceiver".to_string(),
            true,
        );
        assert!(!v.verified);
    }
}
