# Vessel 架构重构与规范化设计规格书

**日期:** 2026-05-30
**目标:** 通过“大刀阔斧”的架构重整，消除代码坏味道，建立前后端统一的开发规范，提升代码的可维护性与扩展性。

---

## 1. 后端重构：强类型与分层架构 (Rust/Tauri)

### 1.1 统一错误处理系统 (`src-tauri/src/error.rs`)
- **设计描述:** 建立全应用通用的错误枚举 `AppError`。
- **技术实现:**
    - 使用 `thiserror` 库实现错误定义的声明式处理。
    - 自动适配 `bollard::errors::Error`, `std::io::Error`, `tauri::Error` 等。
    - 为 `AppError` 实现 `serde::Serialize`，以便 Tauri 能直接将错误传递给前端。
- **规范:** 禁止在命令层使用 `map_err(|e| e.to_string())`，统一使用 `?` 操作符。

### 1.2 领域模型转换层 (`src-tauri/src/docker/models.rs`)
- **设计描述:** 集中定义所有 DTO（Data Transfer Objects），并建立从 Bollard 原始类型到 DTO 的映射。
- **技术实现:**
    - 废除在 `mod.rs` 中散乱定义的结构体。
    - 为每个 DTO 实现 `From<BollardModel>` 特性。
    - 示例: `impl From<bollard::models::ContainerSummary> for ContainerInfo { ... }`
- **收益:** 极大简化命令层的逻辑，使业务代码更具声明性。

### 1.3 统一流注册中心 (`src-tauri/src/docker/stream_manager.rs`)
- **设计描述:** 抽象出一个通用的异步流管理器，替代目前散落在各处的静态 `HashMap`。
- **技术实现:**
    - 使用泛型 `StreamRegistry<T>` 管理 `Sender`。
    - 提供标准化的 `register`, `abort`, `cleanup` 方法。
    - 支持按 ID 和任务类型隔离流。

---

## 2. 前端重构：逻辑抽离与响应式规范 (Vue 3/Pinia)

### 2.1 API 服务层解耦 (`src/api/`)
- **设计描述:** 建立与后端命令一一对应的前端 API 调用层。
- **目录结构:**
    - `src/api/container.ts`
    - `src/api/image.ts`
    - `src/api/compose.ts`
- **规范:** 所有 `invoke` 调用必须通过 `src/api/` 导出，禁止在组件或 Store 中直接调用 `invoke`。

### 2.2 Store 职责精简化
- **设计描述:** Pinia Store 仅保留“状态存储”职责，复杂的交互流程迁移至 `services` 或 `api`。
- **改进点:**
    - `image.ts`: 移除 `pullImage` 中复杂的事件监听逻辑，将其封装为独立的 Service 函数。
    - Store 中的 Actions 仅负责调用 API 并更新 State。

### 2.3 统一接口处理管道 (`src/hooks/useApi.ts`)
- **设计描述:** 建立一个高阶 Hook，自动处理异步任务的状态循环。
- **功能:**
    - 自动管理 `loading`, `error`, `data`。
    - 提供可选的成功/失败全局通知。
    - 示例用法: `const { execute, loading } = useApi(containerApi.start)`

---

## 3. 全栈通信协议标准化

### 3.1 任务事件契约
- 对于长耗时任务（Pull/Export/Import），统一事件格式：
  ```typescript
  {
    taskId: string;
    type: 'progress' | 'log' | 'error' | 'finished';
    payload: any;
  }
  ```

---

## 4. 实施阶段规划

1. **阶段 1 (核心基石):** 实现后端 `error.rs` 和 `models.rs` 的 Trait 转换逻辑。
2. **阶段 2 (分层隔离):** 建立前端 `src/api/` 目录并完成初步解耦。
3. **阶段 3 (流控优化):** 实现后端 `stream_manager.rs` 并重构日志/统计流。
4. **阶段 4 (全栈规范):** 统一 Store 逻辑与任务管道。

---

## 5. 验收标准
- [ ] 没有任何 `tauri::command` 函数的代码行数超过 30 行（逻辑主要被 Trait 承载）。
- [ ] 前端 Store 中不再包含显式的 `try...catch` 状态切换。
- [ ] 所有异步流在窗口关闭时能被正确释放。
- [ ] 全应用错误提示统一，不再出现未经翻译的原始错误堆栈信息。
