#![allow(dead_code)]
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BlogPost {
    pub id: u64,
    pub slug: String,
    pub slug_ar: Option<String>,
    pub title: String,
    pub title_ar: Option<String>,
    pub meta_description: Option<String>,
    pub meta_description_ar: Option<String>,
    pub excerpt: Option<String>,
    pub excerpt_ar: Option<String>,
    pub content_markdown: String,
    pub content_markdown_ar: Option<String>,
    pub content_html: String,
    pub content_html_ar: Option<String>,
    pub featured_image: Option<String>,
    pub author_name: String,
    pub category: String,
    pub tags: Option<serde_json::Value>,
    pub sources: Option<serde_json::Value>,
    pub source_url: Option<String>,
    pub source_name: Option<String>,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub reading_time_minutes: u8,
    pub view_count: u64,
    pub is_featured: i8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PostSummary {
    pub id: u64,
    pub slug: String,
    pub slug_ar: Option<String>,
    pub title: String,
    pub title_ar: Option<String>,
    pub excerpt: Option<String>,
    pub excerpt_ar: Option<String>,
    pub featured_image: Option<String>,
    pub category: String,
    pub published_at: Option<DateTime<Utc>>,
    pub reading_time_minutes: u8,
    pub view_count: u64,
    pub author_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub slug: String,
    pub content_markdown: String,
    pub meta_description: Option<String>,
    pub excerpt: Option<String>,
    pub category: String,
    pub tags: Option<serde_json::Value>,
    pub source_url: Option<String>,
    pub source_name: Option<String>,
    pub featured_image: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content_markdown: Option<String>,
    pub meta_description: Option<String>,
    pub excerpt: Option<String>,
    pub category: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub source_url: Option<String>,
    pub source_name: Option<String>,
    pub featured_image: Option<String>,
    pub status: Option<String>,
}

impl BlogPost {
    pub async fn find_published_by_slug(pool: &MySqlPool, slug: &str) -> Result<Option<BlogPost>, AppError> {
        let post = sqlx::query_as::<_, BlogPost>(
            "SELECT * FROM blog_posts WHERE (slug = ? OR slug_ar = ?) AND status = 'published'"
        )
        .bind(slug)
        .bind(slug)
        .fetch_optional(pool)
        .await?;
        Ok(post)
    }

    pub async fn find_all_published(pool: &MySqlPool, page: u32, per_page: u32) -> Result<Vec<PostSummary>, AppError> {
        let offset = (page.saturating_sub(1)) * per_page;
        let posts = sqlx::query_as::<_, PostSummary>(
            "SELECT id, slug, slug_ar, title, title_ar, excerpt, excerpt_ar, featured_image, category, published_at, reading_time_minutes, view_count, author_name FROM blog_posts WHERE status = 'published' ORDER BY published_at DESC LIMIT ? OFFSET ?"
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        Ok(posts)
    }

    pub async fn find_by_category(pool: &MySqlPool, category: &str, page: u32, per_page: u32) -> Result<Vec<PostSummary>, AppError> {
        let offset = (page.saturating_sub(1)) * per_page;
        let posts = sqlx::query_as::<_, PostSummary>(
            "SELECT id, slug, slug_ar, title, title_ar, excerpt, excerpt_ar, featured_image, category, published_at, reading_time_minutes, view_count, author_name FROM blog_posts WHERE status = 'published' AND (LOWER(category) = LOWER(?) OR LOWER(REPLACE(category, ' ', '-')) = LOWER(?)) ORDER BY published_at DESC LIMIT ? OFFSET ?"
        )
        .bind(category)
        .bind(category)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        Ok(posts)
    }

    pub async fn count_published(pool: &MySqlPool) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM blog_posts WHERE status = 'published'"
        )
        .fetch_one(pool)
        .await?;
        Ok(count.0)
    }

    pub async fn find_all_for_admin(pool: &MySqlPool, status_filter: Option<&str>, page: u32, per_page: u32) -> Result<Vec<PostSummary>, AppError> {
        let offset = (page.saturating_sub(1)) * per_page;
        
        let posts = if let Some(status) = status_filter {
            sqlx::query_as::<_, PostSummary>(
                "SELECT id, slug, slug_ar, title, title_ar, excerpt, excerpt_ar, featured_image, category, published_at, reading_time_minutes, view_count, author_name FROM blog_posts WHERE status = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(status)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, PostSummary>(
                "SELECT id, slug, slug_ar, title, title_ar, excerpt, excerpt_ar, featured_image, category, published_at, reading_time_minutes, view_count, author_name FROM blog_posts ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };
        
        Ok(posts)
    }

    pub async fn find_by_id(pool: &MySqlPool, id: u64) -> Result<Option<BlogPost>, AppError> {
        let post = sqlx::query_as::<_, BlogPost>(
            "SELECT * FROM blog_posts WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(post)
    }

    pub async fn create(pool: &MySqlPool, data: &CreatePost) -> Result<u64, AppError> {
        let result = sqlx::query(
            "INSERT INTO blog_posts (title, slug, content_markdown, meta_description, excerpt, category, tags, source_url, source_name, featured_image, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&data.title)
        .bind(&data.slug)
        .bind(&data.content_markdown)
        .bind(&data.meta_description)
        .bind(&data.excerpt)
        .bind(&data.category)
        .bind(&data.tags)
        .bind(&data.source_url)
        .bind(&data.source_name)
        .bind(&data.featured_image)
        .bind(&data.status)
        .execute(pool)
        .await?;
        Ok(result.last_insert_id())
    }

    pub async fn update(pool: &MySqlPool, id: u64, data: &UpdatePost) -> Result<(), AppError> {
        let current = Self::find_by_id(pool, id).await?.ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;
        
        sqlx::query(
            "UPDATE blog_posts SET title = ?, slug = ?, content_markdown = ?, meta_description = ?, excerpt = ?, category = ?, tags = ?, source_url = ?, source_name = ?, featured_image = ?, status = ?, updated_at = NOW() WHERE id = ?"
        )
        .bind(data.title.as_deref().unwrap_or(&current.title))
        .bind(data.slug.as_deref().unwrap_or(&current.slug))
        .bind(data.content_markdown.as_deref().unwrap_or(&current.content_markdown))
        .bind(data.meta_description.as_deref().or(current.meta_description.as_deref()))
        .bind(data.excerpt.as_deref().or(current.excerpt.as_deref()))
        .bind(data.category.as_deref().unwrap_or(&current.category))
        .bind(data.tags.as_ref().or(current.tags.as_ref()))
        .bind(data.source_url.as_deref().or(current.source_url.as_deref()))
        .bind(data.source_name.as_deref().or(current.source_name.as_deref()))
        .bind(data.featured_image.as_deref().or(current.featured_image.as_deref()))
        .bind(data.status.as_deref().unwrap_or(&current.status))
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(pool: &MySqlPool, id: u64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM blog_posts WHERE id = ?").bind(id).execute(pool).await?;
        Ok(())
    }

    pub async fn publish(pool: &MySqlPool, id: u64) -> Result<(), AppError> {
        sqlx::query("UPDATE blog_posts SET status = 'published', published_at = NOW() WHERE id = ?").bind(id).execute(pool).await?;
        Ok(())
    }

    pub async fn increment_view_count(pool: &MySqlPool, id: u64) -> Result<(), AppError> {
        sqlx::query("UPDATE blog_posts SET view_count = view_count + 1 WHERE id = ?").bind(id).execute(pool).await?;
        Ok(())
    }

    pub async fn find_all_published_slugs(pool: &MySqlPool) -> Result<Vec<(String, Option<String>, Option<DateTime<Utc>>)>, AppError> {
        #[derive(FromRow)]
        struct SlugData {
            slug: String,
            slug_ar: Option<String>,
            published_at: Option<DateTime<Utc>>,
        }
        
        let records = sqlx::query_as::<_, SlugData>(
            "SELECT slug, slug_ar, published_at FROM blog_posts WHERE status = 'published'"
        )
        .fetch_all(pool)
        .await?;
        
        Ok(records.into_iter().map(|r| (r.slug, r.slug_ar, r.published_at)).collect())
    }

    /// Find a published post by its English slug.
    pub async fn find_published_by_slug_ar(pool: &MySqlPool, slug_ar: &str) -> Result<Option<BlogPost>, AppError> {
        let post = sqlx::query_as::<_, BlogPost>(
            "SELECT * FROM blog_posts WHERE slug_ar = ? AND status = 'published'"
        )
        .bind(slug_ar)
        .fetch_optional(pool)
        .await?;
        Ok(post)
    }

    /// Count published posts in a specific category.
    pub async fn count_by_category(pool: &MySqlPool, category: &str) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM blog_posts WHERE status = 'published' AND (LOWER(category) = LOWER(?) OR LOWER(REPLACE(category, ' ', '-')) = LOWER(?))"
        )
        .bind(category)
        .bind(category)
        .fetch_one(pool)
        .await?;
        Ok(count.0)
    }
}

/// Methods on PostSummary for querying lighter post data.
impl PostSummary {
    pub async fn find_all_published(pool: &MySqlPool, page: i64, per_page: i64) -> Result<Vec<PostSummary>, AppError> {
        let offset = (page.max(1) - 1) * per_page;
        let posts = sqlx::query_as::<_, PostSummary>(
            "SELECT id, slug, slug_ar, title, title_ar, excerpt, excerpt_ar, featured_image, category, published_at, reading_time_minutes, view_count, author_name FROM blog_posts WHERE status = 'published' ORDER BY published_at DESC LIMIT ? OFFSET ?"
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        Ok(posts)
    }

    pub async fn find_by_category(pool: &MySqlPool, category: &str, page: i64, per_page: i64) -> Result<Vec<PostSummary>, AppError> {
        let offset = (page.max(1) - 1) * per_page;
        let posts = sqlx::query_as::<_, PostSummary>(
            "SELECT id, slug, slug_ar, title, title_ar, excerpt, excerpt_ar, featured_image, category, published_at, reading_time_minutes, view_count, author_name FROM blog_posts WHERE status = 'published' AND (LOWER(category) = LOWER(?) OR LOWER(REPLACE(category, ' ', '-')) = LOWER(?)) ORDER BY published_at DESC LIMIT ? OFFSET ?"
        )
        .bind(category)
        .bind(category)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        Ok(posts)
    }

    pub async fn find_related(pool: &MySqlPool, category: &str, exclude_id: u64, limit: i32) -> Result<Vec<PostSummary>, AppError> {
        let posts = sqlx::query_as::<_, PostSummary>(
            "SELECT id, slug, slug_ar, title, title_ar, excerpt, excerpt_ar, featured_image, category, published_at, reading_time_minutes, view_count, author_name FROM blog_posts WHERE status = 'published' AND category = ? AND id != ? ORDER BY published_at DESC LIMIT ?"
        )
        .bind(category)
        .bind(exclude_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(posts)
    }
}
