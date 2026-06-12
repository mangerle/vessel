use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Docker 错误: {0}")]
    Docker(#[from] bollard::errors::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tauri 错误: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("Opener 错误: {0}")]
    Opener(#[from] tauri_plugin_opener::Error),

    #[error("未知错误: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("SSH 错误: {0}")]
    Ssh(#[from] russh::Error),

    /// SSH 远端命令执行失败（携带 stdout/stderr/exit_code 用于 UI 展示）
    #[error("远端命令退出码 {exit_code}：{stderr}")]
    RemoteCmd {
        exit_code: i32,
        stdout: String,
        stderr: String,
    },

    /// 远端命令启动阶段就失败（通道未建立）
    #[error("远端命令启动失败: {0}")]
    RemoteSpawn(String),

    /// WSL 桥接错误
    #[error("WSL 桥接错误: {0}")]
    Wsl(String),

    /// SSH 桥接错误（含鉴权、连接、协议错误）
    #[error("SSH 桥接错误: {0}")]
    SshBridge(String),

    /// 容器内文件系统操作错误
    #[error("容器文件系统错误: {0}")]
    ContainerFs(String),

    /// 远端环境配置缺失（例如 SSH 主机未配置）
    #[error("配置缺失: {0}")]
    ConfigMissing(String),

    /// 通用兜底错误（应被具体子类型替代）
    #[error("{0}")]
    Custom(String),
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Custom(s.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Custom(s)
    }
}

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
