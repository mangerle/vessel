import { invoke } from '@tauri-apps/api/core'
import type { ContainerInfo, ContainerDetails } from './types'

/**
 * 容器相关的 API 服务
 */
export const containerApi = {
  /**
   * 获取本地容器列表
   */
  list: () => invoke<ContainerInfo[]>('list_local_containers'),

  /**
   * 启动容器
   */
  start: (id: string) => invoke<void>('start_container', { id }),

  /**
   * 停止容器
   */
  stop: (id: string) => invoke<void>('stop_container', { id }),

  /**
   * 重启容器
   */
  restart: (id: string) => invoke<void>('restart_container', { id }),

  /**
   * 删除容器
   */
  remove: (id: string) => invoke<void>('remove_container', { id }),

  /**
   * 暂停容器
   */
  pause: (id: string) => invoke<void>('pause_container', { id }),

  /**
   * 恢复容器
   */
  unpause: (id: string) => invoke<void>('unpause_container', { id }),

  /**
   * 获取容器详情
   */
  inspect: (id: string) => invoke<ContainerDetails>('inspect_container', { id })
}
