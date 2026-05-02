# Docker Manager 全功能实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重构应用布局并实现 6 大核心管理模块，支持多连接切换、实时监控及全生命周期管理。

**Architecture:** 
- **前端**: Vue Router 路由导航，Pinia 管理全局活动连接。
- **后端**: 扩展 Rust Command 以支持镜像拉取流、性能指标采样。
- **UI**: Naive UI 侧边栏布局，ECharts 实时监控。

---

### Task 1: 侧边栏布局与多页面路由重构 [DONE]

**Files:**
- Modify: `src/App.vue`
- Modify: `src/router/index.ts`
- Create: `src/layout/MainLayout.vue`
- Create: `src/views/Connections.vue`
- Create: `src/views/Containers.vue`
- Create: `src/views/Images.vue`
- Create: `src/views/Compose.vue`
- Create: `src/views/Networks.vue`
- Create: `src/views/Volumes.vue`

- [x] **Step 1: 更新路由定义**
配置 6 个主模块的路由路径。

- [x] **Step 2: 创建 MainLayout 组件**
实现带有侧边栏 (NLayoutSider) 和主内容区 (NLayoutContent) 的基础布局。

- [x] **Step 3: 重写 App.vue**
集成路由视图和全局 Naive UI 配置。

- [x] **Step 4: 提交**

---

### Task 2: Docker 连接管理 (Module 1) [DONE]

**Files:**
- Create: `src-tauri/src/connection/manager.rs`
- Modify: `src-tauri/src/connection/mod.rs`
- Modify: `src/views/Connections.vue`
- Modify: `src/store/connection.ts`

- [x] **Step 1: 实现后端连接 CRUD Command**
支持从数据库读取、添加、删除自定义连接（SSH/TCP/Local）。

- [x] **Step 2: 实现活动连接切换逻辑**
当前端切换连接时，后端更新全局 `DOCKER_CLIENT`。

- [x] **Step 3: 编写连接管理 UI**
实现连接列表展示及“添加连接”模态框。

- [x] **Step 4: 提交**

---

### Task 3: 容器管理增强与实时监控 (Module 3)

**Files:**
- Modify: `src-tauri/src/docker.rs`
- Create: `src/views/ContainerDetail.vue`
- Modify: `src/views/Containers.vue`

- [ ] **Step 1: 实现容器控制 API**
后端增加 `start_container`, `stop_container`, `remove_container` 命令。

- [ ] **Step 2: 实现实时监控数据流**
后端通过 `bollard::container::StatsOptions` 采样数据并推送到前端。

- [ ] **Step 3: 完善容器列表与详情页**
列表显示“运行/总数”统计，详情页集成 ECharts 展示 CPU/内存曲线。

- [ ] **Step 4: 提交**

---

### Task 4: 镜像管理与拉取逻辑 (Module 4)

**Files:**
- Modify: `src-tauri/src/docker.rs`
- Modify: `src/views/Images.vue`

- [ ] **Step 1: 实现镜像操作 API**
支持获取本地镜像列表、删除镜像。

- [ ] **Step 2: 实现镜像拉取 (Pull) 功能**
后端处理拉取流，通过 Tauri Event 反馈进度条。

- [ ] **Step 3: 编写镜像管理 UI**
支持关键词搜索、显示镜像层信息。

- [ ] **Step 4: 提交**

---

### Task 5: Compose、网络与卷管理 (Modules 2, 5, 6)

**Files:**
- Modify: `src-tauri/src/docker.rs`
- Modify: `src/views/Compose.vue`
- Modify: `src/views/Networks.vue`
- Modify: `src/views/Volumes.vue`

- [ ] **Step 1: 实现 Compose 探测逻辑**
后端通过容器 Label 自动归类 Compose 项目。

- [ ] **Step 2: 实现网络与卷的 CRUD**
基础列表显示与删除功能，网络 Prune 功能。

- [ ] **Step 3: 编写对应 UI 页面**
保持与容器列表一致的交互风格。

- [ ] **Step 4: 提交**
