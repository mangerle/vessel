# Docker Manager 设计文档 (Spec)

## 1. 项目概述
一款基于 Rust + Tauri + Vue 3 开发的桌面 Docker 管理工具，支持本地、WSL 及通过 SSH 隧道连接的远程 Docker 实例管理。

## 2. 核心目标
- **跨平台连接**：支持本地 Unix Socket、Windows Named Pipe 及远程 SSH 隧道。
- **混合式 Compose 管理**：既能自动识别现有的 Compose 项目，也支持导入本地配置文件进行持久化管理。
- **集成交互**：提供容器内部终端 (Exec) 和实时日志流。
- **实时监控**：图形化展示容器的 CPU、内存及网络资源占用。

## 3. 技术栈
- **后端 (Rust)**:
  - **Tauri**: 跨平台桌面框架。
  - **Bollard**: 异步 Docker API 客户端。
  - **Tokio**: 异步运行时。
  - **SQLx + SQLite**: 持久化存储服务器配置及 Compose 项目元数据。
  - **russh/ssh2-rs**: SSH 隧道建立。
- **前端 (Vue 3)**:
  - **Vite**: 构建工具。
  - **Pinia**: 状态管理。
  - **Naive UI / Element Plus**: 组件库。
  - **Xterm.js**: 终端集成。
  - **ECharts (vue-echarts)**: 资源监控图表。

## 4. 架构设计

### 4.1 多驱动连接层 (Docker Connection Drivers)
为了支持不同环境，系统采用驱动抽象层：
- **`NativePipe`**: 连接 Windows 默认管道 (`//./pipe/docker_engine`)。
- **`WslBridge`**: 通过 `wsl docker system dial-stdio` 桥接 WSL 内部 Docker。
- **`UnixSocket`**: 连接 Linux/macOS 默认 Socket (`/var/run/docker.sock`)。
- **`Tcp`**: 连接远程或本地 TCP 端口 (2375)。
- **`Ssh`**: 通过 SSH 隧道执行远程 `dial-stdio`。

### 4.2 自动探测逻辑 (Auto-Discovery)
应用启动时按以下优先级探测：
1. **Windows**: `NativePipe` -> `WslBridge`。
2. **Linux/macOS**: `UnixSocket`。
探测成功的连接将标记为“Auto-detected”并作为默认连接。

### 4.3 后端模块
- **Connection Manager**: 管理 Docker 上下文。负责驱动切换、SSH 隧道维护、凭据加密存储及连接健康检查。
- **Docker Engine Bridge**: 统一的 Docker 指令封装，基于当前的活动驱动 (Active Driver) 进行 API 调用。
- **Compose Service**: 处理 `docker-compose` 指令封装，解析项目依赖及标签映射。
- **Stream Dispatcher**: 负责将容器日志流和性能采样数据通过 Tauri Event 广播至前端。

### 4.2 数据库架构 (SQLite)
- **`connections`**:
  - `id`: UUID
  - `name`: 连接名称
  - `driver`: `local`, `ssh`, `wsl`
  - `host`: 远程地址或 Socket 路径
  - `auth_config`: 加密存储的 SSH 密钥或密码
- **`compose_projects`**:
  - `id`: UUID
  - `connection_id`: 关联的连接
  - `name`: 项目名称
  - `working_dir`: 配置文件目录
  - `config_path`: `docker-compose.yml` 路径

## 5. UI/UX 深度交互设计

### 5.1 Compose 页面布局 (嵌套双栏)
在全局侧边栏右侧的主内容区中，Compose 页面进一步划分为两部分：
- **左侧子栏 (Sub-Sider)**:
  - **项目/容器树**: 采用 `NTree` 或自定义列表。
  - **视觉标识**: 运行中项目/容器使用绿色图标，停止则使用灰白色图标。
- **右侧详情区 (Main Detail)**:
  - **顶部操作区**: 显示选中容器的名称、ID、镜像，以及重启、停止、终端三个快捷按钮。
  - **多标签页 (NTabs)**:
    - **日志**: 实时日志流。
    - **设置**: 环境变量、端口、卷配置。
    - **仪表盘**: 实时资源监控图表。

### 5.2 右键菜单 (Context Menu)
针对容器列表项，右键弹出菜单：
- **重启容器**
- **暂停容器**
- **复制容器 ID**
- **复制镜像 ID**
- **创建终端**:
  - 作为容器用户 (Standard User)
  - 作为 Root 用户
- **删除**

### 5.3 终端功能 (Terminal)
- **技术实现**: 集成 `xterm.js`。
- **通信桥接**: 后端通过 `bollard::exec` 开启交互流，利用 Tauri 事件系统进行输入输出的双向转发。
- **权限切换**: 根据用户选择，执行 `docker exec -u <user>`。

## 6. 实现路线图

### 第三阶段：深度交互与终端集成 (当前目标)
1. **重构 Compose 视图布局**: 实现双栏结构与状态图标。
2. **增强容器元数据采集**: 实现 `inspect_container` 获取环境变量、端口和卷。
3. **实现容器详情标签页**: 迁移仪表盘，增加设置页。
4. **集成 Xterm.js 终端**: 完成后端 Exec 逻辑与前端终端组件对接。
5. **实现右键上下文菜单**: 绑定各项快捷操作。

