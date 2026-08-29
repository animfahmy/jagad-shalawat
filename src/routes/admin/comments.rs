use actix_web::{web, HttpResponse, Responder};
use actix_session::Session;
use sqlx::Row;
use crate::AppState;
use crate::error::AppError;
use crate::routes::admin::require_auth;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CommentQuery {
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct RejectForm {
    pub reason: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AdminCommentItem {
    pub id: u64,
    pub post_id: u64,
    pub post_slug: Option<String>,
    pub post_title: Option<String>,
    pub parent_id: Option<u64>,
    pub author_name: String,
    pub author_email: Option<String>,
    pub author_avatar_url: Option<String>,
    pub oauth_provider: Option<String>,
    pub content: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list(
    session: Session,
    state: web::Data<AppState>,
    query: web::Query<CommentQuery>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    
    let status = query.status.clone().unwrap_or_else(|| "all".to_string());
    let comments = if status == "all" {
        sqlx::query_as::<_, AdminCommentItem>(
            r#"
            SELECT 
                c.id, c.post_id, p.slug AS post_slug, p.title AS post_title, c.parent_id,
                c.author_name, c.author_email, c.author_avatar_url, c.oauth_provider,
                c.content, c.status, c.created_at
            FROM blog_comments c
            LEFT JOIN blog_posts p ON c.post_id = p.id
            ORDER BY c.created_at DESC LIMIT 100
            "#
        )
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, AdminCommentItem>(
            r#"
            SELECT 
                c.id, c.post_id, p.slug AS post_slug, p.title AS post_title, c.parent_id,
                c.author_name, c.author_email, c.author_avatar_url, c.oauth_provider,
                c.content, c.status, c.created_at
            FROM blog_comments c
            LEFT JOIN blog_posts p ON c.post_id = p.id
            WHERE c.status = ?
            ORDER BY c.created_at DESC LIMIT 100
            "#
        )
        .bind(status)
        .fetch_all(&state.db)
        .await?
    };

    let mut ctx = tera::Context::new();
    let role = session.get::<String>("user_role").unwrap_or(None).unwrap_or_else(|| "admin".to_string());
    ctx.insert("user_role", &role);
    ctx.insert("comments", &comments);
    
    let html = state.tera.render("admin/comments.html", &ctx)?;
        
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn approve(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<u64>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    let comment_id = path.into_inner();
    
    let comment_row = sqlx::query("SELECT c.post_id, p.slug, p.slug_en FROM blog_comments c LEFT JOIN blog_posts p ON c.post_id = p.id WHERE c.id = ?")
        .bind(comment_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Komentar tidak ditemukan".into()))?;
        
    let post_id: u64 = comment_row.try_get("post_id").unwrap_or(0);
    let post_slug: Option<String> = comment_row.try_get("slug").unwrap_or(None);
    let post_slug_en: Option<String> = comment_row.try_get("slug_en").unwrap_or(None);
        
    sqlx::query("UPDATE blog_comments SET status = 'approved' WHERE id = ?")
        .bind(comment_id)
        .execute(&state.db)
        .await?;
        
    let _ = state.cache.invalidate_comments(post_id).await;
    if let Some(slug) = post_slug {
        let _ = state.cache.invalidate_post(&slug, post_slug_en.as_deref()).await;
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/comments"))
        .finish())
}

pub async fn reject(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<u64>,
    form: web::Form<RejectForm>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    let comment_id = path.into_inner();
    
    let comment_row = sqlx::query("SELECT c.post_id, p.slug, p.slug_en FROM blog_comments c LEFT JOIN blog_posts p ON c.post_id = p.id WHERE c.id = ?")
        .bind(comment_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Komentar tidak ditemukan".into()))?;
        
    let post_id: u64 = comment_row.try_get("post_id").unwrap_or(0);
    let post_slug: Option<String> = comment_row.try_get("slug").unwrap_or(None);
    let post_slug_en: Option<String> = comment_row.try_get("slug_en").unwrap_or(None);
        
    sqlx::query("UPDATE blog_comments SET status = 'rejected', rejection_reason = ? WHERE id = ?")
        .bind(&form.reason)
        .bind(comment_id)
        .execute(&state.db)
        .await?;
        
    let _ = state.cache.invalidate_comments(post_id).await;
    if let Some(slug) = post_slug {
        let _ = state.cache.invalidate_post(&slug, post_slug_en.as_deref()).await;
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/comments"))
        .finish())
}
