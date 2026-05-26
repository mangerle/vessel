use crate::connection::get_docker_client;
use bollard::container::{ListContainersOptions, LogsOptions, StatsOptions};
use bollard::exec::{CreateExecOptions, ResizeExecOptions, StartExecOptions, StartExecResults};
use bollard::image::{CreateImageOptions, ListImagesOptions};
use bollard::network::InspectNetworkOptions;
use futures_util::stream::StreamExt;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

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

/// 镜像搜索结果结构体
#[derive(Serialize)]
pub struct ImageSearchResult {
    pub name: String,
    pub description: String,
    pub is_official: bool,
    pub star_count: i64,
}

/// 镜像历史信息结构体
#[derive(Serialize)]
pub struct ImageHistoryInfo {
    pub id: String,
    pub created: i64,
    pub created_by: String,
    pub size: i64,
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

/// 已连接的容器信息
#[derive(Serialize)]
pub struct ConnectedContainer {
    pub id: String,
    pub name: String,
    pub ipv4_address: String,
    pub ipv6_address: String,
    pub mac_address: String,
}

/// 网络详情结构体
#[derive(Serialize)]
pub struct NetworkDetails {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub created: String,
    pub internal: bool,
    pub attachable: bool,
    pub ingress: bool,
    pub subnet: String,
    pub gateway: String,
    pub containers: Vec<ConnectedContainer>,
    pub options: HashMap<String, String>,
    pub labels: HashMap<String, String>,
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
    pub working_dir: Option<String>,
    pub config_file: Option<String>,
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

/// 卷使用容器信息
#[derive(Serialize)]
pub struct VolumeUser {
    pub container_id: String,
    pub container_name: String,
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

/// 镜像详情结构体
#[derive(Serialize)]
pub struct ImageDetails {
    pub id: String,
    pub tags: Vec<String>,
    pub size: i64,
    pub created: String,
    pub architecture: String,
    pub os: String,
    pub env: Vec<String>,
    pub exposed_ports: Vec<String>,
    pub cmd: Vec<String>,
    pub entrypoint: Vec<String>,
}

/// 获取本地 Docker 容器列表的命令
#[tauri::command]
pub async fn list_local_containers() -> Result<Vec<ContainerInfo>, String> {
    // 使用自动探测的驱动连接 Docker
    let docker = get_docker_client().await?;

    // 列出所有容器 (包括未运行的)
    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
        .map_err(|e| format!("无法获取容器列表: {}", e))?;

    // 转换为前端友好的格式
    Ok(containers
        .into_iter()
        .map(|c| {
            let compose_project = c
                .labels
                .as_ref()
                .and_then(|labels| labels.get("com.docker.compose.project").cloned());
            ContainerInfo {
                id: c.id.unwrap_or_default(),
                // c.names 通常以 ["/container_name"] 格式返回，所以我们取第一个并去掉开头的斜杠
                name: c
                    .names
                    .as_ref()
                    .and_then(|names| names.first())
                    .map(|name| name.trim_start_matches('/').to_string())
                    .unwrap_or_else(|| "未知".to_string()),
                state: c.state.unwrap_or_default(),
                image: c.image.unwrap_or_default(),
                compose_project,
            }
        })
        .collect())
}

/// 获取本地 Docker 镜像列表的命令
#[tauri::command]
pub async fn list_images() -> Result<Vec<ImageInfo>, String> {
    let docker = get_docker_client().await?;

    let images = docker
        .list_images(Some(ListImagesOptions::<String> {
            all: false,
            ..Default::default()
        }))
        .await
        .map_err(|e| format!("无法获取镜像列表: {}", e))?;

    Ok(images
        .into_iter()
        .map(|img| ImageInfo {
            id: img.id,
            tags: img.repo_tags,
            size: img.size,
            created: img.created,
        })
        .collect())
}

/// 获取镜像详情
#[tauri::command]
pub async fn inspect_image(id: String) -> Result<ImageDetails, String> {
    let docker = get_docker_client().await?;
    let details = docker
        .inspect_image(&id)
        .await
        .map_err(|e| format!("获取镜像详情失败: {}", e))?;

    let config = details.config.as_ref();

    Ok(ImageDetails {
        id: details.id.unwrap_or_default(),
        tags: details.repo_tags.unwrap_or_default(),
        size: details.size.unwrap_or_default(),
        created: details.created.unwrap_or_default(),
        architecture: details.architecture.unwrap_or_default(),
        os: details.os.unwrap_or_default(),
        env: config.and_then(|c| c.env.clone()).unwrap_or_default(),
        exposed_ports: config
            .and_then(|c| c.exposed_ports.as_ref())
            .map(|p| p.keys().cloned().collect())
            .unwrap_or_default(),
        cmd: config.and_then(|c| c.cmd.clone()).unwrap_or_default(),
        entrypoint: config.and_then(|c| c.entrypoint.clone()).unwrap_or_default(),
    })
}

/// 删除镜像
#[tauri::command]
pub async fn remove_image(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .remove_image(&id, None, None)
        .await
        .map_err(|e| format!("删除镜像失败: {}", e))?;
    Ok(())
}

/// 搜索镜像
#[tauri::command]
pub async fn search_images(term: String) -> Result<Vec<ImageSearchResult>, String> {
    let docker = get_docker_client().await?;
    let results = docker
        .search_images(bollard::image::SearchImagesOptions {
            term,
            limit: None,
            filters: HashMap::new(),
        })
        .await
        .map_err(|e| format!("搜索镜像失败: {}", e))?;

    Ok(results
        .into_iter()
        .map(|item| ImageSearchResult {
            name: item.name.unwrap_or_default(),
            description: item.description.unwrap_or_default(),
            is_official: item.is_official.unwrap_or_default(),
            star_count: item.star_count.unwrap_or_default(),
        })
        .collect())
}

/// 获取镜像历史
#[tauri::command]
pub async fn get_image_history(id: String) -> Result<Vec<ImageHistoryInfo>, String> {
    let docker = get_docker_client().await?;
    let history = docker
        .image_history(&id)
        .await
        .map_err(|e| format!("获取镜像历史失败: {}", e))?;

    Ok(history
        .into_iter()
        .map(|item| ImageHistoryInfo {
            id: item.id,
            created: item.created,
            created_by: item.created_by,
            size: item.size,
        })
        .collect())
}

/// 拉取镜像
#[tauri::command]
pub async fn pull_image(app: AppHandle, image_name: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    
    // 确保镜像名包含标签，默认为 latest
    let full_image_name = if image_name.contains(':') {
        image_name.clone()
    } else {
        format!("{}:latest", image_name)
    };

    println!("开始拉取镜像: {}", full_image_name);

    let mut stream = docker.create_image(
        Some(CreateImageOptions {
            from_image: full_image_name.clone(),
            ..Default::default()
        }),
        None,
        None,
    );

    let app_handle = app.clone();
    let name_for_events = full_image_name.clone();
    
    tauri::async_runtime::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(info) => {
                    // 发送拉取进度到前端，包含镜像名以便过滤
                    #[derive(Serialize, Clone)]
                    struct ProgressPayload {
                        image: String,
                        info: bollard::models::CreateImageInfo,
                    }
                    if let Err(e) = app_handle.emit("image-pull-progress", ProgressPayload {
                        image: name_for_events.clone(),
                        info,
                    }) {
                        eprintln!("发送拉取进度事件失败: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("拉取镜像 {} 出错: {}", name_for_events, e);
                    #[derive(Serialize, Clone)]
                    struct ErrorPayload {
                        image: String,
                        error: String,
                    }
                    let _ = app_handle.emit("image-pull-error", ErrorPayload {
                        image: name_for_events.clone(),
                        error: e.to_string(),
                    });
                    break;
                }
            }
        }
        // 拉取完成
        println!("镜像拉取任务结束: {}", name_for_events);
        let _ = app_handle.emit("image-pull-finished", name_for_events);
    });

    Ok(())
}

/// 启动容器
#[tauri::command]
pub async fn start_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .start_container::<String>(&id, None)
        .await
        .map_err(|e| format!("启动容器失败: {}", e))
}

/// 停止容器
#[tauri::command]
pub async fn stop_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .stop_container(&id, None)
        .await
        .map_err(|e| format!("停止容器失败: {}", e))
}

/// 重启容器
#[tauri::command]
pub async fn restart_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .restart_container(&id, None)
        .await
        .map_err(|e| format!("重启容器失败: {}", e))
}

/// 获取容器详情
#[tauri::command]
pub async fn inspect_container(id: String) -> Result<ContainerDetails, String> {
    let docker = get_docker_client().await?;
    let details = docker
        .inspect_container(&id, None)
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
                        Some(bindings) => bindings
                            .iter()
                            .map(move |b| PortMapping {
                                private_port,
                                public_port: b
                                    .host_port
                                    .as_ref()
                                    .and_then(|hp| hp.parse::<u16>().ok()),
                                type_: type_.clone(),
                                ip: b.host_ip.clone(),
                            })
                            .collect::<Vec<_>>(),
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

    let mounts = details
        .mounts
        .as_ref()
        .map(|m| {
            m.iter()
                .map(|mi| MountInfo {
                    source: mi.source.clone().unwrap_or_default(),
                    destination: mi.destination.clone().unwrap_or_default(),
                    mode: mi.mode.clone().unwrap_or_default(),
                    rw: mi.rw.unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(ContainerDetails {
        id: details.id.unwrap_or_default(),
        name: details
            .name
            .unwrap_or_default()
            .trim_start_matches('/')
            .to_string(),
        image: config.and_then(|c| c.image.clone()).unwrap_or_default(),
        image_id: details.image.unwrap_or_default(),
        state: details
            .state
            .as_ref()
            .and_then(|s| s.status)
            .map(|s| format!("{:?}", s).to_lowercase())
            .unwrap_or_default(),
        status: details
            .state
            .as_ref()
            .and_then(|s| s.status)
            .map(|s| format!("{:?}", s).to_lowercase())
            .unwrap_or_default(),
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
    docker
        .remove_container(&id, None)
        .await
        .map_err(|e| format!("删除容器失败: {}", e))
}

/// 实时流式传输容器统计信息
#[tauri::command]
pub async fn stream_container_stats(app: AppHandle, id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    let mut stream = docker.stats(
        &id,
        Some(StatsOptions {
            stream: true,
            one_shot: false,
        }),
    );

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
    let mut stream = docker.logs(
        &id,
        Some(LogsOptions {
            follow: true,
            stdout: true,
            stderr: true,
            tail: "100",
            timestamps: true,
            ..Default::default()
        }),
    );

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
    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
        .map_err(|e| format!("无法获取容器列表: {}", e))?;

    struct ProjectData {
        total: usize,
        running: usize,
        working_dir: Option<String>,
        config_file: Option<String>,
    }

    let mut projects_map: HashMap<String, ProjectData> =
        HashMap::new();

    for container in containers {
        if let Some(labels) = container.labels {
            if let Some(project_name) = labels.get("com.docker.compose.project") {
                let data = projects_map
                    .entry(project_name.clone())
                    .or_insert(ProjectData {
                        total: 0,
                        running: 0,
                        working_dir: labels
                            .get("com.docker.compose.project.working_dir")
                            .cloned(),
                        config_file: labels
                            .get("com.docker.compose.project.config_files")
                            .cloned(),
                    });

                data.total += 1;
                if container.state.as_deref() == Some("running") {
                    data.running += 1;
                }
            }
        }
    }

    let projects = projects_map
        .into_iter()
        .map(|(name, data)| ComposeProject {
            name,
            container_count: data.total,
            running_count: data.running,
            status: if data.running > 0 {
                "running".to_string()
            } else {
                "exited".to_string()
            },
            working_dir: data.working_dir,
            config_file: data.config_file,
        })
        .collect();

    Ok(projects)
}

/// 获取网络列表
#[tauri::command]
pub async fn list_networks() -> Result<Vec<NetworkInfo>, String> {
    let docker = get_docker_client().await?;
    let networks = docker
        .list_networks::<String>(None)
        .await
        .map_err(|e| format!("无法获取网络列表: {}", e))?;

    Ok(networks
        .into_iter()
        .map(|n| NetworkInfo {
            id: n.id.unwrap_or_default(),
            name: n.name.unwrap_or_default(),
            driver: n.driver.unwrap_or_default(),
            scope: n.scope.unwrap_or_default(),
            created: n.created.unwrap_or_default(),
        })
        .collect())
}

/// 删除网络
#[tauri::command]
pub async fn remove_network(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .remove_network(&id)
        .await
        .map_err(|e| format!("删除网络失败: {}", e))
}

/// 清理未使用的网络
#[tauri::command]
pub async fn prune_networks() -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .prune_networks::<String>(None)
        .await
        .map_err(|e| format!("清理网络失败: {}", e))?;
    Ok(())
}

/// 断开网络连接
#[tauri::command]
pub async fn disconnect_network(network_id: String, container_id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .disconnect_network(
            &network_id,
            bollard::network::DisconnectNetworkOptions {
                container: container_id,
                force: false,
            },
        )
        .await
        .map_err(|e| format!("断开网络连接失败: {}", e))
}

/// 获取卷列表
#[tauri::command]
pub async fn list_volumes() -> Result<Vec<VolumeInfo>, String> {
    let docker = get_docker_client().await?;
    let response = docker
        .list_volumes::<String>(None)
        .await
        .map_err(|e| format!("无法获取卷列表: {}", e))?;

    let volumes = response.volumes.unwrap_or_default();
    Ok(volumes
        .into_iter()
        .map(|v| VolumeInfo {
            name: v.name,
            driver: v.driver,
            mountpoint: v.mountpoint,
            created: v.created_at.unwrap_or_default(),
        })
        .collect())
}

/// 删除卷
#[tauri::command]
pub async fn remove_volume(name: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .remove_volume(&name, None)
        .await
        .map_err(|e| format!("删除卷失败: {}", e))
}

/// 清理未使用的卷
#[tauri::command]
pub async fn prune_volumes() -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .prune_volumes::<String>(None)
        .await
        .map_err(|e| format!("清理卷失败: {}", e))?;
    Ok(())
}

/// 获取使用特定卷的容器列表
#[tauri::command]
pub async fn list_volume_containers(name: String) -> Result<Vec<VolumeUser>, String> {
    let docker = get_docker_client().await?;
    
    // 获取所有容器
    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
        .map_err(|e| format!("无法获取容器列表: {}", e))?;
    
    let mut users = Vec::new();
    
    for container in containers {
        // 对于每个容器，获取其详细信息以检查挂载点
        if let Some(id) = container.id {
            let details = docker
                .inspect_container(&id, None)
                .await
                .map_err(|e| format!("无法获取容器详情 ({}): {}", id, e))?;
            
            if let Some(mounts) = details.mounts {
                for mount in mounts {
                    // 检查挂载是否匹配指定的卷名
                    if mount.name.as_deref() == Some(&name) || mount.source.as_deref() == Some(&name) {
                        users.push(VolumeUser {
                            container_id: id.clone(),
                            container_name: details.name.clone().unwrap_or_default().trim_start_matches('/').to_string(),
                            source: mount.source.unwrap_or_default(),
                            destination: mount.destination.unwrap_or_default(),
                            mode: mount.mode.unwrap_or_default(),
                            rw: mount.rw.unwrap_or_default(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(users)
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
pub async fn create_container_terminal(
    app: AppHandle,
    id: String,
    user: Option<String>,
) -> Result<String, String> {
    let docker = get_docker_client().await?;

    // 1. 创建 Exec
    let exec = docker
        .create_exec(
            &id,
            CreateExecOptions {
                attach_stdin: Some(true),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                tty: Some(true),
                user,
                cmd: Some(vec!["sh".to_string()]), // 默认使用 sh，前端可以根据需要修改
                ..Default::default()
            },
        )
        .await
        .map_err(|e| format!("创建终端失败: {}", e))?;

    let exec_id = exec.id;
    let exec_id_clone = exec_id.clone();
    let app_clone = app.clone();

    // 2. 启动 Exec
    let start_result = docker
        .start_exec(
            &exec_id,
            Some(StartExecOptions {
                detach: false,
                tty: true,
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| format!("启动终端失败: {}", e))?;

    if let StartExecResults::Attached {
        mut output,
        mut input,
    } = start_result
    {
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
                    if input.flush().await.is_err() {
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
        session
            .stdin_tx
            .send(data)
            .await
            .map_err(|e| format!("写入终端失败: {}", e))?;
        Ok(())
    } else {
        Err("会话不存在".to_string())
    }
}

/// 调整终端大小
#[tauri::command]
pub async fn resize_container_terminal(
    exec_id: String,
    height: u16,
    width: u16,
) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .resize_exec(
            &exec_id,
            ResizeExecOptions {
                height,
                width,
            },
        )
        .await
        .map_err(|e| format!("调整终端大小失败: {}", e))
}

#[tauri::command]
pub async fn read_compose_file(
    path: String,
    mode: String,
    distro: Option<String>,
) -> Result<String, String> {
    if mode == "wsl" {
        let mut cmd = tokio::process::Command::new("wsl");
        if let Some(d) = distro {
            if !d.is_empty() {
                cmd.args(["-d", &d]);
            }
        }
        cmd.args(["-u", "root", "--", "cat", &path]);
        
        #[cfg(windows)]
        cmd.creation_flags(0x08000000);

        let out = cmd.output().await.map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    } else {
        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn write_compose_file(
    path: String, 
    content: String,
    mode: String,
    distro: Option<String>,
) -> Result<(), String> {
    if mode == "wsl" {
        let mut cmd = tokio::process::Command::new("wsl");
        if let Some(d) = distro {
            if !d.is_empty() {
                cmd.args(["-d", &d]);
            }
        }
        // 使用 HEREDOC 写入内容，避免复杂的转义
        let shell_cmd = format!("cat << 'EOF' > \"{}\"\n{}\nEOF", path, content);
        cmd.args(["-u", "root", "--", "sh", "-c", &shell_cmd]);

        #[cfg(windows)]
        cmd.creation_flags(0x08000000);

        let out = cmd.output().await.map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    } else {
        tokio::fs::write(path, content)
            .await
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn run_compose_command(
    app: AppHandle,
    project_dir: String,
    args: Vec<String>,
    mode: String,
    distro: Option<String>,
) -> Result<(), String> {
    let mut cmd = if mode == "wsl" {
        let mut c = tokio::process::Command::new("wsl");
        if let Some(d) = distro {
            if !d.is_empty() {
                c.args(["-d", &d]);
            }
        }
        let args_str = args.join(" ");
        // 进入目录并执行 compose
        c.args(["sh", "-c", &format!("cd \"{}\" && docker compose {}", project_dir, args_str)]);
        c
    } else {
        let mut c = tokio::process::Command::new("docker");
        c.arg("compose")
            .args(args)
            .current_dir(project_dir);
        c
    };

    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start docker compose: {}", e))?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let app_clone = app.clone();
    // Handle stdout
    tauri::async_runtime::spawn(async move {
        let reader = tokio::io::BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone.emit("compose-cmd-output", line);
        }
    });

    let app_clone_err = app.clone();
    // Handle stderr
    tauri::async_runtime::spawn(async move {
        let reader = tokio::io::BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone_err.emit("compose-cmd-output", line);
        }
    });

    let app_clone_finish = app.clone();
    // Wait for process to exit
    tauri::async_runtime::spawn(async move {
        match child.wait().await {
            Ok(status) => {
                if status.success() {
                    let _ = app_clone_finish.emit("compose-cmd-finished", ());
                } else {
                    let _ = app_clone_finish.emit("compose-cmd-error", format!("Process exited with status: {}", status));
                }
            }
            Err(e) => {
                let _ = app_clone_finish.emit("compose-cmd-error", format!("Failed to wait for process: {}", e));
            }
        }
    });

    Ok(())
}

/// 获取网络详情
#[tauri::command]
pub async fn get_network_details(id: String) -> Result<NetworkDetails, String> {
    let docker = get_docker_client().await?;
    let network = docker
        .inspect_network(&id, None::<InspectNetworkOptions<String>>)
        .await
        .map_err(|e| format!("无法获取网络详情: {}", e))?;

    let containers = network
        .containers
        .unwrap_or_default()
        .into_iter()
        .map(|(container_id, details)| ConnectedContainer {
            id: container_id,
            name: details.name.unwrap_or_default(),
            ipv4_address: details.ipv4_address.unwrap_or_default(),
            ipv6_address: details.ipv6_address.unwrap_or_default(),
            mac_address: details.mac_address.unwrap_or_default(),
        })
        .collect();

    let (subnet, gateway) = network
        .ipam
        .and_then(|ipam| ipam.config)
        .and_then(|config| config.first().cloned())
        .map(|cfg| {
            (
                cfg.subnet.unwrap_or_else(|| "N/A".to_string()),
                cfg.gateway.unwrap_or_else(|| "N/A".to_string()),
            )
        })
        .unwrap_or_else(|| ("N/A".to_string(), "N/A".to_string()));

    Ok(NetworkDetails {
        id: network.id.unwrap_or_default(),
        name: network.name.unwrap_or_default(),
        driver: network.driver.unwrap_or_default(),
        scope: network.scope.unwrap_or_default(),
        created: network.created.unwrap_or_default(),
        internal: network.internal.unwrap_or_default(),
        attachable: network.attachable.unwrap_or_default(),
        ingress: network.ingress.unwrap_or_default(),
        subnet,
        gateway,
        containers,
        options: network.options.unwrap_or_default(),
        labels: network.labels.unwrap_or_default(),
    })
}

/// 在文件管理器中打开卷路径
#[tauri::command]
pub async fn open_volume_path(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("无法打开目录: {}", e))
}

/// 获取本地已安装的 WSL 发行版列表
#[tauri::command]
pub async fn list_wsl_distros() -> Result<Vec<String>, String> {
    #[cfg(windows)]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;

        // 执行 wsl -l -q 命令
        let mut cmd = Command::new("wsl.exe");
        cmd.args(["-l", "-q"]);
        // 0x08000000 即 CREATE_NO_WINDOW，防止弹出 cmd 黑色窗口
        cmd.creation_flags(0x08000000);

        let output = cmd.output()
            .map_err(|e| format!("无法执行 wsl 命令，可能未安装 WSL: {}", e))?;

        if !output.status.success() {
            return Err("WSL 命令执行失败".to_string());
        }

        let stdout_raw = output.stdout;
        let mut distros = Vec::new();

        // 优先尝试 UTF-16 LE 解析（因为 Windows 下 wsl -l -q 默认为 UTF-16 字节流）
        if stdout_raw.len() % 2 == 0 {
            let u16_data: Vec<u16> = stdout_raw
                .chunks_exact(2)
                .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
                .collect();
            if let Ok(text) = String::from_utf16(&u16_data) {
                for line in text.lines() {
                    let trimmed = line.trim().trim_start_matches('\u{feff}').to_string();
                    if !trimmed.is_empty() {
                        distros.push(trimmed);
                    }
                }
            }
        }

        // 如果 UTF-16 解析结果为空，回退到 UTF-8 编码解析
        if distros.is_empty() {
            if let Ok(text) = String::from_utf8(stdout_raw) {
                for line in text.lines() {
                    let trimmed = line.trim().trim_start_matches('\u{feff}').to_string();
                    if !trimmed.is_empty() {
                        distros.push(trimmed);
                    }
                }
            }
        }

        Ok(distros)
    }

    #[cfg(not(windows))]
    {
        // 非 Windows 平台不提供 WSL 发行版
        Ok(vec![])
    }
}

/// 打开本地配置文件目录
#[tauri::command]
pub async fn open_config_dir(app: AppHandle) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;
    
    app.opener()
        .open_path(app_dir.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| format!("无法打开目录: {}", e))
}
