# API 层重构：网络与数据卷实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 完成 API 服务层的迁移，封装 Docker 网络和数据卷的相关调用。

**架构：**
- `src/api/types.ts`: 存放所有 API 相关的接口定义。
- `src/api/network.ts`: 封装网络相关的 `invoke` 调用。
- `src/api/volume.ts`: 封装数据卷相关的 `invoke` 调用。
- `src/store/*.ts`: 仅调用 API 层，不再直接使用 `invoke`。

**技术栈：** TypeScript, Pinia, Tauri `@tauri-apps/api/core`.

---

### 任务 1：更新 `src/api/types.ts`

**文件：**
- 修改：`src/api/types.ts`

- [ ] **步骤 1：添加网络相关接口**
- [ ] **步骤 2：添加数据卷相关接口**

### 任务 2：创建 `src/api/network.ts`

**文件：**
- 创建：`src/api/network.ts`

- [ ] **步骤 1：实现网络 API 封装**

### 任务 3：创建 `src/api/volume.ts`

**文件：**
- 创建：`src/api/volume.ts`

- [ ] **步骤 1：实现数据卷 API 封装**

### 任务 4：重构 `src/store/network.ts`

**文件：**
- 修改：`src/store/network.ts`

- [ ] **步骤 1：移除 `invoke` 导入和接口定义**
- [ ] **步骤 2：使用 `src/api/network.ts` 中的函数重构 actions**

### 任务 5：重构 `src/store/volume.ts`

**文件：**
- 修改：`src/store/volume.ts`

- [ ] **步骤 1：移除 `invoke` 导入和接口定义**
- [ ] **步骤 2：使用 `src/api/volume.ts` 中的函数重构 actions**

### 任务 6：验证与提交

- [ ] **步骤 1：运行类型检查确认没有错误**
- [ ] **步骤 2：提交更改**
