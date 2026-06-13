import { defineStore } from 'pinia'
import { ref } from 'vue'
import { containerApi } from '../api/container'
import { runStoreAction } from './helpers'
import type { ContainerInfo } from '../api/types'

/**
 * 容器仓库（setup 风格）
 *
 * 与 settings.ts 的 setup 风格保持一致；loading/error 字段直接以 ref 暴露，
 * runStoreAction 通过 .value 获取 / 写入。
 */
export const useContainerStore = defineStore('container', () => {
  const containers = ref<ContainerInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // runStoreAction 期望的 LoadingErrorState 形状：
  // 由于 ref<boolean> 没有可写的 .value getter/setter 可直接绑定，
  // 这里用 Proxy 视图保留单一真值（loading/error 仍是 ref）。
  const loadingState = {
    get loading() { return loading.value },
    set loading(v: boolean) { loading.value = v },
    get error() { return error.value },
    set error(v: string | null) { error.value = v }
  }

  /** 刷新容器列表（被 runStoreAction 作为 refresh 回调注入） */
  const refresh = async () => {
    containers.value = await containerApi.list()
  }

  const fetchContainers = () =>
    runStoreAction(loadingState, '获取容器', refresh)

  const startContainer = (id: string) =>
    runStoreAction(loadingState, '启动容器', () => containerApi.start(id), refresh)

  const stopContainer = (id: string) =>
    runStoreAction(loadingState, '停止容器', () => containerApi.stop(id), refresh)

  const restartContainer = (id: string) =>
    runStoreAction(loadingState, '重启容器', () => containerApi.restart(id), refresh)

  const removeContainer = (id: string) =>
    runStoreAction(loadingState, '删除容器', () => containerApi.remove(id), refresh)

  const pauseContainer = (id: string) =>
    runStoreAction(loadingState, '暂停容器', () => containerApi.pause(id), refresh)

  const unpauseContainer = (id: string) =>
    runStoreAction(loadingState, '恢复容器', () => containerApi.unpause(id), refresh)

  /**
   * 批量操作：仅执行动作，不触发逐次 list_containers 刷新。
   * 调用方应在批结束后自行调用 fetchContainers() 同步一次即可，
   * 避免 N 个容器操作 × 1 次 list 的 N+1 IPC 放大。
   */
  const batchStart = async (ids: string[]) => {
    if (ids.length === 0) return
    await Promise.all(ids.map(id =>
      runStoreAction(loadingState, '启动容器', () => containerApi.start(id))
    ))
  }
  const batchStop = async (ids: string[]) => {
    if (ids.length === 0) return
    await Promise.all(ids.map(id =>
      runStoreAction(loadingState, '停止容器', () => containerApi.stop(id))
    ))
  }
  const batchRemove = async (ids: string[]) => {
    if (ids.length === 0) return
    await Promise.all(ids.map(id =>
      runStoreAction(loadingState, '删除容器', () => containerApi.remove(id))
    ))
  }

  return {
    containers,
    loading,
    error,
    refresh,
    fetchContainers,
    startContainer,
    stopContainer,
    restartContainer,
    removeContainer,
    pauseContainer,
    unpauseContainer,
    batchStart,
    batchStop,
    batchRemove
  }
})
