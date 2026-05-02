# 数据库与架构质量提升实施计划

> **对于智能代理：** 必需的子技能：使用 superpowers:subagent-driven-development (推荐) 或 superpowers:executing-plans 按任务逐步执行此计划。步骤使用复选框 (`- [ ]`) 语法进行跟踪。

**目标：** 解决 Task 2 代码质量审查中发现的问题，包括错误处理优化、测试增强和单例逻辑鲁棒性。

**架构：**
1. 在 `lib.rs` 中引入 `anyhow` 进行统一错误处理。
2. 在 `db.rs` 中改进 `POOL` 初始化，处理 `POOL.set` 的返回值，并增加对业务表的测试验证。
3. 确保所有代码符合 `cargo clippy` 且无警告。

**技术栈：** Rust, Tauri, sqlx, anyhow

---

### 任务 1：优化 `lib.rs` 错误处理

**文件：**
- 修改：`src-tauri/src/lib.rs`

- [x] **步骤 1：引入 `anyhow` 并重构 `run` 函数**
- [x] **步骤 2：验证编译**

### 任务 2：改进 `db.rs` 单例逻辑与测试

**文件：**
- 修改：`src-tauri/src/db.rs`

- [x] **步骤 1：改进 `init_db` 逻辑**
- [x] **步骤 2：增强测试用例**

### 任务 3：最终验证与清理

- [x] **步骤 1：运行所有测试**
- [x] **步骤 2：运行 Clippy 检查**
- [x] **步骤 3：提交更改**
