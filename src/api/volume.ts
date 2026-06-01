import { invoke } from '@tauri-apps/api/core'
import type { VolumeInfo, VolumeUser } from './types'

/**
 * 数据卷相关的 API 服务
 */
export const volumeApi = {
  /**
   * 获取数据卷列表
   */
  list: () => invoke<VolumeInfo[]>('list_volumes'),

  /**
   * 获取使用特定卷的容器
   * @param name 卷名称
   */
  listContainers: (name: string) => invoke<VolumeUser[]>('list_volume_containers', { name }),

  /**
   * 在文件管理器中打开卷路径
   * @param path 卷路径
   */
  openPath: (path: string) => invoke<void>('open_volume_path', { path }),

  /**
   * 删除数据卷
   * @param name 卷名称
   */
  remove: (name: string) => invoke<void>('remove_volume', { name }),

  /**
   * 清理未使用的数据卷
   */
  prune: () => invoke<void>('prune_volumes')
}
