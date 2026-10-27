use crate::shared::errors::DomainError;

/// Result of inspecting an on-chain transaction.
///
/// `base_units` is the amount actually credited to `to_address`, expressed in the
/// token's smallest on-chain unit (wei / lamports / satoshis / ERC-20 base units).
#[derive(Debug, Clone)]
pub struct TxVerification {
    /// The transaction settles this payment: observed, successful, to the right
    /// address, for at least the expected amount.
    pub verified: bool,
    /// The transaction is on-chain, succeeded, and is deep enough to be final.
    ///
    /// This exists so the caller can tell a transaction that *has not settled
    /// yet* from one that *will never settle*. `verified` alone folds together
    /// confirmation depth, recipient and amount, so a payment sent to the wrong
    /// address or short by a fraction was reported as "not verified yet ... it
    /// will settle automatically once valid". It never would: the caller was
    /// telling people to wait for something that could not happen.
    ///
    /// When this is false the answer is genuinely "not yet". When it is true and
    /// `verified` is false, the recipient or the amount is wrong and no amount
    /// of waiting changes that.
    pub observed: bool,
    pub base_units: u128,
    pub confirmations: u32,
    pub from_address: String,
    pub to_address: String,
}

/// Everything a gateway needs to verify that a specific payment landed on-chain.
///
/// The expected amount is expressed in the token's smallest unit (`expected_base_units`)
/// so verification is a pure integer comparison — no floating point, no USD cents.
#[derive(Debug, Clone)]
pub struct VerifyRequest {
    /// Canonical chain name ("Ethereum", "BSC", "Solana", "Bitcoin").
    pub chain: String,
    /// Canonical token symbol ("ETH", "USDT", ...).
    pub token: String,
    pub tx_hash: String,
    /// Address the platform expects to receive the funds.
    pub expected_to: String,
    /// Amount owed, in the token's smallest unit.
    pub expected_base_units: u128,
    /// Decimal places of the token's smallest unit.
    pub decimals: u8,
    /// ERC-20 contract address when the token is not the chain's native asset.
    pub token_contract: Option<String>,
}

#[async_trait::async_trait]
pub trait BlockchainGateway: Send + Sync {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError>;

    async fn verify_transaction(
        &self,
        request: &VerifyRequest,
    ) -> Result<TxVerification, DomainError>;
}
