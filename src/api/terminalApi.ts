import { invoke } from '@tauri-apps/api/core'
import { CMD } from './commands'

/**
 * 容器内交互式终端 API 服务
 * 后端命令集中在 `src-tauri/src/docker/terminal.rs`。
 */

/** 创建终端返回的 exec id */
export type TerminalExecId = string

/** TTY 终端的列/行尺寸 */
export interface TerminalSize {
  cols: number
  rows: number
}

export const terminalApi = {
  /**
   * 创建容器内的伪终端连接，返回 exec id
   *
   * 入参 `user` 是 UI 层的身份选项：
   * - `'root'`：以 root 身份进入容器（容器内必须存在 root 账户）
   * - `'default'`：不向 Docker 指定 user，沿用镜像/容器自身定义的默认用户
   *   （字符串 `'default'` 只是 UI 标签，并不是真实的 Linux 账号，
   *    若原样下发给后端会导致 Docker 在 /etc/passwd 中找不到该用户而报 400）
   */
  create: (id: string, user: 'root' | 'default' = 'default') =>
    invoke<TerminalExecId>(CMD.createContainerTerminal, {
      id,
      user: user === 'root' ? 'root' : undefined,
    }),

  /**
   * 向已建立的终端写入字节（前端键盘输入）
   */
  write: (execId: TerminalExecId, data: number[]) =>
    invoke<void>(CMD.writeToTerminal, { execId, data }),

  /**
   * 调整 TTY 终端的列/行尺寸
   */
  resize: (execId: TerminalExecId, size: TerminalSize) =>
    invoke<void>(CMD.resizeContainerTerminal, { execId, cols: size.cols, rows: size.rows }),

  /**
   * 关闭已建立的终端连接
   */
  close: (execId: TerminalExecId) =>
    invoke<void>(CMD.closeContainerTerminal, { execId })
}
