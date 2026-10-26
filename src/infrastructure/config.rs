use std::env;

use crate::domain::value_objects::{
    bitcoin_network, check_realm_consistency, evm_network, solana_network, Chain, ConfiguredChain,
    Network,
};

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub host: String,
    pub port: u16,
    pub wallet_btc: String,
    pub wallet_eth: String,
    pub wallet_sol: String,
    pub wallet_usdt: String,
    pub wallet_usdc: String,
    pub wallet_bnb: String,
    pub etherscan_api_key: String,
    pub bscscan_api_key: String,
    pub solana_rpc_url: String,
    pub bitcoin_api_base: String,
    pub frontend_url: String,
    pub app_env: String,
    pub run_migrations: bool,
    pub payment_verification_mode: String,
    pub reconciliation_interval_seconds: u64,
    /// EIP-155 chain id the Ethereum gateway and the frontend must both be on.
    /// Mainnet by default; set to 11155111 for Sepolia, etc.
    pub ethereum_chain_id: u64,
    /// EIP-155 chain id for the BSC gateway. 56 = BSC mainnet, 97 = testnet.
    pub bsc_chain_id: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        if dotenvy::from_filename(".env").is_err() {
            dotenvy::dotenv().ok();
        }

        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        if app_env == "production" {
            let _ = dotenvy::from_filename(".env.production");
        }
        let run_migrations = env::var("RUN_MIGRATIONS")
            .ok()
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(app_env != "production");
        // Fail closed: only development defaults to manual verification; every other
        // environment defaults to live on-chain verification.
        let payment_verification_mode = env::var("PAYMENT_VERIFICATION_MODE")
            .unwrap_or_else(|_| {
                if app_env == "development" {
                    "manual".to_string()
                } else {
                    "live".to_string()
                }
            })
            .to_ascii_lowercase();

        Self {
            database_url: env::var("DATABASE_URL").unwrap_or_default(),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production-min-32-chars".to_string()),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            wallet_btc: env::var("WALLET_ADDRESS_BTC").unwrap_or_default(),
            wallet_eth: env::var("WALLET_ADDRESS_ETH").unwrap_or_default(),
            wallet_sol: env::var("WALLET_ADDRESS_SOL").unwrap_or_default(),
            wallet_usdt: env::var("WALLET_ADDRESS_USDT").unwrap_or_default(),
            wallet_usdc: env::var("WALLET_ADDRESS_USDC").unwrap_or_default(),
            wallet_bnb: env::var("WALLET_ADDRESS_BNB").unwrap_or_default(),
            etherscan_api_key: env::var("ETHERSCAN_API_KEY").unwrap_or_default(),
            bscscan_api_key: env::var("BSCSCAN_API_KEY").unwrap_or_default(),
            solana_rpc_url: env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
            bitcoin_api_base: env::var("BITCOIN_API_BASE")
                .unwrap_or_else(|_| "https://blockstream.info/api".to_string()),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            app_env,
            run_migrations,
            payment_verification_mode,
            reconciliation_interval_seconds: env::var("RECONCILIATION_INTERVAL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            ethereum_chain_id: env::var("ETHEREUM_CHAIN_ID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            bsc_chain_id: env::var("BSC_CHAIN_ID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(56),
        }
    }

    /// The chains this deployment actually advertises, with the network each is
    /// pointed at.
    ///
    /// Only chains with a wallet address configured are included: a route with
    /// no wallet is never offered to a client, so holding a half-finished
    /// deployment to a network it does not serve would be noise.
    pub fn configured_chains(&self) -> Vec<ConfiguredChain> {
        let mut chains = Vec::new();
        if !self.wallet_btc.trim().is_empty() {
            chains.push(ConfiguredChain {
                chain: Chain::Bitcoin,
                network: bitcoin_network(&self.bitcoin_api_base),
            });
        }
        // ETH, USDT and USDC all settle on Ethereum, so one entry covers them.
        if !self.wallet_eth.trim().is_empty()
            || !self.wallet_usdt.trim().is_empty()
            || !self.wallet_usdc.trim().is_empty()
        {
            chains.push(ConfiguredChain {
                chain: Chain::Ethereum,
                network: Some(evm_network(self.ethereum_chain_id)),
            });
        }
        if !self.wallet_bnb.trim().is_empty() {
            chains.push(ConfiguredChain {
                chain: Chain::Bsc,
                network: Some(evm_network(self.bsc_chain_id)),
            });
        }
        if !self.wallet_sol.trim().is_empty() {
            chains.push(ConfiguredChain {
                chain: Chain::Solana,
                network: solana_network(&self.solana_rpc_url),
            });
        }
        chains
    }

    /// The network a given chain is configured against, for the API to report to
    /// clients so the wallet and the verifier cannot disagree.
    pub fn network_for(&self, chain: Chain) -> Option<Network> {
        match chain {
            Chain::Bitcoin => bitcoin_network(&self.bitcoin_api_base),
            Chain::Ethereum => Some(evm_network(self.ethereum_chain_id)),
            Chain::Bsc => Some(evm_network(self.bsc_chain_id)),
            Chain::Solana => solana_network(&self.solana_rpc_url),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.trim().is_empty() {
            return Err("DATABASE_URL must be set".to_string());
        }
        if self.jwt_secret.trim().len() < 32 || is_weak_secret(&self.jwt_secret) {
            return Err(
                "JWT_SECRET must be a strong, non-placeholder value of at least 32 characters"
                    .to_string(),
            );
        }
        if self.ethereum_chain_id == 0 || self.bsc_chain_id == 0 {
            return Err("ETHEREUM_CHAIN_ID and BSC_CHAIN_ID must be non-zero".to_string());
        }
        if !matches!(self.payment_verification_mode.as_str(), "live" | "manual") {
            return Err("PAYMENT_VERIFICATION_MODE must be either 'live' or 'manual'".to_string());
        }
        // Manual mode trusts client-submitted tx hashes — only ever allowed locally.
        if self.payment_verification_mode == "manual" && self.app_env != "development" {
            return Err(
                "PAYMENT_VERIFICATION_MODE=manual is only permitted when APP_ENV=development"
                    .to_string(),
            );
        }
        if !self.bitcoin_api_base.trim().is_empty() && !is_http_url(&self.bitcoin_api_base) {
            return Err(
                "BITCOIN_API_BASE must be a valid http(s) URL (e.g. https://blockstream.info/api)"
                    .to_string(),
            );
        }
        // Every enabled chain must be on the same network. Each setting is
        // individually plausible, so only comparing them catches a deployment
        // that verifies on mainnet while the wallet is told to use devnet.
        check_realm_consistency(&self.configured_chains())?;

        if self.app_env == "production" {
            if self.run_migrations {
                return Err("RUN_MIGRATIONS must be disabled in production; run migrations as a deployment step".to_string());
            }
            if self.payment_verification_mode != "live" {
                return Err("PAYMENT_VERIFICATION_MODE must be 'live' in production".to_string());
            }
            if self.frontend_url.contains("localhost") || self.frontend_url.contains("127.0.0.1") {
                return Err(
                    "FRONTEND_URL must be a public origin in production (not localhost/127.0.0.1)"
                        .to_string(),
                );
            }

            for (name, value, kind) in [
                ("WALLET_ADDRESS_BTC", &self.wallet_btc, WalletKind::Btc),
                ("WALLET_ADDRESS_ETH", &self.wallet_eth, WalletKind::Evm),
                ("WALLET_ADDRESS_USDT", &self.wallet_usdt, WalletKind::Evm),
                ("WALLET_ADDRESS_USDC", &self.wallet_usdc, WalletKind::Evm),
                ("WALLET_ADDRESS_BNB", &self.wallet_bnb, WalletKind::Evm),
                ("WALLET_ADDRESS_SOL", &self.wallet_sol, WalletKind::Solana),
            ] {
                if value.trim().is_empty() {
                    return Err(format!("{} must be configured in production", name));
                }
                if !kind.is_valid(value.trim()) {
                    return Err(format!(
                        "{} is not a valid {} address: '{}'",
                        name,
                        kind.label(),
                        value.trim()
                    ));
                }
            }
            if self.etherscan_api_key.trim().is_empty() {
                return Err("ETHERSCAN_API_KEY must be configured in production".to_string());
            }
            if self.bscscan_api_key.trim().is_empty() {
                return Err("BSCSCAN_API_KEY must be configured in production".to_string());
            }
            if self.solana_rpc_url.trim().is_empty() {
                return Err("SOLANA_RPC_URL must be configured in production".to_string());
            }
            if self.bitcoin_api_base.trim().is_empty() {
                return Err("BITCOIN_API_BASE must be configured in production".to_string());
            }
        }

        Ok(())
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Rejects obvious placeholder / example secrets even when they clear the length bar.
fn is_weak_secret(secret: &str) -> bool {
    let lower = secret.to_ascii_lowercase();
    const MARKERS: &[&str] = &[
        "change-me",
        "change_me",
        "changeme",
        "super-secret",
        "supersecret",
        "your-secret",
        "your_secret",
        "secret-key",
        "example",
        "placeholder",
        "todo",
        "xxxx",
    ];
    MARKERS.iter().any(|m| lower.contains(m))
}

fn is_http_url(value: &str) -> bool {
    let v = value.trim();
    v.starts_with("http://") || v.starts_with("https://")
}

#[derive(Clone, Copy)]
enum WalletKind {
    Evm,
    Btc,
    Solana,
}

impl WalletKind {
    fn label(self) -> &'static str {
        match self {
            WalletKind::Evm => "EVM (0x + 40 hex)",
            WalletKind::Btc => "Bitcoin",
            WalletKind::Solana => "Solana",
        }
    }

    fn is_valid(self, addr: &str) -> bool {
        match self {
            WalletKind::Evm => {
                addr.len() == 42
                    && addr.starts_with("0x")
                    && addr[2..].chars().all(|c| c.is_ascii_hexdigit())
            }
            WalletKind::Btc => {
                let bech32 = addr.starts_with("bc1") && addr.len() >= 14 && addr.len() <= 90;
                let legacy = (addr.starts_with('1') || addr.starts_with('3'))
                    && (26..=35).contains(&addr.len());
                bech32 || legacy
            }
            WalletKind::Solana => {
                (32..=44).contains(&addr.len()) && addr.chars().all(|c| c.is_ascii_alphanumeric())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev_config() -> AppConfig {
        AppConfig {
            database_url: "postgres://db".to_string(),
            jwt_secret: "a-sufficiently-long-and-random-secret-value-1234".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8080,
            wallet_btc: String::new(),
            wallet_eth: String::new(),
            wallet_sol: String::new(),
            wallet_usdt: String::new(),
            wallet_usdc: String::new(),
            wallet_bnb: String::new(),
            etherscan_api_key: String::new(),
            bscscan_api_key: String::new(),
            solana_rpc_url: String::new(),
            bitcoin_api_base: String::new(),
            frontend_url: "http://localhost:3001".to_string(),
            app_env: "development".to_string(),
            run_migrations: true,
            payment_verification_mode: "manual".to_string(),
            reconciliation_interval_seconds: 60,
            ethereum_chain_id: 1,
            bsc_chain_id: 56,
        }
    }

    #[test]
    fn accepts_a_sound_dev_config() {
        assert!(dev_config().validate().is_ok());
    }

    /// The exact defect this repository shipped with: Solana pointed at devnet
    /// while Ethereum was on mainnet. Each value looked fine on its own, and a
    /// payment in that state can never confirm.
    #[test]
    fn rejects_a_deployment_that_mixes_mainnet_and_a_test_network() {
        let mut cfg = dev_config();
        cfg.wallet_sol = "SoLWa11etAddressForTesting111111111111111111".to_string();
        cfg.solana_rpc_url = "https://api.devnet.solana.com".to_string();
        cfg.wallet_eth = "0x1111111111111111111111111111111111111111".to_string();
        cfg.ethereum_chain_id = 1;

        let err = cfg.validate().unwrap_err();
        assert!(err.contains("Solana"), "{}", err);
        assert!(err.contains("Ethereum"), "{}", err);
    }

    #[test]
    fn accepts_an_all_testnet_deployment() {
        let mut cfg = dev_config();
        cfg.wallet_sol = "SoLWa11etAddressForTesting111111111111111111".to_string();
        cfg.solana_rpc_url = "https://api.devnet.solana.com".to_string();
        cfg.wallet_eth = "0x1111111111111111111111111111111111111111".to_string();
        cfg.ethereum_chain_id = 11155111;
        cfg.wallet_btc = "tb1qtestnetaddress".to_string();
        cfg.bitcoin_api_base = "https://blockstream.info/testnet/api".to_string();

        assert!(cfg.validate().is_ok(), "{:?}", cfg.validate());
    }

    /// A chain with no wallet configured is never offered to a client, so it
    /// must not hold back a deployment for a network it does not serve.
    #[test]
    fn ignores_chains_that_have_no_wallet_configured() {
        let mut cfg = dev_config();
        cfg.wallet_sol = "SoLWa11etAddressForTesting111111111111111111".to_string();
        cfg.solana_rpc_url = "https://api.devnet.solana.com".to_string();
        // Ethereum is on mainnet but has no wallet, so it is not advertised.
        cfg.ethereum_chain_id = 1;

        assert!(cfg.validate().is_ok(), "{:?}", cfg.validate());
    }

    #[test]
    fn rejects_default_jwt_secret() {
        let mut cfg = dev_config();
        cfg.jwt_secret = "change-me-in-production-min-32-chars".to_string();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_placeholder_secret_that_clears_length() {
        let mut cfg = dev_config();
        cfg.jwt_secret = "CHANGE_ME_WITH_AT_LEAST_32_CHARACTERS".to_string();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_manual_mode_outside_development() {
        let mut cfg = dev_config();
        cfg.app_env = "staging".to_string();
        cfg.payment_verification_mode = "manual".to_string();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_non_url_bitcoin_base() {
        let mut cfg = dev_config();
        cfg.bitcoin_api_base = "477fde2be4421b775db41972c7fdcdfeaed5e94b".to_string();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_truncated_evm_wallet_in_production() {
        let mut cfg = dev_config();
        cfg.app_env = "production".to_string();
        cfg.run_migrations = false;
        cfg.payment_verification_mode = "live".to_string();
        cfg.frontend_url = "https://app.aeroledger.com".to_string();
        cfg.wallet_btc = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string();
        // 39 hex chars — one short of a valid 20-byte address.
        cfg.wallet_eth = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string();
        cfg.wallet_usdt = cfg.wallet_eth.clone();
        cfg.wallet_usdc = cfg.wallet_eth.clone();
        cfg.wallet_bnb = cfg.wallet_eth.clone();
        cfg.wallet_sol = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK".to_string();
        cfg.etherscan_api_key = "key".to_string();
        cfg.bscscan_api_key = "key".to_string();
        cfg.solana_rpc_url = "https://api.mainnet-beta.solana.com".to_string();
        cfg.bitcoin_api_base = "https://blockstream.info/api".to_string();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn accepts_valid_production_config() {
        let mut cfg = dev_config();
        cfg.app_env = "production".to_string();
        cfg.run_migrations = false;
        cfg.payment_verification_mode = "live".to_string();
        cfg.frontend_url = "https://app.aeroledger.com".to_string();
        cfg.wallet_btc = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string();
        cfg.wallet_eth = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string();
        cfg.wallet_usdt = cfg.wallet_eth.clone();
        cfg.wallet_usdc = cfg.wallet_eth.clone();
        cfg.wallet_bnb = cfg.wallet_eth.clone();
        cfg.wallet_sol = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK".to_string();
        cfg.etherscan_api_key = "key".to_string();
        cfg.bscscan_api_key = "key".to_string();
        cfg.solana_rpc_url = "https://api.mainnet-beta.solana.com".to_string();
        cfg.bitcoin_api_base = "https://blockstream.info/api".to_string();
        assert!(cfg.validate().is_ok());
    }
}
