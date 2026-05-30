# 前端 API 服务层实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 建立 `src/api/` 目录并实现 `container.ts` 和 `image.ts` API 服务层。

**架构：**
- 更新 `src/store/container.ts` 导出必要类型。
- 创建 `src/api/container.ts` 封装容器 API。
- 创建 `src/api/image.ts` 封装镜像 API。

**技术栈：** TypeScript, Tauri API, Pinia (用于类型导入)。

---

### 任务 1：更新 Container Store 类型

**文件：**
- 修改：`src/store/container.ts`

- [ ] **步骤 1：添加并导出 `PortMapping`, `MountInfo`, `ContainerDetails` 接口，并导出 `ContainerInfo`。**

```typescript
export interface ContainerInfo {
  id: string
  name: string
  state: string
  image: string
  compose_project?: string
}

export interface PortMapping {
  private_port: number
  public_port?: number
  type_: string
  ip?: string
}

export interface MountInfo {
  source: string
  destination: string
  mode: string
  rw: boolean
}

export interface ContainerDetails {
  id: string
  name: string
  image: string
  image_id: string
  state: string
  status: string
  created: string
  env: string[]
  ports: PortMapping[]
  mounts: MountInfo[]
}
```

- [ ] **步骤 2：Commit**

```bash
git add src/store/container.ts
git commit -m "refactor(frontend): export container types"
```

### 任务 2：创建 Container API

**文件：**
- 创建：`src/api/container.ts`

- [ ] **步骤 1：创建 `src/api/` 目录**

```bash
mkdir src/api
```

- [ ] **步骤 2：创建 `src/api/container.ts` 并实现方法**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { ContainerInfo, ContainerDetails } from '../store/container'

export const containerApi = {
  list: () => invoke<ContainerInfo[]>('list_local_containers'),
  start: (id: string) => invoke<void>('start_container', { id }),
  stop: (id: string) => invoke<void>('stop_container', { id }),
  restart: (id: string) => invoke<void>('restart_container', { id }),
  remove: (id: string) => invoke<void>('remove_container', { id }),
  pause: (id: string) => invoke<void>('pause_container', { id }),
  unpause: (id: string) => invoke<void>('unpause_container', { id }),
  inspect: (id: string) => invoke<ContainerDetails>('inspect_container', { id })
}
```

- [ ] **步骤 3：Commit**

```bash
git add src/api/container.ts
git commit -m "feat(frontend): add container api service"
```

### 任务 3：创建 Image API

**文件：**
- 创建：`src/api/image.ts`

- [ ] **步骤 1：创建 `src/api/image.ts` 并实现方法**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { ImageInfo, ImageDetails, ImageSearchResult, ImageHistoryInfo } from '../store/image'

export interface PullParams {
  imageName: string
  username?: string | null
  password?: string | null
  serverAddress?: string | null
}

export const imageApi = {
  list: () => invoke<ImageInfo[]>('list_images'),
  inspect: (id: string) => invoke<ImageDetails>('inspect_image', { id }),
  remove: (id: string) => invoke<void>('remove_image', { id }),
  search: (term: string) => invoke<ImageSearchResult[]>('search_images', { term }),
  pull: (params: PullParams) => invoke<void>('pull_image', params),
  prune: () => invoke<{ deleted_count: number; space_reclaimed: number }>('prune_images'),
  history: (id: string) => invoke<ImageHistoryInfo[]>('get_image_history', { id })
}
```

- [ ] **步骤 2：Commit**

```bash
git add src/api/image.ts
git commit -m "feat(frontend): add image api service"
```

### 任务 4：验证与清理

- [ ] **步骤 1：检查文件语法**
- [ ] **步骤 2：最终 Commit**

```bash
git add src/api/
git commit -m "feat(frontend): create api service layer"
```
