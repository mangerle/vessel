import { invoke } from '@tauri-apps/api/core'
import { CMD } from './commands'
import type { ContainerInfo, ContainerDetails } from './types'

/**
 * 容器相关的 API 服务
 */
export const containerApi = {
  /**
   * 获取本地容器列表
   */
  list: () => invoke<ContainerInfo[]>(CMD.listLocalContainers),

  /**
   * 启动容器
   */
  start: (id: string) => invoke<void>(CMD.startContainer, { id }),

  /**
   * 停止容器
   */
  stop: (id: string) => invoke<void>(CMD.stopContainer, { id }),

  /**
   * 重启容器
   */
  restart: (id: string) => invoke<void>(CMD.restartContainer, { id }),

  /**
   * 删除容器
   */
  remove: (id: string) => invoke<void>(CMD.removeContainer, { id }),

  /**
   * 暂停容器
   */
  pause: (id: string) => invoke<void>(CMD.pauseContainer, { id }),

  /**
   * 恢复容器
   */
  unpause: (id: string) => invoke<void>(CMD.unpauseContainer, { id }),

  /**
   * 获取容器详情
   */
  inspect: (id: string) => invoke<ContainerDetails>(CMD.inspectContainer, { id }),

  /** 重命名容器 */
  rename: (id: string, newName: string) =>
    invoke<void>(CMD.renameContainer, { id, newName }),

  /** 提交容器为新镜像，返回新镜像 ID */
  commit: (id: string, repo: string, tag: string, comment: string, author: string) =>
    invoke<string>(CMD.commitContainer, { id, repo, tag, comment, author }),

  /** 关闭 stats 流（按容器 id） */
  closeStats: (id: string) => invoke<void>(CMD.closeContainerStats, { id }),

  /** 启动 stats 流（按容器 id） */
  streamStats: (id: string) => invoke<void>(CMD.streamContainerStats, { id }),

  /** 关闭 logs 流（按容器 id） */
  closeLogs: (id: string) => invoke<void>(CMD.closeContainerLogs, { id }),

  /** 启动 logs 流（按容器 id） */
  streamLogs: (id: string) => invoke<void>(CMD.streamContainerLogs, { id }),

  /** 获取容器进程 Top */
  top: (id: string) =>
    invoke<{ titles: string[]; processes: string[][] }>(CMD.topContainer, { id }),

  /** 容器内 exec 单次命令 */
  exec: (id: string, cmd: string) =>
    invoke<{ exit_code: number | null; output: string }>(CMD.execContainer, { id, cmd })
}
