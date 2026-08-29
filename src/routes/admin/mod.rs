pub mod auth;
pub mod dashboard;
pub mod posts;
pub mod comments;
pub mod blocked_words;
pub mod media;
pub mod contributors;

use actix_session::Session;
use crate::error::AppError;

pub fn require_admin(session: &Session) -> Result<(), AppError> {
    match session.get::<bool>("is_admin") {
        Ok(Some(true)) => {
            // Check if user is strictly admin (not contributor)
            // If user_role is not set, default to "admin" for backward compatibility with old sessions
            let role = session.get::<String>("user_role").unwrap_or(None).unwrap_or_else(|| "admin".to_string());
            if role == "admin" {
                return Ok(());
            }
            Err(AppError::Unauthorized)
        },
        _ => Err(AppError::Unauthorized),
    }
}

pub fn require_auth(session: &Session) -> Result<(), AppError> {
    match session.get::<bool>("is_admin") {
        Ok(Some(true)) => Ok(()),
        _ => Err(AppError::Unauthorized),
    }
}
