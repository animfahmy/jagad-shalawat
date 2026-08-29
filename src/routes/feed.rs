use actix_web::{web, HttpResponse};
use chrono::Utc;

use crate::error::AppError;
use crate::AppState;

/// GET /blog/feed.xml — RSS 2.0 feed (Indonesian)
pub async fn rss_id(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    generate_rss(&state, "id").await
}

/// GET /blog/ar/feed.xml — RSS 2.0 feed (English)
pub async fn rss_ar(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    generate_rss(&state, "en").await
}

async fn generate_rss(state: &AppState, lang: &str) -> Result<HttpResponse, AppError> {
    let cache_key = format!("blog:feed:{}", lang);

    if let Some(cached) = state.cache.get(&cache_key).await? {
        return Ok(HttpResponse::Ok()
            .content_type("application/rss+xml; charset=utf-8")
            .insert_header(("X-Cache", "HIT"))
            .body(cached));
    }

    let posts = crate::models::post::PostSummary::find_all_published(&state.db, 1, 20).await?;

    let (title, description, link) = match lang {
        "en" => (
            "Jagad Shalawat Blog",
            "Articles about digital payment technology, sharia-compliant payments, and automated billing.",
            format!("{}/blog/ar", state.config.base_url),
        ),
        _ => (
            "Blog Jagad Shalawat",
            "Artikel seputar teknologi pembayaran digital, pembayaran syariah, dan penagihan otomatis.",
            format!("{}/blog", state.config.base_url),
        ),
    };

    let now = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();

    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
  <title>{title}</title>
  <description>{description}</description>
  <link>{link}</link>
  <atom:link href="{link}/feed.xml" rel="self" type="application/rss+xml"/>
  <language>{lang_code}</language>
  <lastBuildDate>{now}</lastBuildDate>
  <generator>Jagad Shalawat Blog Engine (Rust)</generator>
"#,
        title = escape_xml(title),
        description = escape_xml(description),
        link = link,
        lang_code = if lang == "en" { "en-us" } else { "id" },
        now = now,
    );

    for post in &posts {
        let (item_title, item_desc, item_link) = match lang {
            "en" => {
                let t = post.title_ar.as_deref().unwrap_or(&post.title);
                let d = post.excerpt_ar.as_deref().unwrap_or(post.excerpt.as_deref().unwrap_or(""));
                let slug = post.slug_ar.as_deref().unwrap_or(&post.slug);
                let l = format!("{}/blog/ar/{}", state.config.base_url, slug);
                (t.to_string(), d.to_string(), l)
            }
            _ => {
                let d = post.excerpt.as_deref().unwrap_or("");
                let l = format!("{}/blog/{}", state.config.base_url, post.slug);
                (post.title.clone(), d.to_string(), l)
            }
        };

        let pub_date = post
            .published_at
            .map(|d| d.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .unwrap_or_default();

        xml.push_str(&format!(
            r#"  <item>
    <title>{title}</title>
    <description>{description}</description>
    <link>{link}</link>
    <guid isPermaLink="true">{link}</guid>
    <pubDate>{pub_date}</pubDate>
    <author>{author}</author>
  </item>
"#,
            title = escape_xml(&item_title),
            description = escape_xml(&item_desc),
            link = item_link,
            pub_date = pub_date,
            author = escape_xml(&post.author_name),
        ));
    }

    xml.push_str("</channel>\n</rss>\n");

    state.cache.set(&cache_key, &xml, 43200).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .insert_header(("X-Cache", "MISS"))
        .body(xml))
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
