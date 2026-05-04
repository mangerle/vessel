import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { useTaskStore } from './task'

export interface ImageInfo {
  id: string
  tags: string[]
  size: number
  created: number
}

export interface ImageDetails extends Omit<ImageInfo, 'created'> {
  created: string
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
  progressDetail?: {
    current?: number
    total?: number
  }
}

export const useImageStore = defineStore('image', {
  state: () => ({
    images: [] as ImageInfo[],
    searchResults: [] as ImageSearchResult[],
    imageHistory: [] as ImageHistoryInfo[],
    loading: false,
    pulling: false,
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
      const taskStore = useTaskStore()
      const taskId = crypto.randomUUID()
      
      taskStore.addTask({
        id: taskId,
        name: `拉取镜像: ${imageName}`,
        status: 'running',
        progress: 0,
        logs: []
      })

      this.pulling = true
      this.error = null

      const unlistenList: UnlistenFn[] = []

      const cleanup = (status: 'success' | 'error', error?: string) => {
        unlistenList.forEach(fn => fn())
        this.pulling = false
        taskStore.updateTask(taskId, { 
          status, 
          progress: status === 'success' ? 100 : undefined,
          error 
        })
      }

      try {
        // 监听拉取进度事件
        const unlistenProgress = await listen<PullProgress>('image-pull-progress', (event) => {
          const payload = event.payload
          const logMsg = payload.status || payload.stream || ''
          
          let progress: number | undefined = undefined
          if (payload.progressDetail?.current && payload.progressDetail?.total) {
             progress = Math.round((payload.progressDetail.current / payload.progressDetail.total) * 100)
          }

          const task = taskStore.tasks.find(t => t.id === taskId)
          if (task) {
            taskStore.updateTask(taskId, {
              progress: progress ?? task.progress,
              logs: [...task.logs, logMsg].slice(-20)
            })
          }
        })
        unlistenList.push(unlistenProgress)

        const unlistenError = await listen<string>('image-pull-error', (event) => {
          console.error('拉取镜像出错:', event.payload)
          this.error = event.payload
          cleanup('error', event.payload)
        })
        unlistenList.push(unlistenError)

        const unlistenFinished = await listen<string>('image-pull-finished', () => {
          cleanup('success')
          this.fetchImages()
        })
        unlistenList.push(unlistenFinished)

        await invoke('pull_image', { imageName })
      } catch (err) {
        console.error('启动拉取任务失败:', err)
        this.error = String(err)
        cleanup('error', String(err))
        throw err
      }
    }
  }
})
