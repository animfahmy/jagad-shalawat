pub mod blog;
pub mod blog_ar;
pub mod comment;
pub mod sitemap;
pub mod feed;
pub mod auth;
pub mod admin;

use actix_web::web;

/// Configure all blog routes.
///
/// All routes are under the `/blog` prefix since this engine sits behind
/// OpenLiteSpeed which reverse-proxies `/blog/*` to this Rust server.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // ── Public blog routes (Indonesian) ──────────────────────────
        .route("/blog", web::get().to(blog::index))
        .route("/blog/", web::get().to(blog::index))
        .route("/blog/kategori/{category}", web::get().to(blog::by_category))
        .route("/blog/category/{category}", web::get().to(blog::by_category))

        // ── Public blog routes (Arabic) ─────────────────────────────
        .route("/blog/ar", web::get().to(blog_ar::index))
        .route("/blog/ar/", web::get().to(blog_ar::index))
        .route("/blog/ar/category/{category}", web::get().to(blog_ar::by_category))
        .route("/blog/ar/kategori/{category}", web::get().to(blog_ar::by_category))

        // ── Comments ─────────────────────────────────────────────────
        .route("/blog/{slug}/komentar", web::post().to(comment::submit))
        .route("/blog/{slug}/comment", web::post().to(comment::submit))
        .route("/blog/ar/{slug}/comment", web::post().to(comment::submit))
        .route("/blog/ar/{slug}/komentar", web::post().to(comment::submit))

        // ── SEO feeds ────────────────────────────────────────────────
        .route("/blog/sitemap.xml", web::get().to(sitemap::sitemap_xml))
        .route("/blog/feed.xml", web::get().to(feed::rss_id))
        .route("/blog/ar/feed.xml", web::get().to(feed::rss_ar))

        // ── OAuth callbacks ──────────────────────────────────────────
        .route("/blog/auth/google", web::get().to(auth::google_login))
        .route("/blog/auth/google/callback", web::get().to(auth::google_callback))
        .route("/blog/auth/github", web::get().to(auth::github_login))
        .route("/blog/auth/github/callback", web::get().to(auth::github_callback))
        .route("/blog/auth/logout", web::get().to(auth::logout))

        // ── Admin panel ──────────────────────────────────────────────
        .route("/blog/admin", web::get().to(admin::dashboard::index))
        .route("/blog/admin/", web::get().to(admin::dashboard::index))
        .route("/blog/admin/dashboard", web::get().to(admin::dashboard::index))
        .route("/blog/admin/login", web::get().to(admin::auth::login_page))
        .route("/blog/admin/login", web::post().to(admin::auth::login_submit))
        .route("/blog/admin/logout", web::get().to(admin::auth::logout))

        // Admin: Posts
        .route("/blog/admin/posts", web::get().to(admin::posts::list))
        .route("/blog/admin/posts/new", web::get().to(admin::posts::new_form))
        .route("/blog/admin/posts", web::post().to(admin::posts::create))
        .route("/blog/admin/posts/{id}/edit", web::get().to(admin::posts::edit_form))
        .route("/blog/admin/posts/edit/{id}", web::get().to(admin::posts::edit_form))
        .route("/blog/admin/posts/{id}", web::put().to(admin::posts::update))
        .route("/blog/admin/posts/{id}", web::post().to(admin::posts::update))
        .route("/blog/admin/posts/edit/{id}", web::post().to(admin::posts::update))
        .route("/blog/admin/posts/{id}", web::delete().to(admin::posts::delete))
        .route("/blog/admin/posts/{id}/delete", web::post().to(admin::posts::delete))
        .route("/blog/admin/posts/delete/{id}", web::post().to(admin::posts::delete))
        .route("/blog/admin/posts/{id}/publish", web::post().to(admin::posts::publish))
        .route("/blog/admin/posts/publish/{id}", web::post().to(admin::posts::publish))
        .route("/blog/admin/posts/{id}/translate", web::post().to(admin::posts::translate))

        // Admin: Comments
        .route("/blog/admin/comments", web::get().to(admin::comments::list))
        .route("/blog/admin/comments/{id}/approve", web::post().to(admin::comments::approve))
        .route("/blog/admin/comments/approve/{id}", web::post().to(admin::comments::approve))
        .route("/blog/admin/comments/{id}/reject", web::post().to(admin::comments::reject))
        .route("/blog/admin/comments/reject/{id}", web::post().to(admin::comments::reject))
        .route("/blog/admin/comments/{id}/delete", web::post().to(admin::comments::reject))
        .route("/blog/admin/comments/delete/{id}", web::post().to(admin::comments::reject))

        // Admin: Blocked words
        .route("/blog/admin/blocked-words", web::get().to(admin::blocked_words::list))
        .route("/blog/admin/blocked-words", web::post().to(admin::blocked_words::create))
        .route("/blog/admin/blocked-words/add", web::post().to(admin::blocked_words::create))
        .route("/blog/admin/blocked-words/{id}", web::delete().to(admin::blocked_words::delete))
        .route("/blog/admin/blocked-words/{id}/delete", web::post().to(admin::blocked_words::delete))
        .route("/blog/admin/blocked-words/delete/{id}", web::post().to(admin::blocked_words::delete))

        // Admin: Media upload
        .route("/blog/admin/media/upload", web::post().to(admin::media::upload))

        // Admin: Cache management
        .route("/blog/admin/cache/clear", web::post().to(admin::dashboard::clear_cache))
        // ── Catch-all dynamic routes (must be last) ──────────────────
        .route("/blog/ar/{slug}", web::get().to(blog_ar::show))
        .route("/blog/{slug}", web::get().to(blog::show));
}
