use actix_web::{web, HttpResponse, Result};
use crate::db::DbPool;
use crate::errors::AppError;
use crate::services::PortfolioService;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/portfolio")
            .route("/{user_id}", web::get().to(get_user_portfolio))
    );
}

async fn get_user_portfolio(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    
    let portfolio = PortfolioService::get_user_portfolio(&pool, &user_id)?;
    
    Ok(HttpResponse::Ok().json(portfolio))
}
