use crate::connection::get_docker_client;
use bollard::container::{ListContainersOptions, StatsOptions, LogsOptions};
use tauri::{AppHandle, Emitter};
use futures_util::stream::StreamExt;
use super::{ContainerInfo, PortMapping, MountInfo, ContainerDetails};

/// 获取本地 Docker 容器列表的命令
#[tauri::command]
pub async fn list_local_containers() -> Result<Vec<ContainerInfo>, String> {
    let docker = get_docker_client().await?;

    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
        .map_err(|e| format!("无法获取容器列表: {}", e))?;

    Ok(containers
        .into_iter()
        .map(|c| {
            let compose_project = c
                .labels
                .as_ref()
                .and_then(|labels| labels.get("com.docker.compose.project").cloned());
            ContainerInfo {
                id: c.id.unwrap_or_default(),
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
                    let event_name = format!("container-stats-{}", id);
                    if let Err(e) = app.emit(&event_name, stats) {
                        log::error!("发送统计事件失败: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    log::error!("获取统计数据失败: {}", e);
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
                        log::error!("发送日志事件失败: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    log::error!("获取日志流出错: {}", e);
                    break;
                }
            }
        }
    });

    Ok(())
}

/// 重命名容器
#[tauri::command]
pub async fn rename_container(id: String, new_name: String) -> Result<(), String> {
    let docker = get_docker_client().await?;

    let options = bollard::container::RenameContainerOptions {
        name: new_name,
    };

    docker
        .rename_container(&id, options)
        .await
        .map_err(|e| format!("重命名容器失败: {}", e))?;

    Ok(())
}

/// 提交容器为新镜像
#[tauri::command]
pub async fn commit_container(
    id: String,
    repo: String,
    tag: Option<String>,
    comment: Option<String>,
    author: Option<String>,
) -> Result<String, String> {
    let docker = get_docker_client().await?;

    let options = bollard::image::CommitContainerOptions {
        container: id.clone(),
        repo,
        tag: tag.unwrap_or_else(|| "latest".to_string()),
        comment: comment.unwrap_or_default(),
        author: author.unwrap_or_default(),
        pause: true,
        changes: None,
    };

    let response = docker
        .commit_container(options, bollard::container::Config::<String>::default())
        .await
        .map_err(|e| format!("提交容器失败: {}", e))?;

    Ok(response.id.unwrap_or_default())
}

/// 暂停容器
#[tauri::command]
pub async fn pause_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .pause_container(&id)
        .await
        .map_err(|e| format!("暂停容器失败: {}", e))
}

/// 恢复容器
#[tauri::command]
pub async fn unpause_container(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .unpause_container(&id)
        .await
        .map_err(|e| format!("恢复容器失败: {}", e))
}

/// 进程 Top 响应结构
#[derive(serde::Serialize)]
pub struct TopResult {
    pub titles: Vec<String>,
    pub processes: Vec<Vec<String>>,
}

/// 获取容器进程列表
#[tauri::command]
pub async fn top_container(id: String) -> Result<TopResult, String> {
    let docker = get_docker_client().await?;
    let top_result = docker
        .top_processes(&id, None::<bollard::container::TopOptions<String>>)
        .await
        .map_err(|e| format!("获取进程列表失败: {}", e))?;

    Ok(TopResult {
        titles: top_result.titles.unwrap_or_default(),
        processes: top_result.processes.unwrap_or_default(),
    })
}

/// Exec 执行结果
#[derive(serde::Serialize)]
pub struct ExecResult {
    pub exit_code: Option<i64>,
    pub output: String,
}

/// 在容器内执行单次命令
#[tauri::command]
pub async fn exec_container(id: String, cmd: String) -> Result<ExecResult, String> {
    use bollard::container::LogOutput;
    use bollard::exec::{CreateExecOptions, StartExecResults};

    let docker = get_docker_client().await?;

    let config = CreateExecOptions {
        cmd: Some(vec!["sh", "-c", &cmd]),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        ..Default::default()
    };

    let exec = docker
        .create_exec(&id, config)
        .await
        .map_err(|e| format!("创建 exec 失败: {}", e))?;

    let start_exec_result = docker
        .start_exec(&exec.id, None)
        .await
        .map_err(|e| format!("启动 exec 失败: {}", e))?;

    let mut output_str = String::new();
    if let StartExecResults::Attached { mut output, .. } = start_exec_result {
        while let Some(msg) = output.next().await {
            match msg {
                Ok(LogOutput::StdOut { message }) => {
                    output_str.push_str(&String::from_utf8_lossy(&message));
                }
                Ok(LogOutput::StdErr { message }) => {
                    output_str.push_str(&String::from_utf8_lossy(&message));
                }
                Err(e) => {
                    output_str.push_str(&format!("\n[读取输出错误: {}]\n", e));
                    break;
                }
                _ => {}
            }
        }
    }

    let inspect_result = docker
        .inspect_exec(&exec.id)
        .await
        .map_err(|e| format!("获取 exec 状态失败: {}", e))?;

    Ok(ExecResult {
        exit_code: inspect_result.exit_code,
        output: output_str,
    })
}

