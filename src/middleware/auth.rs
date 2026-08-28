use actix_session::Session;
use actix_web::HttpResponse;

use crate::error::AppError;

/// Check if the current session belongs to an authenticated admin user.
/// Returns `Ok(())` if authenticated, `Err(Unauthorized)` otherwise.
pub fn require_admin(session: &Session) -> Result<(), AppError> {
    match session.get::<bool>("is_admin") {
        Ok(Some(true)) => Ok(()),
        _ => Err(AppError::Unauthorized),
    }
}

/// Redirect to admin login page if not authenticated.
pub fn redirect_to_login() -> HttpResponse {
    HttpResponse::Found()
        .insert_header(("Location", "/blog/admin/login"))
        .finish()
}
