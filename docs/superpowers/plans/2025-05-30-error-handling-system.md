# 错误处理系统实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 在 Vessel 项目中添加 `thiserror` 依赖并建立统一的 `AppError` 处理规范。

**架构：** 定义一个中央 `AppError` 枚举，它利用 `thiserror` 进行错误转换，并手动实现 `Serialize` 以便 Tauri 前端处理。

**技术栈：** Rust, Tauri, thiserror, serde

---

### 任务 1：添加依赖

**文件：**
- 修改：`src-tauri/Cargo.toml`

- [x] **步骤 1：添加 thiserror 到 Cargo.toml**

在 `[dependencies]` 下添加 `thiserror = "2.0"`。

- [x] **步骤 2：运行 cargo check 验证**

运行：`cd src-tauri && cargo check`
预期：成功下载并检查依赖，无编译错误。

- [x] **步骤 3：Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore(deps): add thiserror dependency"
```

### 任务 2：创建错误模块

**文件：**
- 创建：`src-tauri/src/error.rs`
- 修改：`src-tauri/src/lib.rs`

- [x] **步骤 1：创建 error.rs 并定义 AppError**

```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Docker 错误: {0}")]
    Docker(#[from] bollard::errors::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tauri 错误: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("未知错误: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("{0}")]
    Custom(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

- [x] **步骤 2：在 lib.rs 中导出 error 模块**

在 `src-tauri/src/lib.rs` 顶部添加：
```rust
pub mod error;
```

- [x] **步骤 3：运行 cargo check 验证**

运行：`cd src-tauri && cargo check`
预期：PASS

- [x] **步骤 4：Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/lib.rs
git commit -m "refactor(backend): define AppError and export error module"
```
