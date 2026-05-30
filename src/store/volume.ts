import { defineStore } from 'pinia'
import * as volumeApi from '../api/volume'
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
    /**
     * 获取数据卷列表
     */
    async fetchVolumes() {
      this.loading = true
      this.error = null
      try {
        this.volumes = await volumeApi.listVolumes()
      } catch (err) {
        console.error('获取卷列表失败:', err)
        this.error = String(err)
      } finally {
        this.loading = false
      }
    },
    /**
     * 获取使用特定卷的容器
     */
    async fetchVolumeUsers(name: string) {
      this.loading = true
      this.error = null
      try {
        this.volumeUsers = await volumeApi.listVolumeContainers(name)
      } catch (err) {
        console.error('获取卷使用者失败:', err)
        this.error = String(err)
      } finally {
        this.loading = false
      }
    },
    /**
     * 在文件管理器中打开卷路径
     * @param path 卷路径
     */
    async openPath(path: string) {
      this.loading = true
      this.error = null
      try {
        await volumeApi.openVolumePath(path)
      } catch (err) {
        console.error('打开卷路径失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    /**
     * 删除数据卷
     * @param name 卷名称
     */
    async removeVolume(name: string) {
      this.loading = true
      this.error = null
      try {
        await volumeApi.removeVolume(name)
        await this.fetchVolumes()
      } catch (err) {
        console.error('删除卷失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    /**
     * 清理未使用的数据卷
     */
    async pruneVolumes() {
      this.loading = true
      this.error = null
      try {
        await volumeApi.pruneVolumes()
        await this.fetchVolumes()
      } catch (err) {
        console.error('清理卷失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    }
  }
})
