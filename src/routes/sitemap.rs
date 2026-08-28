use actix_web::{web, HttpResponse};
use chrono::Utc;

use crate::error::AppError;
use crate::AppState;

/// GET /blog/sitemap.xml — XML Sitemap for SEO.
pub async fn sitemap_xml(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let cache_key = "blog:sitemap";

    if let Some(cached) = state.cache.get(cache_key).await? {
        return Ok(HttpResponse::Ok()
            .content_type("application/xml; charset=utf-8")
            .insert_header(("X-Cache", "HIT"))
            .body(cached));
    }

    let slugs = crate::models::post::BlogPost::find_all_published_slugs(&state.db).await?;

    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml">
"#);

    // Blog listing pages
    xml.push_str(&format!(
        r#"  <url>
    <loc>{base}/blog</loc>
    <xhtml:link rel="alternate" hreflang="id" href="{base}/blog"/>
    <xhtml:link rel="alternate" hreflang="en" href="{base}/blog/ar"/>
    <changefreq>daily</changefreq>
    <priority>0.8</priority>
  </url>
  <url>
    <loc>{base}/blog/ar</loc>
    <xhtml:link rel="alternate" hreflang="id" href="{base}/blog"/>
    <xhtml:link rel="alternate" hreflang="en" href="{base}/blog/ar"/>
    <changefreq>daily</changefreq>
    <priority>0.8</priority>
  </url>
"#,
        base = state.config.base_url
    ));

    // Individual articles
    for (slug, slug_en, published_at) in &slugs {
        let lastmod = published_at
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

        // Indonesian version
        xml.push_str(&format!(
            r#"  <url>
    <loc>{base}/blog/{slug}</loc>
    <lastmod>{lastmod}</lastmod>
    <xhtml:link rel="alternate" hreflang="id" href="{base}/blog/{slug}"/>
"#,
            base = state.config.base_url,
            slug = slug,
            lastmod = lastmod,
        ));

        if let Some(ref sen) = slug_en {
            xml.push_str(&format!(
                r#"    <xhtml:link rel="alternate" hreflang="en" href="{base}/blog/ar/{slug_en}"/>
"#,
                base = state.config.base_url,
                slug_en = sen,
            ));
        }
        xml.push_str("    <changefreq>monthly</changefreq>\n    <priority>0.7</priority>\n  </url>\n");

        // English version (if available)
        if let Some(ref sen) = slug_en {
            xml.push_str(&format!(
                r#"  <url>
    <loc>{base}/blog/ar/{slug_en}</loc>
    <lastmod>{lastmod}</lastmod>
    <xhtml:link rel="alternate" hreflang="id" href="{base}/blog/{slug}"/>
    <xhtml:link rel="alternate" hreflang="en" href="{base}/blog/ar/{slug_en}"/>
    <changefreq>monthly</changefreq>
    <priority>0.7</priority>
  </url>
"#,
                base = state.config.base_url,
                slug = slug,
                slug_en = sen,
                lastmod = lastmod,
            ));
        }
    }

    xml.push_str("</urlset>\n");

    // Cache for 12 hours
    state.cache.set(cache_key, &xml, 43200).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/xml; charset=utf-8")
        .insert_header(("X-Cache", "MISS"))
        .body(xml))
}
