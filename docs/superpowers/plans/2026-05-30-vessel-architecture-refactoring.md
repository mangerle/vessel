# Vessel 架构重构实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 通过“大刀阔斧”的架构重整，消除代码坏味道，建立前后端统一的开发规范，提升代码的可维护性与扩展性。

**架构：** 
1. **后端：** 引入 `thiserror` 统一错误处理，建立 `models.rs` 使用 `From` Trait 进行类型转换，实现 `StreamRegistry` 统一流管理。
2. **前端：** 建立 `api/` 层隔离 `invoke` 调用，精简 Pinia Store 职责，引入 `useApi` Hook 标准化请求处理。

**技术栈：** Rust, Tauri 2.0, Vue 3, Pinia, bollard, thiserror, naive-ui

---

## 文件结构预定义

### 后端 (src-tauri/src/)
- `error.rs`: 统一错误定义 (新建)
- `docker/models.rs`: 领域模型与转换逻辑 (新建)
- `docker/stream_manager.rs`: 异步流管理器 (新建)
- `docker/mod.rs`: 模块导出
- `lib.rs`: 注册命令与初始化

### 前端 (src/)
- `api/`: API 服务层 (新建)
- `api/container.ts`, `api/image.ts`, `api/volume.ts`, `api/network.ts`, `api/compose.ts`
- `hooks/useApi.ts`: 统一请求 Hook (新建)
- `store/`: 精简后的状态仓库

---

## 阶段一：后端核心基石

### 任务 1：添加依赖并创建错误处理系统

**文件：**
- 修改：`src-tauri/Cargo.toml`
- 创建：`src-tauri/src/error.rs`
- 修改：`src-tauri/src/lib.rs`

- [ ] **步骤 1：添加 `thiserror` 依赖**
```toml
# src-tauri/Cargo.toml
[dependencies]
thiserror = "2.0"
```

- [ ] **步骤 2：定义 `AppError` 枚举**
```rust
// src-tauri/src/error.rs
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Docker 错误: {0}")]
    Docker(#[from] bollard::errors::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tauri 错误: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("未知错误: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("{0}")]
    Custom(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

- [ ] **步骤 3：在 `lib.rs` 引入模块**
```rust
// src-tauri/src/lib.rs
pub mod error; // 添加这一行
```

- [ ] **步骤 4：Commit**
```bash
git add src-tauri/Cargo.toml src-tauri/src/error.rs src-tauri/src/lib.rs
git commit -m "refactor(backend): add thiserror and define AppError"
```

### 任务 2：重构领域模型与类型转换

**文件：**
- 创建：`src-tauri/src/docker/models.rs`
- 修改：`src-tauri/src/docker/mod.rs`

- [ ] **步骤 1：将模型迁移至 `models.rs` 并实现 `From` Trait**
将 `mod.rs` 中的结构体移动过去，并添加转换逻辑。示例：
```rust
// src-tauri/src/docker/models.rs
use serde::Serialize;
use bollard::models::ContainerSummary;

#[derive(Serialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub state: String,
    pub image: String,
    pub compose_project: Option<String>,
}

impl From<ContainerSummary> for ContainerInfo {
    fn from(c: ContainerSummary) -> Self {
        let compose_project = c.labels.as_ref()
            .and_then(|labels| labels.get("com.docker.compose.project").cloned());
        
        Self {
            id: c.id.unwrap_or_default(),
            name: c.names.as_ref()
                .and_then(|names| names.first())
                .map(|name| name.trim_start_matches('/').to_string())
                .unwrap_or_else(|| "未知".to_string()),
            state: c.state.unwrap_or_default(),
            image: c.image.unwrap_or_default(),
            compose_project,
        }
    }
}
```

- [ ] **步骤 2：更新 `mod.rs` 导出**
```rust
// src-tauri/src/docker/mod.rs
pub mod models;
pub use models::*;
// 移除原有的结构体定义
```

- [ ] **步骤 3：Commit**
```bash
git add src-tauri/src/docker/models.rs src-tauri/src/docker/mod.rs
git commit -m "refactor(backend): move models and implement From traits"
```

### 任务 3：应用新架构重构 `list_local_containers`

**文件：**
- 修改：`src-tauri/src/docker/container.rs`

- [ ] **步骤 1：重构命令逻辑**
```rust
// src-tauri/src/docker/container.rs
use crate::error::AppResult;
use bollard::container::ListContainersOptions;

#[tauri::command]
pub async fn list_local_containers() -> AppResult<Vec<ContainerInfo>> {
    let docker = get_docker_client().await?;
    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?; // 使用 ? 自动转换错误

    Ok(containers.into_iter().map(Into::into).collect())
}
```

- [ ] **步骤 2：Commit**
```bash
git add src-tauri/src/docker/container.rs
git commit -m "refactor(backend): simplify list_local_containers with new architecture"
```

---

## 阶段二：前端 API 层与 Hook 建立

### 任务 4：建立前端 API 服务层

**文件：**
- 创建：`src/api/container.ts`
- 创建：`src/api/image.ts`

- [ ] **步骤 1：创建 `container.ts`**
```typescript
import { invoke } from '@tauri-apps/api/core'
import type { ContainerInfo } from '../store/container'

export const containerApi = {
  list: () => invoke<ContainerInfo[]>('list_local_containers'),
  start: (id: string) => invoke('start_container', { id }),
  stop: (id: string) => invoke('stop_container', { id }),
  // ... 其他容器相关接口
}
```

- [ ] **步骤 2：Commit**
```bash
git add src/api/container.ts src/api/image.ts
git commit -m "feat(frontend): create api service layer"
```

### 任务 5：实现 `useApi` 统一 Hook

**文件：**
- 创建：`src/hooks/useApi.ts`

- [ ] **步骤 1：编写 Hook 逻辑**
```typescript
import { ref } from 'vue'
import { useMessage } from 'naive-ui'

export function useApi<T, Args extends any[]>(
  apiFn: (...args: Args) => Promise<T>,
  options: {
    onSuccess?: (data: T) => void
    onError?: (err: any) => void
    successMsg?: string
  } = {}
) {
  const data = ref<T | null>(null)
  const loading = ref(false)
  const error = ref<any>(null)
  const message = useMessage()

  const execute = async (...args: Args) => {
    loading.value = true
    error.value = null
    try {
      const res = await apiFn(...args)
      data.value = res as any
      if (options.successMsg) message.success(options.successMsg)
      options.onSuccess?.(res)
      return res
    } catch (err) {
      error.value = err
      message.error(String(err))
      options.onError?.(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  return { data, loading, error, execute }
}
```

- [ ] **步骤 2：Commit**
```bash
git add src/hooks/useApi.ts
git commit -m "feat(frontend): add useApi hook"
```

---

## 阶段三：全栈流管理优化 (后续详述)

*(由于篇幅限制，阶段三和四将在执行过程中根据前两阶段结果细化)*
