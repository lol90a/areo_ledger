use std::collections::HashMap;

use crate::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification};
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
}

impl MultiChainGateway {
    pub fn new(
        ethereum: EvmGateway,
        bsc: EvmGateway,
        solana: SolanaGateway,
        bitcoin: BitcoinGateway,
        wallets: HashMap<String, String>,
    ) -> Self {
        Self { ethereum, bsc, solana, bitcoin, wallets }
    }
}

#[async_trait::async_trait]
impl BlockchainGateway for MultiChainGateway {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError> {
        self.wallets.get(token)
            .cloned()
            .ok_or_else(|| DomainError::ValidationError(format!("No wallet configured for {}", token)))
    }

    async fn verify_transaction(
        &self,
        chain: &str,
        tx_hash: &str,
        expected_to: &str,
        expected_amount_cents: i64,
    ) -> Result<TxVerification, DomainError> {
        match chain {
            "Ethereum" => self.ethereum.verify_transaction(chain, tx_hash, expected_to, expected_amount_cents).await,
            "BSC" => self.bsc.verify_transaction(chain, tx_hash, expected_to, expected_amount_cents).await,
            "Solana" => self.solana.verify_transaction(chain, tx_hash, expected_to, expected_amount_cents).await,
            "Bitcoin" => self.bitcoin.verify_transaction(chain, tx_hash, expected_to, expected_amount_cents).await,
            other => Err(DomainError::ValidationError(format!("Unsupported verification chain: {}", other))),
        }
    }
}
