use crate::connection::get_docker_client;
use bollard::container::{ListContainersOptions, StatsOptions, LogsOptions};
use tauri::{AppHandle, Emitter};
use futures_util::stream::StreamExt;
use super::{ContainerInfo, ContainerDetails};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use std::sync::atomic::{AtomicU64, Ordering};

pub static STATS_STREAMS: Lazy<Arc<Mutex<HashMap<String, (oneshot::Sender<()>, u64)>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub static LOGS_STREAMS: Lazy<Arc<Mutex<HashMap<String, (oneshot::Sender<()>, u64)>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static STREAM_COUNTER: AtomicU64 = AtomicU64::new(0);


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

    Ok(containers.into_iter().map(ContainerInfo::from).collect())
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

    Ok(ContainerDetails::from(details))
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

    let (tx, mut rx) = oneshot::channel::<()>();
    let token = STREAM_COUNTER.fetch_add(1, Ordering::SeqCst);

    {
        let mut streams = STATS_STREAMS.lock().await;
        if let Some((old_tx, _)) = streams.insert(id.clone(), (tx, token)) {
            let _ = old_tx.send(());
        }
    }

    let id_clone = id.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut rx => {
                    log::info!("收到停止信号，停止统计流: {}", id_clone);
                    break;
                }
                msg = stream.next() => {
                    match msg {
                        Some(Ok(stats)) => {
                            let event_name = format!("container-stats-{}", id_clone);
                            if let Err(e) = app.emit(&event_name, stats) {
                                log::error!("发送统计事件失败: {}", e);
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            log::error!("获取统计数据失败: {}", e);
                            break;
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
        }
        // 协程退出时清理
        let mut streams = STATS_STREAMS.lock().await;
        if let Some((_, t)) = streams.get(&id_clone) {
            if *t == token {
                streams.remove(&id_clone);
            }
        }
    });

    Ok(())
}

/// 关闭容器统计信息流
#[tauri::command]
pub async fn close_container_stats(id: String) -> Result<(), String> {
    let mut streams = STATS_STREAMS.lock().await;
    if let Some((tx, _)) = streams.remove(&id) {
        let _ = tx.send(());
    }
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

    let (tx, mut rx) = oneshot::channel::<()>();
    let token = STREAM_COUNTER.fetch_add(1, Ordering::SeqCst);

    {
        let mut streams = LOGS_STREAMS.lock().await;
        if let Some((old_tx, _)) = streams.insert(id.clone(), (tx, token)) {
            let _ = old_tx.send(());
        }
    }

    let id_clone = id.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut rx => {
                    log::info!("收到停止信号，停止日志流: {}", id_clone);
                    break;
                }
                msg = stream.next() => {
                    match msg {
                        Some(Ok(log)) => {
                            let event_name = format!("container-logs-{}", id_clone);
                            if let Err(e) = app.emit(&event_name, log.to_string()) {
                                log::error!("发送日志事件失败: {}", e);
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            log::error!("获取日志流出错: {}", e);
                            break;
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
        }
        // 协程退出时清理
        let mut streams = LOGS_STREAMS.lock().await;
        if let Some((_, t)) = streams.get(&id_clone) {
            if *t == token {
                streams.remove(&id_clone);
            }
        }
    });

    Ok(())
}

/// 关闭容器日志流
#[tauri::command]
pub async fn close_container_logs(id: String) -> Result<(), String> {
    let mut streams = LOGS_STREAMS.lock().await;
    if let Some((tx, _)) = streams.remove(&id) {
        let _ = tx.send(());
    }
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

