# 设计文档：建立前端 API 服务层

## 目标
建立一个统一的前端 API 服务层，封装 Tauri 的 `invoke` 调用，提高代码的可维护性和类型安全性。

## 方案设计

### 1. 类型定义更新
为了保证 API 层的类型安全，需要先在 Store 中完善类型定义并导出。

#### 修改 `src/store/container.ts`
- 导出 `ContainerInfo`。
- 添加并导出 `PortMapping` 接口。
- 添加并导出 `MountInfo` 接口。
- 添加并导出 `ContainerDetails` 接口。

#### 修改 `src/store/image.ts`
- 确保 `ImageInfo`, `ImageDetails`, `ImageSearchResult` 等类型已正确导出。

### 2. API 服务层实现

#### 创建 `src/api/container.ts`
封装容器相关的操作。

| 方法 | Tauri 命令 | 参数 | 返回值 |
| :--- | :--- | :--- | :--- |
| `list` | `list_local_containers` | - | `Promise<ContainerInfo[]>` |
| `start` | `start_container` | `id: string` | `Promise<void>` |
| `stop` | `stop_container` | `id: string` | `Promise<void>` |
| `restart` | `restart_container` | `id: string` | `Promise<void>` |
| `remove` | `remove_container` | `id: string` | `Promise<void>` |
| `pause` | `pause_container` | `id: string` | `Promise<void>` |
| `unpause` | `unpause_container` | `id: string` | `Promise<void>` |
| `inspect` | `inspect_container` | `id: string` | `Promise<ContainerDetails>` |

#### 创建 `src/api/image.ts`
封装镜像相关的操作。

| 方法 | Tauri 命令 | 参数 | 返回值 |
| :--- | :--- | :--- | :--- |
| `list` | `list_images` | - | `Promise<ImageInfo[]>` |
| `inspect` | `inspect_image` | `id: string` | `Promise<ImageDetails>` |
| `remove` | `remove_image` | `id: string` | `Promise<void>` |
| `search` | `search_images` | `term: string` | `Promise<ImageSearchResult[]>` |
| `pull` | `pull_image` | `params: PullParams` | `Promise<void>` |
| `prune` | `prune_images` | - | `Promise<{ deleted_count: number; space_reclaimed: number }>` |
| `history` | `get_image_history` | `id: string` | `Promise<ImageHistoryInfo[]>` |

## 验证计划
- 检查文件语法。
- 确认所有 `invoke` 命令名称与 Rust 后端一致。
- 确认类型导入路径正确。
- 在现有 Store 中尝试替换 `invoke` 调用为新的 API 调用（可选，但推荐作为验证）。
