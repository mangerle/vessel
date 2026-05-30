import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { useTaskStore } from './task'
import type { ImageInfo, ImageDetails, ImageSearchResult, ImageHistoryInfo, PullProgress } from '../api/types'

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
    clearSearchResults() {
      this.searchResults = []
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
    async pullImage(imageName: string, auth?: { username?: string; password?: string; serverAddress?: string }) {
      // 调试：确保 imageName 是字符串
      if (typeof imageName !== 'string') {
        throw new TypeError('参数 imageName 必须是字符串')
      }
      
      const taskStore = useTaskStore()
      const taskId = crypto.randomUUID()
      
      // 使用 includes 检查标签
      const targetImageName = imageName.includes(':') ? imageName : `${imageName}:latest`

      taskStore.addTask({
        id: taskId,
        name: `拉取镜像: ${targetImageName}`,
        status: 'running',
        progress: 0,
        logs: []
      })

      this.pulling = true
      this.error = null

      const unlistenList: UnlistenFn[] = []

      const cleanup = (status: 'success' | 'error', error?: string) => {
        unlistenList.forEach(fn => fn())
        
        setTimeout(() => {
          const hasOtherRunning = taskStore.tasks.some(t => t.id !== taskId && t.name.startsWith('拉取镜像:') && t.status === 'running')
          if (!hasOtherRunning) {
            this.pulling = false
          }
        }, 100)

        taskStore.updateTask(taskId, { 
          status, 
          progress: status === 'success' ? 100 : undefined,
          error 
        })
      }

      try {
        const unlistenProgress = await listen<{ image: string, info: PullProgress }>('image-pull-progress', (event) => {
          const { image, info: payload } = event.payload
          if (image !== targetImageName) return

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

        const unlistenError = await listen<{ image: string, error: string }>('image-pull-error', (event) => {
          const { image, error } = event.payload
          if (image !== targetImageName) return
          cleanup('error', error)
        })
        unlistenList.push(unlistenError)

        const unlistenFinished = await listen<string>('image-pull-finished', (event) => {
          const image = event.payload
          if (image !== targetImageName) return
          cleanup('success')
          this.fetchImages()
        })
        unlistenList.push(unlistenFinished)

        // 调用后端，传递 imageName 已经可能包含的登录凭证
        await invoke('pull_image', { 
          imageName,
          username: auth?.username || null,
          password: auth?.password || null,
          serverAddress: auth?.serverAddress || null
        })
      } catch (err) {
        this.error = String(err)
        cleanup('error', String(err))
        throw err
      }
    },
    async pruneDanglingImages() {
      this.loading = true
      this.error = null
      try {
        const result = await invoke<{ deleted_count: number; space_reclaimed: number }>('prune_images')
        await this.fetchImages()
        return result
      } catch (err) {
        console.error('清理虚悬镜像失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    async exportImage(imageId: string, imageName: string) {
      const { save } = await import('@tauri-apps/plugin-dialog')
      
      const safeName = imageName.replace(/[:/]/g, '_') || imageId.substring(0, 12)
      const path = await save({
        title: '导出镜像为 Tar 包',
        defaultPath: `${safeName}.tar`,
        filters: [{ name: 'Tar Archive', extensions: ['tar'] }]
      })

      if (!path) return

      const taskStore = useTaskStore()
      const taskId = crypto.randomUUID()
      const taskName = `导出镜像: ${imageName}`

      taskStore.addTask({
        id: taskId,
        name: taskName,
        status: 'running',
        progress: 0,
        logs: ['正在初始化镜像导出...']
      })

      const unlistenList: (() => void)[] = []

      const exportIdentifier = (imageName && !imageName.includes('<none>')) ? imageName : imageId

      const cleanup = (status: 'success' | 'error', error?: string) => {
        unlistenList.forEach(fn => fn())
        taskStore.updateTask(taskId, {
          status,
          progress: status === 'success' ? 100 : undefined,
          error
        })
      }

      const formatBytes = (bytes: number, decimals = 2) => {
        if (bytes === 0) return '0 Bytes'
        const k = 1024
        const dm = decimals < 0 ? 0 : decimals
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
        const i = Math.floor(Math.log(bytes) / Math.log(k))
        return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
      }

      try {
        const unlistenProgress = await listen<{ image: string; bytes_written: number }>('image-export-progress', (event) => {
          const { image, bytes_written } = event.payload
          if (image !== exportIdentifier) return

          const logMsg = `已导出: ${formatBytes(bytes_written)}`
          const task = taskStore.tasks.find(t => t.id === taskId)
          if (task) {
            taskStore.updateTask(taskId, {
              logs: [logMsg]
            })
          }
        })
        unlistenList.push(unlistenProgress)

        const unlistenError = await listen<{ image: string; error: string }>('image-export-error', (event) => {
          const { image, error } = event.payload
          if (image !== exportIdentifier) return
          cleanup('error', error)
        })
        unlistenList.push(unlistenError)

        const unlistenFinished = await listen<string>('image-export-finished', (event) => {
          const image = event.payload
          if (image !== exportIdentifier) return
          cleanup('success')
        })
        unlistenList.push(unlistenFinished)

        await invoke('export_image', { imageIdOrName: exportIdentifier, path })
      } catch (err) {
        console.error('导出镜像失败:', err)
        cleanup('error', String(err))
        throw err
      }
    },
    async importImage(path: string, customTag?: string) {
      const taskStore = useTaskStore()
      const taskId = crypto.randomUUID()
      
      const fileName = path.split(/[/\\]/).pop() || path
      const taskName = `导入镜像: ${fileName}`

      taskStore.addTask({
        id: taskId,
        name: taskName,
        status: 'running',
        progress: 0,
        logs: ['正在启动镜像导入程序...']
      })

      const unlistenList: UnlistenFn[] = []
      const oldIds = new Set(this.images.map(img => img.id))

      const cleanup = async (status: 'success' | 'error', error?: string) => {
        unlistenList.forEach(fn => fn())
        taskStore.updateTask(taskId, {
          status,
          progress: status === 'success' ? 100 : undefined,
          error
        })
        if (status === 'success') {
          await this.fetchImages()
          
          if (customTag) {
            try {
              const newImage = this.images.find(img => !oldIds.has(img.id))
              if (newImage) {
                const parts = customTag.split(':')
                const repo = parts[0]
                const tag = parts[1] || 'latest'
                
                taskStore.updateTask(taskId, {
                  logs: [`正在为新导入的镜像赋予标签: ${repo}:${tag}...`]
                })
                
                await invoke('tag_image', {
                  imageName: newImage.id,
                  repo,
                  tag
                })
                
                await this.fetchImages()
                taskStore.updateTask(taskId, {
                  logs: [`镜像打标签成功: ${repo}:${tag}`]
                })
              }
            } catch (tagErr) {
              console.error('打标签失败:', tagErr)
              taskStore.updateTask(taskId, {
                logs: [`自动打标签失败: ${tagErr}`]
              })
            }
          }
        }
      }

      try {
        const unlistenProgress = await listen<{ path: string; info: any }>('image-import-progress', (event) => {
          const { path: eventPath, info } = event.payload
          if (eventPath !== path) return

          const logMsg = info.stream || info.status || ''
          const task = taskStore.tasks.find(t => t.id === taskId)
          if (task) {
            taskStore.updateTask(taskId, {
              logs: [...task.logs, logMsg].slice(-20)
            })
          }
        })
        unlistenList.push(unlistenProgress)

        const unlistenError = await listen<{ path: string; error: string }>('image-import-error', (event) => {
          const { path: eventPath, error } = event.payload
          if (eventPath !== path) return
          cleanup('error', error)
        })
        unlistenList.push(unlistenError)

        const unlistenFinished = await listen<string>('image-import-finished', (event) => {
          const eventPath = event.payload
          if (eventPath !== path) return
          cleanup('success')
        })
        unlistenList.push(unlistenFinished)

        await invoke('import_image', { path })
      } catch (err) {
        console.error('流式导入镜像失败:', err)
        cleanup('error', String(err))
        throw err
      }
    }
  }
})
