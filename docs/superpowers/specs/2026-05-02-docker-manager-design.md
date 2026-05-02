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

## 5. UI/UX 设计
- **侧边栏 (Sidebar)**:
  - 顶部：服务器/连接切换器。
  - 中部：导航菜单（概览、容器、镜像、Compose、网络、卷）。
  - 底部：设置与连接状态指示。
- **容器详情页**:
  - **控制栏**：启动/停止/重启/删除。
  - **多页签**：
    - **日志**：实时日志输出。
    - **终端**：交互式 Shell。
    - **监控**：CPU、内存、网络图表。
    - **详情**：环境变量、端口映射、挂载卷等。

## 6. 实现路线图

### 第一阶段：基础设施 (MVP)
- Tauri 项目初始化。
- SQLite 数据库集成与连接配置管理。
- 实现本地 Docker API 桥接及基础容器列表显示。

### 第二阶段：核心功能增强
- 实现 SSH 隧道逻辑，支持管理远程 Docker。
- 集成 Xterm.js，完成容器 Exec 功能。
- 实现基础的日志流查看。

### 第三阶段：Compose 与监控
- 开发 Compose 混合管理模式。
- 集成 ECharts，实现资源采样与可视化展示。
- 完善镜像、网络及卷的管理功能。

### 第四阶段：优化与发布
- 增强 SSH 密钥管理安全性。
- 优化大数据量下的日志滚动性能。
- 跨平台打包与发布。
