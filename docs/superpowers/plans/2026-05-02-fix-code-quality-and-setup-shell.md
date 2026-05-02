# 修复代码质量与配置 Shell 插件实施计划

> **对于代理工作者：** 必需的子技能：使用 superpowers:subagent-driven-development (推荐) 或 superpowers:executing-plans 来逐任务执行此计划。步骤使用复选框 (`- [ ]`) 语法进行跟踪。

**目标：** 按照 Gemini 指令修复代码质量问题，重命名项目，并配置 Shell 插件。

**架构：** 修改配置文件（package.json, Cargo.toml, capabilities/default.json）和 Rust 源代码（main.rs, lib.rs），确保符合命名规范、语言要求和插件配置要求。

**技术栈：** Tauri v2, Rust, Node.js

---

### 任务 1：重命名项目

**文件：**
- 修改：`package.json`
- 修改：`src-tauri/Cargo.toml`

- [ ] **步骤 1：修改 package.json 中的名称**

将 `"name": "tauri-app"` 改为 `"name": "docker-manager"`。

- [ ] **步骤 2：修改 src-tauri/Cargo.toml 中的名称**

将 `[package]` 下的 `name = "tauri-app"` 改为 `name = "docker-manager"`。
将 `[lib]` 下的 `name = "tauri_app_lib"` 改为 `name = "docker_manager_lib"`。

- [ ] **步骤 3：提交更改**

```bash
git add package.json src-tauri/Cargo.toml
git commit -m "refactor: rename project to docker-manager"
```

### 任务 2：更新 Rust 源码以反映库名称更改并翻译注释

**文件：**
- 修改：`src-tauri/src/main.rs`
- 修改：`src-tauri/src/lib.rs`

- [ ] **步骤 1：更新 src-tauri/src/main.rs**

将 `tauri_app_lib::run()` 改为 `docker_manager_lib::run()`。并将注释翻译为中文。

```rust
// 在发布模式下防止在 Windows 上出现额外的控制台窗口，请勿删除！！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    docker_manager_lib::run()
}
```

- [ ] **步骤 2：更新 src-tauri/src/lib.rs**

将注释翻译为中文，并确保所有新增/修改的代码符合中文注释要求。

- [ ] **步骤 3：提交更改**

```bash
git add src-tauri/src/main.rs src-tauri/src/lib.rs
git commit -m "refactor: update rust source with new lib name and chinese comments"
```

### 任务 3：初始化并启用 Shell 插件

**文件：**
- 修改：`src-tauri/src/lib.rs`
- 修改：`src-tauri/capabilities/default.json`

- [ ] **步骤 1：在 src-tauri/src/lib.rs 中初始化 shell 插件**

在 `run` 函数中添加 `.plugin(tauri_plugin_shell::init())`。

- [ ] **步骤 2：在 src-tauri/capabilities/default.json 中启用 shell 插件权限**

在 `permissions` 数组中添加 `"shell:default"`。

- [ ] **步骤 3：运行验证**

运行 `cargo check` 和 `cargo clippy`。

- [ ] **步骤 4：提交更改**

```bash
git add src-tauri/src/lib.rs src-tauri/capabilities/default.json
git commit -m "feat: initialize and enable tauri-plugin-shell"
```
