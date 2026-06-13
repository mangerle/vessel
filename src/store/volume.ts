import { defineStore } from 'pinia'
import { volumeApi } from '../api/volume'
import { runStoreAction } from './helpers'
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
    /** 刷新数据卷列表（runStoreAction 的 refresh 回调） */
    async refresh() {
      this.volumes = await volumeApi.list()
    },
    /**
     * 获取数据卷列表
     */
    async fetchVolumes() {
      await runStoreAction(this, '获取数据卷', () => this.refresh())
    },
    /**
     * 获取使用特定卷的容器
     */
    async fetchVolumeUsers(name: string) {
      await runStoreAction(this, '获取卷使用者', async () => {
        this.volumeUsers = await volumeApi.listContainers(name)
      })
    },
    /**
     * 在文件管理器中打开卷路径
     * @param path 卷路径
     */
    async openPath(path: string) {
      await runStoreAction(this, '打开卷路径', () => volumeApi.openPath(path))
    },
    /**
     * 删除数据卷
     * @param name 卷名称
     */
    async removeVolume(name: string) {
      await runStoreAction(this, '删除卷', () => volumeApi.remove(name), () => this.refresh())
    },
    /**
     * 清理未使用的数据卷
     */
    async pruneVolumes() {
      await runStoreAction(this, '清理卷', () => volumeApi.prune(), () => this.refresh())
    }
  }
})
