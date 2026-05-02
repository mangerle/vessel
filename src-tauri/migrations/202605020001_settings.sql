-- src-tauri/migrations/202605020001_settings.sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- 初始化活动连接为空
INSERT OR IGNORE INTO settings (key, value) VALUES ('active_connection_id', '');
