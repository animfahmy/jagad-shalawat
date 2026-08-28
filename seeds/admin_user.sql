-- Default admin password: 'admin123' (CHANGE THIS IMMEDIATELY)
-- Argon2id hash generated with: echo -n 'admin123' | argon2 salt -id
INSERT INTO blog_admin_users (username, password_hash, display_name)
VALUES ('admin', '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$placeholder', 'Administrator')
ON DUPLICATE KEY UPDATE username = username;
