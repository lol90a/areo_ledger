use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::application::dto::{CreateUserInput, AuthenticateUserInput};
use crate::application::use_cases::create_user::CreateUser;
use crate::application::use_cases::authenticate_user::AuthenticateUser;
use crate::infrastructure::auth::jwt_service::JwtService;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::persistence::postgres::pg_audit_log_repository::PgAuditLogRepository;
use crate::infrastructure::persistence::postgres::pg_user_repository::PgUserRepository;
use crate::interfaces::http::error_response::{into_response, request_metadata};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub token: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/signup", web::post().to(signup))
            .route("/login", web::post().to(login)),
    );
}

async fn signup(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    jwt: web::Data<JwtService>,
    body: web::Json<SignupRequest>,
) -> HttpResponse {
    let use_case = CreateUser::new(PgUserRepository::new(pool.get_ref().clone()));
    match use_case.execute(CreateUserInput {
        email: body.email.clone(),
        name: body.name.clone(),
        password: body.password.clone(),
    }).await {
        Ok(user) => match jwt.create_token(&user.id.to_string(), &user.email, &user.role) {
            Ok(token) => {
                let (ip, ua) = request_metadata(&req);
                let _ = PgAuditLogRepository::new(pool.get_ref().clone()).record(
                    Some(user.id),
                    "user.signup",
                    "user",
                    &user.id.to_string(),
                    serde_json::json!({ "email": user.email, "role": user.role }),
                    ip,
                    ua,
                ).await;
                HttpResponse::Ok().json(AuthResponse {
                    id: user.id.to_string(), email: user.email,
                    name: user.name, role: user.role, token,
                })
            }
            Err(e) => into_response(e),
        },
        Err(e) => into_response(e),
    }
}

async fn login(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    jwt: web::Data<JwtService>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let use_case = AuthenticateUser::new(PgUserRepository::new(pool.get_ref().clone()));
    match use_case.execute(AuthenticateUserInput {
        email: body.email.clone(),
        password: body.password.clone(),
    }).await {
        Ok(user) => match jwt.create_token(&user.id.to_string(), &user.email, &user.role) {
            Ok(token) => {
                let (ip, ua) = request_metadata(&req);
                let _ = PgAuditLogRepository::new(pool.get_ref().clone()).record(
                    Some(user.id),
                    "user.login",
                    "user",
                    &user.id.to_string(),
                    serde_json::json!({ "email": user.email, "role": user.role }),
                    ip,
                    ua,
                ).await;
                HttpResponse::Ok().json(AuthResponse {
                    id: user.id.to_string(), email: user.email,
                    name: user.name, role: user.role, token,
                })
            }
            Err(e) => into_response(e),
        },
        Err(e) => into_response(e),
    }
}
