use bollard::Docker;
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use crate::connection::manager::Connection;

pub mod wsl;
pub mod manager;

/// 全局 Docker 客户端实例
static DOCKER_CLIENT: Lazy<Arc<Mutex<Option<Docker>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

/// 清除 Docker 客户端缓存，强制重新连接
pub async fn clear_client_cache() {
    let mut client_lock = DOCKER_CLIENT.lock().await;
    *client_lock = None;
}

/// 获取 Docker 客户端
/// 优先使用用户选中的活动连接，如果没有则自动探测
pub async fn get_docker_client() -> Result<Docker, String> {
    let mut client_lock = DOCKER_CLIENT.lock().await;
    
    if let Some(client) = &*client_lock {
        return Ok(client.clone());
    }

    // 检查是否有活动连接 ID
    if let Ok(Some(active_id)) = manager::get_active_connection_id().await {
        let conns = manager::get_all_connections().await.map_err(|e| e.to_string())?;
        if let Some(conn) = conns.into_iter().find(|c| c.id == active_id) {
            let docker = match conn.driver.as_str() {
                "NamedPipe" => Docker::connect_with_named_pipe(&conn.host, 120, bollard::API_DEFAULT_VERSION)
                    .map_err(|e| format!("无法通过命名管道连接: {}", e))?,
                "Tcp" => Docker::connect_with_http(&conn.host, 120, bollard::API_DEFAULT_VERSION)
                    .map_err(|e| format!("无法通过 TCP 连接: {}", e))?,
                "WslBridge" => wsl::WslBridge::new(Some(conn.host)).connect().await?,
                _ => return Err(format!("不支持的驱动类型: {}", conn.driver)),
            };

            // 如果明确设置了活动连接，则不再回退到探测逻辑
            docker.ping().await.map_err(|e| format!("活动连接无法访问: {}", e))?;
            
            *client_lock = Some(docker.clone());
            return Ok(docker);
        }
    }

    // 回退到自动探测逻辑 (NamedPipe -> WslBridge)
    // 尝试命名管道 (Windows 默认)
    #[cfg(windows)]
    {
        if let Ok(docker) = Docker::connect_with_named_pipe_defaults() {
            if docker.ping().await.is_ok() {
                *client_lock = Some(docker.clone());
                return Ok(docker);
            }
        }
    }

    // 尝试 WSL 桥接
    match wsl::WslBridge::new(None).connect().await {
        Ok(docker) => {
            *client_lock = Some(docker.clone());
            Ok(docker)
        }
        Err(e) => Err(format!("无法连接到任何 Docker 驱动: {}", e)),
    }
}

// --- Tauri 命令 ---

#[tauri::command]
pub async fn cmd_add_connection(name: String, driver: String, host: String) -> Result<(), String> {
    let conn = Connection {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        driver,
        host,
        auth_config: None,
    };
    manager::add_connection(conn).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_connections() -> Result<Vec<Connection>, String> {
    manager::get_all_connections().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_delete_connection(id: String) -> Result<(), String> {
    manager::delete_connection(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_switch_connection(id: String) -> Result<(), String> {
    manager::set_active_connection_id(&id).await.map_err(|e| e.to_string())?;
    clear_client_cache().await;
    // 尝试预连接以验证
    get_docker_client().await.map(|_| ())
}

#[tauri::command]
pub async fn cmd_get_active_connection_id() -> Result<Option<String>, String> {
    manager::get_active_connection_id().await.map_err(|e| e.to_string())
}
