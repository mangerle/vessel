import { defineStore } from 'pinia'
import { ref } from 'vue'
import { volumeApi } from '../api/volume'
import { runStoreAction } from './helpers'
import type { VolumeInfo, VolumeUser } from '../api/types'

/**
 * Docker 数据卷仓库（setup 风格）
 */
export const useVolumeStore = defineStore('volume', () => {
  // 数据卷列表
  const volumes = ref<VolumeInfo[]>([])
  // 当前选中卷的使用者列表
  const volumeUsers = ref<VolumeUser[]>([])
  // 加载状态
  const loading = ref(false)
  // 错误信息
  const error = ref<string | null>(null)

  const loadingState = {
    get loading() { return loading.value },
    set loading(v: boolean) { loading.value = v },
    get error() { return error.value },
    set error(v: string | null) { error.value = v }
  }

  /** 刷新数据卷列表（runStoreAction 的 refresh 回调） */
  const refresh = async () => {
    volumes.value = await volumeApi.list()
  }

  /** 获取数据卷列表 */
  const fetchVolumes = () =>
    runStoreAction(loadingState, '获取数据卷', refresh)

  /** 获取使用特定卷的容器 */
  const fetchVolumeUsers = (name: string) =>
    runStoreAction(loadingState, '获取卷使用者', async () => {
      volumeUsers.value = await volumeApi.listContainers(name)
    })

  /** 在文件管理器中打开卷路径 */
  const openPath = (path: string) =>
    runStoreAction(loadingState, '打开卷路径', () => volumeApi.openPath(path))

  /** 删除数据卷 */
  const removeVolume = (name: string) =>
    runStoreAction(loadingState, '删除卷', () => volumeApi.remove(name), refresh)

  /** 清理未使用的数据卷 */
  const pruneVolumes = () =>
    runStoreAction(loadingState, '清理卷', () => volumeApi.prune(), refresh)

  return {
    volumes,
    volumeUsers,
    loading,
    error,
    refresh,
    fetchVolumes,
    fetchVolumeUsers,
    openPath,
    removeVolume,
    pruneVolumes
  }
})
