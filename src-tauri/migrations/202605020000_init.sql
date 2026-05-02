-- src-tauri/migrations/202605020000_init.sql
CREATE TABLE IF NOT EXISTS connections (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    driver TEXT NOT NULL, -- local, ssh, wsl
    host TEXT NOT NULL,
    auth_config TEXT -- encrypted json
);

CREATE TABLE IF NOT EXISTS compose_projects (
    id TEXT PRIMARY KEY NOT NULL,
    connection_id TEXT NOT NULL,
    name TEXT NOT NULL,
    working_dir TEXT NOT NULL,
    config_path TEXT NOT NULL,
    FOREIGN KEY(connection_id) REFERENCES connections(id)
);
