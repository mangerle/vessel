/**
 * 前后端事件名常量（前后端协议）
 *
 * 任何对事件名的修改必须前后端同步；新增事件也应在此处集中定义。
 * 后端对应常量见 `src-tauri/src/docker/utils.rs::events`。
 *
 * 同时定义所有事件 payload 的 TS 类型，listen 泛型化后即可获得完整类型推导。
 */

import type { PullProgress } from './types'

export const EVT = {
  // 容器统计/日志流（按容器 id 区分频道）
  containerStats: (id: string) => `container-stats-${id}`,
  containerLogs: (id: string) => `container-logs-${id}`,

  // 终端 exec 流
  containerTerminalStdout: (execId: string) => `container-terminal-stdout-${execId}`,
  containerTerminalExit: (execId: string) => `container-terminal-exit-${execId}`,

  // 镜像拉取/导出/导入进度
  imagePullProgress: 'image-pull-progress',
  imagePullError: 'image-pull-error',
  imagePullFinished: 'image-pull-finished',
  imageExportProgress: 'image-export-progress',
  imageExportFinished: 'image-export-finished',
  imageExportError: 'image-export-error',
  imageImportProgress: 'image-import-progress',
  imageImportFinished: 'image-import-finished',
  imageImportError: 'image-import-error',

  // Compose 命令执行
  composeCmdOutput: 'compose-cmd-output',
  composeCmdFinished: 'compose-cmd-finished',
  composeCmdError: 'compose-cmd-error',

  // 单实例 / 全局
  singleInstanceDetected: 'single-instance-detected',

  // 连接配置变更通知（后端 update_connection_config 末尾 emit）
  connectionUpdated: 'connection-updated',
} as const

// ============= 事件 payload 类型 =============

export interface ImagePullProgressPayload {
  image: string
  info: PullProgress
}

export interface ImagePullErrorPayload {
  image: string
  error: string
}

export type ImagePullFinishedPayload = string

export interface ImageExportProgressPayload {
  image: string
  bytes_written: number
}

export interface ImageExportErrorPayload {
  image: string
  error: string
}

export type ImageExportFinishedPayload = string

export interface ImageImportProgressPayload {
  path: string
  status?: string
  stream?: string
  error?: string
  progress?: string
}

export interface ImageImportErrorPayload {
  path: string
  error: string
}

export type ImageImportFinishedPayload = string

export type ComposeCmdOutputPayload = string
export type ComposeCmdFinishedPayload = void
export interface ComposeCmdErrorPayload {
  error: string
}

