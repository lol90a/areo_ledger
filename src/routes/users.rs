use actix_web::{web, Result};
use serde::{Deserialize, Serialize};
use crate::errors::AppError;
use crate::auth;

#[derive(Deserialize)]
pub struct CreateUserRequest {
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
pub struct UserResponse {
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
            .route("/{id}", web::get().to(get_user))
    );
}

// User registration with password hashing
async fn signup(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<CreateUserRequest>,
) -> Result<web::Json<UserResponse>, AppError> {
    let password_hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
        .map_err(|_| AppError::ValidationError("Password hashing failed".to_string()))?;

    let user_id = uuid::Uuid::new_v4().to_string();
    let role = if payload.email.contains("admin") { "admin" } else { "user" };

    sqlx::query(
        "INSERT INTO users (id, email, name, role, password_hash) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&user_id)
    .bind(&payload.email)
    .bind(&payload.name)
    .bind(role)
    .bind(&password_hash)
    .execute(&**pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let token = auth::create_token(&user_id, &payload.email, role)
        .map_err(|_| AppError::ValidationError("Token generation failed".to_string()))?;

    Ok(web::Json(UserResponse {
        id: user_id,
        email: payload.email.clone(),
        name: payload.name.clone(),
        role: role.to_string(),
        token,
    }))
}

// Login with password verification
async fn login(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<LoginRequest>,
) -> Result<web::Json<UserResponse>, AppError> {
    let user: Option<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, email, name, role, password_hash FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&**pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if let Some((id, email, name, role, password_hash)) = user {
        let valid = bcrypt::verify(&payload.password, &password_hash)
            .map_err(|_| AppError::ValidationError("Password verification failed".to_string()))?;

        if !valid {
            return Err(AppError::ValidationError("Invalid credentials".to_string()));
        }

        let token = auth::create_token(&id, &email, &role)
            .map_err(|_| AppError::ValidationError("Token generation failed".to_string()))?;

        Ok(web::Json(UserResponse { id, email, name, role, token }))
    } else {
        Err(AppError::NotFound)
    }
}

async fn get_user(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<String>,
) -> Result<web::Json<UserResponse>, AppError> {
    let user: (String, String, String, String) = sqlx::query_as(
        "SELECT id, email, name, role FROM users WHERE id = $1"
    )
    .bind(path.as_str())
    .fetch_one(&**pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let token = auth::create_token(&user.0, &user.1, &user.2)
        .map_err(|_| AppError::ValidationError("Token generation failed".to_string()))?;

    Ok(web::Json(UserResponse {
        id: user.0,
        email: user.1,
        name: user.2,
        role: user.3,
        token,
    }))
}
