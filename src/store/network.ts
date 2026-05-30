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
    async executeAction(actionName: string, actionFn: () => Promise<any>, refresh: boolean = true) {
      this.loading = true
      this.error = null
      try {
        const result = await actionFn()
        if (refresh) {
          this.networks = await networkApi.listNetworks()
        }
        return result
      } catch (err) {
        console.error(`${actionName}失败:`, err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    /**
     * 获取网络列表
     */
    async fetchNetworks() {
      await this.executeAction('获取网络', async () => {}, true)
    },
    /**
     * 获取网络详情
     * @param id 网络 ID
     */
    async fetchNetworkDetails(id: string) {
      return await this.executeAction('获取网络详情', async () => {
        this.currentNetwork = await networkApi.getNetworkDetails(id)
        return this.currentNetwork
      }, false)
    },
    /**
     * 删除网络
     * @param id 网络 ID
     */
    async removeNetwork(id: string) {
      await this.executeAction('删除网络', () => networkApi.removeNetwork(id))
    },
    /**
     * 清理未使用的网络
     */
    async pruneNetworks() {
      await this.executeAction('清理网络', () => networkApi.pruneNetworks())
    },
    /**
     * 断开容器网络连接
     * @param networkId 网络 ID
     * @param containerId 容器 ID
     */
    async disconnectContainer(networkId: string, containerId: string) {
      await this.executeAction('断开网络连接', async () => {
        await networkApi.disconnectNetwork(networkId, containerId)
        this.currentNetwork = await networkApi.getNetworkDetails(networkId)
      }, false)
    }
  }
})
