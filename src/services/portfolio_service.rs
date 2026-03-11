use crate::db::DbPool;
use crate::errors::AppError;
use serde::Serialize;

#[derive(Serialize)]
pub struct PortfolioAsset {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub quantity: i32,
    pub purchase_price: f64,
    pub current_value: f64,
    pub status: String,
}

#[derive(Serialize)]
pub struct PortfolioResponse {
    pub total_purchase: f64,
    pub total_current_value: f64,
    pub total_assets: i32,
    pub assets: Vec<PortfolioAsset>,
}

pub struct PortfolioService;

impl PortfolioService {
    pub fn get_user_portfolio(pool: &DbPool, user_id: &str) -> Result<PortfolioResponse, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        // Query user's confirmed orders
        let _rows = conn.query(
            "SELECT id, grand_total, status FROM orders WHERE user_id = $1 AND status = 'confirmed'",
            &[&user_id],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Mock portfolio data - in production, calculate from actual orders
        let assets = vec![
            PortfolioAsset {
                id: "1".to_string(),
                name: "Gulfstream G650".to_string(),
                asset_type: "Private Jet".to_string(),
                quantity: 1,
                purchase_price: 65000000.0,
                current_value: 65000000.0,
                status: "Owned".to_string(),
            },
        ];

        let total_purchase: f64 = assets.iter().map(|a| a.purchase_price).sum();
        let total_current_value: f64 = assets.iter().map(|a| a.current_value).sum();
        let total_assets = assets.len() as i32;

        Ok(PortfolioResponse {
            total_purchase,
            total_current_value,
            total_assets,
            assets,
        })
    }
}
