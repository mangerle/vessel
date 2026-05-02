# Docker Backend Enhancement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement backend commands for container inspection, streaming logs, and interactive terminal.

**Architecture:** Use `bollard` for Docker interaction, Tauri events for streaming data, and a global `HashMap` to manage terminal sessions.

**Tech Stack:** Rust, Tauri, Bollard, Tokio Channels.

---

### Task 1: 容器详情 Inspect 命令实现

**Files:**
- Modify: `src-tauri/src/docker.rs`

- [ ] **Step 1: 定义数据结构**

在 `docker.rs` 中添加 `ContainerDetails`, `PortMapping`, `MountInfo` 结构体。

```rust
#[derive(Serialize)]
pub struct PortMapping {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub type_: String,
    pub ip: Option<String>,
}

#[derive(Serialize)]
pub struct MountInfo {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
}

#[derive(Serialize)]
pub struct ContainerDetails {
    pub id: String,
    pub name: String,
    pub image: String,
    pub image_id: String,
    pub state: String,
    pub status: String,
    pub created: String,
    pub env: Vec<String>,
    pub ports: Vec<PortMapping>,
    pub mounts: Vec<MountInfo>,
}
```

- [ ] **Step 2: 实现 `inspect_container` 函数**

```rust
#[tauri::command]
pub async fn inspect_container(id: String) -> Result<ContainerDetails, String> {
    let docker = get_docker_client().await?;
    let details = docker.inspect_container(&id, None)
        .await
        .map_err(|e| format!("获取容器详情失败: {}", e))?;

    let config = details.config.as_ref();
    let network_settings = details.network_settings.as_ref();

    let ports = network_settings
        .and_then(|ns| ns.ports.as_ref())
        .map(|p| {
            p.iter()
                .flat_map(|(k, v)| {
                    let parts: Vec<&str> = k.split('/').collect();
                    let private_port = parts[0].parse::<u16>().unwrap_or_default();
                    let type_ = parts.get(1).unwrap_or(&"tcp").to_string();

                    match v {
                        Some(bindings) => bindings.iter().map(move |b| PortMapping {
                            private_port,
                            public_port: b.host_port.as_ref().and_then(|hp| hp.parse::<u16>().ok()),
                            type_: type_.clone(),
                            ip: b.host_ip.clone(),
                        }).collect::<Vec<_>>(),
                        None => vec![PortMapping {
                            private_port,
                            public_port: None,
                            type_,
                            ip: None,
                        }],
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let mounts = details.mounts.as_ref().map(|m| {
        m.iter().map(|mi| MountInfo {
            source: mi.source.clone().unwrap_or_default(),
            destination: mi.destination.clone().unwrap_or_default(),
            mode: mi.mode.clone().unwrap_or_default(),
            rw: mi.rw.unwrap_or_default(),
        }).collect()
    }).unwrap_or_default();

    Ok(ContainerDetails {
        id: details.id.unwrap_or_default(),
        name: details.name.unwrap_or_default().trim_start_matches('/').to_string(),
        image: details.config.and_then(|c| c.image).unwrap_or_default(),
        image_id: details.image.unwrap_or_default(),
        state: details.state.and_then(|s| s.status).map(|s| format!("{:?}", s)).unwrap_or_default(),
        status: details.state.and_then(|s| s.status).map(|s| format!("{:?}", s)).unwrap_or_default(), // Simplified
        created: details.created.unwrap_or_default(),
        env: config.and_then(|c| c.env.clone()).unwrap_or_default(),
        ports,
        mounts,
    })
}
```

- [ ] **Step 3: 运行 `cargo check` 验证**

### Task 2: 容器日志流式获取实现

**Files:**
- Modify: `src-tauri/src/docker.rs`

- [ ] **Step 1: 实现 `stream_container_logs` 函数**

```rust
use bollard::container::LogsOptions;

#[tauri::command]
pub async fn stream_container_logs(app: AppHandle, id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    let mut stream = docker.logs(&id, Some(LogsOptions {
        follow: true,
        stdout: true,
        stderr: true,
        tail: "100",
        timestamps: true,
        ..Default::default()
    }));

    tauri::async_runtime::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(log) => {
                    let event_name = format!("container-logs-{}", id);
                    if let Err(e) = app.emit(&event_name, log.to_string()) {
                        eprintln!("发送日志事件失败: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("获取日志流出错: {}", e);
                    break;
                }
            }
        }
    });

    Ok(())
}
```

- [ ] **Step 2: 运行 `cargo check` 验证**

### Task 3: 容器终端 Exec 框架实现

**Files:**
- Modify: `src-tauri/src/docker.rs`

- [ ] **Step 1: 定义会话状态管理器**

```rust
use std::collections::HashMap;
use tokio::sync::mpsc;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TerminalSession {
    pub stdin_tx: mpsc::Sender<Vec<u8>>,
}

static TERMINAL_SESSIONS: Lazy<Arc<Mutex<HashMap<String, TerminalSession>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
```

- [ ] **Step 2: 实现 `create_container_terminal` 函数**

```rust
use bollard::container::{CreateExecOptions, StartExecOptions, StartExecResults};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tauri::command]
pub async fn create_container_terminal(app: AppHandle, id: String, user: Option<String>) -> Result<String, String> {
    let docker = get_docker_client().await?;
    
    // 1. 创建 Exec
    let exec = docker.create_exec(&id, CreateExecOptions {
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        tty: Some(true),
        user,
        cmd: Some(vec!["/bin/sh"]), // 优先尝试 sh
        ..Default::default()
    }).await.map_err(|e| format!("创建终端失败: {}", e))?;

    let exec_id = exec.id;
    let exec_id_clone = exec_id.clone();
    let app_clone = app.clone();

    // 2. 启动 Exec
    let start_result = docker.start_exec(&exec_id, None).await.map_err(|e| format!("启动终端失败: {}", e))?;

    if let StartExecResults::Attached { mut output, mut input } = start_result {
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<Vec<u8>>(100);

        // 存储会话
        {
            let mut sessions = TERMINAL_SESSIONS.lock().await;
            sessions.insert(exec_id.clone(), TerminalSession { stdin_tx });
        }

        // 3. 处理 IO
        tauri::async_runtime::spawn(async move {
            let mut stdout_task = tauri::async_runtime::spawn({
                let app = app_clone.clone();
                let exec_id = exec_id.clone();
                async move {
                    let mut buf = [0u8; 1024];
                    loop {
                        match output.read(&mut buf).await {
                            Ok(0) => break,
                            Ok(n) => {
                                let event_name = format!("container-terminal-stdout-{}", exec_id);
                                if let Err(_) = app.emit(&event_name, buf[..n].to_vec()) {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
            });

            let mut stdin_task = tauri::async_runtime::spawn(async move {
                while let Some(data) = stdin_rx.recv().await {
                    if let Err(_) = input.write_all(&data).await {
                        break;
                    }
                }
            });

            // 等待其中一个任务结束
            tokio::select! {
                _ = &mut stdout_task => {},
                _ = &mut stdin_task => {},
            };

            // 清理会话
            let mut sessions = TERMINAL_SESSIONS.lock().await;
            sessions.remove(&exec_id);
            let _ = app_clone.emit(&format!("container-terminal-exit-{}", exec_id), ());
        });

        Ok(exec_id_clone)
    } else {
        Err("未能连接到终端流".to_string())
    }
}
```

- [ ] **Step 3: 实现交互命令**

```rust
#[tauri::command]
pub async fn write_to_terminal(exec_id: String, data: Vec<u8>) -> Result<(), String> {
    let sessions = TERMINAL_SESSIONS.lock().await;
    if let Some(session) = sessions.get(&exec_id) {
        session.stdin_tx.send(data).await.map_err(|e| format!("写入终端失败: {}", e))?;
        Ok(())
    } else {
        Err("会话不存在".to_string())
    }
}

#[tauri::command]
pub async fn resize_container_terminal(exec_id: String, height: u16, width: u16) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.resize_exec(&exec_id, bollard::container::ResizeExecOptions {
        height,
        width,
    }).await.map_err(|e| format!("调整终端大小失败: {}", e))
}
```

- [ ] **Step 4: 运行 `cargo check` 验证**

### Task 4: 注册命令与最终验证

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 `lib.rs` 中注册新命令**

```rust
        .invoke_handler(tauri::generate_handler![
            // ... 现有命令
            docker::inspect_container,
            docker::stream_container_logs,
            docker::create_container_terminal,
            docker::write_to_terminal,
            docker::resize_container_terminal,
        ])
```

- [ ] **Step 2: 运行 `cargo clippy` 确保代码质量**

Run: `cargo clippy -- -D warnings`
Expected: 没有任何警告。

- [ ] **Step 3: 提交更改**

```bash
git add src-tauri/src/docker.rs src-tauri/src/lib.rs
git commit -m "feat: implement container inspect, logs streaming and interactive terminal"
```
