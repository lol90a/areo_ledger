use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

use crate::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification, VerifyRequest};
use crate::shared::errors::{DomainError, InfraError};

/// Commitment level used for every lookup.
///
/// `finalized` is the only level that is not subject to rollback. The previous
/// implementation left the commitment unset and then synthesised a confirmation
/// count from a slot delta, with a `.max(1)` that made the minimum-confirmation
/// check vacuous: any transaction the RPC returned was treated as settled.
const COMMITMENT: &str = "finalized";

/// Reported for a transaction the RPC returns at `finalized`. Solana has no
/// "confirmations" in the Bitcoin sense; finality is a consensus property, not a
/// depth, so the number is a flag rather than a depth measurement.
const FINALIZED_CONFIRMATIONS: u32 = 32;

/// JSON-RPC "invalid params". From `getTransaction` this means the signature we
/// passed is not a signature at all, i.e. the client sent a malformed hash.
const INVALID_PARAMS: i64 = -32602;

#[derive(Clone)]
pub struct SolanaGateway {
    client: Client,
    rpc_url: String,
    wallets: HashMap<String, String>,
}

/// What inspecting a `getTransaction` result tells us about a native SOL payment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolanaTransferFacts {
    /// Net lamports credited to the expected recipient in this transaction.
    pub lamports_credited: u128,
    /// Whether the expected recipient appears in the transaction at all.
    pub receiver_matched: bool,
    /// Whether the transaction executed without error.
    pub succeeded: bool,
    /// Fee payer / first signer, reported as the sender.
    pub from_address: String,
}

impl SolanaGateway {
    pub fn new(rpc_url: String, wallets: HashMap<String, String>) -> Self {
        let client = crate::infrastructure::http::client(Duration::from_secs(10));
        Self {
            client,
            rpc_url,
            wallets,
        }
    }

    async fn rpc(&self, body: &serde_json::Value) -> Result<serde_json::Value, InfraError> {
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

        let json: serde_json::Value = serde_json::from_str(&text).map_err(|_| {
            InfraError::Blockchain(format!(
                "Provider returned a non-JSON response: {}",
                text.chars().take(200).collect::<String>()
            ))
        })?;

        // A JSON-RPC error body is still HTTP 200; surface it instead of reading
        // `result` as null and calling the transaction "not found".
        if let Some(err) = json.get("error").filter(|e| !e.is_null()) {
            // -32602 is "invalid params", which for these calls means the
            // caller handed us something that is not a signature at all. That is
            // bad input, not a provider fault, and reporting it as an
            // infrastructure error turned a malformed transaction hash into a
            // 500 Internal Server Error.
            if err.get("code").and_then(|c| c.as_i64()) == Some(INVALID_PARAMS) {
                return Err(InfraError::InvalidRequest(
                    "That is not a valid Solana transaction signature.".to_string(),
                ));
            }
            return Err(InfraError::Blockchain(format!(
                "RPC error: {}",
                err.to_string().chars().take(200).collect::<String>()
            )));
        }

        Ok(json)
    }

    /// Account keys of the transaction, as base58 strings. Handles both the plain
    /// string encoding and the `{pubkey, signer, writable}` object encoding.
    fn account_keys(result: &serde_json::Value) -> Vec<String> {
        result["transaction"]["message"]["accountKeys"]
            .as_array()
            .map(|keys| {
                keys.iter()
                    .map(|k| {
                        k.as_str()
                            .map(str::to_string)
                            .or_else(|| {
                                k.get("pubkey").and_then(|p| p.as_str()).map(str::to_string)
                            })
                            .unwrap_or_default()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Pure inspection of a `getTransaction` result.
    ///
    /// Kept free of I/O so the parsing rules below can be unit-tested against real
    /// response shapes:
    ///   * the recipient is located BY ADDRESS, never by a fixed index,
    ///   * the credited amount is that account's own pre/post balance delta,
    ///   * a failed transaction credits nothing regardless of what it attempted.
    pub fn inspect(result: &serde_json::Value, expected_to: &str) -> SolanaTransferFacts {
        let meta = &result["meta"];
        let succeeded = meta.get("err").map(|e| e.is_null()).unwrap_or(false);

        let keys = Self::account_keys(result);
        let index = keys.iter().position(|k| k == expected_to);

        let (lamports_credited, receiver_matched) = match index {
            Some(idx) => {
                let read = |field: &str| {
                    meta[field]
                        .as_array()
                        .and_then(|a| a.get(idx))
                        .and_then(|v| v.as_u64())
                };
                match (read("preBalances"), read("postBalances")) {
                    (Some(pre), Some(post)) if succeeded => {
                        (post.saturating_sub(pre) as u128, true)
                    }
                    // Present in the transaction, but either it failed or the balance
                    // arrays are missing: nothing was credited.
                    _ => (0, true),
                }
            }
            None => (0, false),
        };

        SolanaTransferFacts {
            lamports_credited,
            receiver_matched,
            succeeded,
            from_address: keys.first().cloned().unwrap_or_default(),
        }
    }
}

#[async_trait::async_trait]
impl BlockchainGateway for SolanaGateway {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError> {
        self.wallets.get(token).cloned().ok_or_else(|| {
            DomainError::ValidationError(format!("No wallet configured for {}", token))
        })
    }

    async fn verify_transaction(&self, req: &VerifyRequest) -> Result<TxVerification, DomainError> {
        // Only native SOL is settled on Solana in this deployment. An SPL-token
        // payment would have to be read from `meta.preTokenBalances` /
        // `postTokenBalances` for the right mint and owner, which is deliberately not
        // implemented rather than approximated: see SECURITY_AUDIT.md (SOL-04).
        if req.token_contract.is_some() {
            return Err(DomainError::ValidationError(
                "SPL-token settlement is not supported on Solana by this gateway".to_string(),
            ));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0", "id": 1,
            "method": "getTransaction",
            "params": [
                req.tx_hash,
                {
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 0,
                    // Asking for `finalized` means the RPC only returns the
                    // transaction once it can no longer be rolled back.
                    "commitment": COMMITMENT
                }
            ]
        });

        let data = self.rpc(&body).await.map_err(DomainError::from)?;
        let Some(result) = data.get("result").filter(|r| !r.is_null()) else {
            // Not an error: the transaction may simply not be finalized yet. Report it
            // as unverified with zero confirmations so the caller keeps retrying.
            return Ok(TxVerification {
                verified: false,
                // Nothing seen on-chain yet, so this really is "not yet".
                observed: false,
                base_units: 0,
                confirmations: 0,
                from_address: String::new(),
                to_address: String::new(),
            });
        };

        let facts = Self::inspect(result, &req.expected_to);
        // Exact-or-more; the quote was rounded up to lamport precision when issued.
        let amount_ok = facts.lamports_credited >= req.expected_base_units;

        Ok(TxVerification {
            verified: facts.succeeded && facts.receiver_matched && amount_ok,
            // The RPC returned it at `finalized` and it executed, so the
            // recipient and amount below are final facts, not a timing artefact.
            observed: facts.succeeded,
            base_units: facts.lamports_credited,
            confirmations: if facts.succeeded {
                FINALIZED_CONFIRMATIONS
            } else {
                0
            },
            from_address: facts.from_address,
            to_address: if facts.receiver_matched {
                req.expected_to.clone()
            } else {
                String::new()
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const RECEIVER: &str = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK";
    const SENDER: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";

    fn tx(
        err: serde_json::Value,
        keys: Vec<&str>,
        pre: Vec<u64>,
        post: Vec<u64>,
    ) -> serde_json::Value {
        json!({
            "slot": 1000,
            "meta": { "err": err, "preBalances": pre, "postBalances": post },
            "transaction": { "message": { "accountKeys": keys } }
        })
    }

    #[test]
    fn credits_the_recipients_own_balance_delta() {
        let t = tx(
            json!(null),
            vec![SENDER, RECEIVER],
            vec![10_000_000_000, 5_000],
            vec![8_000_000_000, 2_000_005_000],
        );
        let facts = SolanaGateway::inspect(&t, RECEIVER);
        assert!(facts.succeeded);
        assert!(facts.receiver_matched);
        assert_eq!(facts.lamports_credited, 2_000_000_000);
        assert_eq!(facts.from_address, SENDER);
    }

    #[test]
    fn locates_the_recipient_by_address_not_by_index() {
        // The platform wallet is third here; a fixed index would read the wrong
        // account's delta entirely.
        let t = tx(
            json!(null),
            vec![
                SENDER,
                "SomeProgram1111111111111111111111111111111",
                RECEIVER,
            ],
            vec![10_000_000_000, 0, 0],
            vec![8_000_000_000, 0, 1_500_000_000],
        );
        let facts = SolanaGateway::inspect(&t, RECEIVER);
        assert_eq!(facts.lamports_credited, 1_500_000_000);
    }

    #[test]
    fn a_failed_transaction_credits_nothing() {
        let t = tx(
            json!({"InstructionError": [0, "Custom"]}),
            vec![SENDER, RECEIVER],
            vec![10_000_000_000, 0],
            vec![9_999_995_000, 0],
        );
        let facts = SolanaGateway::inspect(&t, RECEIVER);
        assert!(!facts.succeeded);
        assert_eq!(facts.lamports_credited, 0);
    }

    #[test]
    fn a_transaction_that_never_touches_our_wallet_does_not_match() {
        let t = tx(
            json!(null),
            vec![SENDER, "OtherWallet111111111111111111111111111111111"],
            vec![10_000_000_000, 0],
            vec![8_000_000_000, 2_000_000_000],
        );
        let facts = SolanaGateway::inspect(&t, RECEIVER);
        assert!(!facts.receiver_matched);
        assert_eq!(facts.lamports_credited, 0);
    }

    #[test]
    fn handles_the_object_form_of_account_keys() {
        let t = json!({
            "slot": 1,
            "meta": { "err": null, "preBalances": [0, 0], "postBalances": [0, 7] },
            "transaction": { "message": { "accountKeys": [
                {"pubkey": SENDER, "signer": true, "writable": true},
                {"pubkey": RECEIVER, "signer": false, "writable": true}
            ]}}
        });
        let facts = SolanaGateway::inspect(&t, RECEIVER);
        assert!(facts.receiver_matched);
        assert_eq!(facts.lamports_credited, 7);
    }

    #[test]
    fn a_balance_decrease_is_never_read_as_a_credit() {
        // The platform wallet paying a fee in someone else's transaction.
        let t = tx(
            json!(null),
            vec![RECEIVER, SENDER],
            vec![5_000_000_000, 0],
            vec![4_999_995_000, 0],
        );
        let facts = SolanaGateway::inspect(&t, RECEIVER);
        assert_eq!(facts.lamports_credited, 0);
    }
}
