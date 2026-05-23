# 镜像页面 UI 重构实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 修复损坏的 `Images.vue` 并实现内嵌式镜像仓库搜索与拉取。

**架构：** 将 `Images.vue` 重构为左右布局，右侧使用 `activeTab` 切换“镜像仓库”、“层级结构”和“镜像详情”。移除拉取确认弹窗。

**技术栈：** Vue 3 (Composition API), Naive UI, Tauri, Pinia (Store)

---

### 任务 1：重构并修复 `src/views/Images.vue`

**文件：**
- 修改：`src/views/Images.vue`

- [ ] **步骤 1：准备完整且修复后的代码内容**

需要确保：
1. 包含完整的 `<script setup>` 段，包含所有必要的 import。
2. 包含完整的 `handleMenuSelect` 逻辑。
3. 包含完整的 `<template>` 段，结构清晰。
4. 包含完整的 `<style scoped>` 段。
5. 移除 `showConfirmModal` 和 `pendingPullImage` 相关的逻辑。

- [ ] **步骤 2：执行文件覆写**

使用 `write_file` 覆写 `src/views/Images.vue`。

- [ ] **步骤 3：验证编译与功能**

运行：`npm run type-check` (或类似命令) 确保语法正确。
手动验证：
1. 本地镜像列表显示正常。
2. 点击本地镜像能切换到“层级结构”和“详情”。
3. “镜像仓库”选项卡显示正常，搜索工作正常。
4. 选中搜索结果后立即触发拉取，不显示弹窗。
5. 后台任务进度条显示正常。

### 任务 2：清理与微调

**文件：**
- 修改：`src/views/Images.vue`

- [ ] **步骤 1：移除多余的 CSS 和变量**

检查并删除不再使用的 CSS 类（如 `.pull-modal-body` 等）。

- [ ] **步骤 2：优化视觉体验**

确保选项卡样式符合 Apple 风格，且过渡自然。

- [ ] **步骤 3：Commit**

```bash
git add src/views/Images.vue docs/superpowers/specs/2026-05-23-images-ui-redesign-design.md
git commit -m "feat(ui): integrated image registry into Images view and removed pull confirmation modal"
```
