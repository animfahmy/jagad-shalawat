use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AdminUser {
    pub id: u64,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub is_active: i8,
    pub reset_token: Option<String>,
    pub reset_token_expires_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl AdminUser {
    pub async fn find_by_username(pool: &MySqlPool, username: &str) -> Result<Option<AdminUser>, AppError> {
        let user = sqlx::query_as::<_, AdminUser>(
            "SELECT id, username, email, password_hash, display_name, role, is_active, reset_token, reset_token_expires_at, last_login_at, created_at FROM blog_admin_users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn find_by_email(pool: &MySqlPool, email: &str) -> Result<Option<AdminUser>, AppError> {
        let user = sqlx::query_as::<_, AdminUser>(
            "SELECT id, username, email, password_hash, display_name, role, is_active, reset_token, reset_token_expires_at, last_login_at, created_at FROM blog_admin_users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn find_by_reset_token(pool: &MySqlPool, token: &str) -> Result<Option<AdminUser>, AppError> {
        let user = sqlx::query_as::<_, AdminUser>(
            "SELECT id, username, email, password_hash, display_name, role, is_active, reset_token, reset_token_expires_at, last_login_at, created_at FROM blog_admin_users WHERE reset_token = ? AND reset_token_expires_at > NOW()"
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn update_last_login(pool: &MySqlPool, id: u64) -> Result<(), AppError> {
        sqlx::query("UPDATE blog_admin_users SET last_login_at = NOW() WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn set_reset_token(pool: &MySqlPool, id: u64, token: &str, expires_in_minutes: i64) -> Result<(), AppError> {
        sqlx::query("UPDATE blog_admin_users SET reset_token = ?, reset_token_expires_at = DATE_ADD(NOW(), INTERVAL ? MINUTE) WHERE id = ?")
            .bind(token)
            .bind(expires_in_minutes)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn clear_reset_token_and_update_password(pool: &MySqlPool, id: u64, new_password_hash: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE blog_admin_users SET password_hash = ?, reset_token = NULL, reset_token_expires_at = NULL WHERE id = ?")
            .bind(new_password_hash)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
