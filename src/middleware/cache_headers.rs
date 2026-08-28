/// Generate Cache-Control header value for public content.
///
/// - `max-age=14400` — browser caches for 4 hours
/// - `s-maxage=86400` — CDN (Cloudflare) caches for 24 hours
/// - `stale-while-revalidate=3600` — serve stale while refreshing
pub fn public_cache_header() -> (&'static str, &'static str) {
    (
        "Cache-Control",
        "public, max-age=14400, s-maxage=86400, stale-while-revalidate=3600",
    )
}

/// Cache-Control for static assets (CSS, JS, images).
/// Immutable + long max-age since filenames are versioned/hashed.
pub fn static_cache_header() -> (&'static str, &'static str) {
    (
        "Cache-Control",
        "public, max-age=31536000, immutable",
    )
}

/// Cache-Control for admin pages — no caching.
pub fn no_cache_header() -> (&'static str, &'static str) {
    (
        "Cache-Control",
        "no-store, no-cache, must-revalidate, private",
    )
}
