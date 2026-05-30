use super::{NetworkDetails, NetworkInfo};
use crate::connection::get_docker_client;
use crate::error::AppResult;
use bollard::network::InspectNetworkOptions;

/// 获取网络列表
#[tauri::command]
pub async fn list_networks() -> AppResult<Vec<NetworkInfo>> {
    let docker = get_docker_client().await?;
    let networks = docker.list_networks::<String>(None).await?;

    Ok(networks.into_iter().map(NetworkInfo::from).collect())
}

/// 删除网络
#[tauri::command]
pub async fn remove_network(id: String) -> AppResult<()> {
    log::info!("正在删除网络: {}", id);
    let docker = get_docker_client().await?;
    match docker.remove_network(&id).await {
        Ok(_) => {
            log::info!("网络 {} 删除成功", id);
            Ok(())
        }
        Err(e) => {
            log::error!("删除网络 {} 失败: {}", id, e);
            Err(e.into())
        }
    }
}

/// 清理未使用的网络
#[tauri::command]
pub async fn prune_networks() -> AppResult<()> {
    log::info!("正在清理未使用的网络...");
    let docker = get_docker_client().await?;
    match docker.prune_networks::<String>(None).await {
        Ok(_) => {
            log::info!("网络清理完成");
            Ok(())
        }
        Err(e) => {
            log::error!("网络清理失败: {}", e);
            Err(e.into())
        }
    }
}

/// 断开网络连接
#[tauri::command]
pub async fn disconnect_network(network_id: String, container_id: String) -> AppResult<()> {
    log::info!("正在从网络 {} 断开容器 {}", network_id, container_id);
    let docker = get_docker_client().await?;
    match docker
        .disconnect_network(
            &network_id,
            bollard::network::DisconnectNetworkOptions {
                container: container_id.clone(),
                force: false,
            },
        )
        .await
    {
        Ok(_) => {
            log::info!("容器 {} 已成功从网络 {} 断开", container_id, network_id);
            Ok(())
        }
        Err(e) => {
            log::error!(
                "从网络 {} 断开容器 {} 失败: {}",
                network_id,
                container_id,
                e
            );
            Err(e.into())
        }
    }
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
