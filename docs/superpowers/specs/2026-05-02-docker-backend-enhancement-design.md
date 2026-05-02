# 2026-05-02 后端数据采集与交互增强设计文档

## 目标
增强 Docker 管理器的后端功能，支持容器详情查看、日志流式传输以及交互式终端访问。

## 架构

### 1. 数据结构
在 `src-tauri/src/docker.rs` 中定义以下结构体：

```rust
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

#[derive(Serialize)]
pub struct PortMapping {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub type_: String,
    pub ip: Option<String>,
}

#[derive(Serialize)]
pub struct MountInfo {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
}
```

### 2. 状态管理
由于终端会话需要保持双向 IO 流，我们在 `docker.rs` 中引入全局会话管理器：

```rust
pub struct TerminalSession {
    pub stdin_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
}

static TERMINAL_SESSIONS: Lazy<Arc<Mutex<HashMap<String, TerminalSession>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
```

### 3. 后端命令 (Tauri Commands)

#### Inspect
- `inspect_container(id: String) -> Result<ContainerDetails, String>`
- 使用 `docker.inspect_container` 获取完整元数据并进行转换。

#### Logs
- `stream_container_logs(app: AppHandle, id: String) -> Result<(), String>`
- 启动后台任务监听 `docker.logs` 流。
- 发送 `container-logs-<id>` 事件，数据包含时间戳和原始文本。

#### Terminal Exec
- `create_container_terminal(app: AppHandle, id: String, user: Option<String>) -> Result<String, String>`
    - 依次执行 `exec_create` 和 `exec_start`。
    - 启动后台任务：
        - 从 Docker 读取输出并 `app.emit("container-terminal-stdout-<exec_id>", data)`。
        - 监听 `mpsc` 接收端并将数据写入 Docker stdin。
    - 返回 `exec_id` 作为会话标识。
- `write_to_terminal(exec_id: String, data: Vec<u8>) -> Result<(), String>`
    - 根据 `exec_id` 查找会话并发送数据。
- `resize_container_terminal(exec_id: String, height: u16, width: u16) -> Result<(), String>`
    - 调用 `docker.exec_resize` 动态调整终端大小。

## 错误处理
- 所有的 Docker 操作都应捕获错误并返回人类可读的字符串。
- 终端会话断开时，应自动从全局 Map 中移除并发送结束事件。
