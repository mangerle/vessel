//! 系统凭据库（Keychain / Credential Manager / Secret Service）封装。
//!
//! 修复 P0-3：原 settings.json 直存 SSH/Registry 密码明文，
//! 任何能访问用户家目录的进程或备份都能读到。改用 OS 凭据库：
//! - Windows: Credential Manager
//! - macOS:  Keychain
//! - Linux:  Secret Service / kwallet
//!
//! 前端通过 `set_secret/get_secret/delete_secret` 三条 invoke 命令操作；
//! 后端 `current_config()` 内的 `ssh_password` 字段仍正常使用，仅持久化路径改变。

use crate::error::{AppError, AppResult};

const SERVICE: &str = "vessel.app";

fn entry(key: &str) -> AppResult<keyring::Entry> {
    if key.is_empty() {
        return Err(AppError::Custom("secret key 不能为空".to_string()));
    }
    keyring::Entry::new(SERVICE, key).map_err(|e| AppError::Custom(format!("打开凭据条目失败: {}", e)))
}

/// 写入密钥；空字符串等价于删除（避免前端误把"清空密码"当成保存空串）
#[tauri::command]
pub async fn set_secret(key: String, value: String) -> AppResult<()> {
    let e = entry(&key)?;
    if value.is_empty() {
        // 兼容：旧条目可能存在，先删后置空
        let _ = e.delete_credential();
        return Ok(());
    }
    e.set_password(&value)
        .map_err(|err| AppError::Custom(format!("写入凭据失败: {}", err)))?;
    log::debug!("凭据已写入 ({})", key);
    Ok(())
}

/// 读取密钥；不存在返回 None（前端按需 fallback）
#[tauri::command]
pub async fn get_secret(key: String) -> AppResult<Option<String>> {
    let e = entry(&key)?;
    match e.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(AppError::Custom(format!("读取凭据失败: {}", err))),
    }
}

/// 删除密钥；不存在视为成功
#[tauri::command]
pub async fn delete_secret(key: String) -> AppResult<()> {
    let e = entry(&key)?;
    match e.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(AppError::Custom(format!("删除凭据失败: {}", err))),
    }
}
