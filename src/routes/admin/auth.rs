use actix_web::{web, HttpResponse, Responder};
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


#[derive(Deserialize)]
pub struct LoginQuery {
    pub reset: Option<String>,
}

pub async fn login_page(
    session: Session,
    state: web::Data<AppState>,
    query: web::Query<LoginQuery>
) -> Result<impl Responder, AppError> {
    if let Ok(Some(true)) = session.get::<bool>("is_admin") {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/blog/admin/dashboard"))
            .finish());
    }

    let mut ctx = tera::Context::new();
    if query.reset.as_deref() == Some("success") {
        ctx.insert("success", "Password berhasil diubah. Silakan login.");
    }
    
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

    if user.is_active == 0 {
        return Err(AppError::BadRequest("Akun Anda telah dinonaktifkan.".into()));
    }

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(format!("Hash parse error: {}", e)))?;
    
    Argon2::default().verify_password(form.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::BadRequest("Username atau password salah.".into()))?;

    session.insert("is_admin", true).map_err(|e| AppError::Internal(e.to_string()))?;
    session.insert("admin_id", user.id).map_err(|e| AppError::Internal(e.to_string()))?;
    session.insert("user_role", user.role.unwrap_or_else(|| "admin".to_string())).map_err(|e| AppError::Internal(e.to_string()))?;
    session.insert("display_name", user.display_name.unwrap_or_else(|| "Tim Jagad Shalawat".to_string())).map_err(|e| AppError::Internal(e.to_string()))?;

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

#[derive(Deserialize)]
pub struct ForgotPasswordForm {
    pub email: String,
}

pub async fn forgot_password_page(
    state: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    let ctx = tera::Context::new();
    let html = state.tera.render("admin/forgot_password.html", &ctx)
        .map_err(|e| AppError::Template(e))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn forgot_password_submit(
    state: web::Data<AppState>,
    form: web::Form<ForgotPasswordForm>
) -> Result<impl Responder, AppError> {
    if let Some(user) = AdminUser::find_by_email(&state.db, &form.email).await? {
        let token = uuid::Uuid::new_v4().to_string();
        AdminUser::set_reset_token(&state.db, user.id, &token, 60).await?;
        
        // Base URL could be from config, but for simplicity we hardcode or construct it
        let reset_link = format!("{}/blog/admin/reset-password?token={}", state.config.base_url, token);
        crate::services::email::send_password_reset_email(&form.email, &reset_link, &state.config).await?;
    }
    
    // Always return success to prevent email enumeration
    let mut ctx = tera::Context::new();
    ctx.insert("success_msg", "Jika email terdaftar, link reset password telah dikirim.");
    let html = state.tera.render("admin/forgot_password.html", &ctx)
        .map_err(|e| AppError::Template(e))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

#[derive(Deserialize)]
pub struct ResetPasswordQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordForm {
    pub token: String,
    pub password: String,
}

pub async fn reset_password_page(
    state: web::Data<AppState>,
    query: web::Query<ResetPasswordQuery>
) -> Result<impl Responder, AppError> {
    let mut ctx = tera::Context::new();
    ctx.insert("token", &query.token);
    let html = state.tera.render("admin/reset_password.html", &ctx)
        .map_err(|e| AppError::Template(e))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn reset_password_submit(
    state: web::Data<AppState>,
    form: web::Form<ResetPasswordForm>
) -> Result<impl Responder, AppError> {
    if let Some(user) = AdminUser::find_by_reset_token(&state.db, &form.token).await? {
        use argon2::{password_hash::{rand_core::OsRng, SaltString}, PasswordHasher};
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default().hash_password(form.password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?
            .to_string();
            
        AdminUser::clear_reset_token_and_update_password(&state.db, user.id, &password_hash).await?;
        
        Ok(HttpResponse::Found()
            .append_header(("Location", "/blog/admin/login?reset=success"))
            .finish())
    } else {
        Err(AppError::BadRequest("Token tidak valid atau sudah kedaluwarsa.".into()))
    }
}
