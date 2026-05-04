import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

/**
 * Docker 数据卷接口
 */
interface VolumeInfo {
  name: string
  driver: string
  mountpoint: string
  created: string
}

/**
 * Docker 数据卷仓库
 */
export const useVolumeStore = defineStore('volume', {
  state: () => ({
    // 数据卷列表
    volumes: [] as VolumeInfo[],
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
        this.volumes = await invoke<VolumeInfo[]>('list_volumes')
      } catch (err) {
        console.error('获取卷列表失败:', err)
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
        await invoke('open_volume_path', { path })
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
        await invoke('remove_volume', { name })
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
        await invoke('prune_volumes')
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
