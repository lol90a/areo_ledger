use areo_ledger::domain::ports::price_oracle::PriceOracle;
use areo_ledger::infrastructure::auth::jwt_service::JwtService;
use areo_ledger::infrastructure::blockchain::bitcoin_gateway::BitcoinGateway;
use areo_ledger::infrastructure::blockchain::evm_gateway::EvmGateway;
use areo_ledger::infrastructure::blockchain::router_gateway::MultiChainGateway;
use areo_ledger::infrastructure::blockchain::solana_gateway::SolanaGateway;
use areo_ledger::infrastructure::config::AppConfig;
use areo_ledger::infrastructure::db::{init_pool, run_migrations};
use areo_ledger::infrastructure::pricing::market_price_oracle::MarketPriceOracle;
use areo_ledger::infrastructure::workers::payment_reconciliation::PaymentReconciliationWorker;
use areo_ledger::interfaces::http::handlers::{
    booking_handler, catalog_handler, health_handler, payment_handler, transaction_handler,
    user_handler,
};
use areo_ledger::interfaces::http::middleware::auth_middleware::AuthMiddleware;

use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError};
use actix_web::dev::ServiceRequest;
use actix_web::{middleware, web, App, HttpServer};
use std::collections::HashMap;
use std::sync::Arc;

/// Rate-limit key that trusts the reverse proxy's forwarded client IP so every user
/// behind the nginx container does not share a single bucket.
#[derive(Clone)]
struct RealIpKeyExtractor;

impl KeyExtractor for RealIpKeyExtractor {
    type Key = String;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        let header = |name: &str| {
            req.headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        };
        let ip = header("x-forwarded-for")
            .and_then(|v| v.split(',').next().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .or_else(|| header("x-real-ip").map(|s| s.trim().to_string()))
            .or_else(|| req.peer_addr().map(|a| a.ip().to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        Ok(ip)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = AppConfig::from_env();
    if let Err(e) = config.validate() {
        log::error!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    let pool = init_pool(&config.database_url);
    if config.run_migrations {
        if let Err(e) = run_migrations(&pool).await {
            log::error!("Migration failed: {}", e);
            let mut source = e.source();
            while let Some(err) = source {
                log::error!("Caused by: {}", err);
                source = err.source();
            }
            std::process::exit(1);
        }
    } else {
        log::info!("Skipping migrations on startup.");
    }

    let app_config = web::Data::new(config.clone());
    let jwt = web::Data::new(JwtService::new(config.jwt_secret.clone()));

    let mut wallets = HashMap::new();
    wallets.insert("ETH".to_string(), config.wallet_eth.clone());
    wallets.insert("USDT".to_string(), config.wallet_usdt.clone());
    wallets.insert("USDC".to_string(), config.wallet_usdc.clone());
    wallets.insert("BNB".to_string(), config.wallet_bnb.clone());
    wallets.insert("BTC".to_string(), config.wallet_btc.clone());
    wallets.insert("SOL".to_string(), config.wallet_sol.clone());

    let multichain_gateway = MultiChainGateway::new(
        EvmGateway::new_ethereum(
            config.etherscan_api_key.clone(),
            config.ethereum_chain_id,
            wallets.clone(),
        ),
        // Etherscan V2 serves every chain from one host and key, so BSC uses the
        // same credential. BSCSCAN_API_KEY is kept as an optional override for
        // deployments that still hold a separate key.
        EvmGateway::new_bsc(
            if config.bscscan_api_key.trim().is_empty() {
                config.etherscan_api_key.clone()
            } else {
                config.bscscan_api_key.clone()
            },
            config.bsc_chain_id,
            wallets.clone(),
        ),
        SolanaGateway::new(config.solana_rpc_url.clone(), wallets.clone()),
        BitcoinGateway::new(config.bitcoin_api_base.clone(), config.wallet_btc.clone()),
        wallets,
        config.payment_verification_mode.clone(),
    );
    PaymentReconciliationWorker::new(
        pool.clone(),
        multichain_gateway.clone(),
        config.reconciliation_interval_seconds,
    )
    .spawn();
    let multichain_gateway = web::Data::new(multichain_gateway);

    let price_oracle: Arc<dyn PriceOracle> = Arc::new(MarketPriceOracle::new(
        pool.clone(),
        config.payment_verification_mode == "live",
    ));
    let price_oracle = web::Data::new(price_oracle);

    // Per-client-IP limit (~5 req/s sustained, bursts to 60), keyed on the proxy's
    // forwarded IP so one nginx container does not throttle the whole user base.
    let governor_conf = GovernorConfigBuilder::default()
        .per_millisecond(200)
        .burst_size(60)
        .key_extractor(RealIpKeyExtractor)
        .finish()
        .expect("valid rate limiter configuration");

    let bind_addr = config.bind_address();
    let frontend_url = config.frontend_url.clone();
    let frontend_url_alt = frontend_url.replace("localhost", "127.0.0.1");
    let jwt_secret = config.jwt_secret.clone();
    let is_development = config.app_env == "development";

    log::info!("AeroLedger listening on {}", bind_addr);

    HttpServer::new(move || {
        let cors = if is_development {
            Cors::default()
                .allowed_origin(&frontend_url)
                .allowed_origin(&frontend_url_alt)
                .allowed_origin("http://localhost:3001")
                .allowed_origin("http://127.0.0.1:3000")
                .allowed_origin("http://127.0.0.1:3001")
                .allowed_origin("http://localhost:5173")
                .allowed_origin("http://127.0.0.1:5173")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec!["Content-Type", "Authorization"])
                .max_age(3600)
        } else {
            Cors::default()
                .allowed_origin(&frontend_url)
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec!["Content-Type", "Authorization"])
                .max_age(3600)
        };

        App::new()
            .wrap(AuthMiddleware {
                jwt_secret: jwt_secret.clone(),
            })
            .wrap(Governor::new(&governor_conf))
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(app_config.clone())
            .app_data(jwt.clone())
            .app_data(multichain_gateway.clone())
            .app_data(price_oracle.clone())
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                let msg = format!("{}", err);
                actix_web::error::InternalError::from_response(
                    err,
                    actix_web::HttpResponse::BadRequest().json(serde_json::json!({ "error": msg })),
                )
                .into()
            }))
            .service(
                web::scope("/api")
                    .configure(health_handler::config)
                    .configure(user_handler::config)
                    .configure(catalog_handler::config)
                    .configure(booking_handler::config)
                    .configure(payment_handler::config)
                    .configure(transaction_handler::config),
            )
            .default_service(web::route().to(|| async {
                actix_web::HttpResponse::NotFound()
                    .json(serde_json::json!({ "error": "Route not found" }))
            }))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
