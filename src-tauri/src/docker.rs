use crate::connection::get_docker_client;
use bollard::container::{ListContainersOptions, StatsOptions, LogsOptions};
use bollard::exec::{CreateExecOptions, StartExecResults, ResizeExecOptions};
use bollard::image::{ListImagesOptions, CreateImageOptions};
use serde::Serialize;
use futures_util::stream::StreamExt;
use tauri::{AppHandle, Emitter};
use std::collections::HashMap;
use tokio::sync::mpsc;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncWriteExt;

/// 容器信息结构体
#[derive(Serialize)]
pub struct ContainerInfo {
    /// 容器 ID
    pub id: String,
    /// 容器名称
    pub name: String,
    /// 容器状态 (如: running, exited)
    pub state: String,
    /// 镜像名称
    pub image: String,
    /// 归属的 Compose 项目名
    pub compose_project: Option<String>,
}

/// 镜像信息结构体
#[derive(Serialize)]
pub struct ImageInfo {
    /// 镜像 ID
    pub id: String,
    /// 镜像标签列表
    pub tags: Vec<String>,
    /// 镜像大小 (字节)
    pub size: i64,
    /// 创建时间 (时间戳)
    pub created: i64,
}

/// 网络信息结构体
#[derive(Serialize)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub created: String,
}

/// 卷信息结构体
#[derive(Serialize)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created: String,
}

/// Compose 项目结构体
#[derive(Serialize)]
pub struct ComposeProject {
    pub name: String,
    pub container_count: usize,
    pub running_count: usize,
    pub status: String,
}

/// 端口映射结构体
#[derive(Serialize)]
pub struct PortMapping {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub type_: String,
    pub ip: Option<String>,
}

/// 挂载信息结构体
#[derive(Serialize)]
pub struct MountInfo {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
}

/// 容器详情结构体
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

/// 获取本地 Docker 容器列表的命令
#[tauri::command]
pub async fn list_local_containers() -> Result<Vec<ContainerInfo>, String> {
    // 使用自动探测的驱动连接 Docker
    let docker = get_docker_client().await?;

    // 列出所有容器 (包括未运行的)
    let containers = docker.list_containers(Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    })).await.map_err(|e| format!("无法获取容器列表: {}", e))?;
    
    // 转换为前端友好的格式
    Ok(containers.into_iter().map(|c| {
        let compose_project = c.labels.as_ref().and_then(|labels| labels.get("com.docker.compose.project").cloned());
        ContainerInfo {
            id: c.id.unwrap_or_default(),
            // c.names 通常以 ["/container_name"] 格式返回，所以我们取第一个并去掉开头的斜杠
            name: c.names.as_ref()
                .and_then(|names| names.first())
                .map(|name| name.trim_start_matches('/').to_string())
                .unwrap_or_else(|| "未知".to_string()),
            state: c.state.unwrap_or_default(),
            image: c.image.unwrap_or_default(),
            compose_project,
        }
    }).collect())
}

/// 获取本地 Docker 镜像列表的命令
#[tauri::command]
pub async fn list_images() -> Result<Vec<ImageInfo>, String> {
    let docker = get_docker_client().await?;

    let images = docker.list_images(Some(ListImagesOptions::<String> {
        all: false,
        ..Default::default()
    })).await.map_err(|e| format!("无法获取镜像列表: {}", e))?;

    Ok(images.into_iter().map(|img| ImageInfo {
        id: img.id,
        tags: img.repo_tags,
        size: img.size,
        created: img.created,
    }).collect())
}

/// 删除镜像
#[tauri::command]
pub async fn remove_image(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.remove_image(&id, None, None)
        .await
        .map_err(|e| format!("删除镜像失败: {}", e))?;
    Ok(())
}

/// 拉取镜像
#[tauri::command]
pub async fn pull_image(app: AppHandle, image_name: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    
    let mut stream = docker.create_image(
        Some(CreateImageOptions {
            from_image: image_name.clone(),
            ..Default::default()
        }),
        None,
        None
    );

    tauri::async_runtime::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(info) => {
                    // 发送拉取进度到前端
                    if let Err(e) = app.emit("image-pull-progress", info) {
                        eprintln!("发送拉取进度事件失败: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("拉取镜像出错: {}", e);
                    let _ = app.emit("image-pull-error", e.to_string());
                    break;
                }
            }
        }
        // 拉取完成
        let _ = app.emit("image-pull-finished", image_name);
    });

    Ok(())
}

/// 启动容器
#[tauri::command]
pub async fn start_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.start_container::<String>(&id, None)
        .await
        .map_err(|e| format!("启动容器失败: {}", e))
}

/// 停止容器
#[tauri::command]
pub async fn stop_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.stop_container(&id, None)
        .await
        .map_err(|e| format!("停止容器失败: {}", e))
}

/// 重启容器
#[tauri::command]
pub async fn restart_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.restart_container(&id, None)
        .await
        .map_err(|e| format!("重启容器失败: {}", e))
}

/// 获取容器详情
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
        image: config.and_then(|c| c.image.clone()).unwrap_or_default(),
        image_id: details.image.unwrap_or_default(),
        state: details.state.as_ref().and_then(|s| s.status).map(|s| format!("{:?}", s)).unwrap_or_default(),
        status: details.state.as_ref().and_then(|s| s.status).map(|s| format!("{:?}", s)).unwrap_or_default(),
        created: details.created.unwrap_or_default(),
        env: config.and_then(|c| c.env.clone()).unwrap_or_default(),
        ports,
        mounts,
    })
}

/// 删除容器
#[tauri::command]
pub async fn remove_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.remove_container(&id, None)
        .await
        .map_err(|e| format!("删除容器失败: {}", e))
}

/// 实时流式传输容器统计信息
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
                    // 发送事件到前端，事件名为 container-stats-<id>
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

/// 获取容器日志
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

/// 获取 Compose 项目列表
#[tauri::command]
pub async fn list_compose_projects() -> Result<Vec<ComposeProject>, String> {
    let docker = get_docker_client().await?;
    let containers = docker.list_containers(Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    })).await.map_err(|e| format!("无法获取容器列表: {}", e))?;

    let mut projects_map: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();

    for container in containers {
        if let Some(labels) = container.labels {
            if let Some(project_name) = labels.get("com.docker.compose.project") {
                let counts = projects_map.entry(project_name.clone()).or_insert((0, 0));
                counts.0 += 1; // 总数
                if container.state.as_deref() == Some("running") {
                    counts.1 += 1; // 运行中
                }
            }
        }
    }

    let projects = projects_map.into_iter().map(|(name, (total, running))| ComposeProject {
        name,
        container_count: total,
        running_count: running,
        status: if running > 0 { "running".to_string() } else { "exited".to_string() },
    }).collect();

    Ok(projects)
}

/// 获取网络列表
#[tauri::command]
pub async fn list_networks() -> Result<Vec<NetworkInfo>, String> {
    let docker = get_docker_client().await?;
    let networks = docker.list_networks::<String>(None).await.map_err(|e| format!("无法获取网络列表: {}", e))?;

    Ok(networks.into_iter().map(|n| NetworkInfo {
        id: n.id.unwrap_or_default(),
        name: n.name.unwrap_or_default(),
        driver: n.driver.unwrap_or_default(),
        scope: n.scope.unwrap_or_default(),
        created: n.created.unwrap_or_default(),
    }).collect())
}

/// 删除网络
#[tauri::command]
pub async fn remove_network(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.remove_network(&id).await.map_err(|e| format!("删除网络失败: {}", e))
}

/// 清理未使用的网络
#[tauri::command]
pub async fn prune_networks() -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.prune_networks::<String>(None).await.map_err(|e| format!("清理网络失败: {}", e))?;
    Ok(())
}

/// 获取卷列表
#[tauri::command]
pub async fn list_volumes() -> Result<Vec<VolumeInfo>, String> {
    let docker = get_docker_client().await?;
    let response = docker.list_volumes::<String>(None).await.map_err(|e| format!("无法获取卷列表: {}", e))?;

    let volumes = response.volumes.unwrap_or_default();
    Ok(volumes.into_iter().map(|v| VolumeInfo {
        name: v.name,
        driver: v.driver,
        mountpoint: v.mountpoint,
        created: v.created_at.unwrap_or_default(),
    }).collect())
}

/// 删除卷
#[tauri::command]
pub async fn remove_volume(name: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.remove_volume(&name, None).await.map_err(|e| format!("删除卷失败: {}", e))
}

/// 清理未使用的卷
#[tauri::command]
pub async fn prune_volumes() -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.prune_volumes::<String>(None).await.map_err(|e| format!("清理卷失败: {}", e))?;
    Ok(())
}

/// 终端会话结构体
pub struct TerminalSession {
    pub stdin_tx: mpsc::Sender<Vec<u8>>,
}

/// 全局终端会话管理器
static TERMINAL_SESSIONS: Lazy<Arc<Mutex<HashMap<String, TerminalSession>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// 创建容器终端会话
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
        cmd: Some(vec!["/bin/sh".to_string()]), // 默认使用 sh，前端可以根据需要修改
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
                    while let Some(msg) = output.next().await {
                        match msg {
                            Ok(log_output) => {
                                let data = log_output.into_bytes();
                                let event_name = format!("container-terminal-stdout-{}", exec_id);
                                if app.emit(&event_name, data.to_vec()).is_err() {
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
                    if input.write_all(&data).await.is_err() {
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

/// 向终端写入数据
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

/// 调整终端大小
#[tauri::command]
pub async fn resize_container_terminal(exec_id: String, height: u16, width: u16) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker.resize_exec(&exec_id, ResizeExecOptions {
        height,
        width,
    }).await.map_err(|e| format!("调整终端大小失败: {}", e))
}
