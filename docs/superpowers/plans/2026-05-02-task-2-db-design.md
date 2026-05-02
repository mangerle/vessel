# 数据库设计与集成实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 初始化 SQLite 数据库，实现单例连接池，并完成初始迁移。

**Architecture:** 使用 `sqlx` 异步驱动 SQLite。通过 `OnceLock` 提供全局单例连接池。使用 `sqlx::migrate!` 处理数据库架构演进。

**Tech Stack:** Rust, sqlx (SQLite, runtime-tokio), Tauri 2.0

---

### Task 1: 编写数据库初始化脚本

**Files:**
- Create: `src-tauri/migrations/202605020000_init.sql`

- [ ] **Step 1: 创建迁移目录并编写 SQL 脚本**

```sql
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
```

- [ ] **Step 2: 提交**

```bash
git add src-tauri/migrations/202605020000_init.sql
git commit -m "db: 增加初始迁移脚本"
```

---

### Task 2: 实现 Rust 数据库模块

**Files:**
- Create: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs` (注册模块)

- [ ] **Step 1: 编写 `db.rs` 实现连接池单例**

```rust
// src-tauri/src/db.rs
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::sync::OnceLock;

/// 全局连接池单例
static POOL: OnceLock<Pool<Sqlite>> = OnceLock::new();

/// 初始化数据库连接池并运行迁移
pub async fn init_db(database_url: &str) -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect(database_url).await?;
    // 运行 migrations 目录下的迁移脚本
    sqlx::migrate!("./migrations").run(&pool).await?;
    POOL.set(pool).ok();
    Ok(())
}

/// 获取全局连接池引用
pub fn get_pool() -> &'static Pool<Sqlite> {
    POOL.get().expect("数据库尚未初始化")
}
```

- [ ] **Step 2: 在 `lib.rs` 或 `main.rs` 中暴露模块**

```rust
// src-tauri/src/lib.rs 或 main.rs (取决于项目结构)
pub mod db;
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/db.rs src-tauri/src/lib.rs
git commit -m "db: 实现数据库连接池单例"
```

---

### Task 3: 在应用程序启动时初始化数据库

**Files:**
- Modify: `src-tauri/src/lib.rs` 或 `src-tauri/src/main.rs`

- [ ] **Step 1: 在 `run` 函数或 `setup` 钩子中调用 `init_db`**

需要确定数据库文件存放位置，通常在用户应用数据目录下。

```rust
// src-tauri/src/lib.rs
use tauri::Manager;
use crate::db::init_db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let app_dir = app_handle.path().app_data_dir().expect("无法获取应用数据目录");
                if !app_dir.exists() {
                    std::fs::create_dir_all(&app_dir).expect("无法创建应用数据目录");
                }
                let db_path = app_dir.join("docker-manager.db");
                let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
                init_db(&db_url).await.expect("数据库初始化失败");
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: 提交**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: 应用程序启动时自动初始化数据库"
```

---

### Task 4: 编写测试验证数据库连接

**Files:**
- Create: `src-tauri/src/db_test.rs` (或者在 `db.rs` 中编写集成测试)

- [ ] **Step 1: 编写集成测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tauri::async_runtime;

    #[test]
    fn test_db_init_and_pool() {
        async_runtime::block_on(async {
            // 使用内存数据库进行测试
            let db_url = "sqlite::memory:";
            init_db(db_url).await.expect("测试数据库初始化失败");
            
            let pool = get_pool();
            let row: (i32,) = sqlx::query_as("SELECT 1")
                .fetch_one(pool)
                .await
                .expect("查询失败");
            
            assert_eq!(row.0, 1);
        });
    }
}
```

- [ ] **Step 2: 运行测试并验证**

Run: `cargo test -p docker-manager-lib` (或者对应的包名)

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/db.rs
git commit -m "test: 增加数据库初始化和连接测试"
```
