use crate::connection::get_docker_client;
use crate::error::AppResult;
use bollard::image::{CreateImageOptions, ListImagesOptions, ImportImageOptions, PruneImagesOptions, TagImageOptions};
use bollard::container::Config;
use bollard::models::{HostConfig, PortBinding};
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use tokio_util::bytes::Bytes;
use tokio_util::io::ReaderStream;
use tokio::io::AsyncWriteExt;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;
use serde::Serialize;
use super::{ImageInfo, ImageDetails, ImageSearchResult, ImageHistoryInfo, PruneImagesResult};

/// 获取本地 Docker 镜像列表的命令
#[tauri::command]
pub async fn list_images() -> AppResult<Vec<ImageInfo>> {
    let docker = get_docker_client().await?;

    let images = docker
        .list_images(Some(ListImagesOptions::<String> {
            all: false,
            ..Default::default()
        }))
        .await?;

    Ok(images.into_iter().map(ImageInfo::from).collect())
}

/// 获取镜像详情
#[tauri::command]
pub async fn inspect_image(id: String) -> AppResult<ImageDetails> {
    let docker = get_docker_client().await?;
    let details = docker
        .inspect_image(&id)
        .await?;

    Ok(ImageDetails::from(details))
}

/// 删除镜像
#[tauri::command]
pub async fn remove_image(id: String) -> AppResult<()> {
    let docker = get_docker_client().await?;
    docker
        .remove_image(&id, None, None)
        .await?;
    Ok(())
}

/// 搜索镜像
#[tauri::command]
pub async fn search_images(term: String) -> AppResult<Vec<ImageSearchResult>> {
    let docker = get_docker_client().await?;
    let results = docker
        .search_images(bollard::image::SearchImagesOptions {
            term,
            limit: None,
            filters: HashMap::new(),
        })
        .await?;

    Ok(results.into_iter().map(ImageSearchResult::from).collect())
}

/// 获取镜像历史
#[tauri::command]
pub async fn get_image_history(id: String) -> AppResult<Vec<ImageHistoryInfo>> {
    let docker = get_docker_client().await?;
    let history = docker
        .image_history(&id)
        .await?;

    Ok(history.into_iter().map(ImageHistoryInfo::from).collect())
}

/// 拉取镜像
#[tauri::command]
pub async fn pull_image(
    app: AppHandle,
    image_name: String,
    username: Option<String>,
    password: Option<String>,
    server_address: Option<String>,
) -> AppResult<()> {
    let docker = get_docker_client().await?;
    
    let full_image_name = if image_name.contains(':') {
        image_name.clone()
    } else {
        format!("{}:latest", image_name)
    };

    log::info!("开始拉取镜像: {}", full_image_name);

    let mut credentials = None;
    if let (Some(u), Some(p)) = (username, password) {
        if !u.is_empty() && !p.is_empty() {
            credentials = Some(bollard::auth::DockerCredentials {
                username: Some(u),
                password: Some(p),
                serveraddress: server_address,
                ..Default::default()
            });
        }
    }

    let mut stream = docker.create_image(
        Some(CreateImageOptions {
            from_image: full_image_name.clone(),
            ..Default::default()
        }),
        None,
        credentials,
    );

    let app_handle = app.clone();
    let name_for_events = full_image_name.clone();
    
    tauri::async_runtime::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(info) => {
                    #[derive(Serialize, Clone)]
                    struct ProgressPayload {
                        image: String,
                        info: bollard::models::CreateImageInfo,
                    }
                    let _ = app_handle.emit("image-pull-progress", ProgressPayload {
                        image: name_for_events.clone(),
                        info,
                    });
                }
                Err(e) => {
                    log::error!("拉取镜像 {} 出错: {}", name_for_events, e);
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
        log::info!("镜像拉取任务结束: {}", name_for_events);
        let _ = app_handle.emit("image-pull-finished", name_for_events);
    });

    Ok(())
}

/// 运行镜像（创建并启动容器）
#[tauri::command]
pub async fn run_image(
    image: String,
    name: Option<String>,
    ports: Vec<String>,
    env: Vec<String>,
    restart_policy: Option<String>,
    binds: Option<Vec<String>>,
    tty: Option<bool>,
    open_stdin: Option<bool>,
    cmd: Option<Vec<String>>,
    overwrite: Option<bool>,
) -> AppResult<String> {
    let docker = get_docker_client().await?;

    if overwrite.unwrap_or(false) {
        if let Some(ref container_name) = name {
            let remove_options = bollard::container::RemoveContainerOptions {
                v: true,
                force: true,
                link: false,
            };
            let _ = docker.remove_container(container_name, Some(remove_options)).await;
        }
    }

    let mut port_bindings = HashMap::new();
    let mut exposed_ports = HashMap::new();

    for p in ports {
        let parts: Vec<&str> = p.split(':').collect();
        let (host_port, container_part) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else if parts.len() == 1 {
            ("", parts[0])
        } else {
            continue;
        };

        let container_parts: Vec<&str> = container_part.split('/').collect();
        let container_port = container_parts[0];
        let protocol = if container_parts.len() > 1 { container_parts[1] } else { "tcp" };

        if container_port.trim().is_empty() {
            continue;
        }

        let container_key = format!("{}/{}", container_port.trim(), protocol);
        exposed_ports.insert(container_key.clone(), HashMap::new());
        
        let host_port_opt = if host_port.trim().is_empty() {
            None
        } else {
            Some(host_port.trim().to_string())
        };
        let binding = PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: host_port_opt,
        };
        port_bindings.insert(container_key, Some(vec![binding]));
    }

    let restart = restart_policy.map(|p| {
        use bollard::models::RestartPolicyNameEnum;
        let name = match p.as_str() {
            "always" => RestartPolicyNameEnum::ALWAYS,
            "unless-stopped" => RestartPolicyNameEnum::UNLESS_STOPPED,
            "on-failure" => RestartPolicyNameEnum::ON_FAILURE,
            _ => RestartPolicyNameEnum::EMPTY,
        };
        bollard::models::RestartPolicy {
            name: Some(name),
            maximum_retry_count: None,
        }
    });

    let container_cmd = match cmd {
        Some(v) if !v.is_empty() => Some(v),
        _ => None,
    };

    let host_binds = match binds {
        Some(v) if !v.is_empty() => Some(v),
        _ => None,
    };

    let config = Config {
        image: Some(image),
        env: Some(env),
        exposed_ports: Some(exposed_ports),
        tty,
        open_stdin,
        cmd: container_cmd,
        host_config: Some(HostConfig {
            port_bindings: Some(port_bindings),
            restart_policy: restart,
            binds: host_binds,
            ..Default::default()
        }),
        ..Default::default()
    };

    let container = docker
        .create_container(
            name.as_ref().map(|n| bollard::container::CreateContainerOptions {
                name: n.clone(),
                ..Default::default()
            }),
            config,
        )
        .await?;

    docker
        .start_container::<String>(&container.id, None)
        .await?;

    Ok(container.id)
}

/// 获取本地已安装的 WSL 发行版列表
#[tauri::command]
pub async fn list_wsl_distros() -> AppResult<Vec<String>> {
    #[cfg(windows)]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;

        let mut cmd = Command::new("wsl.exe");
        cmd.args(["-l", "-q"]);
        cmd.creation_flags(0x08000000);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err("WSL 命令执行失败".into());
        }

        let stdout_raw = output.stdout;
        let mut distros = Vec::new();

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
        Ok(vec![])
    }
}

/// 打开本地配置文件目录
#[tauri::command]
pub async fn open_config_dir(app: AppHandle) -> AppResult<()> {
    let app_dir = app
        .path()
        .app_data_dir()?;
    
    app.opener()
        .open_path(app_dir.to_string_lossy().to_string(), None::<String>)?;
    Ok(())
}

/// 导出镜像为 tar 文件
#[tauri::command]
pub async fn export_image(
    app: AppHandle,
    image_id_or_name: String,
    path: String,
) -> AppResult<()> {
    let docker = get_docker_client().await?;
    let mut stream = docker.export_image(&image_id_or_name);

    let app_handle = app.clone();
    let path_clone = path.clone();
    let name_clone = image_id_or_name.clone();

    tauri::async_runtime::spawn(async move {
        let mut file = match tokio::fs::File::create(&path_clone).await {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("创建目标文件失败: {}", e);
                log::error!("{}", err_msg);
                #[derive(Clone, serde::Serialize)]
                struct ExportErrPayload {
                    image: String,
                    error: String,
                }
                let _ = app_handle.emit("image-export-error", ExportErrPayload {
                    image: name_clone,
                    error: err_msg,
                });
                return;
            }
        };

        let mut total_bytes = 0i64;

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    total_bytes += bytes.len() as i64;
                    if let Err(e) = file.write_all(&bytes).await {
                        let err_msg = format!("写入镜像数据失败: {}", e);
                        log::error!("{}", err_msg);
                        #[derive(Clone, serde::Serialize)]
                        struct ExportErrPayload {
                            image: String,
                            error: String,
                        }
                        let _ = app_handle.emit("image-export-error", ExportErrPayload {
                            image: name_clone,
                            error: err_msg,
                        });
                        return;
                    }

                    #[derive(Clone, serde::Serialize)]
                    struct ExportProgressPayload {
                        image: String,
                        bytes_written: i64,
                    }
                    let _ = app_handle.emit("image-export-progress", ExportProgressPayload {
                        image: name_clone.clone(),
                        bytes_written: total_bytes,
                    });
                }
                Err(e) => {
                    let err_msg = format!("读取镜像导出流失败: {}", e);
                    log::error!("{}", err_msg);
                    #[derive(Clone, serde::Serialize)]
                    struct ExportErrPayload {
                        image: String,
                        error: String,
                    }
                    let _ = app_handle.emit("image-export-error", ExportErrPayload {
                        image: name_clone,
                        error: err_msg,
                    });
                    return;
                }
            }
        }

        if let Err(e) = file.flush().await {
            let err_msg = format!("刷新文件失败: {}", e);
            log::error!("{}", err_msg);
            #[derive(Clone, serde::Serialize)]
            struct ExportErrPayload {
                image: String,
                error: String,
            }
            let _ = app_handle.emit("image-export-error", ExportErrPayload {
                image: name_clone,
                error: err_msg,
            });
            return;
        }

        log::info!("镜像导出任务结束: {}", name_clone);
        let _ = app_handle.emit("image-export-finished", name_clone);
    });

    Ok(())
}

/// 导入镜像文件
#[tauri::command]
pub async fn import_image(
    app: AppHandle,
    path: String,
) -> AppResult<()> {
    let docker = get_docker_client().await?;

    let file = tokio::fs::File::open(&path)
        .await?;

    let byte_stream = ReaderStream::new(file)
        .map(|res| {
            res.unwrap_or_else(|e| {
                log::error!("读取 Tar 文件出错: {}", e);
                Bytes::new()
            })
        });

    let mut stream = docker.import_image_stream(
        ImportImageOptions { quiet: false },
        byte_stream,
        None,
    );

    let app_handle = app.clone();
    let path_clone = path.clone();

    tauri::async_runtime::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(info) => {
                    #[derive(Clone, serde::Serialize)]
                    struct ImportProgressPayload {
                        path: String,
                        status: Option<String>,
                        stream: Option<String>,
                        error: Option<String>,
                        progress: Option<String>,
                    }
                    let payload = ImportProgressPayload {
                        path: path_clone.clone(),
                        status: info.status,
                        stream: info.stream,
                        error: info.error,
                        progress: info.progress,
                    };
                    let _ = app_handle.emit("image-import-progress", payload);
                }
                Err(e) => {
                    log::error!("导入镜像 {} 出错: {}", path_clone, e);
                    #[derive(Clone, serde::Serialize)]
                    struct ImportErrPayload {
                        path: String,
                        error: String,
                    }
                    let _ = app_handle.emit("image-import-error", ImportErrPayload {
                        path: path_clone.clone(),
                        error: e.to_string(),
                    });
                    return;
                }
            }
        }
        log::info!("镜像导入任务结束: {}", path_clone);
        let _ = app_handle.emit("image-import-finished", path_clone);
    });

    Ok(())
}

/// 清理无用的虚悬镜像
#[tauri::command]
pub async fn prune_images() -> AppResult<PruneImagesResult> {
    let docker = get_docker_client().await?;

    let mut filters = HashMap::new();
    filters.insert("dangling".to_string(), vec!["true".to_string()]);

    let options = PruneImagesOptions { filters };
    let response = docker
        .prune_images(Some(options))
        .await?;

    Ok(PruneImagesResult::from(response))
}

/// 为镜像打标签
#[tauri::command]
pub async fn tag_image(
    image_name: String,
    repo: String,
    tag: String,
) -> AppResult<()> {
    let docker = get_docker_client().await?;

    let options = TagImageOptions {
        repo,
        tag,
    };

    docker
        .tag_image(&image_name, Some(options))
        .await?;

    Ok(())
}
