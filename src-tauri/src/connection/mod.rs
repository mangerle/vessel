use bollard::Docker;
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use crate::error::AppResult;

pub mod wsl;

/// 连接配置结构体
#[derive(Clone)]
pub struct ConnectionConfig {
    pub mode: String,
    pub distro: Option<String>,
}

/// 全局连接配置
pub static CONNECTION_CONFIG: Lazy<Arc<Mutex<ConnectionConfig>>> = Lazy::new(|| Arc::new(Mutex::new(ConnectionConfig {
    mode: "wsl".to_string(),
    distro: None,
})));

/// 全局 Docker 客户端实例
static DOCKER_CLIENT: Lazy<Arc<Mutex<Option<Docker>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

/// 清除 Docker 客户端缓存，强制重新连接
pub async fn clear_client_cache() {
    let mut client_lock = DOCKER_CLIENT.lock().await;
    *client_lock = None;
}

/// 更新全局连接配置的命令
#[tauri::command]
pub async fn update_connection_config(mode: String, distro: Option<String>) {
    let mut config = CONNECTION_CONFIG.lock().await;
    config.mode = mode;
    config.distro = distro;
    // 配置改变后，必须清除客户端缓存以触发重新连接
    clear_client_cache().await;
}

/// 获取 Docker 客户端
pub async fn get_docker_client() -> AppResult<Docker> {
    let mut client_lock = DOCKER_CLIENT.lock().await;
    
    if let Some(client) = &*client_lock {
        return Ok(client.clone());
    }

    let config = CONNECTION_CONFIG.lock().await.clone();

    // 根据配置选择连接方式
    if config.mode == "wsl" {
        match wsl::WslBridge::new(config.distro).connect().await {
            Ok(docker) => {
                *client_lock = Some(docker.clone());
                Ok(docker)
            }
            Err(e) => {
                // 如果 WSL 失败，回退到探测命名管道 (Windows 默认，兼容 Docker Desktop)
                #[cfg(windows)]
                {
                    if let Ok(docker) = Docker::connect_with_named_pipe_defaults() {
                        if docker.ping().await.is_ok() {
                            *client_lock = Some(docker.clone());
                            return Ok(docker);
                        }
                    }
                }
                Err(format!("无法通过 WSL 连接到 Docker: {}", e).into())
            }
        }
    } else {
        // SSH 或其他模式暂未完全实现，回退到命名管道
        #[cfg(windows)]
        {
            if let Ok(docker) = Docker::connect_with_named_pipe_defaults() {
                if docker.ping().await.is_ok() {
                    *client_lock = Some(docker.clone());
                    return Ok(docker);
                }
            }
        }
        Err("当前连接模式暂未支持或无法连接到本地 Docker".to_string().into())
    }
}

