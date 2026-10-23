use uuid::Uuid;

use crate::domain::repositories::catalog_repository::CatalogRepository;
use crate::domain::value_objects::{FiatCurrency, Money};
use crate::infrastructure::db::DbPool;
use crate::shared::errors::{DomainError, InfraError};

/// A public flight-catalog row (safe to expose unauthenticated).
#[derive(serde::Serialize)]
pub struct FlightListItem {
    pub id: String,
    pub route: String,
    pub origin: String,
    pub destination: String,
    pub aircraft: String,
    pub price_usd: f64,
    pub duration: Option<String>,
    pub seats: i32,
    pub status: String,
}

/// A public marketplace-asset row.
#[derive(serde::Serialize)]
pub struct AssetListItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub total_value: f64,
    pub token_price: f64,
    pub available_tokens: i32,
    pub total_tokens: i32,
    pub status: String,
}

pub struct PgCatalogRepository {
    pool: DbPool,
}

impl PgCatalogRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    async fn client(&self) -> Result<deadpool_postgres::Client, DomainError> {
        self.pool
            .get()
            .await
            .map_err(|e| InfraError::Database(e.to_string()).into())
    }

    fn money_from_decimal(raw: &str) -> Result<Money, DomainError> {
        Money::from_decimal_str(raw, FiatCurrency::Usd)
            .map_err(|e| DomainError::InternalError(format!("invalid catalog price: {}", e)))
    }

    pub async fn list_flights(&self) -> Result<Vec<FlightListItem>, DomainError> {
        let client = self.client().await?;
        let rows = client
            .query(
                "SELECT id::TEXT, route, origin, destination, aircraft,
                        CAST(price_usd AS FLOAT8), duration, seats, status
                 FROM flights
                 WHERE status = 'available'
                 ORDER BY price_usd ASC",
                &[],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(rows
            .iter()
            .map(|row| FlightListItem {
                id: row.get(0),
                route: row.get(1),
                origin: row.get(2),
                destination: row.get(3),
                aircraft: row.get(4),
                price_usd: row.get(5),
                duration: row.get(6),
                seats: row.get(7),
                status: row.get(8),
            })
            .collect())
    }

    pub async fn list_assets(&self) -> Result<Vec<AssetListItem>, DomainError> {
        let client = self.client().await?;
        let rows = client
            .query(
                "SELECT id::TEXT, name, type, description, image,
                        CAST(total_value AS FLOAT8), CAST(token_price AS FLOAT8),
                        available_tokens, total_tokens, status
                 FROM assets
                 WHERE status = 'active'
                 ORDER BY token_price ASC",
                &[],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?;
        Ok(rows
            .iter()
            .map(|row| AssetListItem {
                id: row.get(0),
                name: row.get(1),
                asset_type: row.get(2),
                description: row.get(3),
                image: row.get(4),
                total_value: row.get(5),
                token_price: row.get(6),
                available_tokens: row.get(7),
                total_tokens: row.get(8),
                status: row.get(9),
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl CatalogRepository for PgCatalogRepository {
    async fn flight_price(&self, flight_id: &Uuid) -> Result<Money, DomainError> {
        let client = self.client().await?;
        let row = client
            .query_opt(
                "SELECT CAST(price_usd AS TEXT) FROM flights WHERE id = $1 AND status = 'available'",
                &[flight_id],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound(format!("Flight {}", flight_id)))?;
        let raw: String = row.get(0);
        Self::money_from_decimal(&raw)
    }

    async fn asset_price(&self, asset_id: &Uuid) -> Result<Money, DomainError> {
        let client = self.client().await?;
        let row = client
            .query_opt(
                "SELECT CAST(token_price AS TEXT) FROM assets WHERE id = $1 AND status = 'active'",
                &[asset_id],
            )
            .await
            .map_err(|e| InfraError::Database(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound(format!("Asset {}", asset_id)))?;
        let raw: String = row.get(0);
        Self::money_from_decimal(&raw)
    }
}
