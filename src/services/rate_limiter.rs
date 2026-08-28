use crate::error::AppError;
use redis::{aio::ConnectionManager, AsyncCommands};

#[derive(Clone)]
pub struct RateLimiter {
    conn_manager: ConnectionManager,
}

impl RateLimiter {
    pub fn new(conn: ConnectionManager) -> Self {
        Self {
            conn_manager: conn,
        }
    }

    pub async fn check_rate_limit(&mut self, ip: &str, max_requests: u32, window_secs: u64) -> Result<bool, AppError> {
        let key = format!("blog:rate:{}", ip);
        
        let mut pipe = redis::pipe();
        pipe.atomic()
            .incr(&key, 1)
            .expire(&key, window_secs as i64);
            
        let result: (u32, bool) = pipe.query_async(&mut self.conn_manager).await?;
        let count = result.0;
        
        Ok(count <= max_requests)
    }

    pub async fn get_remaining(&mut self, ip: &str, max_requests: u32) -> Result<u32, AppError> {
        let key = format!("blog:rate:{}", ip);
        let count: Option<u32> = self.conn_manager.get(&key).await?;
        
        let current = count.unwrap_or(0);
        if current >= max_requests {
            Ok(0)
        } else {
            Ok(max_requests - current)
        }
    }
}
