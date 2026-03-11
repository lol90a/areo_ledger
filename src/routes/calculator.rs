use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use crate::errors::AppError;
use crate::services::CalculatorService;

#[derive(Deserialize)]
pub struct ConversionRequest {
    pub amount: f64,
    pub from_currency: String,
    pub to_currency: String,
}

#[derive(Serialize)]
pub struct ConversionResponse {
    pub amount: f64,
    pub from_currency: String,
    pub to_currency: String,
    pub result: f64,
    pub rate: f64,
}

#[derive(Serialize)]
pub struct PricesResponse {
    pub crypto: std::collections::HashMap<String, crate::services::calculator_service::CurrencyInfo>,
    pub fiat: std::collections::HashMap<String, crate::services::calculator_service::CurrencyInfo>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/calculator")
            .route("/convert", web::post().to(convert))
            .route("/prices", web::get().to(get_prices))
    );
}

async fn convert(
    payload: web::Json<ConversionRequest>,
) -> Result<HttpResponse, AppError> {
    let (result, rate) = CalculatorService::convert(
        payload.amount,
        &payload.from_currency,
        &payload.to_currency,
    );

    Ok(HttpResponse::Ok().json(ConversionResponse {
        amount: payload.amount,
        from_currency: payload.from_currency.clone(),
        to_currency: payload.to_currency.clone(),
        result,
        rate,
    }))
}

async fn get_prices() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(PricesResponse {
        crypto: CalculatorService::get_crypto_prices(),
        fiat: CalculatorService::get_fiat_rates(),
    }))
}
