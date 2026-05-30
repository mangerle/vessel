use serde::Serialize;
use std::collections::HashMap;
use bollard::models::{
    ContainerSummary, ImageSummary, ImageSearchResponseItem, ImageHistoryResponseItem, 
    Network, Volume, ContainerInspectResponse, ImageInspect,
    NetworkContainer, PortBinding, MountPoint
};

/// 容器信息结构体
#[derive(Serialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub state: String,
    pub image: String,
    pub compose_project: Option<String>,
}

impl From<ContainerSummary> for ContainerInfo {
    fn from(c: ContainerSummary) -> Self {
        let compose_project = c.labels.as_ref()
            .and_then(|labels| labels.get("com.docker.compose.project").cloned());
        
        Self {
            id: c.id.unwrap_or_default(),
            name: c.names.as_ref()
                .and_then(|names| names.first())
                .map(|name| name.trim_start_matches('/').to_string())
                .unwrap_or_else(|| "未知".to_string()),
            state: c.state.unwrap_or_default(),
            image: c.image.unwrap_or_default(),
            compose_project,
        }
    }
}

/// 镜像信息结构体
#[derive(Serialize)]
pub struct ImageInfo {
    pub id: String,
    pub tags: Vec<String>,
    pub size: i64,
    pub created: i64,
}

impl From<ImageSummary> for ImageInfo {
    fn from(img: ImageSummary) -> Self {
        Self {
            id: img.id,
            tags: img.repo_tags,
            size: img.size,
            created: img.created,
        }
    }
}

/// 镜像搜索结果结构体
#[derive(Serialize)]
pub struct ImageSearchResult {
    pub name: String,
    pub description: String,
    pub is_official: bool,
    pub star_count: i64,
}

impl From<ImageSearchResponseItem> for ImageSearchResult {
    fn from(item: ImageSearchResponseItem) -> Self {
        Self {
            name: item.name.unwrap_or_default(),
            description: item.description.unwrap_or_default(),
            is_official: item.is_official.unwrap_or_default(),
            star_count: item.star_count.unwrap_or_default(),
        }
    }
}

/// 镜像历史信息结构体
#[derive(Serialize)]
pub struct ImageHistoryInfo {
    pub id: String,
    pub created: i64,
    pub created_by: String,
    pub size: i64,
}

impl From<ImageHistoryResponseItem> for ImageHistoryInfo {
    fn from(item: ImageHistoryResponseItem) -> Self {
        Self {
            id: item.id,
            created: item.created,
            created_by: item.created_by,
            size: item.size,
        }
    }
}

/// 网络信息结构体
#[derive(Serialize)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub created: String,
}

impl From<Network> for NetworkInfo {
    fn from(n: Network) -> Self {
        Self {
            id: n.id.unwrap_or_default(),
            name: n.name.unwrap_or_default(),
            driver: n.driver.unwrap_or_default(),
            scope: n.scope.unwrap_or_default(),
            created: n.created.unwrap_or_default(),
        }
    }
}

/// 已连接的容器信息
#[derive(Serialize)]
pub struct ConnectedContainer {
    pub id: String,
    pub name: String,
    pub ipv4_address: String,
    pub ipv6_address: String,
    pub mac_address: String,
}

impl From<(String, NetworkContainer)> for ConnectedContainer {
    fn from((id, details): (String, NetworkContainer)) -> Self {
        Self {
            id,
            name: details.name.unwrap_or_default(),
            ipv4_address: details.ipv4_address.unwrap_or_default(),
            ipv6_address: details.ipv6_address.unwrap_or_default(),
            mac_address: details.mac_address.unwrap_or_default(),
        }
    }
}

/// 网络详情结构体
#[derive(Serialize)]
pub struct NetworkDetails {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub created: String,
    pub internal: bool,
    pub attachable: bool,
    pub ingress: bool,
    pub subnet: String,
    pub gateway: String,
    pub containers: Vec<ConnectedContainer>,
    pub options: HashMap<String, String>,
    pub labels: HashMap<String, String>,
}

impl From<Network> for NetworkDetails {
    fn from(network: Network) -> Self {
        let containers = network
            .containers
            .unwrap_or_default()
            .into_iter()
            .map(ConnectedContainer::from)
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

        Self {
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
        }
    }
}

/// 卷信息结构体
#[derive(Serialize)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created: String,
}

impl From<Volume> for VolumeInfo {
    fn from(v: Volume) -> Self {
        Self {
            name: v.name,
            driver: v.driver,
            mountpoint: v.mountpoint,
            created: v.created_at.unwrap_or_default(),
        }
    }
}

/// Compose 项目结构体
#[derive(Serialize)]
pub struct ComposeProject {
    pub name: String,
    pub container_count: usize,
    pub running_count: usize,
    pub status: String,
    pub working_dir: Option<String>,
    pub config_file: Option<String>,
}

/// 端口映射结构体
#[derive(Serialize)]
pub struct PortMapping {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub type_: String,
    pub ip: Option<String>,
}

impl PortMapping {
    pub fn from_port_binding(private_port: u16, type_: String, b: &PortBinding) -> Self {
        Self {
            private_port,
            public_port: b.host_port.as_ref().and_then(|hp| hp.parse::<u16>().ok()),
            type_,
            ip: b.host_ip.clone(),
        }
    }
}

/// 挂载信息结构体
#[derive(Serialize)]
pub struct MountInfo {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
}

impl From<&MountPoint> for MountInfo {
    fn from(mi: &MountPoint) -> Self {
        Self {
            source: mi.source.clone().unwrap_or_default(),
            destination: mi.destination.clone().unwrap_or_default(),
            mode: mi.mode.clone().unwrap_or_default(),
            rw: mi.rw.unwrap_or_default(),
        }
    }
}

/// 卷使用容器信息
#[derive(Serialize)]
pub struct VolumeUser {
    pub container_id: String,
    pub container_name: String,
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
}

/// 容器详情结构体
#[derive(Serialize)]
pub struct ContainerDetails {
    pub id: String,
    pub name: String,
    pub image: String,
    pub image_id: String,
    pub state: String,
    pub status: String,
    pub created: String,
    pub env: Vec<String>,
    pub ports: Vec<PortMapping>,
    pub mounts: Vec<MountInfo>,
}

impl From<ContainerInspectResponse> for ContainerDetails {
    fn from(details: ContainerInspectResponse) -> Self {
        let config = details.config.as_ref();
        let network_settings = details.network_settings.as_ref();

        let ports = network_settings
            .and_then(|ns| ns.ports.as_ref())
            .map(|p| {
                p.iter()
                    .flat_map(|(k, v)| {
                        let parts: Vec<&str> = k.split('/').collect();
                        let private_port = parts[0].parse::<u16>().unwrap_or_default();
                        let type_ = parts.get(1).unwrap_or(&"tcp").to_string();

                        match v {
                            Some(bindings) => bindings
                                .iter()
                                .map(move |b| PortMapping::from_port_binding(private_port, type_.clone(), b))
                                .collect::<Vec<_>>(),
                            None => vec![PortMapping {
                                private_port,
                                public_port: None,
                                type_,
                                ip: None,
                            }],
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mounts = details
            .mounts
            .as_ref()
            .map(|m| m.iter().map(MountInfo::from).collect())
            .unwrap_or_default();

        Self {
            id: details.id.unwrap_or_default(),
            name: details
                .name
                .unwrap_or_default()
                .trim_start_matches('/')
                .to_string(),
            image: config.and_then(|c| c.image.clone()).unwrap_or_default(),
            image_id: details.image.unwrap_or_default(),
            state: details
                .state
                .as_ref()
                .and_then(|s| s.status)
                .map(|s| format!("{:?}", s).to_lowercase())
                .unwrap_or_default(),
            status: details
                .state
                .as_ref()
                .and_then(|s| s.status)
                .map(|s| format!("{:?}", s).to_lowercase())
                .unwrap_or_default(),
            created: details.created.unwrap_or_default(),
            env: config.and_then(|c| c.env.clone()).unwrap_or_default(),
            ports,
            mounts,
        }
    }
}

/// 镜像详情结构体
#[derive(Serialize)]
pub struct ImageDetails {
    pub id: String,
    pub tags: Vec<String>,
    pub size: i64,
    pub created: String,
    pub architecture: String,
    pub os: String,
    pub env: Vec<String>,
    pub exposed_ports: Vec<String>,
    pub cmd: Vec<String>,
    pub entrypoint: Vec<String>,
}

impl From<ImageInspect> for ImageDetails {
    fn from(details: ImageInspect) -> Self {
        let config = details.config.as_ref();

        Self {
            id: details.id.unwrap_or_default(),
            tags: details.repo_tags.unwrap_or_default(),
            size: details.size.unwrap_or_default(),
            created: details.created.unwrap_or_default(),
            architecture: details.architecture.unwrap_or_default(),
            os: details.os.unwrap_or_default(),
            env: config.and_then(|c| c.env.clone()).unwrap_or_default(),
            exposed_ports: config
                .and_then(|c| c.exposed_ports.as_ref())
                .map(|p| p.keys().cloned().collect())
                .unwrap_or_default(),
            cmd: config.and_then(|c| c.cmd.clone()).unwrap_or_default(),
            entrypoint: config.and_then(|c| c.entrypoint.clone()).unwrap_or_default(),
        }
    }
}

/// 清理无用的虚悬镜像结果结构体
#[derive(Serialize)]
pub struct PruneImagesResult {
    pub deleted_count: usize,
    pub space_reclaimed: i64,
}
