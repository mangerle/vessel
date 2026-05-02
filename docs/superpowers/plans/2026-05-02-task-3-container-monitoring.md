# 容器管理增强与实时监控实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现容器的生命周期管理（启动、停止、重启、删除）以及实时的 CPU/内存监控。

**Architecture:** 后端通过 `bollard` 调用 Docker Engine API，并通过 Tauri Event 系统实现实时数据推送。前端使用 Pinia 管理状态，Vue-Echarts 展示监控数据。

**Tech Stack:** Rust (bollard), Vue 3, Pinia, Naive UI, ECharts, Vue-Echarts.

---

### Task 1: 安装前端依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装 echarts 和 vue-echarts**

Run: `npm install echarts vue-echarts`

- [ ] **Step 2: 验证安装**

检查 `package.json` 是否包含相关依赖。

### Task 2: 后端 - 实现容器控制 API

**Files:**
- Modify: `src-tauri/src/docker.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 `src-tauri/src/docker.rs` 中添加控制命令**

```rust
#[tauri::command]
pub async fn start_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.start_container::<String>(&id, None)
        .await
        .map_err(|e| format!("启动容器失败: {}", e))
}

#[tauri::command]
pub async fn stop_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.stop_container(&id, None)
        .await
        .map_err(|e| format!("停止容器失败: {}", e))
}

#[tauri::command]
pub async fn restart_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.restart_container(&id, None)
        .await
        .map_err(|e| format!("重启容器失败: {}", e))
}

#[tauri::command]
pub async fn remove_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.remove_container(&id, None)
        .await
        .map_err(|e| format!("删除容器失败: {}", e))
}
```

- [ ] **Step 2: 在 `src-tauri/src/lib.rs` 中注册新命令**

```rust
        .invoke_handler(tauri::generate_handler![
            greet,
            docker::list_local_containers,
            docker::start_container,
            docker::stop_container,
            docker::restart_container,
            docker::remove_container,
            // ... 现有命令
        ])
```

- [ ] **Step 3: 提交**

### Task 3: 后端 - 实现实时监控流

**Files:**
- Modify: `src-tauri/src/docker.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 `src-tauri/src/docker.rs` 中添加流式统计命令**

```rust
use bollard::container::StatsOptions;
use futures_util::stream::StreamExt;
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
pub async fn stream_container_stats(app: AppHandle, id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    let mut stream = docker.stats(&id, Some(StatsOptions {
        stream: true,
        one_shot: false,
    }));

    tauri::async_runtime::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(stats) => {
                    let event_name = format!("container-stats-{}", id);
                    if let Err(e) = app.emit(&event_name, stats) {
                        eprintln!("发送统计事件失败: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("获取统计数据失败: {}", e);
                    break;
                }
            }
        }
    });

    Ok(())
}
```

- [ ] **Step 2: 注册命令并提交**

### Task 4: 前端 - 完善容器 Store

**Files:**
- Modify: `src/store/container.ts`

- [ ] **Step 1: 更新 Store 添加控制方法**

```typescript
  actions: {
    // ... 现有 fetchContainers
    async startContainer(id: string) {
      await invoke('start_container', { id })
      await this.fetchContainers()
    },
    async stopContainer(id: string) {
      await invoke('stop_container', { id })
      await this.fetchContainers()
    },
    async restartContainer(id: string) {
      await invoke('restart_container', { id })
      await this.fetchContainers()
    },
    async removeContainer(id: string) {
      await invoke('remove_container', { id })
      await this.fetchContainers()
    }
  }
```

- [ ] **Step 2: 提交**

### Task 5: 前端 - 容器列表交互增强

**Files:**
- Modify: `src/views/Containers.vue`

- [ ] **Step 1: 添加操作按钮和统计信息**

更新 `columns` 定义，添加 "操作" 列，渲染启动/停止/重启/删除/详情按钮。

- [ ] **Step 2: 提交**

### Task 6: 前端 - 路由与详情页

**Files:**
- Create: `src/views/ContainerDetail.vue`
- Modify: `src/router/index.ts`

- [ ] **Step 1: 创建 `ContainerDetail.vue`**

集成 `vue-echarts`，使用 `listen` 监听 `container-stats-{id}` 事件，并更新图表。

- [ ] **Step 2: 更新路由配置**

添加 `/containers/:id` 路由。

- [ ] **Step 3: 最终验证**

Run: `npm run build` 确保没有编译错误。
Check: 运行应用并测试各项功能。

- [ ] **Step 4: 提交**
