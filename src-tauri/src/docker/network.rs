use crate::connection::get_docker_client;
use crate::error::AppResult;
use bollard::network::InspectNetworkOptions;
use super::{NetworkInfo, NetworkDetails};

/// 获取网络列表
#[tauri::command]
pub async fn list_networks() -> AppResult<Vec<NetworkInfo>> {
    let docker = get_docker_client().await?;
    let networks = docker
        .list_networks::<String>(None)
        .await?;

    Ok(networks.into_iter().map(NetworkInfo::from).collect())
}

/// 删除网络
#[tauri::command]
pub async fn remove_network(id: String) -> AppResult<()> {
    let docker = get_docker_client().await?;
    docker
        .remove_network(&id)
        .await?;
    Ok(())
}

/// 清理未使用的网络
#[tauri::command]
pub async fn prune_networks() -> AppResult<()> {
    let docker = get_docker_client().await?;
    docker
        .prune_networks::<String>(None)
        .await?;
    Ok(())
}

/// 断开网络连接
#[tauri::command]
pub async fn disconnect_network(network_id: String, container_id: String) -> AppResult<()> {
    let docker = get_docker_client().await?;
    docker
        .disconnect_network(
            &network_id,
            bollard::network::DisconnectNetworkOptions {
                container: container_id,
                force: false,
            },
        )
        .await?;
    Ok(())
}

/// 获取网络详情
#[tauri::command]
pub async fn get_network_details(id: String) -> AppResult<NetworkDetails> {
    let docker = get_docker_client().await?;
    let network = docker
        .inspect_network(&id, None::<InspectNetworkOptions<String>>)
        .await?;

    Ok(NetworkDetails::from(network))
}
