# 镜像、网络与数据卷视图 macOS 风格重构实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将镜像、网络和数据卷视图重构为 macOS 三栏式卡片布局，并增加镜像拉取联想、镜像层级预览、网络连接容器列表及数据卷使用追踪功能。

**架构：** 采用 Vue 3 + Pinia + Naive UI 的组合，通过扩展 Rust 后端 API 提供更深层的 Docker 数据，并在前端利用分段选择器（Segmented Control）实现多维度的资源详情展示。

**技术栈：** Vue 3, Pinia, Naive UI, Tauri, Rust (Bollard).

---

## 任务列表

### 任务 1：后端 Docker 命令扩展

**文件：**
- 修改：`src-tauri/src/docker.rs`
- 修改：`src-tauri/src/lib.rs`

- [ ] **步骤 1：在 docker.rs 中定义数据结构和新命令**
- [ ] **步骤 2：注册新命令到 lib.rs**
- [ ] **步骤 3：Commit**

### 任务 2：前端 Store 升级

**文件：**
- 修改：`src/store/image.ts`
- 修改：`src/store/network.ts`
- 修改：`src/store/volume.ts`

- [ ] **步骤 1：更新 image.ts 增加搜索和历史逻辑**
- [ ] **步骤 2：更新 network.ts 增加获取连接容器逻辑**
- [ ] **步骤 3：更新 volume.ts 增加打开路径逻辑**
- [ ] **步骤 4：Commit**

### 任务 3：镜像视图 (Images.vue) 重构

**文件：**
- 修改：`src/views/Images.vue`

- [ ] **步骤 1：实现三栏布局与拉取联想功能**
- [ ] **步骤 2：实现详情页概览、层级（Timeline）和配置标签页**
- [ ] **步骤 3：Commit**

### 任务 4：网络视图 (Networks.vue) 重构

**文件：**
- 修改：`src/views/Networks.vue`

- [ ] **步骤 1：实现详情页连接容器表格**
- [ ] **步骤 2：应用 macOS 三栏样式**
- [ ] **步骤 3：Commit**

### 任务 5：数据卷视图 (Volumes.vue) 重构

**文件：**
- 修改：`src/views/Volumes.vue`

- [ ] **步骤 1：实现详情页挂载容器列表**
- [ ] **步骤 2：添加“在资源管理器中打开”功能**
- [ ] **步骤 3：Commit**
