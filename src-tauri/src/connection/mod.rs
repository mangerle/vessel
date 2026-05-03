use bollard::Docker;
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

pub mod wsl;

/// 全局 Docker 客户端实例
static DOCKER_CLIENT: Lazy<Arc<Mutex<Option<Docker>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

/// 清除 Docker 客户端缓存，强制重新连接
pub async fn clear_client_cache() {
    let mut client_lock = DOCKER_CLIENT.lock().await;
    *client_lock = None;
}

/// 获取 Docker 客户端
/// 默认使用 WSL 桥接
pub async fn get_docker_client() -> Result<Docker, String> {
    let mut client_lock = DOCKER_CLIENT.lock().await;
    
    if let Some(client) = &*client_lock {
        return Ok(client.clone());
    }

    // 默认使用 WSL 桥接
    match wsl::WslBridge::new(None).connect().await {
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
            Err(format!("无法连接到 Docker: {}", e))
        }
    }
}
