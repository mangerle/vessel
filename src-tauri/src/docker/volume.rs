use crate::connection::get_docker_client;
use bollard::container::ListContainersOptions;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use super::{VolumeInfo, VolumeUser};

/// 获取卷列表
#[tauri::command]
pub async fn list_volumes() -> Result<Vec<VolumeInfo>, String> {
    let docker = get_docker_client().await?;
    let response = docker
        .list_volumes::<String>(None)
        .await
        .map_err(|e| format!("无法获取卷列表: {}", e))?;

    let volumes = response.volumes.unwrap_or_default();
    Ok(volumes
        .into_iter()
        .map(|v| VolumeInfo {
            name: v.name,
            driver: v.driver,
            mountpoint: v.mountpoint,
            created: v.created_at.unwrap_or_default(),
        })
        .collect())
}

/// 删除卷
#[tauri::command]
pub async fn remove_volume(name: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .remove_volume(&name, None)
        .await
        .map_err(|e| format!("删除卷失败: {}", e))
}

/// 清理未使用的卷
#[tauri::command]
pub async fn prune_volumes() -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .prune_volumes::<String>(None)
        .await
        .map_err(|e| format!("清理卷失败: {}", e))?;
    Ok(())
}

/// 获取使用特定卷的容器列表
#[tauri::command]
pub async fn list_volume_containers(name: String) -> Result<Vec<VolumeUser>, String> {
    let docker = get_docker_client().await?;
    
    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
        .map_err(|e| format!("无法获取容器列表: {}", e))?;
    
    let mut users = Vec::new();
    
    for container in containers {
        if let Some(id) = container.id {
            let details = docker
                .inspect_container(&id, None)
                .await
                .map_err(|e| format!("无法获取容器详情 ({}): {}", id, e))?;
            
            if let Some(mounts) = details.mounts {
                for mount in mounts {
                    if mount.name.as_deref() == Some(&name) || mount.source.as_deref() == Some(&name) {
                        users.push(VolumeUser {
                            container_id: id.clone(),
                            container_name: details.name.clone().unwrap_or_default().trim_start_matches('/').to_string(),
                            source: mount.source.unwrap_or_default(),
                            destination: mount.destination.unwrap_or_default(),
                            mode: mount.mode.unwrap_or_default(),
                            rw: mount.rw.unwrap_or_default(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(users)
}

/// 在文件管理器中打开卷路径
#[tauri::command]
pub async fn open_volume_path(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("无法打开目录: {}", e))
}
