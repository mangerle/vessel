# Vessel

Vessel 是一款为开发者打造的极致简约、Docker 管理工具。它专为 WSL (Windows Subsystem for Linux)
环境优化，旨在提供一种如空气般轻盈、无感的容器管理体验。

## 🌟 核心理念

- **极致简约**：移除一切冗余的 UI 元素，只保留最核心的功能。没有 Logo，没有复杂的设置，只有你的容器。
- **专为 WSL 而生**：内置高效的 WSL 桥接技术，无需繁琐的端口映射或配置，开箱即用。
- **原生触感**：采用类 macOS 的设计语言，提供流畅、直观的交互体验。

## ✨ 主要功能

- **🚀 Docker Compose 集成**：
    - 自动识别并管理 Compose 项目。
    - 支持一键 Up / Down / Restart。
    - 内置 YAML 编辑器，实时修改并保存配置。
- **📦 容器管理**：
    - 实时状态监控（CPU、内存、网络、IO 可视化图表）。
    - 交互式终端支持（支持普通用户与 Root 用户切换）。
    - 实时日志流查看。
- **🖼️ 镜像与资源**：
    - 镜像快速拉取与清理。
    - 网络、数据卷的列表展示与一键瘦身（Prune）。
- **🔌 零配置连接**：
    - 自动探测并连接 WSL 中的 Docker 守护进程。
    - 兼容 Docker Desktop 命名管道。

## 🛠️ 技术栈

- **后端**：[Rust](https://www.rust-lang.org/) + [Tauri v2](https://v2.tauri.app/)
- **Docker 驱动**：[Bollard](https://github.com/fussybeaver/bollard)
- **前端**：[Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/)
- **UI 组件**：[Naive UI](https://www.naiveui.com/)
- **状态管理**：[Pinia](https://pinia.vuejs.org/)
- **数据可视化**：[Apache ECharts](https://echarts.apache.org/)
- **数据库**：[SQLite](https://www.sqlite.org/) (通过 SQLx)

## 🚀 快速开始

### 预备条件

- Windows 10/11 (推荐)
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://v2.tauri.app/reference/cli/) (`cargo install tauri-cli`)

### 安装与运行

1.  **克隆仓库**
    ```bash
    git clone https://github.com/your-username/vessel.git
    cd vessel
    ```

2.  **安装依赖**
    ```bash
    npm install
    ```

3.  **启动开发环境**
    ```bash
    npm run tauri dev
    ```

4.  **构建应用**
    ```bash
    npm run tauri build
    ```

## 🏗️ 架构说明

Vessel 采用了一种独特的 **WSL 桥接模式**。在 Windows 环境下，它不会直接尝试连接 TCP 端口，而是通过 `wsl docker system dial-stdio` 与 Linux 内部的 Docker 守护进程建立双向流。这种方式比传统的 TCP 转发更安全、更快速，且能完美处理权限问题。

## 📄 开源协议

本项目采用 [MIT](LICENSE) 协议开源。
