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

### 5.1 侧边栏导航 (Sidebar)
采用 Naive UI 的 `NLayoutSider`，包含以下菜单项：
1. **Docker 管理**: 连接切换与添加。
2. **Compose 管理**: 项目列表、服务拓扑。
3. **容器管理**: 运行状态、控制台。
4. **镜像管理**: 镜像列表、仓库拉取。
5. **网络管理**: 网络拓扑、配置查看。
6. **卷管理**: 挂载点管理、清理。

### 5.2 核心模块功能

| 模块 | 核心功能 | 统计信息 (Dashboard) |
| :--- | :--- | :--- |
| **Docker 管理** | 添加 (Local/WSL/SSH/TCP)、切换当前连接、健康检查 | 已连接服务器数 / 总连接数 |
| **Compose 管理** | 导入/探测项目、Up/Down/Restart、查看关联容器 | 运行项目数 / 总项目数 |
| **容器管理** | 启动/停止/删除、查看详情、Exec 终端 | 正在运行 / 总容器数 |
| **镜像管理** | 拉取 (Pull)、删除 (RMI)、查看层 (Layers)、创建容器 | 本地镜像总数 / 总占用空间 |
| **网络管理** | 创建、删除、查看容器绑定 | 网络总数 |
| **卷管理** | 创建、删除、一键清理 (Prune) | 卷总数 / 挂载点总数 |

### 5.3 监控与统计 (Monitoring)
- **详情页面**: 每个容器和 Compose 项目都有一个独立的监控视图。
- **数据指标**: CPU 使用率、内存占用、磁盘 I/O、网络流量。
- **实时性**: 通过 Tauri Event 每秒采样并推送至前端 ECharts。

## 6. 实现路线图

### 第二阶段：全功能模块开发 (当前目标)
1. **重构导航与布局**: 实现侧边栏与主工作区的多连接切换逻辑。
2. **容器与监控增强**: 实现容器控制（起/停/删）及 ECharts 实时图表。
3. **镜像与仓库管理**: 实现镜像拉取进度条显示及创建容器向导。
4. **网络与卷管理**: 实现基础的 CRUD 页面。
5. **Compose 深度集成**: 实现 Compose 项目与其下属容器的联动管理。

