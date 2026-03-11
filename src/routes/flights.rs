use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use crate::services::FlightService;

#[derive(Deserialize)]
pub struct SearchFlightsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub date: Option<String>,
    pub passengers: Option<i32>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/flights")
            .route("", web::get().to(search_flights))
    );
}

async fn search_flights(query: web::Query<SearchFlightsQuery>) -> Result<HttpResponse> {
    let flights = FlightService::search_flights(
        query.from.as_deref(),
        query.to.as_deref(),
        query.passengers,
    );

    Ok(HttpResponse::Ok().json(flights))
}
