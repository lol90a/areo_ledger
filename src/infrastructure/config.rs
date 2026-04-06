use std::env;

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
    pub reconciliation_interval_seconds: u64,
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
            reconciliation_interval_seconds: env::var("RECONCILIATION_INTERVAL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.trim().is_empty() {
            return Err("DATABASE_URL must be set".to_string());
        }
        if self.jwt_secret.trim().len() < 32
            || self.jwt_secret == "change-me-in-production-min-32-chars"
        {
            return Err(
                "JWT_SECRET must be set to a strong value with at least 32 characters".to_string(),
            );
        }

        if self.app_env == "production" {
            if self.run_migrations {
                return Err("RUN_MIGRATIONS must be disabled in production; run migrations as a deployment step".to_string());
            }
            if self.frontend_url.contains("localhost") {
                return Err("FRONTEND_URL must not point to localhost in production".to_string());
            }
            for (name, value) in [
                ("WALLET_ADDRESS_BTC", &self.wallet_btc),
                ("WALLET_ADDRESS_ETH", &self.wallet_eth),
                ("WALLET_ADDRESS_USDT", &self.wallet_usdt),
                ("WALLET_ADDRESS_USDC", &self.wallet_usdc),
                ("WALLET_ADDRESS_BNB", &self.wallet_bnb),
                ("WALLET_ADDRESS_SOL", &self.wallet_sol),
            ] {
                if value.trim().is_empty() {
                    return Err(format!("{} must be configured in production", name));
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

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn rejects_default_jwt_secret() {
        let cfg = AppConfig {
            database_url: "postgres://db".to_string(),
            jwt_secret: "change-me-in-production-min-32-chars".to_string(),
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
            reconciliation_interval_seconds: 60,
        };
        assert!(cfg.validate().is_err());
    }
}

