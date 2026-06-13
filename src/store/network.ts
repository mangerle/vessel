import { defineStore } from 'pinia'
import { ref } from 'vue'
import { networkApi } from '../api/network'
import { runStoreAction } from './helpers'
import type { NetworkInfo, NetworkDetails } from '../api/types'

/**
 * Docker 网络仓库（setup 风格）
 */
export const useNetworkStore = defineStore('network', () => {
  // 网络列表
  const networks = ref<NetworkInfo[]>([])
  // 当前网络详情
  const currentNetwork = ref<NetworkDetails | null>(null)
  // 加载状态
  const loading = ref(false)
  // 错误信息
  const error = ref<string | null>(null)

  // 适配 runStoreAction 的 LoadingErrorState：保持单一真值在 ref 上
  const loadingState = {
    get loading() { return loading.value },
    set loading(v: boolean) { loading.value = v },
    get error() { return error.value },
    set error(v: string | null) { error.value = v }
  }

  /** 刷新网络列表（runStoreAction 的 refresh 回调） */
  const refresh = async () => {
    networks.value = await networkApi.list()
  }

  /** 获取网络列表 */
  const fetchNetworks = () =>
    runStoreAction(loadingState, '获取网络', refresh)

  /** 获取网络详情 */
  const fetchNetworkDetails = (id: string) =>
    runStoreAction(loadingState, '获取网络详情', async () => {
      currentNetwork.value = await networkApi.getDetails(id)
      return currentNetwork.value
    })

  /** 删除网络 */
  const removeNetwork = (id: string) =>
    runStoreAction(loadingState, '删除网络', () => networkApi.remove(id), refresh)

  /** 清理未使用的网络 */
  const pruneNetworks = () =>
    runStoreAction(loadingState, '清理网络', () => networkApi.prune(), refresh)

  /** 断开容器网络连接 */
  const disconnectContainer = (networkId: string, containerId: string) =>
    runStoreAction(loadingState, '断开网络连接', async () => {
      await networkApi.disconnect(networkId, containerId)
      currentNetwork.value = await networkApi.getDetails(networkId)
    })

  return {
    networks,
    currentNetwork,
    loading,
    error,
    refresh,
    fetchNetworks,
    fetchNetworkDetails,
    removeNetwork,
    pruneNetworks,
    disconnectContainer
  }
})
