# Docker Manager 实施计划 - 第一阶段 (MVP)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 完成基础架构搭建，实现本地 Docker 容器列表显示及基础的数据库持久化配置。

**Architecture:** 采用 Tauri 提供的后端能力，通过 `bollard` 与 Docker 通信，并使用 `sqlx` (SQLite) 存储配置。前端使用 Vue 3 + Naive UI 提供界面。

**Tech Stack:** Rust, Tauri v2, Vue 3, Naive UI, SQLx (SQLite), Bollard.

---

### Task 1: 项目初始化与环境搭建 [DONE]

**Files:**
- Create: `package.json`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`

- [x] **Step 1: 使用 Tauri CLI 初始化项目**

Run: `npm create tauri-app@latest -- --template vue-ts --manager npm --yes .`
Expected: 项目骨架生成成功。

- [x] **Step 2: 配置后端依赖**

在 `src-tauri/Cargo.toml` 中添加必要依赖：
```toml
[dependencies]
tauri = { version = "2.0", features = [] }
tauri-plugin-shell = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
bollard = "0.15"
sqlx = { version = "0.7", features = ["runtime-tokio", "tls-native-tls", "sqlite"] }
uuid = { version = "1", features = ["v4", "serde"] }
```

- [x] **Step 3: 运行基础项目验证**

Run: `npm run tauri dev`
Expected: 看到 Tauri 默认欢迎界面。

- [x] **Step 4: 提交**

```bash
git add .
git commit -m "chore: initialize tauri project with dependencies"
```

---

### Task 2: 数据库设计与集成 [DONE]

**Files:**
- Create: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/main.rs`
- Create: `migrations/202605020000_init.sql`

- [x] **Step 1: 编写数据库初始化脚本**

```sql
-- migrations/202605020000_init.sql
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

- [x] **Step 2: 实现 Rust 数据库连接单例**

```rust
// src-tauri/src/db.rs
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::sync::OnceLock;

static POOL: OnceLock<Pool<Sqlite>> = OnceLock::new();

pub async fn init_db(database_url: &str) -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    POOL.set(pool).ok();
    Ok(())
}

pub fn get_pool() -> &'static Pool<Sqlite> {
    POOL.get().expect("Database not initialized")
}
```

- [x] **Step 3: 在 main.rs 中初始化数据库**

- [x] **Step 4: 编写测试验证数据库连接**

- [x] **Step 5: 提交**

---

### Task 3: 本地 Docker API 桥接实现

**Files:**
- Create: `src-tauri/src/docker.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 实现获取本地容器列表的 Command**

```rust
// src-tauri/src/docker.rs
use bollard::Docker;
use bollard::container::ListContainersOptions;
use serde::Serialize;

#[derive(Serialize)]
pub struct ContainerInfo {
    id: String,
    name: String,
    state: String,
    image: String,
}

#[tauri::command]
pub async fn list_local_containers() -> Result<Vec<ContainerInfo>, String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| e.to_string())?;
    let containers = docker.list_containers(Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    })).await.map_err(|e| e.to_string())?;
    
    Ok(containers.into_iter().map(|c| ContainerInfo {
        id: c.id.unwrap_or_default(),
        name: c.names.unwrap_or_default().join(", "),
        state: c.state.unwrap_or_default(),
        image: c.image.unwrap_or_default(),
    }).collect())
}
```

- [ ] **Step 2: 注册 Command**

- [ ] **Step 3: 运行并验证 (确保本地 Docker 已启动)**

- [ ] **Step 4: 提交**

---

### Task 4: 前端基础 UI 搭建 (容器列表)

**Files:**
- Create: `src/views/ContainerList.vue`
- Modify: `src/App.vue`

- [ ] **Step 1: 安装 Naive UI**

Run: `npm install naive-ui vfonts`

- [ ] **Step 2: 编写容器列表页面**

使用 `invoke('list_local_containers')` 获取数据并渲染表格。

- [ ] **Step 3: 验证 UI 显示**

- [ ] **Step 4: 提交**
