use actix_web::{web, App, HttpServer, middleware};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_session::config::PersistentSession;
use actix_files::Files;
use tera::Tera;
use sqlx::mysql::MySqlPoolOptions;
use std::sync::Arc;
use tokio::sync::RwLock;

mod config;
mod error;
mod models;
mod routes;
mod services;

use config::Config;
use services::content_filter::ContentFilter;
use services::cache::CacheService;

pub struct AppState {
    pub db: sqlx::MySqlPool,
    pub cache: CacheService,
    pub tera: Tera,
    pub config: Config,
    pub content_filter: Arc<RwLock<ContentFilter>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let config = Config::from_env();
    
    // Setup database
    let db = MySqlPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to MySQL");
        
    // Setup Redis & Cache
    let cache_service = CacheService::new(&config.redis_url)
        .await
        .expect("Failed to connect to Redis");

    // Setup Tera templates
    let mut tera = Tera::new("src/templates/**/*.html").expect("Failed to parse templates");
    tera.autoescape_on(vec![".html", ".sql"]);

    // Initialize Content Filter
    let content_filter = ContentFilter::new(&db).await.expect("Failed to init content filter");
    let filter_arc = Arc::new(RwLock::new(content_filter));

    let state = web::Data::new(AppState {
        db: db.clone(),
        cache: cache_service,
        tera,
        config: config.clone(),
        content_filter: filter_arc,
    });
    
    let session_key = actix_web::cookie::Key::derive_from(config.session_secret.as_bytes());
    
    // Extract bind address, defaulting to 127.0.0.1:8080 if not set or just using the config
    let bind_addr = config.bind_address.clone();

    log::info!("🚀 Tagih Otomatis Blog running on {}", bind_addr);

    HttpServer::new(move || {
        let session_mw = SessionMiddleware::builder(CookieSessionStore::default(), session_key.clone())
            .cookie_name(String::from("tagih_blog_session"))
            .cookie_path(String::from("/"))
            .cookie_same_site(actix_web::cookie::SameSite::Lax)
            .cookie_secure(false)
            .cookie_http_only(true)
            .session_lifecycle(PersistentSession::default().session_ttl(actix_web::cookie::time::Duration::days(30)))
            .build();

        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(session_mw)
            
            // Serve static files
            .service(
                Files::new("/blog/static", "src/static")
            )
            
            // Configure routes
            .configure(routes::configure)
    })
    .bind(bind_addr)?
    .run()
    .await
}
