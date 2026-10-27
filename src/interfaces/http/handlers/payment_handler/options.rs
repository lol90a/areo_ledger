use actix_web::{web, HttpResponse};
use serde::Serialize;

use crate::domain::value_objects::{resolve_chain_token, Chain};
use crate::infrastructure::config::AppConfig;

/// A payment route the client may select.
///
/// `chain_id`, `rpc_url` and `token_contract` come from server configuration
/// rather than being hard-coded in the browser, so the wallet connects to the
/// chain and token the backend will actually verify against. A mismatch here is
/// how funds get sent to the right address on the wrong network.
#[derive(Serialize)]
pub(super) struct PaymentOptionResponse {
    method: String,
    token: String,
    chain: String,
    display_name: String,
    wallet_address: Option<String>,
    available: bool,
    /// Wallet family the client should use: "evm", "solana" or "manual".
    wallet_kind: &'static str,
    /// EIP-155 chain id for EVM routes.
    chain_id: Option<u64>,
    /// ERC-20 contract address when the token is not the chain's native asset.
    token_contract: Option<String>,
    /// Decimals of the token's smallest unit.
    decimals: u8,
    /// Canonical network name this route settles on ("devnet", "sepolia",
    /// "mainnet-beta", ...), or null when the endpoint does not name one.
    network: Option<String>,
    /// RPC endpoint for chains whose network is not identified by a chain id.
    ///
    /// Solana has no EIP-155 equivalent, so without this the browser had to
    /// supply its own endpoint and silently disagreed with the verifier about
    /// which cluster the payment was on. Serving it here puts Solana under the
    /// same rule as `chain_id`: the server decides the network, not the client.
    rpc_url: Option<String>,
}

fn build_payment_options(config: &AppConfig) -> Vec<PaymentOptionResponse> {
    [
        ("btc", "Bitcoin (BTC)", config.wallet_btc.as_str()),
        ("eth", "Ethereum (ETH)", config.wallet_eth.as_str()),
        ("usdt", "Tether USD (USDT)", config.wallet_usdt.as_str()),
        ("usdc", "USD Coin (USDC)", config.wallet_usdc.as_str()),
        ("bsc", "BNB on BSC", config.wallet_bnb.as_str()),
        ("sol", "Solana (SOL)", config.wallet_sol.as_str()),
    ]
    .into_iter()
    .filter_map(|(method, display_name, wallet_address)| {
        // Chain and token come from the single source of truth in the domain layer,
        // so the API can never advertise a route the verifier does not understand.
        let (chain, token) = resolve_chain_token(method)?;
        let chain_id = match chain {
            Chain::Ethereum => Some(config.ethereum_chain_id),
            Chain::Bsc => Some(config.bsc_chain_id),
            _ => None,
        };
        let wallet_kind = match chain {
            Chain::Ethereum | Chain::Bsc => "evm",
            Chain::Solana => "solana",
            Chain::Bitcoin => "manual",
        };
        Some(PaymentOptionResponse {
            method: method.to_string(),
            token: token.as_str().to_string(),
            chain: chain.as_str().to_string(),
            display_name: display_name.to_string(),
            wallet_address: (!wallet_address.trim().is_empty()).then(|| wallet_address.to_string()),
            available: !wallet_address.trim().is_empty(),
            wallet_kind,
            chain_id,
            token_contract: token.erc20_contract().map(|s| s.to_string()),
            decimals: token.decimals(),
            network: config.network_for(chain).map(|n| n.name),
            // Only Solana needs this; EVM clients key off chain_id and Bitcoin
            // is settled manually.
            rpc_url: (chain == Chain::Solana).then(|| config.solana_rpc_url.clone()),
        })
    })
    .collect()
}

pub(super) async fn payment_options(config: web::Data<AppConfig>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "options": build_payment_options(config.get_ref())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOL_WALLET: &str = "SoLWa11etAddressForTesting111111111111111111";
    const EVM_WALLET: &str = "0x1111111111111111111111111111111111111111";

    /// An all-testnet deployment, which is what the demo and CI both run on.
    fn testnet_config() -> AppConfig {
        let mut cfg = AppConfig::fixture();
        cfg.wallet_sol = SOL_WALLET.to_string();
        cfg.solana_rpc_url = "https://api.devnet.solana.com".to_string();
        cfg.wallet_eth = EVM_WALLET.to_string();
        cfg.wallet_usdt = EVM_WALLET.to_string();
        cfg.ethereum_chain_id = 11155111;
        cfg
    }

    fn option_for<'a>(
        opts: &'a [PaymentOptionResponse],
        method: &str,
    ) -> &'a PaymentOptionResponse {
        opts.iter()
            .find(|o| o.method == method)
            .unwrap_or_else(|| panic!("no payment option for '{}'", method))
    }

    /// The client must never have to pick its own Solana cluster: that is the
    /// disagreement that let a devnet wallet pay against a mainnet verifier.
    #[test]
    fn solana_carries_the_servers_own_rpc_endpoint_and_network() {
        let opts = build_payment_options(&testnet_config());
        let sol = option_for(&opts, "sol");
        assert_eq!(sol.wallet_kind, "solana");
        assert_eq!(
            sol.rpc_url.as_deref(),
            Some("https://api.devnet.solana.com")
        );
        assert_eq!(sol.network.as_deref(), Some("devnet"));
        // Solana has no EIP-155 id; offering one would be meaningless.
        assert_eq!(sol.chain_id, None);
    }

    #[test]
    fn evm_routes_carry_a_chain_id_and_no_rpc_url() {
        let opts = build_payment_options(&testnet_config());
        let eth = option_for(&opts, "eth");
        assert_eq!(eth.wallet_kind, "evm");
        assert_eq!(eth.chain_id, Some(11155111));
        assert_eq!(eth.network.as_deref(), Some("sepolia"));
        assert_eq!(eth.rpc_url, None);
    }

    /// Token metadata has to come from the domain layer, because the verifier
    /// reads the same values. A hard-coded 18 here would under-charge USDT by
    /// twelve orders of magnitude.
    #[test]
    fn token_decimals_and_contract_come_from_the_domain_layer() {
        let opts = build_payment_options(&testnet_config());
        let eth = option_for(&opts, "eth");
        assert_eq!(eth.decimals, 18);
        assert_eq!(eth.token_contract, None, "ETH is a native asset");

        let usdt = option_for(&opts, "usdt");
        assert_eq!(usdt.decimals, 6);
        assert!(
            usdt.token_contract.is_some(),
            "USDT is an ERC-20 and must name its contract"
        );
    }

    /// A route with no wallet is still listed, so the UI can show it greyed out
    /// rather than silently omitting a payment method -- but it must not hand
    /// out an empty address for the user to pay into.
    #[test]
    fn a_route_without_a_wallet_is_listed_but_unavailable() {
        let mut cfg = testnet_config();
        cfg.wallet_btc = String::new();

        let opts = build_payment_options(&cfg);
        let btc = option_for(&opts, "btc");
        assert!(!btc.available);
        assert_eq!(btc.wallet_address, None);
    }

    #[test]
    fn whitespace_only_wallet_is_treated_as_unconfigured() {
        let mut cfg = testnet_config();
        cfg.wallet_btc = "   ".to_string();

        let opts = build_payment_options(&cfg);
        assert!(!option_for(&opts, "btc").available);
    }

    #[test]
    fn every_advertised_method_resolves_to_a_chain_and_token() {
        let opts = build_payment_options(&testnet_config());
        assert_eq!(
            opts.len(),
            6,
            "a method that no longer resolves would be dropped silently"
        );
    }
}
