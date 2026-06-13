use crate::error::AppResult;
use bollard::Docker;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tauri::{AppHandle, Emitter};
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

    /// 校验配置自身的完整性，按 mode 检查必要字段。
    ///
    /// 修复 P1-9：原 update_connection_config 直接吞下任意 ConnectionConfig，
    /// SSH 模式下空 host/user 也会被原样写入全局状态，要等到第一次 ping_docker 才暴露问题，
    /// 排查路径长且会导致 docker 客户端缓存被污染（被迫整体重连）。
    pub fn validate(&self) -> Result<(), String> {
        match self.mode {
            ConnectionMode::Wsl => {
                // WSL distro 可空（fallback 到默认发行版），不强制校验
                Ok(())
            }
            ConnectionMode::Ssh => {
                let host = self.ssh_host.as_deref().unwrap_or("").trim();
                if host.is_empty() {
                    return Err("SSH 主机地址不能为空".to_string());
                }
                let user = self.ssh_user.as_deref().unwrap_or("").trim();
                if user.is_empty() {
                    return Err("SSH 用户名不能为空".to_string());
                }
                if let Some(port) = self.ssh_port
                    && port == 0
                {
                    return Err("SSH 端口非法".to_string());
                }
                Ok(())
            }
            ConnectionMode::Desktop => Ok(()),
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
///
/// 幂等：仅当新配置与当前配置不一致时才清空客户端缓存与各模式代理。
/// 这样可以避免 App.vue 启动时 / 重复调用 ping_docker 等场景下误关连接。
/// 配置实际变更时通过 emit("connection-updated", &config) 通知所有前端实例
/// 刷新 activeConnection，避免多窗口/托盘切换时前后端失同步。
#[tauri::command]
pub async fn update_connection_config(app: AppHandle, config: ConnectionConfig) -> AppResult<()> {
    // 修复 P1-9：进入即校验输入，避免坏配置写入全局并连带污染客户端缓存
    if let Err(e) = config.validate() {
        log::warn!("update_connection_config 校验失败: {}", e);
        return Err(crate::error::AppError::ConfigMissing(e));
    }

    let new_config = config;
    // 备份旧配置用于失败回滚（emit 失败本身不算致命，仅日志告警）
    let prev_config = {
        let guard = CONNECTION_CONFIG.read().await;
        if *guard == new_config {
            log::debug!("连接配置未变化，跳过客户端缓存清理");
            return Ok(());
        }
        guard.clone()
    };
    log::info!(
        "正在更新连接配置: mode={:?}, name={}, distro={:?}, ssh={:?}@{:?}:{:?}",
        new_config.mode,
        new_config.name,
        new_config.wsl_distro,
        new_config.ssh_user,
        new_config.ssh_host,
        new_config.ssh_port
    );
    {
        let mut guard = CONNECTION_CONFIG.write().await;
        *guard = new_config.clone();
    }
    // 配置改变后，清空客户端缓存与各模式代理
    clear_client_cache().await;
    // 通知前端刷新 activeConnection（多窗口/托盘切换场景同步通道）
    if let Err(e) = app.emit("connection-updated", &new_config) {
        log::error!("发送 connection-updated 事件失败: {}", e);
        // emit 失败仅告警；下方 prev_config 引用保留供未来扩展失败回滚链路时复用
        let _ = &prev_config;
    }
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
            // 修复 P1-12：先校验 ConnectionConfig 自身（host/user 必填），
            // 再组装 SshConfig；早 fail 比让 SshConfig::validate 报「主机地址为空」
            // 路径更短，且在 SSH 分支里立刻给出 ConfigMissing 错误码。
            if let Err(e) = config.validate() {
                return Err(crate::error::AppError::ConfigMissing(e));
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ssh_config(host: &str, user: &str) -> ConnectionConfig {
        ConnectionConfig {
            mode: ConnectionMode::Ssh,
            name: "test".into(),
            wsl_distro: None,
            ssh_host: Some(host.into()),
            ssh_port: Some(22),
            ssh_user: Some(user.into()),
            ssh_password: Some("p".into()),
            use_sudo: false,
        }
    }

    #[test]
    fn validate_ssh_requires_host_and_user() {
        let mut cfg = ssh_config("192.168.1.1", "root");
        assert!(cfg.validate().is_ok(), "完整 SSH 配置应通过校验");

        cfg.ssh_host = Some("".into());
        assert!(cfg.validate().is_err(), "空 host 应拒绝");

        cfg.ssh_host = Some("   ".into());
        assert!(cfg.validate().is_err(), "全空白 host 应拒绝");

        cfg.ssh_host = Some("192.168.1.1".into());
        cfg.ssh_user = Some("".into());
        assert!(cfg.validate().is_err(), "空 user 应拒绝");
    }

    #[test]
    fn validate_ssh_rejects_zero_port() {
        let mut cfg = ssh_config("h", "u");
        cfg.ssh_port = Some(0);
        assert!(cfg.validate().is_err(), "零端口应拒绝");
    }

    #[test]
    fn validate_wsl_and_desktop_pass_through() {
        let wsl = ConnectionConfig {
            mode: ConnectionMode::Wsl,
            name: "wsl".into(),
            wsl_distro: None, // 允许为空
            ssh_host: None,
            ssh_port: None,
            ssh_user: None,
            ssh_password: None,
            use_sudo: false,
        };
        assert!(wsl.validate().is_ok(), "WSL distro 可选");

        let desktop = ConnectionConfig::desktop_default();
        assert!(desktop.validate().is_ok(), "Desktop 模式默认通过");
    }

    #[test]
    fn connection_mode_from_string_lowercases_and_falls_back_to_desktop() {
        assert_eq!(ConnectionMode::from("WSL".to_string()), ConnectionMode::Wsl);
        assert_eq!(ConnectionMode::from("ssh".to_string()), ConnectionMode::Ssh);
        assert_eq!(
            ConnectionMode::from("garbage".to_string()),
            ConnectionMode::Desktop
        );
    }
}
