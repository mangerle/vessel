use crate::connection::get_docker_client;
use bollard::exec::{CreateExecOptions, ResizeExecOptions, StartExecOptions, StartExecResults};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tauri::{AppHandle, Emitter};
use futures_util::stream::StreamExt;
use tokio::io::AsyncWriteExt;

/// 终端会话结构体
pub struct TerminalSession {
    pub stdin_tx: mpsc::Sender<Vec<u8>>,
}

/// 全局终端会话管理器
pub static TERMINAL_SESSIONS: Lazy<Arc<Mutex<HashMap<String, TerminalSession>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// 创建容器终端会话
#[tauri::command]
pub async fn create_container_terminal(
    app: AppHandle,
    id: String,
    user: Option<String>,
) -> Result<String, String> {
    let docker = get_docker_client().await?;

    let exec = docker
        .create_exec(
            &id,
            CreateExecOptions {
                attach_stdin: Some(true),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                tty: Some(true),
                user,
                cmd: Some(vec!["sh".to_string()]),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| format!("创建终端失败: {}", e))?;

    let exec_id = exec.id;
    let exec_id_clone = exec_id.clone();
    let app_clone = app.clone();

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

        {
            let mut sessions = TERMINAL_SESSIONS.lock().await;
            sessions.insert(exec_id.clone(), TerminalSession { stdin_tx });
        }

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

            tokio::select! {
                _ = &mut stdout_task => {},
                _ = &mut stdin_task => {},
            };

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
