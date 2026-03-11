use actix_web::{web, HttpResponse, Result};
use crate::db::DbPool;
use crate::errors::AppError;
use crate::services::TransactionService;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/transactions")
            .route("/{user_id}", web::get().to(get_user_transactions))
    );
}

async fn get_user_transactions(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    
    let transactions = TransactionService::get_user_transactions(&pool, &user_id)?;
    
    Ok(HttpResponse::Ok().json(transactions))
}
