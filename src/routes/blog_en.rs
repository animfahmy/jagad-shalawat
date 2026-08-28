use actix_web::{web, HttpRequest, HttpResponse};

use crate::error::AppError;
use crate::services::seo;
use crate::AppState;

const POSTS_PER_PAGE: i64 = 12;

/// GET /blog/en — Blog listing page (English)
pub async fn index(
    state: web::Data<AppState>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let cache_key = format!("blog:listing:en:page:{}", page);

    if let Some(cached) = state.cache.get(&cache_key).await? {
        return Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("X-Cache", "HIT"))
            .insert_header(cache_control_public())
            .body(cached));
    }

    let posts = crate::models::post::PostSummary::find_all_published(&state.db, page, POSTS_PER_PAGE).await?;
    let total = crate::models::post::BlogPost::count_published(&state.db).await?;
    let total_pages = (total as f64 / POSTS_PER_PAGE as f64).ceil() as i64;

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("current_page", &page);
    ctx.insert("total_pages", &total_pages);
    ctx.insert("lang", "en");
    ctx.insert("base_url", &state.config.base_url);
    ctx.insert("page_title", "Blog — Jagad Shalawat");
    ctx.insert("page_description", "Articles about digital payment technology, sharia-compliant payments, and automated billing solutions.");
    ctx.insert("canonical_url", &format!("{}/blog/en", state.config.base_url));
    ctx.insert("og_tags", &seo::generate_og_tags(
        "Blog — Jagad Shalawat",
        "Articles about digital payment technology, sharia-compliant payments, and automated billing solutions.",
        None,
        &format!("{}/blog/en", state.config.base_url),
        "en_US",
    ));
    ctx.insert("hreflang_tags", "");

    let html = state.tera.render("blog/index_en.html", &ctx)?;
    state.cache.set(&cache_key, &html, 86400).await?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header(("X-Cache", "MISS"))
        .insert_header(cache_control_public())
        .body(html))
}

/// GET /blog/en/category/{category} or /blog/en/kategori/{category} — Articles filtered by category (English)
pub async fn by_category(
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let category = path.into_inner();
    let page = query.page.unwrap_or(1).max(1);
    let cache_key = format!("blog:listing:en:cat:{}:page:{}", category, page);

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
    let page_title = format!("{} — Jagad Shalawat Blog", formatted_title);

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("current_page", &page);
    ctx.insert("total_pages", &total_pages);
    ctx.insert("category", &category);
    ctx.insert("category_name", &formatted_title);
    ctx.insert("category_title", &formatted_title);
    ctx.insert("lang", "en");
    ctx.insert("alternate_lang_url", &format!("/blog/kategori/{}", category));
    ctx.insert("base_url", &state.config.base_url);
    ctx.insert("page_title", &page_title);
    ctx.insert("page_description", &format!("Articles in category {} on Jagad Shalawat Blog", formatted_title));
    ctx.insert("canonical_url", &format!("{}/blog/en/category/{}", state.config.base_url, category));

    let html = state.tera.render("blog/category.html", &ctx)?;
    state.cache.set(&cache_key, &html, 86400).await?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header(("X-Cache", "MISS"))
        .insert_header(cache_control_public())
        .body(html))
}

use actix_session::Session;

/// GET /blog/en/{slug} — Single article page (English)
pub async fn show(
    state: web::Data<AppState>,
    path: web::Path<String>,
    _req: HttpRequest,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let slug_en = path.into_inner();

    if slug_en == "feed.xml" || slug_en == "category" || slug_en == "kategori" {
        return Err(AppError::NotFound("Not found".into()));
    }

    let user_name = session.get::<String>("user_name").ok().flatten();
    let user_email = session.get::<String>("user_email").ok().flatten();
    let user_avatar = session.get::<String>("user_avatar").ok().flatten();
    let is_logged_in = user_name.is_some();

    let cache_key = format!("blog:page:en:{}", slug_en);

    if !is_logged_in {
        if let Some(cached) = state.cache.get(&cache_key).await? {
            let stats_key = format!("blog:stats:slug_en:{}", slug_en);
            let _ = state.cache.increment(&stats_key).await;

            return Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .insert_header(("X-Cache", "HIT"))
                .insert_header(cache_control_public())
                .body(cached));
        }
    }

    // Fetch post by English slug
    let post = crate::models::post::BlogPost::find_published_by_slug_en(&state.db, &slug_en)
        .await?
        .ok_or_else(|| AppError::NotFound("Article not found".into()))?;

    let comments = crate::models::comment::BlogComment::find_by_post_nested(&state.db, post.id).await?;

    let related = {
        let cat = &post.category;
        crate::models::post::PostSummary::find_related(&state.db, cat, post.id, 3).await?
    };

    let title_en = post.title_en.as_deref().unwrap_or(&post.title);
    let desc_en = post.meta_description_en.as_deref()
        .or(post.excerpt_en.as_deref())
        .unwrap_or("");

    let alternate_lang_url = format!("/blog/{}", post.slug);

    let mut ctx = tera::Context::new();
    ctx.insert("post", &post);
    ctx.insert("comments", &comments);
    ctx.insert("related_posts", &related);
    ctx.insert("lang", "en");
    ctx.insert("alternate_lang_url", &alternate_lang_url);
    ctx.insert("base_url", &state.config.base_url);
    ctx.insert("page_title", &format!("{} — Jagad Shalawat Blog", title_en));
    ctx.insert("page_description", desc_en);
    ctx.insert("canonical_url", &format!("{}/blog/en/{}", state.config.base_url, slug_en));
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

    ctx.insert("json_ld", &seo::generate_json_ld_article(&post, &state.config.base_url, "en"));
    ctx.insert("og_tags", &seo::generate_og_tags(
        title_en,
        desc_en,
        post.featured_image.as_deref(),
        &format!("{}/blog/en/{}", state.config.base_url, slug_en),
        "en_US",
    ));
    ctx.insert("hreflang_tags", &seo::generate_hreflang_tags(
        &post.slug,
        post.slug_en.as_deref(),
        &state.config.base_url,
    ));
    ctx.insert("breadcrumb_ld", &seo::generate_json_ld_breadcrumb(&[
        ("Home".into(), state.config.base_url.clone()),
        ("Blog".into(), format!("{}/blog/en", state.config.base_url)),
        (title_en.to_string(), format!("{}/blog/en/{}", state.config.base_url, slug_en)),
    ]));

    let html = state.tera.render("blog/post_en.html", &ctx)?;

    let stats_key = format!("blog:stats:slug_en:{}", slug_en);
    let _ = state.cache.increment(&stats_key).await;

    if is_logged_in {
        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("X-Cache", "BYPASS"))
            .insert_header(("Cache-Control", "private, no-cache, no-store, must-revalidate"))
            .body(html))
    } else {
        state.cache.set(&cache_key, &html, 86400).await?;

        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("X-Cache", "MISS"))
            .insert_header(cache_control_public())
            .body(html))
    }
}

#[derive(serde::Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
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
