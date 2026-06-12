use crate::error::AppResult;
use bollard::Docker;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tokio::sync::RwLock;

pub mod ssh;
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
            _ => ConnectionMode::Desktop,
        }
    }
}

/// 统一的连接配置结构体（前后端共用形状）
///
/// 同一份配置里同时携带 WSL 与 SSH 的可选字段，按 `mode` 取用对应子集。
/// `name` 仅用于前端展示与日志，后端按 mode 决定行为。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub mode: ConnectionMode,
    #[serde(default)]
    pub name: String,

    // WSL 专用
    #[serde(default)]
    pub wsl_distro: Option<String>,

    // SSH 专用
    #[serde(default)]
    pub ssh_host: Option<String>,
    #[serde(default)]
    pub ssh_port: Option<u16>,
    #[serde(default)]
    pub ssh_user: Option<String>,
    #[serde(default)]
    pub ssh_password: Option<String>,
    /// 是否使用 sudo 提升权限调用 docker（远端用户不在 docker 组时使用）
    #[serde(default)]
    pub use_sudo: bool,
}

impl ConnectionConfig {
    /// 构造一个默认的本地桌面连接配置
    pub fn desktop_default() -> Self {
        Self {
            mode: ConnectionMode::Desktop,
            name: "Docker Desktop".to_string(),
            wsl_distro: None,
            ssh_host: None,
            ssh_port: None,
            ssh_user: None,
            ssh_password: None,
            use_sudo: false,
        }
    }
}

/// 全局活动连接配置
pub static CONNECTION_CONFIG: LazyLock<RwLock<ConnectionConfig>> = LazyLock::new(|| {
    RwLock::new(ConnectionConfig {
        mode: ConnectionMode::Wsl,
        name: "WSL".to_string(),
        wsl_distro: None,
        ssh_host: None,
        ssh_port: None,
        ssh_user: None,
        ssh_password: None,
        use_sudo: false,
    })
});

/// 全局 Docker 客户端实例
static DOCKER_CLIENT: LazyLock<RwLock<Option<Docker>>> = LazyLock::new(|| RwLock::new(None));

/// 清除 Docker 客户端缓存并关闭各模式代理，强制重新连接
pub async fn clear_client_cache() {
    let mut client_lock = DOCKER_CLIENT.write().await;
    *client_lock = None;
    wsl::reset_proxy_port().await;
    ssh::reset_proxy().await;
}

/// 获取当前活动连接配置的快照
pub async fn current_config() -> ConnectionConfig {
    CONNECTION_CONFIG.read().await.clone()
}

/// 更新全局活动连接配置
#[tauri::command]
pub async fn update_connection_config(config: ConnectionConfig) -> AppResult<()> {
    log::info!(
        "正在更新连接配置: mode={:?}, name={}, distro={:?}, ssh={:?}@{:?}:{:?}",
        config.mode,
        config.name,
        config.wsl_distro,
        config.ssh_user,
        config.ssh_host,
        config.ssh_port
    );
    {
        let mut guard = CONNECTION_CONFIG.write().await;
        *guard = config;
    }
    // 配置改变后，清空客户端缓存与各模式代理
    clear_client_cache().await;
    Ok(())
}

/// 获取 Docker 客户端（按当前活动连接配置分派）
pub async fn get_docker_client() -> AppResult<Docker> {
    // 1. 检查缓存
    {
        let client_lock = DOCKER_CLIENT.read().await;
        if let Some(client) = &*client_lock {
            return Ok(client.clone());
        }
    }

    // 2. 缓存不存在，获取写锁
    let mut client_lock = DOCKER_CLIENT.write().await;

    // Double-check pattern
    if let Some(client) = &*client_lock {
        return Ok(client.clone());
    }

    // 3. 取出当前配置
    let config = CONNECTION_CONFIG.read().await.clone();
    log::info!("正在尝试建立新的 Docker 连接: {:?}", config.mode);

    let result: Result<Docker, String> = match config.mode {
        ConnectionMode::Wsl => {
            let bridge = wsl::WslBridge::new(config.wsl_distro.clone());
            bridge.connect().await.map_err(|e| e.to_string())
        }
        ConnectionMode::Ssh => {
            let ssh_config = ssh::SshConfig {
                host: config.ssh_host.clone().unwrap_or_default(),
                port: config.ssh_port.unwrap_or(22),
                user: config.ssh_user.clone().unwrap_or_default(),
                password: config.ssh_password.clone(),
                use_sudo: config.use_sudo,
            };
            let bridge = ssh::SshBridge::new(ssh_config);
            bridge.connect().await.map_err(|e| e.to_string())
        }
        ConnectionMode::Desktop => connect_desktop().await,
    };

    match result {
        Ok(docker) => {
            log::info!("Docker 连接成功: {:?}", config.mode);
            *client_lock = Some(docker.clone());
            Ok(docker)
        }
        Err(e) => {
            log::error!("Docker 连接失败 (mode={:?}): {}", config.mode, e);
            Err(e.into())
        }
    }
}

/// 探测并连接本地 Docker Desktop 命名管道
#[cfg(windows)]
async fn connect_desktop() -> Result<Docker, String> {
    use bollard::Docker;
    if let Ok(docker) = Docker::connect_with_named_pipe_defaults()
        && docker.ping().await.is_ok()
    {
        return Ok(docker);
    }
    Err("无法连接到本地 Docker Desktop 命名管道".to_string())
}

#[cfg(not(windows))]
async fn connect_desktop() -> Result<Docker, String> {
    Err("Desktop 模式仅在 Windows 平台受支持".to_string())
}

/// 轻量级 Docker 连通性测试命令
#[tauri::command]
pub async fn ping_docker() -> AppResult<()> {
    let docker = get_docker_client().await?;
    docker.ping().await?;
    Ok(())
}

/// 诊断 SSH 远端 Docker 环境：返回用户、组、socket 权限、sudo 状态及修复建议
#[tauri::command]
pub async fn diagnose_ssh_connection(config: ConnectionConfig) -> AppResult<ssh::SshDiagnostic> {
    let ssh_cfg = ssh::SshConfig {
        host: config
            .ssh_host
            .clone()
            .ok_or_else(|| "SSH 主机未配置".to_string())?,
        port: config.ssh_port.unwrap_or(22),
        user: config
            .ssh_user
            .clone()
            .ok_or_else(|| "SSH 用户未配置".to_string())?,
        password: config.ssh_password.clone(),
        use_sudo: config.use_sudo,
    };
    ssh_cfg.validate()?;
    let bridge = ssh::SshBridge::new(ssh_cfg);
    Ok(bridge.diagnose().await)
}
