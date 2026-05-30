# 重构 Container Store 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 重构 `src/store/container.ts`，使用 `containerApi` 替代直接的 `invoke` 调用。

**架构：** 将 Store 中的 `invoke` 调用迁移到 `containerApi` 层，简化 Actions 逻辑，同时保持全局状态管理。

**技术栈：** Vue 3, Pinia, TypeScript, Tauri API

---

### 任务 1：重构 `src/store/container.ts`

**文件：**
- 修改：`src/store/container.ts`

- [ ] **步骤 1：引入 `containerApi` 并更新 `fetchContainers`**

修改 `src/store/container.ts` 的导入部分和 `fetchContainers` 方法。

```typescript
import { defineStore } from 'pinia'
import { containerApi } from '../api/container'
import type { ContainerInfo } from '../api/types'

// ...
    async fetchContainers() {
      this.loading = true
      this.error = null
      try {
        this.containers = await containerApi.list()
      } catch (err) {
        console.error('获取容器失败:', err)
        this.error = String(err)
      } finally {
        this.loading = false
      }
    },
// ...
```

- [ ] **步骤 2：更新所有操作方法（start, stop, restart, remove, pause, unpause）**

将剩余的 `invoke` 调用替换为 `containerApi` 的对应方法。

```typescript
    async startContainer(id: string) {
      this.loading = true
      this.error = null
      try {
        await containerApi.start(id)
        await this.fetchContainers()
      } catch (err) {
        console.error('启动容器失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    // ... 对其他方法进行类似修改
```

- [ ] **步骤 3：验证编译并 Commit**

运行：`npm run type-check` (如果有的话) 或确保无语法错误。

```bash
git add src/store/container.ts
git commit -m "refactor(frontend): use containerApi in container store"
```
