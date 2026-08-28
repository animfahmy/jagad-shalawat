use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AdminUser {
    pub id: u64,
    pub username: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub is_active: i8,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl AdminUser {
    pub async fn find_by_username(pool: &MySqlPool, username: &str) -> Result<Option<AdminUser>, AppError> {
        let user = sqlx::query_as::<_, AdminUser>(
            "SELECT id, username, password_hash, display_name, is_active, last_login_at, created_at FROM blog_admin_users WHERE username = ?"
        )
        .bind(username)
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
}
