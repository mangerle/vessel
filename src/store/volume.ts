import { defineStore } from 'pinia'
import { volumeApi } from '../api/volume'
import type { VolumeInfo, VolumeUser } from '../api/types'

/**
 * Docker 数据卷仓库
 */
export const useVolumeStore = defineStore('volume', {
  state: () => ({
    // 数据卷列表
    volumes: [] as VolumeInfo[],
    // 当前选中卷的使用者列表
    volumeUsers: [] as VolumeUser[],
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
          this.volumes = await volumeApi.list()
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
     * 获取数据卷列表
     */
    async fetchVolumes() {
      await this.executeAction('获取数据卷', async () => {}, true)
    },
    /**
     * 获取使用特定卷的容器
     */
    async fetchVolumeUsers(name: string) {
      await this.executeAction('获取卷使用者', async () => {
        this.volumeUsers = await volumeApi.listContainers(name)
      }, false)
    },
    /**
     * 在文件管理器中打开卷路径
     * @param path 卷路径
     */
    async openPath(path: string) {
      await this.executeAction('打开卷路径', () => volumeApi.openPath(path), false)
    },
    /**
     * 删除数据卷
     * @param name 卷名称
     */
    async removeVolume(name: string) {
      await this.executeAction('删除卷', () => volumeApi.remove(name))
    },
    /**
     * 清理未使用的数据卷
     */
    async pruneVolumes() {
      await this.executeAction('清理卷', () => volumeApi.prune())
    }
  }
})
