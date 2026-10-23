use std::collections::HashMap;

use crate::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification, VerifyRequest};
use crate::infrastructure::blockchain::bitcoin_gateway::BitcoinGateway;
use crate::infrastructure::blockchain::evm_gateway::EvmGateway;
use crate::infrastructure::blockchain::solana_gateway::SolanaGateway;
use crate::shared::errors::DomainError;

#[derive(Clone)]
pub struct MultiChainGateway {
    ethereum: EvmGateway,
    bsc: EvmGateway,
    solana: SolanaGateway,
    bitcoin: BitcoinGateway,
    wallets: HashMap<String, String>,
    verification_mode: String,
}

impl MultiChainGateway {
    pub fn new(
        ethereum: EvmGateway,
        bsc: EvmGateway,
        solana: SolanaGateway,
        bitcoin: BitcoinGateway,
        wallets: HashMap<String, String>,
        verification_mode: String,
    ) -> Self {
        Self {
            ethereum,
            bsc,
            solana,
            bitcoin,
            wallets,
            verification_mode,
        }
    }

    pub fn is_manual(&self) -> bool {
        self.verification_mode == "manual"
    }
}

#[async_trait::async_trait]
impl BlockchainGateway for MultiChainGateway {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError> {
        self.wallets.get(token).cloned().ok_or_else(|| {
            DomainError::ValidationError(format!("No wallet configured for {}", token))
        })
    }

    async fn verify_transaction(
        &self,
        request: &VerifyRequest,
    ) -> Result<TxVerification, DomainError> {
        // Manual mode: no on-chain lookup. Only permitted in development (enforced by
        // AppConfig::validate); treats a well-formed hash as a settled payment so the
        // demo checkout flow works without a live chain.
        if self.verification_mode == "manual" {
            return Ok(TxVerification {
                verified: true,
                base_units: request.expected_base_units,
                confirmations: 99,
                from_address: "manual-verification".to_string(),
                to_address: request.expected_to.clone(),
            });
        }

        match request.chain.as_str() {
            "Ethereum" => self.ethereum.verify_transaction(request).await,
            "BSC" => self.bsc.verify_transaction(request).await,
            "Solana" => self.solana.verify_transaction(request).await,
            "Bitcoin" => self.bitcoin.verify_transaction(request).await,
            other => Err(DomainError::ValidationError(format!(
                "Unsupported verification chain: {}",
                other
            ))),
        }
    }
}
