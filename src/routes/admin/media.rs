// Requires the following dependencies in Cargo.toml:
// actix-multipart = "0.7"
// futures-util = "0.3"
// urlencoding = "2"

use actix_web::{web, Responder};
use actix_session::Session;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use crate::error::AppError;
use crate::routes::admin::require_auth;
use image::imageops::FilterType;
use uuid::Uuid;
use std::fs;


pub async fn upload(
    session: Session,
    mut payload: Multipart
) -> Result<impl Responder, AppError> {
    require_auth(&session)?;
    
    let mut file_url = None;
    
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::BadRequest(e.to_string()))?;
        
        let content_type = field.content_type().map(|c| c.to_string()).unwrap_or_default();
        if !content_type.starts_with("image/") {
            continue;
        }
        
        let mut bytes = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| AppError::BadRequest(e.to_string()))?;
            bytes.extend_from_slice(&data);
            if bytes.len() > 10 * 1024 * 1024 { // 10MB
                return Err(AppError::BadRequest("File too large".into()));
            }
        }
        
        let img = image::load_from_memory(&bytes)
            .map_err(|e| AppError::BadRequest(format!("Invalid image: {}", e)))?;
            
        let (w, h) = (img.width(), img.height());
        let max_dim = 1024u32;
        let resized = if w > max_dim || h > max_dim {
            img.resize(max_dim, max_dim, FilterType::Lanczos3)
        } else {
            img
        };
        
        let uuid = Uuid::new_v4().to_string();
        let filename = format!("{}.webp", uuid);
        let path = format!("src/static/uploads/{}", filename);
        
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        resized.save_with_format(&path, image::ImageFormat::WebP)
            .map_err(|e| AppError::Internal(format!("Failed to save image: {}", e)))?;
            
        file_url = Some(format!("/blog/static/uploads/{}", filename));
        break;
    }
    
    match file_url {
        Some(url) => Ok(web::Json(serde_json::json!({ "url": url }))),
        None => Err(AppError::BadRequest("No image provided".into()))
    }
}
