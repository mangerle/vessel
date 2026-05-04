# Docker Manager 优化：WSL 进程管理与镜像拉取 UX

## 1. 问题分析

### 1.1 WSL 进程泄露
目前 `WslBridge` 为每个 TCP 连接都启动一个 `wsl docker system dial-stdio` 进程。
*   **根因**：连接频繁（如 Stats 监控）导致进程数激增。虽然有 `tokio::io::copy` 后的 `child.kill()`，但如果连接半关闭或挂起，进程可能残留。
*   **方案**：实现连接池或减少连接频率。由于 Bollard/Docker API 本身是基于 HTTP 的，频繁建立/断开连接是常态。

### 1.2 镜像拉取 UX 差
*   **问题 1**：点击搜索结果直接拉取，缺乏二次确认。
*   **问题 2**：报错后进度条卡死，没有自动关闭或明确的错误提示。

---

## 2. 设计方案

### 2.1 WSL 连接池与生命周期优化
我们将优化 `src-tauri/src/connection/wsl.rs`：
1.  **进程复用 (可选)**：`dial-stdio` 是流式的，通常一个连接对应一个进程是标准做法，但我们需要更严谨的清理。
2.  **强制超时与 Drop 处理**：使用 `tokio::select!` 确保在 IO 拷贝结束、连接关闭或超时后，进程被立即杀死。
3.  **终端会话清理**：在 `docker.rs` 中，确保 `TERMINAL_SESSIONS` 在前端组件销毁或连接断开时，后端能感知并回收资源。

### 2.2 镜像拉取流程重构
1.  **确认弹窗**：点击搜索建议后，不立即执行 `pull_image`，而是弹出 `NModal`。
    *   展示内容：镜像名、Stars、是否官方、描述。
    *   操作：确认拉取 / 取消。
2.  **错误处理增强**：
    *   在 `useImageStore` 中增加 `pullError` 状态。
    *   当 `image-pull-error` 触发时，更新状态，进度条弹窗显示错误信息，并提供“关闭”或“重试”按钮。
    *   防止卡死：如果拉取流由于网络原因断开，设置超时逻辑。

---

## 3. 任务清单

- [ ] **Backend: WSL 进程加固**
  - 修改 `ensure_proxy` 中的循环，确保 `child` 进程在 `copy` 任务结束后被 100% 杀死。
  - 增加对 `Wait` 的处理，防止僵尸进程。
- [ ] **Backend: 镜像拉取错误透传**
  - 确保 `bollard` 返回的错误能完整传递给前端。
- [ ] **Frontend: 镜像确认拉取 Modal**
  - 在 `Images.vue` 中新增 `ConfirmPullModal`。
- [ ] **Frontend: 拉取状态机优化**
  - 优化 `pulling` 状态，报错时自动停止加载并显示错误。
  - 增加“手动关闭”进度弹窗的功能，无论成功还是失败。
