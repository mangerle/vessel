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
