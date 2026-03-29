use crate::shared::errors::DomainError;

#[derive(Debug, Clone)]
pub struct TxVerification {
    pub verified:      bool,
    pub amount_cents:  i64,
    pub confirmations: u32,
    pub from_address:  String,
    pub to_address:    String,
}

#[async_trait::async_trait]
pub trait BlockchainGateway: Send + Sync {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError>;

    async fn verify_transaction(
        &self,
        chain: &str,
        tx_hash: &str,
        expected_to: &str,
        expected_amount_cents: i64,
    ) -> Result<TxVerification, DomainError>;
}
