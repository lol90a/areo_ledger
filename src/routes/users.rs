use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use crate::db::DbPool;
use crate::errors::AppError;
use crate::services::UserService;
use crate::auth;

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
            .route("/login", web::post().to(login))
    );
}

async fn signup(
    pool: web::Data<DbPool>,
    payload: web::Json<SignupRequest>,
) -> Result<HttpResponse, AppError> {
    let user = UserService::create_user(
        &pool,
        &payload.email,
        &payload.name,
        &payload.password,
    )?;

    let token = auth::generate_token(&user.id, &user.role)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        id: user.id.to_string(),
        email: user.email,
        name: user.name,
        role: user.role,
        token,
    }))
}

async fn login(
    pool: web::Data<DbPool>,
    payload: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let user = UserService::authenticate_user(
        &pool,
        &payload.email,
        &payload.password,
    )?;

    let token = auth::generate_token(&user.id, &user.role)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        id: user.id.to_string(),
        email: user.email,
        name: user.name,
        role: user.role,
        token,
    }))
}
