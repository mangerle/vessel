import { defineStore } from 'pinia'
import * as networkApi from '../api/network'
import type { NetworkInfo, NetworkDetails } from '../api/types'

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
        this.networks = await networkApi.listNetworks()
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
        this.currentNetwork = await networkApi.getNetworkDetails(id)
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
        await networkApi.removeNetwork(id)
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
        await networkApi.pruneNetworks()
        await this.fetchNetworks()
      } catch (err) {
        console.error('清理网络失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    /**
     * 断开容器网络连接
     * @param networkId 网络 ID
     * @param containerId 容器 ID
     */
    async disconnectContainer(networkId: string, containerId: string) {
      this.loading = true
      this.error = null
      try {
        await networkApi.disconnectNetwork(networkId, containerId)
        await this.fetchNetworkDetails(networkId)
      } catch (err) {
        console.error('断开网络连接失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    }
  }
})
