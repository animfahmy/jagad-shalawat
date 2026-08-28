use actix_session::Session;
use actix_web::{web, HttpRequest, HttpResponse};

use crate::error::AppError;
use crate::services::seo;
use crate::AppState;

const POSTS_PER_PAGE: i64 = 12;

/// GET /blog — Blog listing page (Indonesian)
pub async fn index(
    state: web::Data<AppState>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.halaman.unwrap_or(1).max(1);
    let cache_key = format!("blog:listing:id:page:{}", page);

    // Try cache first
    if let Some(cached) = state.cache.get(&cache_key).await? {
        return Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("X-Cache", "HIT"))
            .insert_header(cache_control_public())
            .body(cached));
    }

    // Cache miss — fetch from DB
    let posts = crate::models::post::PostSummary::find_all_published(&state.db, page, POSTS_PER_PAGE).await?;
    let total = crate::models::post::BlogPost::count_published(&state.db).await?;
    let total_pages = (total as f64 / POSTS_PER_PAGE as f64).ceil() as i64;

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("current_page", &page);
    ctx.insert("total_pages", &total_pages);
    ctx.insert("lang", "id");
    ctx.insert("base_url", &state.config.base_url);
    ctx.insert("page_title", "Blog — Tagih Otomatis");
    ctx.insert("page_description", "Artikel seputar teknologi pembayaran digital, pembayaran syariah, dan kemudahan penagihan otomatis.");
    ctx.insert("canonical_url", &format!("{}/blog", state.config.base_url));
    ctx.insert("og_tags", &seo::generate_og_tags(
        "Blog — Tagih Otomatis",
        "Artikel seputar teknologi pembayaran digital, pembayaran syariah, dan kemudahan penagihan otomatis.",
        None,
        &format!("{}/blog", state.config.base_url),
        "id_ID",
    ));
    ctx.insert("hreflang_tags", "");

    let html = state.tera.render("blog/index.html", &ctx)?;

    // Cache the rendered page for 24 hours
    state.cache.set(&cache_key, &html, 86400).await?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header(("X-Cache", "MISS"))
        .insert_header(cache_control_public())
        .body(html))
}

/// GET /blog/kategori/{category} — Articles filtered by category (Indonesian)
pub async fn by_category(
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let category = path.into_inner();
    let page = query.halaman.unwrap_or(1).max(1);
    let cache_key = format!("blog:listing:id:cat:{}:page:{}", category, page);

    if let Some(cached) = state.cache.get(&cache_key).await? {
        return Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("X-Cache", "HIT"))
            .insert_header(cache_control_public())
            .body(cached));
    }

    let posts = crate::models::post::PostSummary::find_by_category(&state.db, &category, page, POSTS_PER_PAGE).await?;
    let total = crate::models::post::BlogPost::count_by_category(&state.db, &category).await?;
    let total_pages = (total as f64 / POSTS_PER_PAGE as f64).ceil() as i64;

    let category_title = category.replace('-', " ");
    let formatted_title = capitalize_words(&category_title);
    let page_title = format!("{} — Blog Tagih Otomatis", formatted_title);

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("current_page", &page);
    ctx.insert("total_pages", &total_pages);
    ctx.insert("category", &category);
    ctx.insert("category_name", &formatted_title);
    ctx.insert("category_title", &formatted_title);
    ctx.insert("lang", "id");
    ctx.insert("alternate_lang_url", &format!("/blog/en/category/{}", category));
    ctx.insert("base_url", &state.config.base_url);
    ctx.insert("page_title", &page_title);
    ctx.insert("page_description", &format!("Artikel kategori {} di Blog Tagih Otomatis", formatted_title));
    ctx.insert("canonical_url", &format!("{}/blog/kategori/{}", state.config.base_url, category));

    let html = state.tera.render("blog/category.html", &ctx)?;
    state.cache.set(&cache_key, &html, 86400).await?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header(("X-Cache", "MISS"))
        .insert_header(cache_control_public())
        .body(html))
}

/// GET /blog/{slug} — Single article page (Indonesian)
pub async fn show(
    state: web::Data<AppState>,
    path: web::Path<String>,
    _req: HttpRequest,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let slug = path.into_inner();

    // Don't intercept admin, auth, static, sitemap, feed routes
    if slug == "admin" || slug == "auth" || slug == "static"
        || slug == "sitemap.xml" || slug == "feed.xml"
        || slug == "en" || slug == "kategori"
    {
        return Err(AppError::NotFound("Halaman tidak ditemukan".into()));
    }

    let user_name = session.get::<String>("user_name").ok().flatten();
    let user_email = session.get::<String>("user_email").ok().flatten();
    let user_avatar = session.get::<String>("user_avatar").ok().flatten();
    let is_logged_in = user_name.is_some();

    let cache_key = format!("blog:page:id:{}", slug);

    // Only serve cached response for anonymous visitors (so logged-in commenters see their profile)
    if !is_logged_in {
        if let Some(cached) = state.cache.get(&cache_key).await? {
            let stats_key = format!("blog:stats:slug:{}", slug);
            let _ = state.cache.increment(&stats_key).await;

            return Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .insert_header(("X-Cache", "HIT"))
                .insert_header(cache_control_public())
                .body(cached));
        }
    }

    // Cache miss — fetch from DB
    let post = crate::models::post::BlogPost::find_published_by_slug(&state.db, &slug)
        .await?
        .ok_or_else(|| AppError::NotFound("Artikel tidak ditemukan".into()))?;

    // Fetch approved comments
    let comments = crate::models::comment::BlogComment::find_by_post_nested(&state.db, post.id).await?;

    // Fetch related posts
    let related = {
        let cat = &post.category;
        crate::models::post::PostSummary::find_related(&state.db, cat, post.id, 3).await?
    };

    let alternate_lang_url = if let Some(ref slug_en) = post.slug_en {
        if !slug_en.trim().is_empty() {
            format!("/blog/en/{}", slug_en)
        } else {
            "/blog/en".to_string()
        }
    } else {
        "/blog/en".to_string()
    };

    // Build template context
    let mut ctx = tera::Context::new();
    ctx.insert("post", &post);
    ctx.insert("comments", &comments);
    ctx.insert("related_posts", &related);
    ctx.insert("lang", "id");
    ctx.insert("alternate_lang_url", &alternate_lang_url);
    ctx.insert("base_url", &state.config.base_url);
    ctx.insert("page_title", &format!("{} — Blog Tagih Otomatis", post.title.clone()));
    ctx.insert("page_description", &post.meta_description.as_deref().unwrap_or(&post.excerpt.as_deref().unwrap_or("")));
    ctx.insert("canonical_url", &format!("{}/blog/{}", state.config.base_url, post.slug));
    ctx.insert("turnstile_site_key", &state.config.turnstile_site_key);

    // Commenter user info
    ctx.insert("is_logged_in", &is_logged_in);
    if let Some(ref name) = user_name {
        ctx.insert("user_name", name);
    }
    if let Some(ref email) = user_email {
        ctx.insert("user_email", email);
    }
    if let Some(ref avatar) = user_avatar {
        ctx.insert("user_avatar", avatar);
    }

    // SEO tags
    ctx.insert("json_ld", &seo::generate_json_ld_article(&post, &state.config.base_url, "id"));
    ctx.insert("og_tags", &seo::generate_og_tags(
        &post.title,
        post.meta_description.as_deref().unwrap_or(""),
        post.featured_image.as_deref(),
        &format!("{}/blog/{}", state.config.base_url, post.slug),
        "id_ID",
    ));
    ctx.insert("hreflang_tags", &seo::generate_hreflang_tags(
        &post.slug,
        post.slug_en.as_deref(),
        &state.config.base_url,
    ));
    ctx.insert("breadcrumb_ld", &seo::generate_json_ld_breadcrumb(&[
        ("Beranda".into(), state.config.base_url.clone()),
        ("Blog".into(), format!("{}/blog", state.config.base_url)),
        (post.title.clone(), format!("{}/blog/{}", state.config.base_url, post.slug)),
    ]));

    let html = state.tera.render("blog/post.html", &ctx)?;

    // Track view count
    let stats_key = format!("blog:stats:slug:{}", slug);
    let _ = state.cache.increment(&stats_key).await;

    if is_logged_in {
        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("X-Cache", "BYPASS"))
            .insert_header(("Cache-Control", "private, no-cache, no-store, must-revalidate"))
            .body(html))
    } else {
        // Cache for 24 hours only for anonymous visitors
        state.cache.set(&cache_key, &html, 86400).await?;

        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("X-Cache", "MISS"))
            .insert_header(cache_control_public())
            .body(html))
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct PaginationQuery {
    pub halaman: Option<i64>,
}

fn cache_control_public() -> (&'static str, &'static str) {
    ("Cache-Control", "public, max-age=0, must-revalidate, s-maxage=86400")
}

fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
