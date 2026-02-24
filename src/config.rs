use std::env;

pub struct AppConfig {
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/aeroledger".to_string());
        
        Self { database_url }
    }
}
