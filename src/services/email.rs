use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::config::Config;
use crate::error::AppError;

pub async fn send_password_reset_email(to_email: &str, reset_link: &str, config: &Config) -> Result<(), AppError> {
    let smtp_username = config.smtp_username.as_deref().ok_or_else(|| AppError::Internal("SMTP_USERNAME belum diatur di .env".into()))?;
    let smtp_password = config.smtp_password.as_deref().ok_or_else(|| AppError::Internal("SMTP_PASSWORD belum diatur di .env".into()))?;
    let smtp_server = config.smtp_server.as_deref().ok_or_else(|| AppError::Internal("SMTP_SERVER belum diatur di .env".into()))?;
    
    let html_content = format!(
        r#"
        <p>Halo,</p>
        <p>Anda menerima email ini karena ada permintaan reset password untuk akun Anda.</p>
        <p>Silakan klik link di bawah ini untuk mengatur ulang password Anda:</p>
        <p><a href="{}">Reset Password</a></p>
        <p>Jika Anda tidak meminta reset password, abaikan email ini.</p>
        <p>Terima kasih,<br>Tim Jagad Shalawat</p>
        "#,
        reset_link
    );

    let email = Message::builder()
        .from("Jagad Shalawat <no-reply@jagadshalawat.org>".parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject("Reset Password Anda - Jagad Shalawat")
        .header(ContentType::TEXT_HTML)
        .body(html_content)
        .map_err(|e| AppError::Internal(format!("Failed to build email: {}", e)))?;

    let creds = Credentials::new(smtp_username.to_owned(), smtp_password.to_owned());

    // We use a sync transport here, wrapped in spawn_blocking for async context
    let mailer = SmtpTransport::relay(smtp_server)
        .map_err(|e| AppError::Internal(format!("Failed to connect to SMTP relay: {}", e)))?
        .credentials(creds)
        .build();

    tokio::task::spawn_blocking(move || {
        mailer.send(&email)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task execution failed: {}", e)))?
    .map_err(|e| AppError::Internal(format!("Failed to send email: {}", e)))?;

    Ok(())
}
