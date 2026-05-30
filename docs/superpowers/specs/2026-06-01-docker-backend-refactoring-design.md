# Docker 模块后端架构适配设计文档

## 目标
将 Docker 相关的后端模块重构为使用 `AppResult<T>` 和基于 Trait 的模型转换，以提升代码一致性和可维护性。

## 涉及文件
- `src-tauri/src/docker/models.rs` (DTO 定义与转换)
- `src-tauri/src/docker/image.rs`
- `src-tauri/src/docker/volume.rs`
- `src-tauri/src/docker/network.rs`
- `src-tauri/src/docker/compose.rs`
- `src-tauri/src/docker/fs.rs`

## 设计方案

### 1. `models.rs` 增强
- **迁移 DTO**: 将 `ContainerFileInfo` 从 `fs.rs` 移动到 `models.rs`。
- **完善转换 Trait**:
    - 为 `PruneImagesResult` 实现 `From<bollard::models::ImagePruneResponse>`。
    - 为 `ComposeProject` 实现从 `bollard` 容器列表聚合的逻辑（或保持现有聚合逻辑但返回 DTO）。
- **统一命名**: 确保所有 DTO 结构体在 `models.rs` 中统一定义。

### 2. 错误处理重构
- 所有 `#[tauri::command]` 函数返回类型改为 `AppResult<T>`。
- 移除 `.map_err(|e| format!(...))`，直接使用 `?`。`AppError` 已实现对 `bollard::errors::Error`、`std::io::Error` 等的转换。
- 对于特定的业务逻辑错误，使用 `AppError::Custom(String)` 或在 `AppError` 中添加新变体。

### 3. 各模块适配细节

#### Image 模块 (`image.rs`)
- `list_images`, `inspect_image` 等函数直接使用 `?` 并通过 `into()` 转换结果。
- `pull_image` 和 `export_image` 中的异步任务通过 `Emitter` 发送错误，命令本身返回 `AppResult<()>`。

#### Volume 模块 (`volume.rs`)
- 重构 `list_volumes`, `remove_volume`, `prune_volumes`。
- `list_volume_containers` 保持现有逻辑，但返回 `AppResult<Vec<VolumeUser>>`。

#### Network 模块 (`network.rs`)
- 重构所有网络操作函数。

#### Compose 模块 (`compose.rs`)
- `list_compose_projects` 返回 `AppResult<Vec<ComposeProject>>`。
- `read_compose_file`, `write_compose_file` 使用 `?` 处理 IO 错误。

#### FS 模块 (`fs.rs`)
- 将 `ContainerFileInfo` 移至 `models.rs`。
- `run_exec_to_string` 辅助函数改为返回 `AppResult<(String, String)>`。
- 优化 shell 脚本执行的错误处理。

## 验证计划
- 确保代码编译通过。
- 检查各命令在前端调用时的错误捕获是否正常。
