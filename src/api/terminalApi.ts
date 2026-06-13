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
   */
  create: (id: string, user: 'root' | 'default' = 'default') =>
    invoke<TerminalExecId>(CMD.createContainerTerminal, { id, user }),

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
