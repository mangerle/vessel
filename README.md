<p align="center">
  <img src="public/logo-with-name.svg" alt="Vessel" width="380" />
</p>

<p align="center">
  <a href="README_zh.md">简体中文</a> | <b>English</b>
</p>

<p align="center">
  <a href="https://github.com/tauri-apps/tauri"><img src="https://img.shields.io/badge/Tauri-v2-blue.svg?logo=tauri" alt="Tauri v2" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024_Edition-orange.svg?logo=rust" alt="Rust" /></a>
  <a href="https://vuejs.org/"><img src="https://img.shields.io/badge/Vue-3-brightgreen.svg?logo=vue.js" alt="Vue 3" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License" /></a>
</p>

Vessel is an ultra-minimalist, amphibious desktop Docker management client crafted for modern developers. Optimized specifically for WSL (Windows Subsystem for Linux) and featuring comprehensive multi-node remote SSH deployment, Vessel delivers a featherlight, blazing-fast, and distraction-free container management experience.

---

## Core Philosophy

- **Geek Aesthetic**: Dark futuristic visual design eliminating redundant abstractions of traditional desktop software, focusing purely on real-time container metrics and smooth ergonomics.
- **Amphibious Integration**: Built-in local WSL pipe bridging alongside remote multi-node SSH engines. Effortlessly pivot between local development environments and remote servers within a unified UI.
- **Secure Updates**: Fully signed end-to-end auto-update pipeline supporting hot-updates and instant one-click restarts.

---

## Key Features

- **Amphibious Docker Engine Switching**:
  - **WSL Pipe Sideloading**: Automatic detection and direct streaming communication via `wsl docker system dial-stdio` with host Linux subsystems (e.g., Ubuntu, Debian) without port forwarding.
  - **Remote SSH Connections**: Store and manage SSH credentials for multiple remote Linux hosts; activate and switch engines with a single click from the sidebar.
  - **Docker Desktop Compatibility**: Fully compatible with native Windows Named Pipe connections.
- **Docker Compose Project-Level Management**:
  - Automatically discover and orchestrate multi-container Compose projects.
  - One-click Up / Down / Restart with real-time YAML editing.
- **Granular Container Monitoring**:
  - Real-time system load metrics (interactive charts for CPU, Memory, Network I/O, and Disk I/O).
  - Interactive Terminal (quick switching between normal user and root) alongside real-time live log streams.
- **Image & Registry Management**:
  - Image listing, rapid pull, and one-click pruning.
  - Rapid configuration and persistent storage for third-party image mirrors and private Harbor registries.
- **Signed In-App Hot Updates**:
  - Integrated native update detection showing cloud changelogs and release dates.
  - Glowing gradient progress indicator and seamless one-click relaunch upon installation.
- **Automated CI/CD Releases**:
  - Full GitHub Actions pipeline building, signing, and deploying tagged releases automatically.

---

## Tech Stack

- **Backend**: [Rust](https://www.rust-lang.org/) + [Tauri v2](https://v2.tauri.app/) (Multi-platform desktop framework)
- **Docker Engine Driver**: [Bollard](https://github.com/fussybeaver/bollard)
- **Auto Update**: `tauri-plugin-updater` & `tauri-plugin-process`
- **Frontend**: [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/)
- **UI Components**: [Naive UI](https://www.naiveui.com/) + [@vicons/ionicons5](https://github.com/07akioni/xicons)
- **State Management**: [Pinia](https://pinia.vuejs.org/)
- **Data Visualization**: [Apache ECharts](https://echarts.apache.org/) (bridged with vue-echarts)
- **Configuration Persistence**: `tauri-plugin-store` (Lightweight flat key-value persistence)

---

## Getting Started

### Prerequisites

- Windows 10/11
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://v2.tauri.app/reference/cli/) (`cargo install tauri-cli`)

### Installation & Development

1. **Clone the repository**
   ```bash
   git clone https://github.com/mangerle/vessel.git
   cd vessel
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Start development mode**
   ```bash
   npm run tauri dev
   ```

4. **Build release package**
   ```bash
   npm run tauri build
   ```

---

## Release & Updates

This project integrates `standard-version` and GitHub Actions for an automated versioning and cloud release pipeline.

### Local Release Workflow

To release a new version locally:

1. **Bump version**:
   Run the following command in the project root:
   ```bash
   npm run release
   ```
   This command automatically:
   - Scans commits following Conventional Commits to calculate the next semantic version.
   - Synchronizes version numbers across `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.
   - Creates a local Git commit and tags it (e.g., `v0.1.4`).

   *Tip: To specify a version manually, use:*
   ```bash
   npm run release -- --release-as 0.1.5
   ```

2. **Push tags to trigger CI**:
   ```bash
   git push --follow-tags
   ```

### Cloud CI Build & Changelog Generation

When GitHub receives a new tag push, `.github/workflows/release.yml` triggers automatically:
- **Categorized Changelog Generation**: CI parses commit history via `git-cliff` to generate categorized release notes automatically.
- **Artifact Packaging & Upload**: Automatically builds, signs, and publishes installation packages to the repository Releases.

---

## Architecture Overview

Vessel adopts a dedicated **WSL Bridging Mode**. On Windows, instead of connecting over raw TCP ports, it communicates with the Linux Docker daemon via `wsl docker system dial-stdio`. This provides enhanced security, reduced latency, and resolves permission boundaries naturally. In remote mode, it establishes direct bidirectional streaming to the Docker socket over SSH, enabling an agentless multi-node container management experience.

---

## License

This project is open-sourced under the [MIT](LICENSE) License.
