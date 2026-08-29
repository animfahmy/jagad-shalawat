#![allow(dead_code)]
use crate::error::AppError;
use sqlx::MySqlPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct BlockedWord {
    pub word: String,
    pub category: String,
    pub is_regex: bool,
}

#[derive(Debug)]
pub enum FilterResult {
    Clean,
    Blocked { category: String, matched_word: String },
    Suspicious { reason: String },
}

#[derive(Clone)]
pub struct ContentFilter {
    blocked_words: Arc<RwLock<Vec<BlockedWord>>>,
}

impl ContentFilter {
    pub async fn new(pool: &MySqlPool) -> Result<Self, AppError> {
        let filter = Self {
            blocked_words: Arc::new(RwLock::new(Vec::new())),
        };
        filter.reload(pool).await?;
        Ok(filter)
    }

    pub async fn reload(&self, pool: &MySqlPool) -> Result<(), AppError> {
        #[derive(sqlx::FromRow)]
        struct WordRow {
            word: String,
            category: String,
            is_regex: i8,
        }

        let rows: Vec<WordRow> = sqlx::query_as::<_, WordRow>(
            "SELECT word, category, is_regex FROM blog_blocked_words"
        )
        .fetch_all(pool)
        .await?;

        let mut words = Vec::with_capacity(rows.len());
        for row in rows {
            words.push(BlockedWord {
                word: row.word.to_lowercase(),
                category: row.category,
                is_regex: row.is_regex != 0,
            });
        }

        let mut write_guard = self.blocked_words.write().await;
        *write_guard = words;
        
        Ok(())
    }

    pub async fn check(&self, text: &str) -> FilterResult {
        let text_lower = text.to_lowercase();
        let normalized = Self::normalize_evasion(&text_lower);

        let url_count = text_lower.matches("http://").count() + text_lower.matches("https://").count();
        if url_count > 2 {
            return FilterResult::Suspicious {
                reason: "Too many URLs".to_string(),
            };
        }

        let words_guard = self.blocked_words.read().await;

        for blocked in words_guard.iter() {
            if normalized.contains(&blocked.word) || text_lower.contains(&blocked.word) {
                return FilterResult::Blocked {
                    category: blocked.category.clone(),
                    matched_word: blocked.word.clone(),
                };
            }
        }

        FilterResult::Clean
    }

    fn normalize_evasion(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        
        for c in text.chars() {
            match c {
                '.' | '*' | '-' | '_' | ' ' => continue,
                '0' => result.push('o'),
                '1' => result.push('i'),
                '3' => result.push('e'),
                '4' => result.push('a'),
                '5' => result.push('s'),
                '7' => result.push('t'),
                '@' => result.push('a'),
                '$' => result.push('s'),
                _ => result.push(c),
            }
        }
        
        result
    }
}

