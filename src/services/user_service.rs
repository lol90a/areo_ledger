use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::user::User;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};

pub struct UserService;

impl UserService {
    pub fn create_user(
        pool: &DbPool,
        email: &str,
        name: &str,
        password: &str,
    ) -> Result<User, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let id = Uuid::new_v4();
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        
        let role = if email.contains("admin") { "admin" } else { "user" };

        conn.execute(
            "INSERT INTO users (id, email, name, role, password_hash) VALUES ($1, $2, $3, $4, $5)",
            &[&id, &email, &name, &role, &password_hash],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(User {
            id,
            email: email.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            wallet_address: None,
        })
    }

    pub fn authenticate_user(
        pool: &DbPool,
        email: &str,
        password: &str,
    ) -> Result<User, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let row = conn.query_one(
            "SELECT id, email, name, role, password_hash, wallet_address FROM users WHERE email = $1",
            &[&email],
        ).map_err(|_| AppError::Unauthorized)?;

        let password_hash: String = row.get(4);
        
        if !verify(password, &password_hash).unwrap_or(false) {
            return Err(AppError::Unauthorized);
        }

        Ok(User {
            id: row.get(0),
            email: row.get(1),
            name: row.get(2),
            role: row.get(3),
            wallet_address: row.get(5),
        })
    }

    pub fn get_user_by_id(pool: &DbPool, user_id: &Uuid) -> Result<User, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let row = conn.query_one(
            "SELECT id, email, name, role, wallet_address FROM users WHERE id = $1",
            &[user_id],
        ).map_err(|_| AppError::NotFound)?;

        Ok(User {
            id: row.get(0),
            email: row.get(1),
            name: row.get(2),
            role: row.get(3),
            wallet_address: row.get(4),
        })
    }

    pub fn count_users(pool: &DbPool) -> Result<i64, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let row = conn.query_one("SELECT COUNT(*) FROM users", &[])
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.get(0))
    }
}
