use actix_web::{web, HttpResponse, Responder, get, post};
use actix_session::Session;
use crate::AppState;
use crate::error::AppError;
use crate::models::admin_user::AdminUser;
use serde::Deserialize;
use argon2::{Argon2, PasswordHash, PasswordVerifier};

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}


pub async fn login_page(
    session: Session,
    state: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    if let Ok(Some(true)) = session.get::<bool>("is_admin") {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/blog/admin/dashboard"))
            .finish());
    }

    let ctx = tera::Context::new();
    let html = state.tera.render("admin/login.html", &ctx)
        .map_err(|e| AppError::Template(e))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}


pub async fn login_submit(
    session: Session,
    state: web::Data<AppState>,
    form: web::Form<LoginForm>
) -> Result<impl Responder, AppError> {
    let user = crate::models::admin_user::AdminUser::find_by_username(&state.db, &form.username).await?;

    let user = user.ok_or_else(|| AppError::BadRequest("Username atau password salah.".into()))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(format!("Hash parse error: {}", e)))?;
    
    Argon2::default().verify_password(form.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::BadRequest("Username atau password salah.".into()))?;

    session.insert("is_admin", true).map_err(|e| AppError::Internal(e.to_string()))?;
    session.insert("admin_id", user.id).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/dashboard"))
        .finish())
}


pub async fn logout(session: Session) -> Result<impl Responder, AppError> {
    session.purge();
    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/login"))
        .finish())
}
