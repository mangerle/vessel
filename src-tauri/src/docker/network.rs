use crate::connection::get_docker_client;
use bollard::network::InspectNetworkOptions;
use super::{NetworkInfo, NetworkDetails, ConnectedContainer};

/// 获取网络列表
#[tauri::command]
pub async fn list_networks() -> Result<Vec<NetworkInfo>, String> {
    let docker = get_docker_client().await?;
    let networks = docker
        .list_networks::<String>(None)
        .await
        .map_err(|e| format!("无法获取网络列表: {}", e))?;

    Ok(networks
        .into_iter()
        .map(|n| NetworkInfo {
            id: n.id.unwrap_or_default(),
            name: n.name.unwrap_or_default(),
            driver: n.driver.unwrap_or_default(),
            scope: n.scope.unwrap_or_default(),
            created: n.created.unwrap_or_default(),
        })
        .collect())
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

    let containers = network
        .containers
        .unwrap_or_default()
        .into_iter()
        .map(|(container_id, details)| ConnectedContainer {
            id: container_id,
            name: details.name.unwrap_or_default(),
            ipv4_address: details.ipv4_address.unwrap_or_default(),
            ipv6_address: details.ipv6_address.unwrap_or_default(),
            mac_address: details.mac_address.unwrap_or_default(),
        })
        .collect();

    let (subnet, gateway) = network
        .ipam
        .and_then(|ipam| ipam.config)
        .and_then(|config| config.first().cloned())
        .map(|cfg| {
            (
                cfg.subnet.unwrap_or_else(|| "N/A".to_string()),
                cfg.gateway.unwrap_or_else(|| "N/A".to_string()),
            )
        })
        .unwrap_or_else(|| ("N/A".to_string(), "N/A".to_string()));

    Ok(NetworkDetails {
        id: network.id.unwrap_or_default(),
        name: network.name.unwrap_or_default(),
        driver: network.driver.unwrap_or_default(),
        scope: network.scope.unwrap_or_default(),
        created: network.created.unwrap_or_default(),
        internal: network.internal.unwrap_or_default(),
        attachable: network.attachable.unwrap_or_default(),
        ingress: network.ingress.unwrap_or_default(),
        subnet,
        gateway,
        containers,
        options: network.options.unwrap_or_default(),
        labels: network.labels.unwrap_or_default(),
    })
}
