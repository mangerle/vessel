use crate::connection::get_docker_client;
use bollard::container::{ListContainersOptions, StatsOptions};
use bollard::image::{ListImagesOptions, CreateImageOptions};
use serde::Serialize;
use futures_util::stream::StreamExt;
use tauri::{AppHandle, Emitter};

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
    Ok(containers.into_iter().map(|c| ContainerInfo {
        id: c.id.unwrap_or_default(),
        // c.names 通常以 ["/container_name"] 格式返回，所以我们取第一个并去掉开头的斜杠
        name: c.names.as_ref()
            .and_then(|names| names.first())
            .map(|name| name.trim_start_matches('/').to_string())
            .unwrap_or_else(|| "未知".to_string()),
        state: c.state.unwrap_or_default(),
        image: c.image.unwrap_or_default(),
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
