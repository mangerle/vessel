# Compose 与容器交互重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重构 Compose 页面为两栏布局，支持层级列表、多标签页详情（日志/设置/仪表盘）及集成终端。

**Architecture:** 
- **后端**: 增加容器 Inspect、日志流和终端 Exec 交互。
- **前端**: 使用 `NSplit` 或 CSS Grid 实现双栏布局，集成 `xterm.js`。

---

### Task 1: 后端数据采集与交互增强

**Files:**
- Modify: `src-tauri/src/docker.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 实现容器 Inspect 命令**
返回环境变量、端口映射、卷挂载详情。

- [ ] **Step 2: 实现容器日志获取命令**
支持获取最近 N 行日志并建立流式监听。

- [ ] **Step 3: 实现终端 Exec 基础框架**
定义 `open_container_terminal` 命令，启动 shell 进程并建立事件转发。

- [ ] **Step 4: 提交**

---

### Task 2: Compose 视图两栏布局重构

**Files:**
- Modify: `src/views/Compose.vue`
- Create: `src/components/ComposeTree.vue`
- Create: `src/components/ContainerDetailView.vue`

- [ ] **Step 1: 实现嵌套双栏布局**
在 `Compose.vue` 内部，使用 `NLayout` 或 `NSplit` 实现左侧子栏（项目树）和右侧工作区（详情）。

- [ ] **Step 2: 实现状态图标逻辑**
运行中（绿色）、已停止（灰白色）。

- [ ] **Step 3: 实现点击交互**
点击容器后，右侧详情区展示其元数据。

- [ ] **Step 4: 提交**

---

### Task 3: 容器详情标签页与设置页实现

**Files:**
- Create: `src/components/container/ContainerLogs.vue`
- Create: `src/components/container/ContainerSettings.vue`
- Create: `src/components/container/ContainerDashboard.vue`

- [ ] **Step 1: 实现多标签页切换**
日志、设置、仪表盘。

- [ ] **Step 2: 实现设置页 UI**
列表展示环境变量、端口和卷。

- [ ] **Step 3: 迁移仪表盘**
将原有的监控图表集成到 Dashboard 标签页中。

- [ ] **Step 4: 提交**

---

### Task 4: 右键菜单与操作集成

**Files:**
- Modify: `src/components/ComposeTree.vue`
- Modify: `src/components/ContainerDetailView.vue`

- [ ] **Step 1: 实现全局右键菜单组件**
集成“重启、暂停、复制 ID、删除、创建终端”操作。

- [ ] **Step 2: 实现终端二级菜单**
作为普通用户、作为 Root 用户。

- [ ] **Step 3: 提交**

---

### Task 5: 集成 Xterm.js 终端

**Files:**
- Create: `src/components/container/ContainerTerminal.vue`
- Modify: `package.json`

- [ ] **Step 1: 安装依赖**
`npm install xterm xterm-addon-fit`

- [ ] **Step 2: 实现终端组件**
与后端事件流对接，实现命令输入与输出显示。

- [ ] **Step 3: 提交**
