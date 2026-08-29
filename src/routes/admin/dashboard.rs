use actix_web::{web, HttpResponse, Responder};
use actix_session::Session;
use crate::AppState;
use crate::error::AppError;
use crate::routes::admin::require_auth;
use chrono::{DateTime, Utc};

#[derive(serde::Serialize, sqlx::FromRow)]
struct DashboardStats {
    total_published_posts: i64,
    total_draft_posts: i64,
    total_pending_comments: i64,
    total_views: i64,
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct RecentPost {
    id: u64,
    title: String,
    status: String,
    created_at: DateTime<Utc>,
}


pub async fn index(
    session: Session,
    state: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;

    let total_published: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blog_posts WHERE status = 'published'")
        .fetch_one(&state.db)
        .await?;

    let total_draft: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blog_posts WHERE status = 'draft'")
        .fetch_one(&state.db)
        .await?;

    let total_pending: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blog_comments WHERE status = 'pending'")
        .fetch_one(&state.db)
        .await?;

    let total_views: (i64,) = sqlx::query_as("SELECT CAST(COALESCE(SUM(view_count), 0) AS SIGNED) FROM blog_posts")
        .fetch_one(&state.db)
        .await?;
        
    let views_val = total_views.0;

    let recent_posts: Vec<RecentPost> = sqlx::query_as(
        "SELECT id, title, status, created_at FROM blog_posts ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let mut ctx = tera::Context::new();
    let role = session.get::<String>("user_role").unwrap_or(None).unwrap_or_else(|| "admin".to_string());
    ctx.insert("user_role", &role);
    ctx.insert("stats", &DashboardStats {
        total_published_posts: total_published.0,
        total_draft_posts: total_draft.0,
        total_pending_comments: total_pending.0,
        total_views: views_val,
    });
    ctx.insert("recent_posts", &recent_posts);

    let html = state.tera.render("admin/dashboard.html", &ctx)?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}


pub async fn clear_cache(
    session: Session,
    state: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;

    state.cache.delete_pattern("blog:*").await?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/dashboard"))
        .finish())
}
