# Vessel

Vessel 是一款为现代开发者打造的极致简约、两栖式 Docker 桌面容器管理客户端。它不仅专为 WSL (Windows Subsystem for Linux) 环境优化，更全面支持远程多节点 SSH 部署，旨在提供一种如空气般轻盈、极速、无感的容器管理体验。

## 🌟 核心理念

- **极客质感**：界面设计融合暗黑发光科技美学，移除了传统桌面软件的冗余抽象，追求最纯粹的容器数据呈现与流畅操作。
- **两栖融合**：内置 WSL 本地管道桥接与远程多节点 SSH 连接引擎，同一套界面，无缝在本地开发与远程测试环境中来回穿梭。
- **安全更新**：内置全通道加签自动更新机制，支持热升级与闪电式一键重启生效。

---

## ✨ 主要功能

- **🔌 两栖 Docker 引擎连接与切换**：
    - **WSL 管道侧载**：自动探测并使用 `wsl docker system dial-stdio` 与宿主机 Linux 子系统（如 Ubuntu、Debian 等）直连，免端口映射。
    - **远程 SSH 连接**：支持保存多台远程 Linux 服务器的 SSH 鉴权凭证，一键在侧边栏激活并切换对应的 Docker 引擎。
    - **Docker Desktop**：兼容 Windows 本地原生 Named Pipe (命名管道) 连接。
- **🚀 Docker Compose 项目级管理**：
    - 自动识别并管理多容器 Compose 项目。
    - 支持一键 Up / Down / Restart，内置实时 YAML 编辑。
- **📦 容器细粒度监控**：
    - 实时系统负载监控（CPU、内存、网络 IO、磁盘读写等可视化折线图表）。
    - 提供交互式 Terminal（支持普通用户/Root 快速切换）与实时日志流监控。
- **🖼️ 镜像与加速器管理**：
    - 镜像列表管理、快速 Pull 与镜像一键瘦身（Prune）。
    - 极速配置并持久化第三方镜像加速器、私有 Harbor 账户机密。
- **🔒 自动加签与真实热更新**：
    - 软件内集成真实更新机制，展示云端新版本的详细日志与日期。
    - 配备发光渐变进度条实时展示下载百分比，完成自愈安装后支持软重启（Relaunch）一秒装载新版。
- **🤖 GitHub Actions 自动化构建发布**：
    - 完整的 Actions CI/CD 流水线，当源码仓库（可为私有）发布版本 Tag 时，自动为安装包加签并跨仓库发布至公开的 Releases 分发仓库。

---

## 🛠️ 技术栈

- **后端**：[Rust](https://www.rust-lang.org/) + [Tauri v2](https://v2.tauri.app/) (多平台应用构建框架)
- **Docker 驱动**：[Bollard](https://github.com/fussybeaver/bollard)
- **自动更新**：`tauri-plugin-updater` & `tauri-plugin-process`
- **前端**：[Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/)
- **UI 组件**：[Naive UI](https://www.naiveui.com/) + [@vicons/ionicons5](https://github.com/07akioni/xicons)
- **状态管理**：[Pinia](https://pinia.vuejs.org/)
- **数据可视化**：[Apache ECharts](https://echarts.apache.org/) (使用 vue-echarts 桥接)
- **配置持久化**：`tauri-plugin-store` (轻量级本地键值对扁平持久化)

---

## 🚀 快速开始

### 预备条件

- Windows 10/11
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://v2.tauri.app/reference/cli/) (`cargo install tauri-cli`)

### 安装与运行

1.  **克隆仓库**
    ```bash
    git clone https://github.com/mangerle/vessel.git
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

---

## 🏗️ 架构说明

Vessel 采用了一种独特的 **WSL 桥接模式**。在 Windows 环境下，它不会直接尝试连接 TCP 端口，而是通过 `wsl docker system dial-stdio` 与 Linux 内部的 Docker 守护进程建立双向流。这种方式比传统的 TCP 转发更安全、更快速，且能完美处理权限问题。而在远程部署模式中，它会基于 SSH 协议打通与 Docker Socket 守护进程的连接，实现极简的免 agent 容器跨端控制。

---

## 📄 开源协议

本项目采用 [MIT](LICENSE) 协议开源。
