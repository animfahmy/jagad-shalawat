ALTER TABLE blog_admin_users ADD COLUMN email VARCHAR(255) UNIQUE;
ALTER TABLE blog_admin_users ADD COLUMN role ENUM('admin', 'contributor') DEFAULT 'admin';
ALTER TABLE blog_admin_users ADD COLUMN reset_token VARCHAR(255) UNIQUE;
ALTER TABLE blog_admin_users ADD COLUMN reset_token_expires_at TIMESTAMP NULL;
