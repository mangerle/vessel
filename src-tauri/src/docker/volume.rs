use crate::handle_docker_op;
use super::{VolumeInfo, VolumeUser};
use crate::connection::get_docker_client;
use crate::error::AppResult;
use bollard::container::ListContainersOptions;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

/// 获取卷列表
#[tauri::command]
pub async fn list_volumes() -> AppResult<Vec<VolumeInfo>> {
    let docker = get_docker_client().await?;
    let response = docker.list_volumes::<String>(None).await?;

    let volumes = response.volumes.unwrap_or_default();
    Ok(volumes.into_iter().map(VolumeInfo::from).collect())
}

/// 删除卷
#[tauri::command]
pub async fn remove_volume(name: String) -> AppResult<()> {
    log::info!("正在删除卷: {}", name);
    let docker = get_docker_client().await?;
    handle_docker_op!("删除卷", name, docker.remove_volume(&name, None))
}

/// 清理未使用的卷
#[tauri::command]
pub async fn prune_volumes() -> AppResult<()> {
    log::info!("正在清理未使用的卷...");
    let docker = get_docker_client().await?;
    handle_docker_op!("卷清理", "所有未使用的卷", docker.prune_volumes::<String>(None))
}

/// 获取使用特定卷的容器列表
#[tauri::command]
pub async fn list_volume_containers(name: String) -> AppResult<Vec<VolumeUser>> {
    let docker = get_docker_client().await?;

    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;

    let inspect_futures = containers.into_iter().filter_map(|c| {
        let id = c.id?;
        let docker_clone = docker.clone();
        Some(async move {
            let result = docker_clone.inspect_container(&id, None).await;
            (id, result)
        })
    });

    let inspect_results = futures_util::future::join_all(inspect_futures).await;
    let mut users = Vec::new();

    for (id, result) in inspect_results {
        if let Ok(details) = result {
            if let Some(mounts) = details.mounts {
                for mount in mounts {
                    if mount.name.as_deref() == Some(&name)
                        || mount.source.as_deref() == Some(&name)
                    {
                        users.push(VolumeUser {
                            container_id: id.clone(),
                            container_name: details
                                .name
                                .clone()
                                .unwrap_or_default()
                                .trim_start_matches('/')
                                .to_string(),
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
pub async fn open_volume_path(app: AppHandle, path: String) -> AppResult<()> {
    log::info!("正在文件管理器中打开路径: {}", path);
    handle_docker_op!("打开路径", path, async { app.opener().open_path(&path, None::<String>) })
}
