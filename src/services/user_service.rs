use uuid::Uuid;
use crate::models::user::User;
use crate::errors::AppError;

pub async fn create_user(
    pool: &sqlx::PgPool,
    email: String,
    name: String,
    wallet_address: Option<String>,
) -> Result<User, AppError> {
    let user_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, name, wallet_address)
        VALUES (?,?,?,?)
        "#,
    )
    .bind(user_id)
    .bind(email.clone())
    .bind(name.clone())
    .bind(&wallet_address)
    .execute(pool)
    .await?;

    Ok(User {
        id: user_id,
        email,
        name,
        wallet_address,
    })
}

pub async fn get_user_by_id(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, name, wallet_address FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}
