import { invoke } from '@tauri-apps/api/core'
import type { NetworkInfo, NetworkDetails } from './types'

/**
 * 获取网络列表
 */
export async function listNetworks(): Promise<NetworkInfo[]> {
  return await invoke<NetworkInfo[]>('list_networks')
}

/**
 * 获取网络详情
 * @param id 网络 ID
 */
export async function getNetworkDetails(id: string): Promise<NetworkDetails> {
  return await invoke<NetworkDetails>('get_network_details', { id })
}

/**
 * 删除网络
 * @param id 网络 ID
 */
export async function removeNetwork(id: string): Promise<void> {
  await invoke('remove_network', { id })
}

/**
 * 清理未使用的网络
 */
export async function pruneNetworks(): Promise<void> {
  await invoke('prune_networks')
}

/**
 * 断开容器网络连接
 * @param networkId 网络 ID
 * @param containerId 容器 ID
 */
export async function disconnectNetwork(networkId: string, containerId: string): Promise<void> {
  await invoke('disconnect_network', { networkId, containerId })
}
