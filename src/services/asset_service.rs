use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub description: String,
    pub image: String,
    pub total_value: f64,
    pub token_price: f64,
    pub available_tokens: i32,
    pub total_tokens: i32,
    pub roi: f64,
    pub location: String,
    pub specifications: serde_json::Value,
}

pub struct AssetService;

impl AssetService {
    pub fn get_all_assets() -> Vec<Asset> {
        vec![
            Asset {
                id: "1".to_string(),
                name: "Gulfstream G650".to_string(),
                asset_type: "jet".to_string(),
                description: "The Gulfstream G650 represents the pinnacle of private aviation excellence with ultra-long range capability.".to_string(),
                image: "https://images.unsplash.com/photo-1540962351504-03099e0a754b?w=800".to_string(),
                total_value: 65000000.0,
                token_price: 10000.0,
                available_tokens: 3250,
                total_tokens: 6500,
                roi: 18.5,
                location: "Miami, FL".to_string(),
                specifications: serde_json::json!({
                    "range": "7,000nm",
                    "speed": "Mach 0.925",
                    "seats": "19",
                    "manufacturer": "Gulfstream"
                }),
            },
            Asset {
                id: "2".to_string(),
                name: "Sunseeker 131 Yacht".to_string(),
                asset_type: "yacht".to_string(),
                description: "British yacht building masterpiece with 5 luxurious cabins and beach club.".to_string(),
                image: "https://images.unsplash.com/photo-1567899378494-47b22a2ae96a?w=800".to_string(),
                total_value: 28000000.0,
                token_price: 5000.0,
                available_tokens: 2800,
                total_tokens: 5600,
                roi: 15.2,
                location: "Monaco".to_string(),
                specifications: serde_json::json!({
                    "length": "131ft",
                    "cabins": "5",
                    "crew": "8",
                    "builder": "Sunseeker"
                }),
            },
            Asset {
                id: "3".to_string(),
                name: "Manhattan Penthouse".to_string(),
                asset_type: "real-estate".to_string(),
                description: "Extraordinary penthouse in Manhattan with 360-degree views and private elevator.".to_string(),
                image: "https://images.unsplash.com/photo-1545324418-cc1a3fa10c00?w=800".to_string(),
                total_value: 45000000.0,
                token_price: 15000.0,
                available_tokens: 1500,
                total_tokens: 3000,
                roi: 12.8,
                location: "New York, NY".to_string(),
                specifications: serde_json::json!({
                    "sqft": "8,500",
                    "bedrooms": "5",
                    "bathrooms": "6",
                    "city": "NYC"
                }),
            },
            Asset {
                id: "4".to_string(),
                name: "Bugatti Chiron".to_string(),
                asset_type: "car".to_string(),
                description: "Hypercar pushing boundaries with quad-turbo W16 engine and carbon fiber construction.".to_string(),
                image: "https://images.unsplash.com/photo-1544636331-e26879cd4d9b?w=800".to_string(),
                total_value: 3500000.0,
                token_price: 2500.0,
                available_tokens: 700,
                total_tokens: 1400,
                roi: 22.3,
                location: "Dubai, UAE".to_string(),
                specifications: serde_json::json!({
                    "horsepower": "1,500 HP",
                    "acceleration": "0-60: 2.4s",
                    "top_speed": "261mph",
                    "manufacturer": "Bugatti"
                }),
            },
            Asset {
                id: "5".to_string(),
                name: "Bombardier Global 7500".to_string(),
                asset_type: "jet".to_string(),
                description: "Sets new standards in business aviation with longest range and four living spaces.".to_string(),
                image: "https://images.unsplash.com/photo-1583792928584-5e9d36229fc8?w=800".to_string(),
                total_value: 73000000.0,
                token_price: 12000.0,
                available_tokens: 4050,
                total_tokens: 6083,
                roi: 19.7,
                location: "London, UK".to_string(),
                specifications: serde_json::json!({
                    "range": "7,700nm",
                    "speed": "Mach 0.925",
                    "seats": "19",
                    "manufacturer": "Bombardier"
                }),
            },
            Asset {
                id: "6".to_string(),
                name: "Azimut Grande 35M".to_string(),
                asset_type: "yacht".to_string(),
                description: "Italian luxury yacht with 4 cabins and exceptional design.".to_string(),
                image: "https://images.unsplash.com/photo-1605281317010-fe5ffe798166?w=800".to_string(),
                total_value: 18000000.0,
                token_price: 4000.0,
                available_tokens: 2250,
                total_tokens: 4500,
                roi: 14.5,
                location: "Ibiza, Spain".to_string(),
                specifications: serde_json::json!({
                    "length": "115ft",
                    "cabins": "4",
                    "crew": "6",
                    "builder": "Azimut"
                }),
            },
        ]
    }

    pub fn get_asset_by_id(id: &str) -> Option<Asset> {
        Self::get_all_assets().into_iter().find(|a| a.id == id)
    }
}
