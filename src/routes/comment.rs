use actix_session::Session;
use actix_web::{web, HttpRequest, HttpResponse};
use log::{info, warn};
use md5::{Digest, Md5};

use crate::error::AppError;
use crate::models::comment::{BlogComment, CreateComment, CommentDisplay};
use crate::services::content_filter::FilterResult;
use crate::AppState;

/// Form data for submitting a comment.
#[derive(serde::Deserialize)]
pub struct CommentForm {
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub content: String,
    pub parent_id: Option<u64>,
    /// Honeypot field — must be empty for legitimate submissions.
    pub website: Option<String>,
    /// Cloudflare Turnstile token.
    #[serde(alias = "cf-turnstile-response")]
    pub cf_turnstile_response: Option<String>,
}

/// POST /blog/{slug}/komentar — Submit a comment on a blog post.
pub async fn submit(
    state: web::Data<AppState>,
    path: web::Path<String>,
    form: web::Form<CommentForm>,
    req: HttpRequest,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let slug = path.into_inner();

    // ── Layer 1: Honeypot check ──────────────────────────────────
    if let Some(ref website) = form.website {
        if !website.is_empty() {
            info!("🍯 Honeypot triggered for slug={}", slug);
            // Silently accept (don't reveal to bot that it was caught)
            return Ok(redirect_to_post(&slug));
        }
    }

    // ── Layer 2: Cloudflare Turnstile ────────────────────────────
    if !state.config.turnstile_secret_key.is_empty() && state.config.turnstile_secret_key != "dummy" {
        if let Some(ref token) = form.cf_turnstile_response {
            if !verify_turnstile(token, &state.config.turnstile_secret_key).await? {
                warn!("🛡️ Turnstile verification failed for slug={}", slug);
                return Err(AppError::BadRequest("Verifikasi CAPTCHA gagal. Silakan coba lagi.".into()));
            }
        } else {
            return Err(AppError::BadRequest("CAPTCHA diperlukan.".into()));
        }
    }

    // ── Layer 3: Rate limit check ────────────────────────────────
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    let mut rate_limiter = crate::services::rate_limiter::RateLimiter::new(
        state.cache.connection().await?,
    );
    if !rate_limiter.check_rate_limit(&ip, 3, 600).await? {
        warn!("⏱️ Rate limit exceeded for IP={}", ip);
        return Err(AppError::RateLimited);
    }

    // ── Find the post ────────────────────────────────────────────
    let post = crate::models::post::BlogPost::find_published_by_slug(&state.db, &slug)
        .await?
        .ok_or_else(|| AppError::NotFound("Artikel tidak ditemukan".into()))?;

    // ── Determine author info ────────────────────────────────────
    // Check if user is logged in via OAuth
    let (author_name, author_email, avatar_url, oauth_provider, oauth_id) =
        if let (Ok(Some(name)), Ok(Some(provider))) = (
            session.get::<String>("user_name"),
            session.get::<String>("oauth_provider"),
        ) {
            let email = session.get::<String>("user_email").ok().flatten();
            let avatar = session.get::<String>("user_avatar").ok().flatten();
            let oid = session.get::<String>("oauth_id").ok().flatten();
            (name, email, avatar, Some(provider), oid)
        } else {
            // Anonymous commenter
            let name = form.author_name.clone()
                .filter(|n| !n.trim().is_empty())
                .ok_or_else(|| AppError::BadRequest("Nama wajib diisi.".into()))?;
            let email = form.author_email.clone().filter(|e| !e.trim().is_empty());

            // Generate Gravatar URL from email
            let avatar = email.as_ref().map(|e| gravatar_url(e));

            (name, email, avatar, None, None)
        };

    // ── Layer 4 & 5: Content filter ──────────────────────────────
    let content = form.content.trim().to_string();
    if content.is_empty() {
        return Err(AppError::BadRequest("Komentar tidak boleh kosong.".into()));
    }
    if content.len() > 5000 {
        return Err(AppError::BadRequest("Komentar terlalu panjang (maks 5000 karakter).".into()));
    }

    let filter = state.content_filter.read().await;
    let filter_result = filter.check(&content).await;

    // Also check the author name
    let name_filter = filter.check(&author_name).await;

    let status = match (&filter_result, &name_filter) {
        (FilterResult::Blocked { category, matched_word }, _) => {
            warn!("🚫 Comment blocked ({}): matched '{}' in content", category, matched_word);
            return Err(AppError::BadRequest("Komentar mengandung kata yang diblokir.".into()));
        }
        (_, FilterResult::Blocked { category, matched_word }) => {
            warn!("🚫 Comment blocked ({}): matched '{}' in name", category, matched_word);
            return Err(AppError::BadRequest("Nama mengandung kata yang diblokir.".into()));
        }
        (FilterResult::Suspicious { reason }, _) | (_, FilterResult::Suspicious { reason }) => {
            info!("⏳ Comment flagged for review: {}", reason);
            "pending".to_string()
        }
        (FilterResult::Clean, FilterResult::Clean) => {
            "approved".to_string()
        }
    };

    // ── Sanitize HTML in comment content ─────────────────────────
    let sanitized_content = crate::services::markdown::sanitize_html(&content);

    // ── Layer 6: Save to database ────────────────────────────────
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let create_data = CreateComment {
        post_id: post.id,
        parent_id: form.parent_id,
        author_name,
        author_email,
        author_avatar_url: avatar_url,
        oauth_provider,
        oauth_id,
        content: sanitized_content,
        ip_address: Some(ip),
        user_agent: Some(user_agent),
    };

    BlogComment::create(&state.db, &create_data, &status).await?;

    // ── Invalidate comment cache ─────────────────────────────────
    if status == "approved" {
        state.cache.invalidate_comments(post.id).await?;
        // Also invalidate the post page cache so comments appear
        state.cache.invalidate_post(&post.slug, post.slug_en.as_deref()).await?;
    }

    info!(
        "💬 Comment submitted for '{}' — status: {}",
        slug, status
    );

    Ok(redirect_to_post(&slug))
}

/// Verify Cloudflare Turnstile CAPTCHA token.
async fn verify_turnstile(token: &str, secret: &str) -> Result<bool, AppError> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&[("secret", secret), ("response", token)])
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Turnstile verification failed: {}", e)))?;

    #[derive(serde::Deserialize)]
    struct TurnstileResponse {
        success: bool,
    }

    let result: TurnstileResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Turnstile response parse error: {}", e)))?;

    Ok(result.success)
}

/// Generate Gravatar URL from email address.
fn gravatar_url(email: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(email.trim().to_lowercase().as_bytes());
    let hash = hex::encode(hasher.finalize());
    format!("https://www.gravatar.com/avatar/{}?d=mp&s=80", hash)
}

/// Redirect back to the blog post after comment submission.
fn redirect_to_post(slug: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header(("Location", format!("/blog/{}#komentar", slug)))
        .finish()
}
