#![allow(dead_code)]
use actix_web::{web, HttpResponse, Responder};
use actix_session::Session;
use crate::AppState;
use crate::error::AppError;
use crate::routes::admin::require_auth;
use serde::Deserialize;
use slug::slugify;
use crate::services::markdown::{render_markdown, calculate_reading_time, extract_first_paragraph};
use crate::models::post::BlogPost;
use chrono::Utc;
use sqlx::Row;

#[derive(Deserialize)]
pub struct PostQuery {
    pub status: Option<String>,
    pub page: Option<u32>,
}

#[derive(Deserialize)]
pub struct PostForm {
    pub title: String,
    pub slug: Option<String>,
    pub content_markdown: String,
    pub title_en: Option<String>,
    pub slug_en: Option<String>,
    pub excerpt_en: Option<String>,
    pub content_markdown_en: Option<String>,
    pub meta_description: Option<String>,
    pub meta_description_en: Option<String>,
    pub excerpt: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub sources: Option<String>,
    pub source_url: Option<String>,
    pub source_name: Option<String>,
    pub featured_image: Option<String>,
    pub status: Option<String>,
}


pub async fn list(
    session: Session,
    state: web::Data<AppState>,
    query: web::Query<PostQuery>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    
    let status = query.status.clone().unwrap_or_else(|| "all".to_string());
    
    let posts = if status == "all" {
        sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts ORDER BY created_at DESC LIMIT 50")
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts WHERE status = ? ORDER BY created_at DESC LIMIT 50")
            .bind(&status)
            .fetch_all(&state.db)
            .await?
    };

    let mut ctx = tera::Context::new();
    let role = session.get::<String>("user_role").unwrap_or(None).unwrap_or_else(|| "admin".to_string());
    ctx.insert("user_role", &role);
    ctx.insert("posts", &posts);
    
    let html = state.tera.render("admin/posts_list.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}


pub async fn new_form(
    session: Session,
    state: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    let mut ctx = tera::Context::new();
    let role = session.get::<String>("user_role").unwrap_or(None).unwrap_or_else(|| "admin".to_string());
    ctx.insert("user_role", &role);
    let html = state.tera.render("admin/post_editor.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}


fn parse_tags_json(raw: Option<&str>) -> Option<String> {
    let t = raw?.trim();
    if t.is_empty() {
        return None;
    }
    let cleaned = t.trim_start_matches('[').trim_end_matches(']');
    let tags: Vec<String> = cleaned
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if tags.is_empty() {
        None
    } else {
        serde_json::to_string(&tags).ok()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SourceItem {
    pub title: String,
    pub url: String,
}

fn parse_sources_json(raw: Option<&str>) -> Option<String> {
    let t = raw?.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(t) {
        if val.is_array() {
            return Some(val.to_string());
        }
    }
    let mut items = Vec::new();
    for line in t.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (title, url) = if let Some((left, right)) = trimmed.split_once('|') {
            (left.trim().to_string(), right.trim().to_string())
        } else {
            let domain = url::Url::parse(trimmed)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string()))
                .unwrap_or_else(|| trimmed.to_string());
            (domain, trimmed.to_string())
        };
        items.push(SourceItem { title, url });
    }
    if items.is_empty() {
        None
    } else {
        serde_json::to_string(&items).ok()
    }
}

pub async fn create(
    session: Session,
    state: web::Data<AppState>,
    form: web::Form<PostForm>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    
    let slug = form.slug.as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| slugify(&form.title));
    let content_html = render_markdown(&form.content_markdown);
    let reading_time = (calculate_reading_time(&form.content_markdown).min(255)) as u8;
    let excerpt = form.excerpt.as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| extract_first_paragraph(&content_html));
    let status = form.status.as_deref().unwrap_or("draft");
    let tags_json = parse_tags_json(form.tags.as_deref());
    let sources_json = parse_sources_json(form.sources.as_deref());
    
    let title_en = form.title_en.as_deref().filter(|s| !s.trim().is_empty());
    let slug_en = form.slug_en.as_deref().filter(|s| !s.trim().is_empty());
    let excerpt_en = form.excerpt_en.as_deref().filter(|s| !s.trim().is_empty());
    let content_markdown_en = form.content_markdown_en.as_deref().filter(|s| !s.trim().is_empty());
    let content_html_en = content_markdown_en.map(render_markdown);
    let meta_description_en = form.meta_description_en.as_deref().filter(|s| !s.trim().is_empty());
    
    let published_at = if status == "published" {
        Some(Utc::now())
    } else {
        None
    };

    let display_name = session.get::<String>("display_name").unwrap_or(None).unwrap_or_else(|| "Tim Jagad Shalawat".to_string());

    sqlx::query(
        r#"
        INSERT INTO blog_posts (
            title, slug, content_markdown, content_html, 
            title_en, slug_en, excerpt_en, content_markdown_en, content_html_en, meta_description_en,
            meta_description, excerpt,
            category, tags, sources, source_url, source_name, featured_image, status,
            reading_time_minutes, published_at, author_name
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&form.title)
        .bind(&slug)
        .bind(&form.content_markdown)
        .bind(&content_html)
        .bind(title_en)
        .bind(slug_en)
        .bind(excerpt_en)
        .bind(content_markdown_en)
        .bind(content_html_en.as_deref())
        .bind(meta_description_en)
        .bind(&form.meta_description)
        .bind(&excerpt)
        .bind(&form.category)
        .bind(tags_json.as_deref())
        .bind(sources_json.as_deref())
        .bind(&form.source_url)
        .bind(&form.source_name)
        .bind(&form.featured_image)
        .bind(status)
        .bind(reading_time)
        .bind(published_at)
        .bind(&display_name)
        .execute(&state.db)
        .await?;
    
    let _ = state.cache.invalidate_all_listings().await;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/posts"))
        .finish())
}


pub async fn edit_form(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<u64>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    let post_id = path.into_inner();
    
    let post = sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts WHERE id = ?")
        .bind(post_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Artikel tidak ditemukan".into()))?;
        
    let mut ctx = tera::Context::new();
    let role = session.get::<String>("user_role").unwrap_or(None).unwrap_or_else(|| "admin".to_string());
    ctx.insert("user_role", &role);
    ctx.insert("post", &post);
    let html = state.tera.render("admin/post_editor.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}


pub async fn update(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<u64>,
    form: web::Form<PostForm>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    let post_id = path.into_inner();
    
    let slug = form.slug.as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| slugify(&form.title));
    let content_html = render_markdown(&form.content_markdown);
    let reading_time = (calculate_reading_time(&form.content_markdown).min(255)) as u8;
    let excerpt = form.excerpt.as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| extract_first_paragraph(&content_html));
    let status = form.status.as_deref().unwrap_or("draft");
    let tags_json = parse_tags_json(form.tags.as_deref());
    let sources_json = parse_sources_json(form.sources.as_deref());
    
    let title_en = form.title_en.as_deref().filter(|s| !s.trim().is_empty());
    let slug_en = form.slug_en.as_deref().filter(|s| !s.trim().is_empty());
    let excerpt_en = form.excerpt_en.as_deref().filter(|s| !s.trim().is_empty());
    let content_markdown_en = form.content_markdown_en.as_deref().filter(|s| !s.trim().is_empty());
    let content_html_en = content_markdown_en.map(render_markdown);
    let meta_description_en = form.meta_description_en.as_deref().filter(|s| !s.trim().is_empty());
    
    let existing = sqlx::query("SELECT slug, slug_en, published_at FROM blog_posts WHERE id = ?")
        .bind(post_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Artikel tidak ditemukan".into()))?;
        
    let old_slug: String = existing.try_get("slug").unwrap_or_default();
    let old_slug_en: Option<String> = existing.try_get("slug_en").unwrap_or(None);
    let mut published_at: Option<chrono::DateTime<chrono::Utc>> = existing.try_get("published_at").unwrap_or(None);
    if status == "published" && published_at.is_none() {
        published_at = Some(Utc::now());
    }

    sqlx::query(
        r#"
        UPDATE blog_posts SET
            title = ?, slug = ?, content_markdown = ?, content_html = ?,
            title_en = ?, slug_en = ?, excerpt_en = ?, content_markdown_en = ?, content_html_en = ?, meta_description_en = ?,
            meta_description = ?, excerpt = ?, category = ?, tags = ?, sources = ?, source_url = ?, source_name = ?,
            featured_image = ?, status = ?, reading_time_minutes = ?, published_at = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#)
        .bind(&form.title)
        .bind(&slug)
        .bind(&form.content_markdown)
        .bind(&content_html)
        .bind(title_en)
        .bind(slug_en)
        .bind(excerpt_en)
        .bind(content_markdown_en)
        .bind(content_html_en.as_deref())
        .bind(meta_description_en)
        .bind(&form.meta_description)
        .bind(&excerpt)
        .bind(&form.category)
        .bind(tags_json.as_deref())
        .bind(sources_json.as_deref())
        .bind(&form.source_url)
        .bind(&form.source_name)
        .bind(&form.featured_image)
        .bind(status)
        .bind(reading_time)
        .bind(published_at)
        .bind(post_id)
        .execute(&state.db)
        .await?;
    
    let _ = state.cache.invalidate_post(&slug, slug_en).await;
    if !old_slug.is_empty() && old_slug != slug {
        let _ = state.cache.invalidate_post(&old_slug, old_slug_en.as_deref()).await;
    }
    let _ = state.cache.invalidate_all_listings().await;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/posts"))
        .finish())
}


pub async fn delete(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<u64>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    let post_id = path.into_inner();
    
    let post = sqlx::query("SELECT slug FROM blog_posts WHERE id = ?")
        .bind(post_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Artikel tidak ditemukan".into()))?;
        
    let slug: String = post.try_get("slug").unwrap_or_default();
        
    sqlx::query("DELETE FROM blog_posts WHERE id = ?")
        .bind(post_id)
        .execute(&state.db)
        .await?;
        
    let _ = state.cache.invalidate_post(&slug, None).await;
    let _ = state.cache.invalidate_all_listings().await;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/posts"))
        .finish())
}


pub async fn publish(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<u64>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    let post_id = path.into_inner();
    
    let post = sqlx::query("SELECT slug, published_at FROM blog_posts WHERE id = ?")
        .bind(post_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Artikel tidak ditemukan".into()))?;
        
    let slug: String = post.try_get("slug").unwrap_or_default();
    let existing_pub: Option<chrono::DateTime<chrono::Utc>> = post.try_get("published_at").unwrap_or(None);
    let published_at = existing_pub.unwrap_or_else(|| Utc::now());
    
    sqlx::query("UPDATE blog_posts SET status = 'published', published_at = ? WHERE id = ?")
        .bind(published_at)
        .bind(post_id)
        .execute(&state.db)
        .await?;
        
    let _ = state.cache.invalidate_post(&slug, None).await;
    let _ = state.cache.invalidate_all_listings().await;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/blog/admin/posts"))
        .finish())
}


pub async fn translate(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<u64>
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    let post_id = path.into_inner();
    
    let post = sqlx::query("SELECT title, content_markdown FROM blog_posts WHERE id = ?")
        .bind(post_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Artikel tidak ditemukan".into()))?;
        
    let title: String = post.try_get("title").unwrap_or_default();
    let content_markdown: String = post.try_get("content_markdown").unwrap_or_default();
        
    let slug_en = slugify(&title) + "-en";
    let content_en = content_markdown; // Placeholder, real translate should happen here
    let content_html_en = render_markdown(&content_en);
    
    sqlx::query(
        "UPDATE blog_posts SET slug_en = ?, content_markdown_en = ?, content_html_en = ? WHERE id = ?"
    )
    .bind(slug_en)
    .bind(content_en)
    .bind(content_html_en)
    .bind(post_id)
    .execute(&state.db)
    .await?;

    Ok(web::Json(serde_json::json!({ "success": true })))
}

