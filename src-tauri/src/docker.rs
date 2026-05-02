use bollard::Docker;
use bollard::container::ListContainersOptions;
use serde::Serialize;

/// 容器信息结构体
#[derive(Serialize)]
pub struct ContainerInfo {
    /// 容器 ID
    pub id: String,
    /// 容器名称
    pub name: String,
    /// 容器状态 (如: running, exited)
    pub state: String,
    /// 镜像名称
    pub image: String,
}

/// 获取本地 Docker 容器列表的命令
#[tauri::command]
pub async fn list_local_containers() -> Result<Vec<ContainerInfo>, String> {
    // 使用本地默认配置连接 Docker (Windows 上通常是命名管道，Linux 上是 Unix Socket)
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("无法连接到 Docker: {}", e))?;

    // 列出所有容器 (包括未运行的)
    let containers = docker.list_containers(Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    })).await.map_err(|e| format!("无法获取容器列表: {}", e))?;
    
    // 转换为前端友好的格式
    Ok(containers.into_iter().map(|c| ContainerInfo {
        id: c.id.unwrap_or_default(),
        // c.names 通常以 ["/container_name"] 格式返回，所以我们取第一个并去掉开头的斜杠
        name: c.names.as_ref()
            .and_then(|names| names.first())
            .map(|name| name.trim_start_matches('/').to_string())
            .unwrap_or_else(|| "未知".to_string()),
        state: c.state.unwrap_or_default(),
        image: c.image.unwrap_or_default(),
    }).collect())
}
