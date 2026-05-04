import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

/**
 * Docker 网络信息接口
 */
interface NetworkInfo {
  id: string
  name: string
  driver: string
  scope: string
  created: string
}

/**
 * 已连接的容器信息
 */
export interface ConnectedContainer {
  id: string
  name: string
  ipv4_address: string
  ipv6_address: string
  mac_address: string
}

/**
 * 网络详情接口
 */
export interface NetworkDetails {
  id: string
  name: string
  driver: string
  scope: string
  created: string
  internal: boolean
  attachable: boolean
  ingress: boolean
  subnet: string
  gateway: string
  containers: ConnectedContainer[]
  options: Record<string, string>
  labels: Record<string, string>
}

/**
 * Docker 网络仓库
 */
export const useNetworkStore = defineStore('network', {
  state: () => ({
    // 网络列表
    networks: [] as NetworkInfo[],
    // 当前网络详情
    currentNetwork: null as NetworkDetails | null,
    // 加载状态
    loading: false,
    // 错误信息
    error: null as string | null
  }),
  actions: {
    /**
     * 获取网络列表
     */
    async fetchNetworks() {
      this.loading = true
      this.error = null
      try {
        this.networks = await invoke<NetworkInfo[]>('list_networks')
      } catch (err) {
        console.error('获取网络列表失败:', err)
        this.error = String(err)
      } finally {
        this.loading = false
      }
    },
    /**
     * 获取网络详情
     * @param id 网络 ID
     */
    async fetchNetworkDetails(id: string) {
      this.loading = true
      this.error = null
      try {
        this.currentNetwork = await invoke<NetworkDetails>('get_network_details', { id })
        return this.currentNetwork
      } catch (err) {
        console.error('获取网络详情失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    /**
     * 删除网络
     * @param id 网络 ID
     */
    async removeNetwork(id: string) {
      this.loading = true
      this.error = null
      try {
        await invoke('remove_network', { id })
        await this.fetchNetworks()
      } catch (err) {
        console.error('删除网络失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    /**
     * 清理未使用的网络
     */
    async pruneNetworks() {
      this.loading = true
      this.error = null
      try {
        await invoke('prune_networks')
        await this.fetchNetworks()
      } catch (err) {
        console.error('清理网络失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    }
  }
})
