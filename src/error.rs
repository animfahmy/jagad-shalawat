use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Database(sqlx::Error),
    Redis(redis::RedisError),
    Template(tera::Error),
    Internal(String),
    Unauthorized,
    BadRequest(String),
    RateLimited,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::Redis(err) => write!(f, "Redis error: {}", err),
            AppError::Template(err) => write!(f, "Template error: {}", err),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized access"),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::RateLimited => write!(f, "Too Many Requests"),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::NotFound(ref message) => HttpResponse::NotFound().json(serde_json::json!({ "error": message })),
            AppError::Unauthorized => HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" })),
            AppError::BadRequest(ref message) => HttpResponse::BadRequest().json(serde_json::json!({ "error": message })),
            AppError::RateLimited => HttpResponse::TooManyRequests().json(serde_json::json!({ "error": "Too Many Requests" })),
            _ => {
                log::error!("Internal server error: {:?}", self);
                HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Internal Server Error" }))
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Record not found".to_string()),
            _ => AppError::Database(err),
        }
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::Redis(err)
    }
}

impl From<tera::Error> for AppError {
    fn from(err: tera::Error) -> Self {
        AppError::Template(err)
    }
}
