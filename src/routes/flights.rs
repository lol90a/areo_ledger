use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchFlightsQuery {
    pub search: Option<String>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/flights")
            .route("", web::get().to(search_flights))
            .route("/{id}", web::get().to(get_flight))
    );
}

async fn search_flights(query: web::Query<SearchFlightsQuery>) -> Result<HttpResponse> {
    // TODO: integrate real flight API
    let flights = if let Some(search) = &query.search {
        vec![
            serde_json::json!({
                "id": "1",
                "origin": "New York",
                "destination": search.clone(),
                "departure_time": "2026-01-25T10:00:00Z",
                "price_usd": 500.0
            }),
            serde_json::json!({
                "id": "2",
                "origin": "Los Angeles",
                "destination": search.clone(),
                "departure_time": "2026-01-26T14:00:00Z",
                "price_usd": 750.0
            })
        ]
    } else {
        vec![
            serde_json::json!({
                "id": "1",
                "origin": "New York",
                "destination": "London",
                "departure_time": "2026-01-25T10:00:00Z",
                "price_usd": 500.0
            }),
            serde_json::json!({
                "id": "2",
                "origin": "Los Angeles",
                "destination": "Paris",
                "departure_time": "2026-01-26T14:00:00Z",
                "price_usd": 750.0
            }),
            serde_json::json!({
                "id": "3",
                "origin": "Chicago",
                "destination": "Tokyo",
                "departure_time": "2026-01-27T18:00:00Z",
                "price_usd": 1200.0
            })
        ]
    };

    Ok(HttpResponse::Ok().json(flights))
}

async fn get_flight(_path: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json("Flight details"))
}
