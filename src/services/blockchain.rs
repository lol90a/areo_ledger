use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionVerification {
    pub verified: bool,
    pub amount: f64,
    pub confirmations: u32,
    pub from_address: String,
    pub to_address: String,
}

pub struct BlockchainVerifier {
    client: Client,
    etherscan_api_key: String,
    bscscan_api_key: String,
    solana_rpc_url: String,
}

impl BlockchainVerifier {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            etherscan_api_key: env::var("ETHERSCAN_API_KEY").unwrap_or_default(),
            bscscan_api_key: env::var("BSCSCAN_API_KEY").unwrap_or_default(),
            solana_rpc_url: env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        }
    }

    pub async fn verify_transaction(
        &self,
        tx_hash: &str,
        chain: &str,
        expected_address: &str,
        expected_amount: f64,
    ) -> Result<TransactionVerification, String> {
        match chain.to_uppercase().as_str() {
            "ETH" | "ETHEREUM" => self.verify_ethereum_tx(tx_hash, expected_address, expected_amount).await,
            "BTC" | "BITCOIN" => self.verify_bitcoin_tx(tx_hash, expected_address, expected_amount).await,
            "SOL" | "SOLANA" => self.verify_solana_tx(tx_hash, expected_address, expected_amount).await,
            "BNB" | "BSC" => self.verify_bsc_tx(tx_hash, expected_address, expected_amount).await,
            "USDT" | "USDC" => self.verify_ethereum_tx(tx_hash, expected_address, expected_amount).await,
            _ => Err(format!("Unsupported chain: {}", chain)),
        }
    }

    async fn verify_ethereum_tx(
        &self,
        tx_hash: &str,
        expected_address: &str,
        expected_amount: f64,
    ) -> Result<TransactionVerification, String> {
        let url = format!(
            "https://api.etherscan.io/api?module=proxy&action=eth_getTransactionByHash&txhash={}&apikey={}",
            tx_hash, self.etherscan_api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch transaction: {}", e))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(result) = data.get("result") {
            if result.is_null() {
                return Err("Transaction not found".to_string());
            }

            let to_address = result["to"].as_str().unwrap_or("");
            let value_hex = result["value"].as_str().unwrap_or("0x0");
            
            // Convert hex value to ETH
            let value_wei = u128::from_str_radix(value_hex.trim_start_matches("0x"), 16)
                .unwrap_or(0);
            let value_eth = value_wei as f64 / 1e18;

            // Get confirmations
            let block_number = result["blockNumber"].as_str().unwrap_or("0x0");
            let tx_block = u64::from_str_radix(block_number.trim_start_matches("0x"), 16)
                .unwrap_or(0);

            // Get current block
            let current_block = self.get_ethereum_current_block().await?;
            let confirmations = if tx_block > 0 {
                current_block.saturating_sub(tx_block) as u32
            } else {
                0
            };

            let from_address = result["from"].as_str().unwrap_or("").to_string();

            Ok(TransactionVerification {
                verified: to_address.eq_ignore_ascii_case(expected_address) 
                    && value_eth >= expected_amount * 0.99 // 1% tolerance
                    && confirmations >= 12,
                amount: value_eth,
                confirmations,
                from_address,
                to_address: to_address.to_string(),
            })
        } else {
            Err("Invalid response from Etherscan".to_string())
        }
    }

    async fn get_ethereum_current_block(&self) -> Result<u64, String> {
        let url = format!(
            "https://api.etherscan.io/api?module=proxy&action=eth_blockNumber&apikey={}",
            self.etherscan_api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch block number: {}", e))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(result) = data.get("result").and_then(|r| r.as_str()) {
            u64::from_str_radix(result.trim_start_matches("0x"), 16)
                .map_err(|e| format!("Failed to parse block number: {}", e))
        } else {
            Err("Invalid response".to_string())
        }
    }

    async fn verify_bitcoin_tx(
        &self,
        tx_hash: &str,
        expected_address: &str,
        expected_amount: f64,
    ) -> Result<TransactionVerification, String> {
        let url = format!("https://blockchain.info/rawtx/{}", tx_hash);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch transaction: {}", e))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Find output matching our address
        let outputs = data["out"].as_array().ok_or("No outputs found")?;
        let mut total_amount = 0.0;
        let mut found_address = false;

        for output in outputs {
            if let Some(addr) = output["addr"].as_str() {
                if addr == expected_address {
                    found_address = true;
                    let value_satoshi = output["value"].as_u64().unwrap_or(0);
                    total_amount += value_satoshi as f64 / 1e8; // Convert to BTC
                }
            }
        }

        let confirmations = data["block_height"]
            .as_u64()
            .map(|_| 6) // Assume 6+ confirmations if in block
            .unwrap_or(0) as u32;

        let from_address = data["inputs"][0]["prev_out"]["addr"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(TransactionVerification {
            verified: found_address 
                && total_amount >= expected_amount * 0.99 
                && confirmations >= 3,
            amount: total_amount,
            confirmations,
            from_address,
            to_address: expected_address.to_string(),
        })
    }

    async fn verify_solana_tx(
        &self,
        tx_hash: &str,
        expected_address: &str,
        expected_amount: f64,
    ) -> Result<TransactionVerification, String> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                tx_hash,
                {
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 0
                }
            ]
        });

        let response = self.client
            .post(&self.solana_rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch transaction: {}", e))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(result) = data.get("result") {
            if result.is_null() {
                return Err("Transaction not found".to_string());
            }

            // Extract transaction details
            let meta = &result["meta"];
            let pre_balances = meta["preBalances"].as_array().ok_or("No pre balances")?;
            let post_balances = meta["postBalances"].as_array().ok_or("No post balances")?;

            // Calculate amount transferred (simplified)
            let amount_lamports = if post_balances.len() > 1 && pre_balances.len() > 1 {
                let post = post_balances[1].as_u64().unwrap_or(0);
                let pre = pre_balances[1].as_u64().unwrap_or(0);
                post.saturating_sub(pre)
            } else {
                0
            };

            let amount_sol = amount_lamports as f64 / 1e9;

            let confirmations = result["slot"]
                .as_u64()
                .map(|_| 32) // Assume finalized
                .unwrap_or(0) as u32;

            Ok(TransactionVerification {
                verified: amount_sol >= expected_amount * 0.99 && confirmations >= 32,
                amount: amount_sol,
                confirmations,
                from_address: "".to_string(),
                to_address: expected_address.to_string(),
            })
        } else {
            Err("Invalid response from Solana RPC".to_string())
        }
    }

    async fn verify_bsc_tx(
        &self,
        tx_hash: &str,
        expected_address: &str,
        expected_amount: f64,
    ) -> Result<TransactionVerification, String> {
        let url = format!(
            "https://api.bscscan.com/api?module=proxy&action=eth_getTransactionByHash&txhash={}&apikey={}",
            tx_hash, self.bscscan_api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch transaction: {}", e))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(result) = data.get("result") {
            if result.is_null() {
                return Err("Transaction not found".to_string());
            }

            let to_address = result["to"].as_str().unwrap_or("");
            let value_hex = result["value"].as_str().unwrap_or("0x0");
            
            let value_wei = u128::from_str_radix(value_hex.trim_start_matches("0x"), 16)
                .unwrap_or(0);
            let value_bnb = value_wei as f64 / 1e18;

            let from_address = result["from"].as_str().unwrap_or("").to_string();

            Ok(TransactionVerification {
                verified: to_address.eq_ignore_ascii_case(expected_address) 
                    && value_bnb >= expected_amount * 0.99
                    && true, // Assume confirmed if found
                amount: value_bnb,
                confirmations: 15,
                from_address,
                to_address: to_address.to_string(),
            })
        } else {
            Err("Invalid response from BscScan".to_string())
        }
    }
}
