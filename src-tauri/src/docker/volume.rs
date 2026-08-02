use super::{VolumeInfo, VolumeUser};
use crate::connection::get_docker_client;
use crate::error::AppResult;
use crate::handle_docker_op;
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
    handle_docker_op!(
        "卷清理",
        "所有未使用的卷",
        docker.prune_volumes::<String>(None)
    )
}

/// 获取使用特定卷的容器列表
/// 修复 P1-9：ContainerSummary 自身已带 mounts 字段，无需逐个 inspect_container。
/// 由 N 次 IPC（O(N) inspect）→ 0 次额外 IPC，仅一次 list_containers。
#[tauri::command]
pub async fn list_volume_containers(name: String) -> AppResult<Vec<VolumeUser>> {
    let docker = get_docker_client().await?;

    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;

    let mut users = Vec::new();

    for c in containers {
        let id = match c.id {
            Some(id) => id,
            None => continue,
        };
        let container_name = c
            .names
            .as_ref()
            .and_then(|names| names.first())
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_default();
        if let Some(mounts) = c.mounts {
            for mount in mounts {
                // 命名卷：mount.name 命中；bind mount：mount.source 命中
                if mount.name.as_deref() == Some(&name) || mount.source.as_deref() == Some(&name) {
                    users.push(VolumeUser {
                        container_id: id.clone(),
                        container_name: container_name.clone(),
                        source: mount.source.unwrap_or_default(),
                        destination: mount.destination.unwrap_or_default(),
                        mode: mount.mode.unwrap_or_default(),
                        rw: mount.rw.unwrap_or_default(),
                    });
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
    handle_docker_op!("打开路径", path, async {
        app.opener().open_path(&path, None::<String>)
    })
}
