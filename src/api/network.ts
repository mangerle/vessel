import { invoke } from '@tauri-apps/api/core'
import type { NetworkInfo, NetworkDetails } from './types'

/**
 * 网络相关的 API 服务
 */
export const networkApi = {
  /**
   * 获取网络列表
   */
  list: () => invoke<NetworkInfo[]>('list_networks'),

  /**
   * 获取网络详情
   * @param id 网络 ID
   */
  getDetails: (id: string) => invoke<NetworkDetails>('get_network_details', { id }),

  /**
   * 删除网络
   * @param id 网络 ID
   */
  remove: (id: string) => invoke<void>('remove_network', { id }),

  /**
   * 清理未使用的网络
   */
  prune: () => invoke<void>('prune_networks'),

  /**
   * 断开容器网络连接
   * @param networkId 网络 ID
   * @param containerId 容器 ID
   */
  disconnect: (networkId: string, containerId: string) =>
    invoke<void>('disconnect_network', { networkId, containerId })
}
