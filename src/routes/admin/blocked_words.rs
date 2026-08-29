use actix_web::{web, HttpResponse, Responder};
use actix_session::Session;
use crate::AppState;
use crate::error::AppError;
use crate::routes::admin::require_admin;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BlockedWordForm {
    pub word: String,
    pub category: String,
    pub is_regex: Option<bool>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct BlockedWord {
    id: u32,
    word: String,
    category: String,
    is_regex: i8, // mysql tinyint(1)
}


pub async fn list(
    session: Session,
    state: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    require_admin(&session)?;
    
    let words = sqlx::query_as::<_, BlockedWord>(
        "SELECT id, word, category, is_regex FROM blog_blocked_words ORDER BY category, word"
    )
    .fetch_all(&state.db)
    .await?;

    let mut ctx = tera::Context::new();
    ctx.insert("blocked_words", &words);
    
    let html = state.tera.render("admin/blocked_words.html", &ctx)?;
        
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}


pub async fn create(
    session: Session,
    state: web::Data<AppState>,
    form: web::Form<BlockedWordForm>
) -> Result<impl Responder, AppError> {
    require_admin(&session)?;
    
    let is_regex = form.is_regex.unwrap_or(false);
    
    sqlx::query("INSERT INTO blog_blocked_words (word, category, is_regex) VALUES (?, ?, ?)")
        .bind(&form.word)
        .bind(&form.category)
        .bind(is_regex)
        .execute(&state.db)
        .await?;
    
    let filter = state.content_filter.write().await;
    filter.reload(&state.db).await?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/blocked-words"))
        .finish())
}


pub async fn delete(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<u64>
) -> Result<impl Responder, AppError> {
    require_admin(&session)?;
    let word_id = path.into_inner();
    
    sqlx::query("DELETE FROM blog_blocked_words WHERE id = ?")
        .bind(word_id)
        .execute(&state.db)
        .await?;
        
    let filter = state.content_filter.write().await;
    filter.reload(&state.db).await?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/blocked-words"))
        .finish())
}
