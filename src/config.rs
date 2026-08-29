use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_address: String,
    pub base_url: String,
    pub database_url: String,
    pub redis_url: String,
    pub session_secret: String,
    pub turnstile_site_key: String,
    pub turnstile_secret_key: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_url: Option<String>,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub github_redirect_url: Option<String>,
    pub gcs_bucket: Option<String>,
    pub gcs_credentials_path: Option<String>,
    pub translation_api_key: Option<String>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_server: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        Config {
            bind_address: env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
            base_url: env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set"),
            session_secret: env::var("SESSION_SECRET").expect("SESSION_SECRET must be set"),
            turnstile_site_key: env::var("TURNSTILE_SITE_KEY").expect("TURNSTILE_SITE_KEY must be set"),
            turnstile_secret_key: env::var("TURNSTILE_SECRET_KEY").expect("TURNSTILE_SECRET_KEY must be set"),
            google_client_id: env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").ok(),
            google_redirect_url: env::var("GOOGLE_REDIRECT_URL").ok(),
            github_client_id: env::var("GITHUB_CLIENT_ID").ok(),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET").ok(),
            github_redirect_url: env::var("GITHUB_REDIRECT_URL").ok(),
            gcs_bucket: env::var("GCS_BUCKET").ok(),
            gcs_credentials_path: env::var("GCS_CREDENTIALS_PATH").ok(),
            translation_api_key: env::var("TRANSLATION_API_KEY").ok(),
            smtp_username: env::var("SMTP_USERNAME").ok(),
            smtp_password: env::var("SMTP_PASSWORD").ok(),
            smtp_server: env::var("SMTP_SERVER").ok(),
        }
    }
}
