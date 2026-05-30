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
    async executeAction(actionName: string, actionFn: () => Promise<any>, refresh: boolean = true) {
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
    }
  }
})
