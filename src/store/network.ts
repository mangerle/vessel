import { defineStore } from 'pinia'
import { networkApi } from '../api/network'
import { runStoreAction } from './helpers'
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
    /** 刷新网络列表（runStoreAction 的 refresh 回调） */
    async refresh() {
      this.networks = await networkApi.list()
    },
    /**
     * 获取网络列表
     */
    async fetchNetworks() {
      await runStoreAction(this, '获取网络', () => this.refresh())
    },
    /**
     * 获取网络详情
     * @param id 网络 ID
     */
    async fetchNetworkDetails(id: string) {
      return await runStoreAction(this, '获取网络详情', async () => {
        this.currentNetwork = await networkApi.getDetails(id)
        return this.currentNetwork
      })
    },
    /**
     * 删除网络
     * @param id 网络 ID
     */
    async removeNetwork(id: string) {
      await runStoreAction(this, '删除网络', () => networkApi.remove(id), () => this.refresh())
    },
    /**
     * 清理未使用的网络
     */
    async pruneNetworks() {
      await runStoreAction(this, '清理网络', () => networkApi.prune(), () => this.refresh())
    },
    /**
     * 断开容器网络连接
     * @param networkId 网络 ID
     * @param containerId 容器 ID
     */
    async disconnectContainer(networkId: string, containerId: string) {
      await runStoreAction(this, '断开网络连接', async () => {
        await networkApi.disconnect(networkId, containerId)
        this.currentNetwork = await networkApi.getDetails(networkId)
      })
    }
  }
})
