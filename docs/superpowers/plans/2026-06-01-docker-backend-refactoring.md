# Docker 后端重构计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标**：将所有 Docker 后端模块迁移至 `AppResult` 并实现基于 Trait 的模型转换。

**架构**：利用 Rust 的 `From` Trait 统一 Bollard 模型与应用 DTO 的转换，通过 `AppResult` (Result<T, AppError>) 简化错误传播。

**技术栈**：Rust, Tauri, Bollard.

---

### 任务 1：模型与 DTO 增强

**文件**：
- 修改：`src-tauri/src/docker/models.rs`
- 修改：`src-tauri/src/docker/fs.rs`

- [ ] **步骤 1：在 `models.rs` 中迁移 `ContainerFileInfo` 并添加 `From` Trait**
    - 从 `fs.rs` 移动 `ContainerFileInfo` 到 `models.rs`。
    - 为 `PruneImagesResult` 添加 `From<bollard::models::ImagePruneResponse>`。

- [ ] **步骤 2：Commit**
    - `git add src-tauri/src/docker/models.rs src-tauri/src/docker/fs.rs`
    - `git commit -m "refactor(backend): migrate ContainerFileInfo and add From traits to models"`

### 任务 2：重构 Image 模块

**文件**：
- 修改：`src-tauri/src/docker/image.rs`

- [ ] **步骤 1：重构 `image.rs` 中的所有命令**
    - 修改函数签名，返回 `AppResult<T>`。
    - 移除 `.map_err`，改用 `?`。
    - 使用 `into()` 或 `from()` 进行模型转换。

- [ ] **步骤 2：Commit**
    - `git add src-tauri/src/docker/image.rs`
    - `git commit -m "refactor(backend): adapt image.rs to AppResult and Trait conversion"`

### 任务 3：重构 Volume 模块

**文件**：
- 修改：`src-tauri/src/docker/volume.rs`

- [ ] **步骤 1：重构 `volume.rs` 中的所有命令**
    - 修改签名，应用 `?` 操作符。

- [ ] **步骤 2：Commit**
    - `git add src-tauri/src/docker/volume.rs`
    - `git commit -m "refactor(backend): adapt volume.rs to AppResult"`

### 任务 4：重构 Network 模块

**文件**：
- 修改：`src-tauri/src/docker/network.rs`

- [ ] **步骤 1：重构 `network.rs` 中的所有命令**

- [ ] **步骤 2：Commit**
    - `git add src-tauri/src/docker/network.rs`
    - `git commit -m "refactor(backend): adapt network.rs to AppResult"`

### 任务 5：重构 Compose 模块

**文件**：
- 修改：`src-tauri/src/docker/compose.rs`

- [ ] **步骤 1：重构 `compose.rs` 中的命令**
    - 特别是 `read_compose_file` 和 `write_compose_file`。

- [ ] **步骤 2：Commit**
    - `git add src-tauri/src/docker/compose.rs`
    - `git commit -m "refactor(backend): adapt compose.rs to AppResult"`

### 任务 6：重构 FS 模块

**文件**：
- 修改：`src-tauri/src/docker/fs.rs`

- [ ] **步骤 1：重构 `fs.rs` 中的辅助函数和命令**
    - 迁移 `ContainerFileInfo` 后更新引用。
    - 更新 `run_exec_to_string` 返回类型。

- [ ] **步骤 2：Commit**
    - `git add src-tauri/src/docker/fs.rs`
    - `git commit -m "refactor(backend): adapt fs.rs to AppResult"`

### 任务 7：最终验证

- [ ] **步骤 1：全量编译检查**
    - 运行 `cd src-tauri && cargo check`。

- [ ] **步骤 2：完成报告**
    - 确认所有目标文件已按规范重构。
