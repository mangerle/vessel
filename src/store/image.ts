import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface ImageInfo {
  id: string
  tags: string[]
  size: number
  created: number
}

export interface PullProgress {
  status?: string
  progress?: string
  id?: string
  stream?: string
  error?: string
}

export const useImageStore = defineStore('image', {
  state: () => ({
    images: [] as ImageInfo[],
    loading: false,
    pulling: false,
    pullLogs: [] as PullProgress[],
    error: null as string | null
  }),
  actions: {
    async fetchImages() {
      this.loading = true
      this.error = null
      try {
        this.images = await invoke<ImageInfo[]>('list_images')
      } catch (err) {
        console.error('获取镜像失败:', err)
        this.error = String(err)
      } finally {
        this.loading = false
      }
    },
    async removeImage(id: string) {
      try {
        await invoke('remove_image', { id })
        await this.fetchImages()
      } catch (err) {
        console.error('删除镜像失败:', err)
        throw err
      }
    },
    async pullImage(imageName: string) {
      this.pulling = true
      this.pullLogs = []
      try {
        // 监听拉取进度事件
        const unlistenProgress = await listen<PullProgress>('image-pull-progress', (event) => {
          this.pullLogs.push(event.payload)
          // 保持最近的 100 条日志
          if (this.pullLogs.length > 100) {
            this.pullLogs.shift()
          }
        })

        const unlistenError = await listen<string>('image-pull-error', (event) => {
          console.error('拉取镜像出错:', event.payload)
          this.error = event.payload
          this.pulling = false
          unlistenProgress()
          unlistenError()
          unlistenFinished()
        })

        const unlistenFinished = await listen<string>('image-pull-finished', () => {
          this.pulling = false
          this.fetchImages()
          unlistenProgress()
          unlistenError()
          unlistenFinished()
        })

        await invoke('pull_image', { imageName })
      } catch (err) {
        console.error('启动拉取任务失败:', err)
        this.pulling = false
        throw err
      }
    }
  }
})
