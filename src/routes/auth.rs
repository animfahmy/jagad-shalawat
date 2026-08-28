use actix_session::Session;
use actix_web::{web, HttpRequest, HttpResponse};
use log::info;

use crate::error::AppError;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct AuthLoginQuery {
    pub return_to: Option<String>,
}

fn sanitize_return_url(raw: Option<&str>, referer: Option<&str>) -> String {
    if let Some(r) = raw {
        if r.starts_with("/blog") {
            return r.to_string();
        }
    }
    if let Some(ref_str) = referer {
        if let Ok(u) = url::Url::parse(ref_str) {
            let path = u.path();
            if path.starts_with("/blog") {
                return path.to_string();
            }
        } else if ref_str.starts_with("/blog") {
            return ref_str.to_string();
        }
    }
    "/blog".to_string()
}

/// GET /blog/auth/google — Redirect to Google OAuth consent screen.
pub async fn google_login(
    req: HttpRequest,
    state: web::Data<AppState>,
    session: Session,
    query: web::Query<AuthLoginQuery>,
) -> Result<HttpResponse, AppError> {
    let client_id = state.config.google_client_id.as_ref()
        .ok_or_else(|| AppError::Internal("Google OAuth not configured".into()))?;
    let redirect_url = state.config.google_redirect_url.as_ref()
        .ok_or_else(|| AppError::Internal("Google OAuth redirect URL not configured".into()))?;

    let referer = req.headers().get("Referer").and_then(|h| h.to_str().ok());
    let return_url = sanitize_return_url(query.return_to.as_deref(), referer);

    session.insert("return_url", &return_url).ok();

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&access_type=online&state={}",
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_url),
        urlencoding::encode(&return_url),
    );

    Ok(HttpResponse::Found()
        .insert_header(("Location", auth_url))
        .finish())
}

/// GET /blog/auth/google/callback — Handle Google OAuth callback.
pub async fn google_callback(
    state: web::Data<AppState>,
    query: web::Query<OAuthCallback>,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let code = &query.code;
    let client_id = state.config.google_client_id.as_ref()
        .ok_or_else(|| AppError::Internal("Google OAuth not configured".into()))?;
    let client_secret = state.config.google_client_secret.as_ref()
        .ok_or_else(|| AppError::Internal("Google OAuth secret not configured".into()))?;
    let redirect_url = state.config.google_redirect_url.as_ref()
        .ok_or_else(|| AppError::Internal("Google OAuth redirect URL not configured".into()))?;

    // Exchange code for tokens
    let client = reqwest::Client::new();
    let token_resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("redirect_uri", redirect_url.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Token exchange failed: {}", e)))?;

    let token_data: GoogleTokenResponse = token_resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Token parse failed: {}", e)))?;

    // Get user info
    let user_resp = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(&token_data.access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("User info request failed: {}", e)))?;

    let user_info: GoogleUserInfo = user_resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("User info parse failed: {}", e)))?;

    // Store user info in session
    session.insert("user_name", &user_info.name).ok();
    session.insert("user_email", &user_info.email).ok();
    session.insert("user_avatar", &user_info.picture).ok();
    session.insert("oauth_provider", "google").ok();
    session.insert("oauth_id", &user_info.id).ok();

    info!("🔑 Google OAuth login: {} ({})", user_info.name, user_info.email);

    // Determine return URL
    let return_url = query.state.clone()
        .filter(|s| !s.trim().is_empty() && s.starts_with("/blog"))
        .or_else(|| {
            session.get::<String>("return_url").ok().flatten()
                .filter(|s| s.starts_with("/blog"))
        })
        .unwrap_or_else(|| "/blog".to_string());

    Ok(HttpResponse::Found()
        .insert_header(("Location", return_url))
        .finish())
}

/// GET /blog/auth/github — Redirect to GitHub OAuth.
pub async fn github_login(
    req: HttpRequest,
    state: web::Data<AppState>,
    session: Session,
    query: web::Query<AuthLoginQuery>,
) -> Result<HttpResponse, AppError> {
    let client_id = state.config.github_client_id.as_ref()
        .ok_or_else(|| AppError::Internal("GitHub OAuth not configured".into()))?;
    let redirect_url = state.config.github_redirect_url.as_ref()
        .ok_or_else(|| AppError::Internal("GitHub OAuth redirect URL not configured".into()))?;

    let referer = req.headers().get("Referer").and_then(|h| h.to_str().ok());
    let return_url = sanitize_return_url(query.return_to.as_deref(), referer);

    session.insert("return_url", &return_url).ok();

    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user%20user:email&state={}",
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_url),
        urlencoding::encode(&return_url),
    );

    Ok(HttpResponse::Found()
        .insert_header(("Location", auth_url))
        .finish())
}

/// GET /blog/auth/github/callback — Handle GitHub OAuth callback.
pub async fn github_callback(
    state: web::Data<AppState>,
    query: web::Query<OAuthCallback>,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let code = &query.code;
    let client_id = state.config.github_client_id.as_ref()
        .ok_or_else(|| AppError::Internal("GitHub OAuth not configured".into()))?;
    let client_secret = state.config.github_client_secret.as_ref()
        .ok_or_else(|| AppError::Internal("GitHub OAuth secret not configured".into()))?;

    let client = reqwest::Client::new();

    // Exchange code for access token
    let token_resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GitHub token exchange failed: {}", e)))?;

    let token_data: GitHubTokenResponse = token_resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("GitHub token parse failed: {}", e)))?;

    // Get user info
    let user_resp = client
        .get("https://api.github.com/user")
        .header("User-Agent", "TagihOtomatisBlog")
        .bearer_auth(&token_data.access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GitHub user info failed: {}", e)))?;

    let user_info: GitHubUserInfo = user_resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("GitHub user parse failed: {}", e)))?;

    // Get primary email
    let email = if let Some(email) = user_info.email {
        email
    } else {
        // Fetch from emails endpoint
        let emails_resp = client
            .get("https://api.github.com/user/emails")
            .header("User-Agent", "TagihOtomatisBlog")
            .bearer_auth(&token_data.access_token)
            .send()
            .await
            .ok();

        if let Some(resp) = emails_resp {
            let emails: Vec<GitHubEmail> = resp.json().await.unwrap_or_default();
            emails.into_iter()
                .find(|e| e.primary)
                .map(|e| e.email)
                .unwrap_or_default()
        } else {
            String::new()
        }
    };

    let display_name = user_info.name.unwrap_or(user_info.login.clone());

    session.insert("user_name", &display_name).ok();
    session.insert("user_email", &email).ok();
    session.insert("user_avatar", &user_info.avatar_url).ok();
    session.insert("oauth_provider", "github").ok();
    session.insert("oauth_id", &user_info.id.to_string()).ok();

    info!("🔑 GitHub OAuth login: {} ({})", display_name, email);

    let return_url = query.state.clone()
        .filter(|s| !s.trim().is_empty() && s.starts_with("/blog"))
        .or_else(|| {
            session.get::<String>("return_url").ok().flatten()
                .filter(|s| s.starts_with("/blog"))
        })
        .unwrap_or_else(|| "/blog".to_string());

    Ok(HttpResponse::Found()
        .insert_header(("Location", return_url))
        .finish())
}

/// GET /blog/auth/logout — Clear session and redirect to return_to or blog.
pub async fn logout(
    req: HttpRequest,
    session: Session,
    query: web::Query<AuthLoginQuery>,
) -> HttpResponse {
    let referer = req.headers().get("Referer").and_then(|h| h.to_str().ok());
    let return_url = sanitize_return_url(query.return_to.as_deref(), referer);

    session.purge();
    HttpResponse::Found()
        .insert_header(("Location", return_url))
        .finish()
}

// ── Data structures for OAuth responses ──────────────────────────────

#[derive(serde::Deserialize)]
pub struct OAuthCallback {
    pub code: String,
    pub state: Option<String>,
}

#[derive(serde::Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
}

#[derive(serde::Deserialize)]
struct GoogleUserInfo {
    id: String,
    name: String,
    email: String,
    picture: Option<String>,
}

#[derive(serde::Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
}

#[derive(serde::Deserialize)]
struct GitHubUserInfo {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: String,
}

#[derive(serde::Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
}
