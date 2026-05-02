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
 * Docker 网络仓库
 */
export const useNetworkStore = defineStore('network', {
  state: () => ({
    // 网络列表
    networks: [] as NetworkInfo[],
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
     * 删除网络
     * @param id 网络 ID
     */
    async removeNetwork(id: string) {
      try {
        await invoke('remove_network', { id })
        await this.fetchNetworks()
      } catch (err) {
        console.error('删除网络失败:', err)
        throw err
      }
    },
    /**
     * 清理未使用的网络
     */
    async pruneNetworks() {
      try {
        await invoke('prune_networks')
        await this.fetchNetworks()
      } catch (err) {
        console.error('清理网络失败:', err)
        throw err
      }
    }
  }
})
