use actix_web::{web, HttpResponse, Responder, get, post};
use actix_session::Session;
use crate::AppState;
use crate::error::AppError;
use crate::routes::admin::require_admin;
use crate::models::admin_user::AdminUser;
use serde::Deserialize;
use argon2::{Argon2, PasswordHash, PasswordHasher, password_hash::{rand_core::OsRng, SaltString}};

#[derive(Deserialize)]
pub struct ContributorForm {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password: Option<String>,
    pub is_active: Option<String>, // 'on' if checked
}

pub async fn list(
    session: Session,
    state: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    require_admin(&session)?;
    
    let users = sqlx::query_as::<_, AdminUser>(
        "SELECT id, username, email, password_hash, display_name, role, is_active, reset_token, reset_token_expires_at, last_login_at, created_at FROM blog_admin_users WHERE role = 'contributor' ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let mut ctx = tera::Context::new();
    ctx.insert("users", &users);
    
    let html = state.tera.render("admin/contributors/list.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn new_form(
    session: Session,
    state: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    require_admin(&session)?;
    let ctx = tera::Context::new();
    let html = state.tera.render("admin/contributors/form.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn create(
    session: Session,
    state: web::Data<AppState>,
    form: web::Form<ContributorForm>
) -> Result<impl Responder, AppError> {
    require_admin(&session)?;
    
    let password = form.password.as_deref().unwrap_or("password123");
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default().hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?
        .to_string();
        
    let is_active = if form.is_active.is_some() { 1 } else { 0 };

    sqlx::query(
        "INSERT INTO blog_admin_users (username, email, password_hash, display_name, role, is_active) VALUES (?, ?, ?, ?, 'contributor', ?)"
    )
    .bind(&form.username)
    .bind(&form.email)
    .bind(password_hash)
    .bind(&form.display_name)
    .bind(is_active)
    .execute(&state.db)
    .await?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/contributors"))
        .finish())
}

pub async fn edit_form(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<u64>
) -> Result<impl Responder, AppError> {
    require_admin(&session)?;
    let user_id = path.into_inner();
    
    let user = sqlx::query_as::<_, AdminUser>(
        "SELECT id, username, email, password_hash, display_name, role, is_active, reset_token, reset_token_expires_at, last_login_at, created_at FROM blog_admin_users WHERE id = ? AND role = 'contributor'"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Kontributor tidak ditemukan".into()))?;

    let mut ctx = tera::Context::new();
    ctx.insert("user", &user);
    let html = state.tera.render("admin/contributors/form.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn update(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<u64>,
    form: web::Form<ContributorForm>
) -> Result<impl Responder, AppError> {
    require_admin(&session)?;
    let user_id = path.into_inner();
    let is_active = if form.is_active.is_some() { 1 } else { 0 };

    if let Some(password) = &form.password {
        if !password.trim().is_empty() {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = Argon2::default().hash_password(password.as_bytes(), &salt)
                .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?
                .to_string();
                
            sqlx::query(
                "UPDATE blog_admin_users SET username = ?, email = ?, display_name = ?, is_active = ?, password_hash = ? WHERE id = ? AND role = 'contributor'"
            )
            .bind(&form.username)
            .bind(&form.email)
            .bind(&form.display_name)
            .bind(is_active)
            .bind(password_hash)
            .bind(user_id)
            .execute(&state.db)
            .await?;
            
            return Ok(HttpResponse::Found().append_header(("Location", "/blog/admin/contributors")).finish());
        }
    }

    sqlx::query(
        "UPDATE blog_admin_users SET username = ?, email = ?, display_name = ?, is_active = ? WHERE id = ? AND role = 'contributor'"
    )
    .bind(&form.username)
    .bind(&form.email)
    .bind(&form.display_name)
    .bind(is_active)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/contributors"))
        .finish())
}
