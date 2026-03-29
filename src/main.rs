use areo_ledger::infrastructure::auth::jwt_service::JwtService;
use areo_ledger::infrastructure::blockchain::bitcoin_gateway::BitcoinGateway;
use areo_ledger::infrastructure::blockchain::evm_gateway::EvmGateway;
use areo_ledger::infrastructure::blockchain::router_gateway::MultiChainGateway;
use areo_ledger::infrastructure::blockchain::solana_gateway::SolanaGateway;
use areo_ledger::infrastructure::config::AppConfig;
use areo_ledger::infrastructure::db::{init_pool, run_migrations};
use areo_ledger::infrastructure::workers::payment_reconciliation::PaymentReconciliationWorker;
use areo_ledger::interfaces::http::handlers::{
    booking_handler, health_handler, payment_handler, transaction_handler, user_handler,
};
use areo_ledger::interfaces::http::middleware::auth_middleware::AuthMiddleware;

use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware, web, App, HttpServer};
use std::collections::HashMap;

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

    let jwt = web::Data::new(JwtService::new(config.jwt_secret.clone()));

    let mut wallets = HashMap::new();
    wallets.insert("ETH".to_string(), config.wallet_eth.clone());
    wallets.insert("USDT".to_string(), config.wallet_usdt.clone());
    wallets.insert("USDC".to_string(), config.wallet_usdc.clone());
    wallets.insert("BNB".to_string(), config.wallet_bnb.clone());
    wallets.insert("BTC".to_string(), config.wallet_btc.clone());
    wallets.insert("SOL".to_string(), config.wallet_sol.clone());

    let multichain_gateway = MultiChainGateway::new(
        EvmGateway::new_ethereum(config.etherscan_api_key.clone(), wallets.clone()),
        EvmGateway::new_bsc(config.bscscan_api_key.clone(), wallets.clone()),
        SolanaGateway::new(config.solana_rpc_url.clone(), wallets.clone()),
        BitcoinGateway::new(config.bitcoin_api_base.clone(), config.wallet_btc.clone()),
        wallets,
    );
    PaymentReconciliationWorker::new(
        pool.clone(),
        multichain_gateway.clone(),
        config.reconciliation_interval_seconds,
    ).spawn();
    let multichain_gateway = web::Data::new(multichain_gateway);

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(100)
        .finish()
        .unwrap();

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
            .wrap(cors)
            .wrap(AuthMiddleware { jwt_secret: jwt_secret.clone() })
            .wrap(Governor::new(&governor_conf))
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(jwt.clone())
            .app_data(multichain_gateway.clone())
            .app_data(
                web::JsonConfig::default().error_handler(|err, _req| {
                    let msg = format!("{}", err);
                    actix_web::error::InternalError::from_response(
                        err,
                        actix_web::HttpResponse::BadRequest().json(serde_json::json!({ "error": msg })),
                    )
                    .into()
                }),
            )
            .service(
                web::scope("/api")
                    .configure(health_handler::config)
                    .configure(user_handler::config)
                    .configure(booking_handler::config)
                    .configure(payment_handler::config)
                    .configure(transaction_handler::config),
            )
            .default_service(web::route().to(|| async {
                actix_web::HttpResponse::NotFound().json(serde_json::json!({ "error": "Route not found" }))
            }))
    })
    .bind(&bind_addr)?
    .run()
    .await
}

