use serde::Serialize;
use std::collections::HashMap;

pub mod container;
pub mod image;
pub mod volume;
pub mod network;
pub mod compose;
pub mod terminal;
pub mod fs;

pub use container::*;
pub use image::*;
pub use volume::*;
pub use network::*;
pub use compose::*;
pub use terminal::*;
pub use fs::*;

/// 容器信息结构体
#[derive(Serialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub state: String,
    pub image: String,
    pub compose_project: Option<String>,
}

/// 镜像信息结构体
#[derive(Serialize)]
pub struct ImageInfo {
    pub id: String,
    pub tags: Vec<String>,
    pub size: i64,
    pub created: i64,
}

/// 镜像搜索结果结构体
#[derive(Serialize)]
pub struct ImageSearchResult {
    pub name: String,
    pub description: String,
    pub is_official: bool,
    pub star_count: i64,
}

/// 镜像历史信息结构体
#[derive(Serialize)]
pub struct ImageHistoryInfo {
    pub id: String,
    pub created: i64,
    pub created_by: String,
    pub size: i64,
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

/// 已连接的容器信息
#[derive(Serialize)]
pub struct ConnectedContainer {
    pub id: String,
    pub name: String,
    pub ipv4_address: String,
    pub ipv6_address: String,
    pub mac_address: String,
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

/// 卷信息结构体
#[derive(Serialize)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created: String,
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

/// 挂载信息结构体
#[derive(Serialize)]
pub struct MountInfo {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
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

/// 清理无用的虚悬镜像结果结构体
#[derive(Serialize)]
pub struct PruneImagesResult {
    pub deleted_count: usize,
    pub space_reclaimed: i64,
}
