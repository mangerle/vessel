-- src-tauri/migrations/202605020000_init.sql
CREATE TABLE IF NOT EXISTS compose_projects (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    working_dir TEXT NOT NULL,
    config_path TEXT NOT NULL
);
