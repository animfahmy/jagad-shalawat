CREATE TABLE IF NOT EXISTS blog_blocked_words (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    word VARCHAR(255) NOT NULL,
    category ENUM('gambling','sara','pornography','spam','other') NOT NULL,
    is_regex BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE INDEX idx_word (word)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
