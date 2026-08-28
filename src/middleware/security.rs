/// Security headers to add to all responses.
///
/// These complement Cloudflare's security headers with application-specific policies.
pub fn security_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("X-Content-Type-Options", "nosniff"),
        ("X-Frame-Options", "SAMEORIGIN"),
        ("Referrer-Policy", "strict-origin-when-cross-origin"),
        (
            "Permissions-Policy",
            "camera=(), microphone=(), geolocation=(), interest-cohort=()",
        ),
        (
            "Content-Security-Policy",
            "default-src 'self'; \
             script-src 'self' https://challenges.cloudflare.com; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' https://www.gravatar.com https://storage.googleapis.com https://*.googleusercontent.com https://avatars.githubusercontent.com data:; \
             font-src 'self'; \
             connect-src 'self' https://challenges.cloudflare.com; \
             frame-src https://challenges.cloudflare.com; \
             base-uri 'self'; \
             form-action 'self' https://accounts.google.com https://github.com",
        ),
    ]
}
