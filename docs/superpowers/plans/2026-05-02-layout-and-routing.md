# 侧边栏布局与多页面路由重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 Docker 管理工具从单页面列表重构为具有侧边栏导航的多页面专业布局。

**Architecture:** 使用 Naive UI 的 `NLayout` 组件族实现标准侧边栏布局。通过 Vue Router 管理不同功能模块的导航。将现有容器列表功能迁移到新路由结构中。

**Tech Stack:** Vue 3, Naive UI, Vue Router, @vicons/ionicons5

---

### Task 1: 安装图标库依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装 @vicons/ionicons5**

运行: `npm install @vicons/ionicons5`

- [ ] **Step 2: 验证安装**

检查 `package.json` 中的 `dependencies`。

---

### Task 2: 创建功能视图占位组件

**Files:**
- Create: `src/views/Connections.vue`
- Create: `src/views/Containers.vue` (迁移自 ContainerList.vue)
- Create: `src/views/Images.vue`
- Create: `src/views/Compose.vue`
- Create: `src/views/Networks.vue`
- Create: `src/views/Volumes.vue`

- [ ] **Step 1: 迁移 ContainerList.vue 到 Containers.vue**

将 `src/views/ContainerList.vue` 重命名/移动到 `src/views/Containers.vue`。

- [ ] **Step 2: 创建其他视图的占位组件**

为 Connections, Images, Compose, Networks, Volumes 创建简单的 `n-result` 或标题占位组件。

---

### Task 3: 实现 MainLayout 基础布局

**Files:**
- Create: `src/layout/MainLayout.vue`

- [ ] **Step 1: 创建 src/layout 目录**

- [ ] **Step 2: 编写 MainLayout.vue 逻辑与模板**

实现 `n-layout` 嵌套，包含 `n-layout-sider` (内部使用 `n-menu`) 和 `n-layout-content` (内部包含 `router-view`)。

---

### Task 4: 更新路由配置

**Files:**
- Modify: `src/router/index.ts`

- [ ] **Step 1: 配置嵌套路由**

将 `MainLayout` 设置为根路由的组件，并配置 6 个功能模块为子路由。

---

### Task 5: 重构 App.vue

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 简化 App.vue**

集成全局 Naive UI 提供者（`n-config-provider`, `n-message-provider` 等），并直接包含顶层 `router-view`。

---

### Task 6: 验证与提交

- [ ] **Step 1: 运行构建检查**

运行: `npm run build` 或 `npx tsc`

- [ ] **Step 2: 提交代码**

```bash
git add .
git commit -m "feat: 重构侧边栏布局并增加多页面支持"
```
