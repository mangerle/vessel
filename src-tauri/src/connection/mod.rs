use crate::error::AppResult;
use bollard::Docker;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tokio::sync::RwLock;

pub mod wsl;

/// 连接模式枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionMode {
    Wsl,
    Ssh,
    Desktop,
}

impl From<String> for ConnectionMode {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "wsl" => ConnectionMode::Wsl,
            "ssh" => ConnectionMode::Ssh,
            "desktop" => ConnectionMode::Desktop,
            _ => ConnectionMode::Desktop, // 默认回退
        }
    }
}

/// 连接配置结构体
#[derive(Clone)]
pub struct ConnectionConfig {
    pub mode: ConnectionMode,
    pub distro: Option<String>,
}

/// 全局连接配置
pub static CONNECTION_CONFIG: LazyLock<RwLock<ConnectionConfig>> = LazyLock::new(|| {
    RwLock::new(ConnectionConfig {
        mode: ConnectionMode::Wsl,
        distro: None,
    })
});

/// 全局 Docker 客户端实例
static DOCKER_CLIENT: LazyLock<RwLock<Option<Docker>>> = LazyLock::new(|| RwLock::new(None));

/// 清除 Docker 客户端缓存，强制重新连接
pub async fn clear_client_cache() {
    let mut client_lock = DOCKER_CLIENT.write().await;
    *client_lock = None;
    // 重置 WSL 代理端口缓存
    wsl::reset_proxy_port().await;
}

/// 更新全局连接配置的命令
#[tauri::command]
pub async fn update_connection_config(mode: String, distro: Option<String>) {
    let mode_enum = ConnectionMode::from(mode);
    log::info!(
        "正在更新连接配置: mode={:?}, distro={:?}",
        mode_enum,
        distro
    );
    {
        let mut config = CONNECTION_CONFIG.write().await;
        config.mode = mode_enum;
        config.distro = distro;
    }
    // 配置改变后，必须清除客户端缓存以触发重新连接
    let mut client_lock = DOCKER_CLIENT.write().await;
    *client_lock = None;
}

/// 获取 Docker 客户端
pub async fn get_docker_client() -> AppResult<Docker> {
    // 1. 获取配置 (使用读锁)
    let (mode, distro) = {
        let config = CONNECTION_CONFIG.read().await;
        (config.mode, config.distro.clone())
    };

    // 2. 检查缓存 (先试读锁)
    {
        let client_lock = DOCKER_CLIENT.read().await;
        if let Some(client) = &*client_lock {
            return Ok(client.clone());
        }
    }

    // 3. 缓存不存在，获取写锁并创建客户端
    let mut client_lock = DOCKER_CLIENT.write().await;

    // Double-check pattern
    if let Some(client) = &*client_lock {
        return Ok(client.clone());
    }

    log::info!("正在尝试建立新的 Docker 连接...");

    // 根据配置选择连接方式
    if mode == ConnectionMode::Wsl {
        match wsl::WslBridge::new(distro).connect().await {
            Ok(docker) => {
                log::info!("Docker WSL 连接成功");
                *client_lock = Some(docker.clone());
                Ok(docker)
            }
            Err(e) => {
                // 如果 WSL 失败，回退到探测命名管道 (Windows 默认，兼容 Docker Desktop)
                #[cfg(windows)]
                {
                    log::info!("WSL 连接失败，尝试回退到命名管道...");
                    if let Ok(docker) = Docker::connect_with_named_pipe_defaults()
                        && docker.ping().await.is_ok()
                    {
                        log::info!("命名管道连接成功");
                        *client_lock = Some(docker.clone());
                        return Ok(docker);
                    }
                }
                log::error!("无法通过 WSL 连接到 Docker: {}", e);
                Err(format!("无法通过 WSL 连接到 Docker: {}", e).into())
            }
        }
    } else {
        // SSH 或其他模式暂未完全实现，回退到命名管道
        #[cfg(windows)]
        {
            log::info!("当前非 WSL 模式 ({:?})，尝试通过命名管道连接...", mode);
            if let Ok(docker) = Docker::connect_with_named_pipe_defaults()
                && docker.ping().await.is_ok()
            {
                log::info!("命名管道连接成功");
                *client_lock = Some(docker.clone());
                return Ok(docker);
            }
        }
        log::error!("当前连接模式暂未支持或无法连接到本地 Docker");
        Err("当前连接模式暂未支持或无法连接到本地 Docker"
            .to_string()
            .into())
    }
}

/// 轻量级 Docker 连通性测试命令
#[tauri::command]
pub async fn ping_docker() -> AppResult<()> {
    let docker = get_docker_client().await?;
    docker.ping().await?;
    Ok(())
}
