use crate::error::AppError;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::future::Future;

/// Redis cache service for pre-rendered HTML pages and other cached data.
///
/// Uses `ConnectionManager` which handles reconnection automatically and
/// can be safely shared across threads (implements Clone and is Send + Sync).
#[derive(Clone)]
pub struct CacheService {
    conn: ConnectionManager,
}

impl CacheService {
    /// Create a new cache service connected to Redis.
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    /// Get a cached value by key.
    pub async fn get(&self, key: &str) -> Result<Option<String>, AppError> {
        let mut conn = self.conn.clone();
        let value: Option<String> = conn.get(key).await?;
        Ok(value)
    }

    /// Set a cached value with TTL in seconds.
    pub async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(key, value, ttl_secs).await?;
        Ok(())
    }

    /// Delete a single cached key.
    pub async fn delete(&self, key: &str) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let _: () = conn.del(key).await?;
        Ok(())
    }

    /// Delete all keys matching a glob pattern using SCAN.
    pub async fn delete_pattern(&self, pattern: &str) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await?;
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }
        Ok(())
    }

    /// Increment a counter key. Returns the new value.
    pub async fn increment(&self, key: &str) -> Result<i64, AppError> {
        let mut conn = self.conn.clone();
        let value: i64 = conn.incr(key, 1).await?;
        Ok(value)
    }

    /// Get a cloned connection manager for direct use (e.g., rate limiter).
    pub async fn connection(&self) -> Result<ConnectionManager, AppError> {
        Ok(self.conn.clone())
    }

    /// Cache-aside pattern: get from cache, or compute and store.
    pub async fn get_or_set<F, Fut>(
        &self,
        key: &str,
        ttl_secs: u64,
        f: F,
    ) -> Result<String, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<String, AppError>>,
    {
        if let Some(cached) = self.get(key).await? {
            return Ok(cached);
        }

        let fresh = f().await?;
        self.set(key, &fresh, ttl_secs).await?;
        Ok(fresh)
    }

    /// Invalidate cache for a specific blog post (both languages).
    pub async fn invalidate_post(
        &self,
        slug: &str,
        slug_en: Option<&str>,
    ) -> Result<(), AppError> {
        self.delete(&format!("blog:page:id:{}", slug)).await?;

        if let Some(en) = slug_en {
            self.delete(&format!("blog:page:en:{}", en)).await?;
        }

        // Also invalidate listings and feeds since they may include this post
        self.invalidate_all_listings().await?;
        self.delete("blog:sitemap").await?;
        self.delete("blog:feed:id").await?;
        self.delete("blog:feed:en").await?;
        Ok(())
    }

    /// Invalidate all listing page caches.
    pub async fn invalidate_all_listings(&self) -> Result<(), AppError> {
        self.delete_pattern("blog:listing:*").await?;
        Ok(())
    }

    /// Invalidate comment cache for a specific post.
    pub async fn invalidate_comments(&self, post_id: u64) -> Result<(), AppError> {
        self.delete(&format!("blog:comments:{}", post_id)).await?;
        Ok(())
    }
}
