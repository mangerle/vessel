# 完成 Compose API 层迁移及类型清理实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将 Docker Compose 相关的 API 调用从 Store 层迁移到统一的 API 服务层，并清理和统一类型定义。

**架构：** 
- 创建 `src/api/compose.ts` 作为 Compose 相关的 API 封装。
- 将 `ComposeProject` 类型定义移动到 `src/api/types.ts`。
- 更新 `src/store/compose.ts` 以调用 `src/api/compose.ts`。
- 修复任何相关的类型引用错误。

**技术栈：** Vue 3, Pinia, Tauri, TypeScript

---

### 任务 1：更新类型定义

**文件：**
- 修改：`src/api/types.ts`
- 修改：`src/store/compose.ts`

- [ ] **步骤 1：在 `src/api/types.ts` 中添加 `ComposeProject` 接口**

```typescript
export interface ComposeProject {
  name: string
  container_count: number
  running_count: number
  status: string
  working_dir?: string
  config_file?: string
}
```

- [ ] **步骤 2：从 `src/store/compose.ts` 中删除 `ComposeProject` 定义，改从 `../api/types` 导入**

- [ ] **步骤 3：验证类型定义一致性**

### 任务 2：创建 Compose API 服务层

**文件：**
- 创建：`src/api/compose.ts`

- [ ] **步骤 1：创建 `src/api/compose.ts` 并封装 Tauri 命令**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { ComposeProject } from './types'

/**
 * Docker Compose 相关的 API 服务
 */
export const composeApi = {
  /**
   * 获取 Compose 项目列表
   */
  listProjects: () => invoke<ComposeProject[]>('list_compose_projects'),

  /**
   * 读取项目的 Compose 文件
   */
  readFile: (path: string, mode?: string, distro?: string) => 
    invoke<string>('read_compose_file', { path, mode, distro }),

  /**
   * 保存项目的 Compose 文件
   */
  writeFile: (path: string, content: string, mode?: string, distro?: string) => 
    invoke<void>('write_compose_file', { path, content, mode, distro }),

  /**
   * 运行 Compose 命令
   */
  runCommand: (projectDir: string, args: string[], mode?: string, distro?: string) => 
    invoke<void>('run_compose_command', { projectDir, args, mode, distro })
}
```

### 任务 3：重构 Compose Store

**文件：**
- 修改：`src/store/compose.ts`

- [ ] **步骤 1：导入 `composeApi` 并替换 `invoke` 调用**

- [ ] **步骤 2：更新 `fetchProjects`, `fetchComposeFile`, `saveComposeFile`, `runComposeCommand` 方法**

### 任务 4：清理与验证

**文件：**
- 修改：`src/views/Volumes.vue` (如果需要)

- [ ] **步骤 1：检查 `src/views/Volumes.vue` 是否有类型引用错误并修复**

- [ ] **步骤 2：运行类型检查 (如果环境支持)**

运行：`npm run type-check` 或 `tsc --noEmit`

- [ ] **步骤 3：Commit 变更**

```bash
git add src/api/ src/store/
git commit -m "refactor(frontend): complete api layer migration for compose and cleanup types"
```
