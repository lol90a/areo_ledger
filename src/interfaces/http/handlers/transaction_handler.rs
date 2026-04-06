use actix_web::{web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::application::use_cases::get_transactions::GetTransactions;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::persistence::postgres::pg_transaction_repository::PgTransactionRepository;
use crate::interfaces::http::auth::require_same_user_or_admin;
use crate::interfaces::http::error_response::into_response;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/transactions").route("/{user_id}", web::get().to(get_user_transactions)),
    );
}

async fn get_user_transactions(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user_id = path.into_inner();
    if let Err(e) = require_same_user_or_admin(&req, &user_id) {
        return into_response(e);
    }

    let use_case = GetTransactions::new(PgTransactionRepository::new(pool.get_ref().clone()));
    match use_case.execute(user_id).await {
        Ok(txs) => HttpResponse::Ok().json(txs),
        Err(e) => into_response(e),
    }
}
