import { defineStore } from 'pinia'
import { containerApi } from '../api/container'
import { runStoreAction } from './helpers'
import type { ContainerInfo } from '../api/types'

export const useContainerStore = defineStore('container', {
  state: () => ({
    containers: [] as ContainerInfo[],
    loading: false,
    error: null as string | null
  }),
  actions: {
    /** 刷新容器列表（被 runStoreAction 作为 refresh 回调注入） */
    async refresh() {
      this.containers = await containerApi.list()
    },
    async fetchContainers() {
      await runStoreAction(this, '获取容器', () => this.refresh())
    },
    async startContainer(id: string) {
      await runStoreAction(this, '启动容器', () => containerApi.start(id), () => this.refresh())
    },
    async stopContainer(id: string) {
      await runStoreAction(this, '停止容器', () => containerApi.stop(id), () => this.refresh())
    },
    async restartContainer(id: string) {
      await runStoreAction(this, '重启容器', () => containerApi.restart(id), () => this.refresh())
    },
    async removeContainer(id: string) {
      await runStoreAction(this, '删除容器', () => containerApi.remove(id), () => this.refresh())
    },
    async pauseContainer(id: string) {
      await runStoreAction(this, '暂停容器', () => containerApi.pause(id), () => this.refresh())
    },
    async unpauseContainer(id: string) {
      await runStoreAction(this, '恢复容器', () => containerApi.unpause(id), () => this.refresh())
    },
    /**
     * 批量操作：仅执行动作，不触发逐次 list_containers 刷新。
     * 调用方应在批结束后自行调用 fetchContainers() 同步一次即可，
     * 避免 N 个容器操作 × 1 次 list 的 N+1 IPC 放大。
     */
    async batchStart(ids: string[]) {
      if (ids.length === 0) return
      await Promise.all(ids.map(id => runStoreAction(this, '启动容器', () => containerApi.start(id))))
    },
    async batchStop(ids: string[]) {
      if (ids.length === 0) return
      await Promise.all(ids.map(id => runStoreAction(this, '停止容器', () => containerApi.stop(id))))
    },
    async batchRemove(ids: string[]) {
      if (ids.length === 0) return
      await Promise.all(ids.map(id => runStoreAction(this, '删除容器', () => containerApi.remove(id))))
    }
  }
})
