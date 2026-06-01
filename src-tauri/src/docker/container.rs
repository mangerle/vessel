use crate::handle_docker_op;
use super::{ContainerDetails, ContainerInfo, LOGS_STREAMS, STATS_STREAMS, spawn_stream_handler};
use crate::connection::get_docker_client;
use crate::error::AppResult;
use bollard::container::{ListContainersOptions, LogsOptions, StatsOptions};
use futures_util::stream::StreamExt;
use tauri::AppHandle;

/// 获取本地 Docker 容器列表的命令
#[tauri::command]
pub async fn list_local_containers() -> AppResult<Vec<ContainerInfo>> {
    let docker = get_docker_client().await?;

    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;

    Ok(containers.into_iter().map(ContainerInfo::from).collect())
}

/// 启动容器
#[tauri::command]
pub async fn start_container(id: String) -> AppResult<()> {
    log::info!("正在启动容器: {}", id);
    let docker = get_docker_client().await?;
    handle_docker_op!("启动容器", id, docker.start_container::<String>(&id, None))
}

/// 停止容器
#[tauri::command]
pub async fn stop_container(id: String) -> AppResult<()> {
    log::info!("正在停止容器: {}", id);
    let docker = get_docker_client().await?;
    handle_docker_op!("停止容器", id, docker.stop_container(&id, None))
}

/// 重启容器
#[tauri::command]
pub async fn restart_container(id: String) -> AppResult<()> {
    log::info!("正在重启容器: {}", id);
    let docker = get_docker_client().await?;
    handle_docker_op!("重启容器", id, docker.restart_container(&id, None))
}

/// 获取容器详情
#[tauri::command]
pub async fn inspect_container(id: String) -> AppResult<ContainerDetails> {
    let docker = get_docker_client().await?;
    let details = docker.inspect_container(&id, None).await?;

    Ok(ContainerDetails::from(details))
}

/// 删除容器
#[tauri::command]
pub async fn remove_container(id: String) -> AppResult<()> {
    log::info!("正在删除容器: {}", id);
    let docker = get_docker_client().await?;
    handle_docker_op!("删除容器", id, docker.remove_container(&id, None))
}

/// 实时流式传输容器统计信息
#[tauri::command]
pub async fn stream_container_stats(app: AppHandle, id: String) -> AppResult<()> {
    let docker = get_docker_client().await?;
    let stream = docker.stats(
        &id,
        Some(StatsOptions {
            stream: true,
            one_shot: false,
        }),
    );

    spawn_stream_handler(
        app,
        id.clone(),
        stream,
        &*STATS_STREAMS,
        format!("container-stats-{}", id),
        "统计",
    )
    .await;

    Ok(())
}

/// 关闭容器统计信息流
#[tauri::command]
pub async fn close_container_stats(id: String) -> AppResult<()> {
    let mut streams = STATS_STREAMS.lock().await;
    if let Some((tx, _)) = streams.remove(&id) {
        let _ = tx.send(());
    }
    Ok(())
}

/// 获取容器日志
#[tauri::command]
pub async fn stream_container_logs(app: AppHandle, id: String) -> AppResult<()> {
    let docker = get_docker_client().await?;
    let stream = docker.logs(
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

    let mapped_stream = stream.map(|res| res.map(|log| log.to_string()));

    spawn_stream_handler(
        app,
        id.clone(),
        mapped_stream,
        &*LOGS_STREAMS,
        format!("container-logs-{}", id),
        "日志",
    )
    .await;

    Ok(())
}

/// 关闭容器日志流
#[tauri::command]
pub async fn close_container_logs(id: String) -> AppResult<()> {
    let mut streams = LOGS_STREAMS.lock().await;
    if let Some((tx, _)) = streams.remove(&id) {
        let _ = tx.send(());
    }
    Ok(())
}

/// 重命名容器
#[tauri::command]
pub async fn rename_container(id: String, new_name: String) -> AppResult<()> {
    log::info!("正在重命名容器 {} 为 {}", id, new_name);
    let docker = get_docker_client().await?;

    let options = bollard::container::RenameContainerOptions {
        name: new_name.clone(),
    };

    handle_docker_op!("重命名容器", id, docker.rename_container(&id, options))
}

/// 提交容器为新镜像
#[tauri::command]
pub async fn commit_container(
    id: String,
    repo: String,
    tag: Option<String>,
    comment: Option<String>,
    author: Option<String>,
) -> AppResult<String> {
    log::info!("正在提交容器 {} 为镜像 {}:{:?}", id, repo, tag);
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

    match docker
        .commit_container(options, bollard::container::Config::<String>::default())
        .await
    {
        Ok(response) => {
            let new_id = response.id.unwrap_or_default();
            log::info!("容器 {} 提交成功，新镜像 ID: {}", id, new_id);
            Ok(new_id)
        }
        Err(e) => {
            log::error!("提交容器 {} 失败: {}", id, e);
            Err(e.into())
        }
    }
}

/// 暂停容器
#[tauri::command]
pub async fn pause_container(id: String) -> AppResult<()> {
    log::info!("正在暂停容器: {}", id);
    let docker = get_docker_client().await?;
    handle_docker_op!("暂停容器", id, docker.pause_container(&id))
}

/// 恢复容器
#[tauri::command]
pub async fn unpause_container(id: String) -> AppResult<()> {
    log::info!("正在恢复容器: {}", id);
    let docker = get_docker_client().await?;
    handle_docker_op!("恢复容器", id, docker.unpause_container(&id))
}

/// 进程 Top 响应结构
#[derive(serde::Serialize)]
pub struct TopResult {
    pub titles: Vec<String>,
    pub processes: Vec<Vec<String>>,
}

/// 获取容器进程列表
#[tauri::command]
pub async fn top_container(id: String) -> AppResult<TopResult> {
    let docker = get_docker_client().await?;
    let top_result = docker
        .top_processes(&id, None::<bollard::container::TopOptions<String>>)
        .await?;

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
pub async fn exec_container(id: String, cmd: String) -> AppResult<ExecResult> {
    use bollard::container::LogOutput;
    use bollard::exec::{CreateExecOptions, StartExecResults};

    log::info!("正在容器 {} 中执行命令: {}", id, cmd);
    let docker = get_docker_client().await?;

    let config = CreateExecOptions {
        cmd: Some(vec!["sh", "-c", &cmd]),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        ..Default::default()
    };

    let exec = match docker.create_exec(&id, config).await {
        Ok(e) => e,
        Err(e) => {
            log::error!("在容器 {} 中创建 exec 失败: {}", id, e);
            return Err(e.into());
        }
    };

    let start_exec_result = match docker.start_exec(&exec.id, None).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("在容器 {} 中启动 exec 失败: {}", id, e);
            return Err(e.into());
        }
    };

    let mut output_str = String::new();
    if let StartExecResults::Attached { mut output, .. } = start_exec_result {
        while let Some(msg) = output.next().await {
            match msg? {
                LogOutput::StdOut { message } => {
                    output_str.push_str(&String::from_utf8_lossy(&message));
                }
                LogOutput::StdErr { message } => {
                    output_str.push_str(&String::from_utf8_lossy(&message));
                }
                _ => {}
            }
        }
    }

    let inspect_result = docker.inspect_exec(&exec.id).await?;

    log::info!(
        "容器 {} 命令执行完成，退出码: {:?}",
        id,
        inspect_result.exit_code
    );
    Ok(ExecResult {
        exit_code: inspect_result.exit_code,
        output: output_str,
    })
}
