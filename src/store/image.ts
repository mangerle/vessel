import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

export interface ImageInfo {
  id: string
  tags: string[]
  size: number
  created: number
}

export interface ImageDetails extends ImageInfo {
  architecture: string
  os: string
  env: string[]
  exposed_ports: string[]
  cmd: string[]
  entrypoint: string[]
}

export interface ImageSearchResult {
  name: string
  description: string
  is_official: boolean
  star_count: number
}

export interface ImageHistoryInfo {
  id: string
  created: number
  created_by: string
  size: number
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
    searchResults: [] as ImageSearchResult[],
    imageHistory: [] as ImageHistoryInfo[],
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
    async searchImages(term: string) {
      this.loading = true
      this.error = null
      try {
        this.searchResults = await invoke<ImageSearchResult[]>('search_images', { term })
      } catch (err) {
        console.error('搜索镜像失败:', err)
        this.error = String(err)
      } finally {
        this.loading = false
      }
    },
    async inspectImage(id: string) {
      this.loading = true
      this.error = null
      try {
        return await invoke<ImageDetails>('inspect_image', { id })
      } catch (err) {
        console.error('获取镜像详情失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    async fetchImageHistory(id: string) {
      this.loading = true
      this.error = null
      try {
        this.imageHistory = await invoke<ImageHistoryInfo[]>('get_image_history', { id })
      } catch (err) {
        console.error('获取镜像历史失败:', err)
        this.error = String(err)
      } finally {
        this.loading = false
      }
    },
    async removeImage(id: string) {
      this.loading = true
      this.error = null
      try {
        await invoke('remove_image', { id })
        await this.fetchImages()
      } catch (err) {
        console.error('删除镜像失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    async pullImage(imageName: string) {
      this.pulling = true
      this.pullLogs = []
      this.error = null

      const unlistenList: UnlistenFn[] = []

      const cleanup = () => {
        unlistenList.forEach(fn => fn())
        this.pulling = false
      }

      try {
        // 监听拉取进度事件
        const unlistenProgress = await listen<PullProgress>('image-pull-progress', (event) => {
          this.pullLogs.push(event.payload)
          // 保持最近的 100 条日志
          if (this.pullLogs.length > 100) {
            this.pullLogs.shift()
          }
        })
        unlistenList.push(unlistenProgress)

        const unlistenError = await listen<string>('image-pull-error', (event) => {
          console.error('拉取镜像出错:', event.payload)
          this.error = event.payload
          cleanup()
        })
        unlistenList.push(unlistenError)

        const unlistenFinished = await listen<string>('image-pull-finished', () => {
          cleanup()
          this.fetchImages()
        })
        unlistenList.push(unlistenFinished)

        await invoke('pull_image', { imageName })
      } catch (err) {
        console.error('启动拉取任务失败:', err)
        this.error = String(err)
        cleanup()
        throw err
      }
    }
  }
})
