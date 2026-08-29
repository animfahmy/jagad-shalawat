#![allow(dead_code)]
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BlogComment {
    pub id: u64,
    pub post_id: u64,
    pub parent_id: Option<u64>,
    pub author_name: String,
    pub author_email: Option<String>,
    pub author_avatar_url: Option<String>,
    pub content: String,
    pub oauth_provider: Option<String>,
    pub oauth_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: String,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateComment {
    pub post_id: u64,
    pub parent_id: Option<u64>,
    pub author_name: String,
    pub author_email: Option<String>,
    pub content: String,
    pub oauth_provider: Option<String>,
    pub oauth_id: Option<String>,
    pub author_avatar_url: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentDisplay {
    pub id: u64,
    pub author_name: String,
    pub author_avatar_url: Option<String>,
    pub oauth_provider: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub parent_id: Option<u64>,
    pub replies: Vec<CommentDisplay>,
}

impl BlogComment {
    pub async fn find_approved_by_post(pool: &MySqlPool, post_id: u64) -> Result<Vec<BlogComment>, AppError> {
        let comments = sqlx::query_as::<_, BlogComment>(
            "SELECT * FROM blog_comments WHERE post_id = ? AND status = 'approved' ORDER BY created_at ASC"
        )
        .bind(post_id)
        .fetch_all(pool)
        .await?;
        Ok(comments)
    }

    pub async fn create(pool: &MySqlPool, data: &CreateComment, status: &str) -> Result<u64, AppError> {
        let result = sqlx::query(
            "INSERT INTO blog_comments (post_id, parent_id, author_name, author_email, content, oauth_provider, oauth_id, author_avatar_url, ip_address, user_agent, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(data.post_id)
        .bind(data.parent_id)
        .bind(&data.author_name)
        .bind(&data.author_email)
        .bind(&data.content)
        .bind(&data.oauth_provider)
        .bind(&data.oauth_id)
        .bind(&data.author_avatar_url)
        .bind(&data.ip_address)
        .bind(&data.user_agent)
        .bind(status)
        .execute(pool)
        .await?;
        Ok(result.last_insert_id())
    }

    pub async fn count_pending(pool: &MySqlPool) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM blog_comments WHERE status = 'pending'"
        )
        .fetch_one(pool)
        .await?;
        Ok(count.0)
    }

    pub async fn find_pending(pool: &MySqlPool, page: u32, per_page: u32) -> Result<Vec<BlogComment>, AppError> {
        let offset = (page.saturating_sub(1)) * per_page;
        let comments = sqlx::query_as::<_, BlogComment>(
            "SELECT * FROM blog_comments WHERE status = 'pending' ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        Ok(comments)
    }

    pub async fn approve(pool: &MySqlPool, id: u64) -> Result<(), AppError> {
        sqlx::query("UPDATE blog_comments SET status = 'approved' WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn reject(pool: &MySqlPool, id: u64, reason: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE blog_comments SET status = 'rejected', rejection_reason = ? WHERE id = ?")
            .bind(reason)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn find_by_post_nested(pool: &MySqlPool, post_id: u64) -> Result<Vec<CommentDisplay>, AppError> {
        let comments = Self::find_approved_by_post(pool, post_id).await?;
        
        let display_comments: Vec<CommentDisplay> = comments.into_iter().map(|c| CommentDisplay {
            id: c.id,
            author_name: c.author_name,
            author_avatar_url: c.author_avatar_url,
            oauth_provider: c.oauth_provider,
            content: c.content,
            created_at: c.created_at,
            parent_id: c.parent_id,
            replies: Vec::new(),
        }).collect();

        // Reconstruct hierarchy
        let mut root_comments = Vec::new();
        let mut child_comments = Vec::new();

        for comment in display_comments.into_iter() {
            if comment.parent_id.is_none() {
                root_comments.push(comment);
            } else {
                child_comments.push(comment);
            }
        }

        // Extremely basic nesting (just 1 level deep usually)
        for child in child_comments {
            if let Some(parent_id) = child.parent_id {
                if let Some(parent) = root_comments.iter_mut().find(|p| p.id == parent_id) {
                    parent.replies.push(child);
                }
            }
        }

        Ok(root_comments)
    }
}

