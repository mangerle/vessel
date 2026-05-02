use bollard::Docker;
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

pub mod wsl;

/// Docker 连接驱动类型
#[derive(Debug, Clone, Copy)]
pub enum DriverType {
    /// Windows 命名管道 (Docker Desktop)
    NamedPipe,
    /// WSL 桥接 (WSL 中的 Docker)
    WslBridge,
}

/// 全局 Docker 客户端实例
static DOCKER_CLIENT: Lazy<Arc<Mutex<Option<Docker>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

/// 获取 Docker 客户端
/// 按照 NamedPipe -> WslBridge 的优先级自动探测
pub async fn get_docker_client() -> Result<Docker, String> {
    let mut client_lock = DOCKER_CLIENT.lock().await;
    
    if let Some(client) = &*client_lock {
        return Ok(client.clone());
    }

    // 尝试命名管道 (Windows 默认)
    #[cfg(windows)]
    {
        if let Ok(docker) = Docker::connect_with_named_pipe_defaults() {
            // 尝试执行一个简单的操作来验证连接
            if docker.ping().await.is_ok() {
                *client_lock = Some(docker.clone());
                return Ok(docker);
            }
        }
    }

    // 尝试 WSL 桥接
    match wsl::WslBridge::new().connect().await {
        Ok(docker) => {
            *client_lock = Some(docker.clone());
            Ok(docker)
        }
        Err(e) => Err(format!("无法连接到任何 Docker 驱动: {}", e)),
    }
}
