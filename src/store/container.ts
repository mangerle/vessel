import { defineStore } from 'pinia'
import { containerApi } from '../api/container'
import type { ContainerInfo } from '../api/types'

export const useContainerStore = defineStore('container', {
  state: () => ({
    containers: [] as ContainerInfo[],
    loading: false,
    error: null as string | null
  }),
  actions: {
    async executeAction<T = void>(actionName: string, actionFn: () => Promise<T>, refresh: boolean = true): Promise<T> {
      this.loading = true
      this.error = null
      try {
        const result = await actionFn()
        if (refresh) {
          this.containers = await containerApi.list()
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
    async fetchContainers() {
      await this.executeAction('获取容器', async () => {}, true)
    },
    async startContainer(id: string) {
      await this.executeAction('启动容器', () => containerApi.start(id))
    },
    async stopContainer(id: string) {
      await this.executeAction('停止容器', () => containerApi.stop(id))
    },
    async restartContainer(id: string) {
      await this.executeAction('重启容器', () => containerApi.restart(id))
    },
    async removeContainer(id: string) {
      await this.executeAction('删除容器', () => containerApi.remove(id))
    },
    async pauseContainer(id: string) {
      await this.executeAction('暂停容器', () => containerApi.pause(id))
    },
    async unpauseContainer(id: string) {
      await this.executeAction('恢复容器', () => containerApi.unpause(id))
    },
    /**
     * 批量操作：仅执行动作，不触发逐次 list_containers 刷新。
     * 调用方应在批结束后自行调用 fetchContainers() 同步一次即可，
     * 避免 N 个容器操作 × 1 次 list 的 N+1 IPC 放大。
     */
    async batchStart(ids: string[]) {
      if (ids.length === 0) return
      await Promise.all(ids.map(id => this.executeAction('启动容器', () => containerApi.start(id), false)))
    },
    async batchStop(ids: string[]) {
      if (ids.length === 0) return
      await Promise.all(ids.map(id => this.executeAction('停止容器', () => containerApi.stop(id), false)))
    },
    async batchRemove(ids: string[]) {
      if (ids.length === 0) return
      await Promise.all(ids.map(id => this.executeAction('删除容器', () => containerApi.remove(id), false)))
    }
  }
})
