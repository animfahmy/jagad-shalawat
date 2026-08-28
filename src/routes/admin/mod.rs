pub mod auth;
pub mod dashboard;
pub mod posts;
pub mod comments;
pub mod blocked_words;
pub mod media;

use actix_session::Session;
use crate::error::AppError;

pub fn require_admin(session: &Session) -> Result<(), AppError> {
    match session.get::<bool>("is_admin") {
        Ok(Some(true)) => Ok(()),
        _ => Err(AppError::Unauthorized),
    }
}
