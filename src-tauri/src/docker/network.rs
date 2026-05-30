use crate::connection::get_docker_client;
use bollard::network::InspectNetworkOptions;
use super::{NetworkInfo, NetworkDetails};

/// 获取网络列表
#[tauri::command]
pub async fn list_networks() -> Result<Vec<NetworkInfo>, String> {
    let docker = get_docker_client().await?;
    let networks = docker
        .list_networks::<String>(None)
        .await
        .map_err(|e| format!("无法获取网络列表: {}", e))?;

    Ok(networks.into_iter().map(NetworkInfo::from).collect())
}

/// 删除网络
#[tauri::command]
pub async fn remove_network(id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .remove_network(&id)
        .await
        .map_err(|e| format!("删除网络失败: {}", e))
}

/// 清理未使用的网络
#[tauri::command]
pub async fn prune_networks() -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .prune_networks::<String>(None)
        .await
        .map_err(|e| format!("清理网络失败: {}", e))?;
    Ok(())
}

/// 断开网络连接
#[tauri::command]
pub async fn disconnect_network(network_id: String, container_id: String) -> Result<(), String> {
    let docker = get_docker_client().await?;
    docker
        .disconnect_network(
            &network_id,
            bollard::network::DisconnectNetworkOptions {
                container: container_id,
                force: false,
            },
        )
        .await
        .map_err(|e| format!("断开网络连接失败: {}", e))
}

/// 获取网络详情
#[tauri::command]
pub async fn get_network_details(id: String) -> Result<NetworkDetails, String> {
    let docker = get_docker_client().await?;
    let network = docker
        .inspect_network(&id, None::<InspectNetworkOptions<String>>)
        .await
        .map_err(|e| format!("无法获取网络详情: {}", e))?;

    Ok(NetworkDetails::from(network))
}
