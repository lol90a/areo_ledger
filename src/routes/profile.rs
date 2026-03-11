use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub wallet_address: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn get_profile(
    pool: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
) -> impl Responder {
    let user_id = user_id.into_inner();

    let user = sqlx::query_as!(
        ProfileResponse,
        "SELECT id, email, name, role, wallet_address, created_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn update_profile(
    pool: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
    update: web::Json<UpdateProfileRequest>,
) -> impl Responder {
    let user_id = user_id.into_inner();

    if let Some(ref email) = update.email {
        let exists = sqlx::query!("SELECT id FROM users WHERE email = $1 AND id != $2", email, user_id)
            .fetch_optional(pool.get_ref())
            .await;

        if let Ok(Some(_)) = exists {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": "Email already in use"}));
        }
    }

    let result = sqlx::query!(
        r#"
        UPDATE users 
        SET name = COALESCE($1, name),
            email = COALESCE($2, email),
            wallet_address = COALESCE($3, wallet_address)
        WHERE id = $4
        "#,
        update.name,
        update.email,
        update.wallet_address,
        user_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Profile updated successfully"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn change_password(
    pool: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
    request: web::Json<ChangePasswordRequest>,
) -> impl Responder {
    let user_id = user_id.into_inner();

    let user = sqlx::query!("SELECT password_hash FROM users WHERE id = $1", user_id)
        .fetch_optional(pool.get_ref())
        .await;

    match user {
        Ok(Some(user)) => {
            if !verify(&request.current_password, &user.password_hash).unwrap_or(false) {
                return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Current password is incorrect"}));
            }

            let new_hash = match hash(&request.new_password, DEFAULT_COST) {
                Ok(h) => h,
                Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
            };

            let result = sqlx::query!("UPDATE users SET password_hash = $1 WHERE id = $2", new_hash, user_id)
                .execute(pool.get_ref())
                .await;

            match result {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Password changed successfully"})),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/profile")
            .route("/{user_id}", web::get().to(get_profile))
            .route("/{user_id}", web::put().to(update_profile))
            .route("/{user_id}/password", web::put().to(change_password))
    );
}
