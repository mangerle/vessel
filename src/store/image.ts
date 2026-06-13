import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { error as logError } from '@tauri-apps/plugin-log'
import { useTaskStore } from './task'
import { runStoreAction } from './helpers'
import { imageApi } from '../api/image'
import {
  EVT,
  type ImagePullProgressPayload,
  type ImagePullErrorPayload,
  type ImagePullFinishedPayload,
  type ImageExportProgressPayload,
  type ImageExportErrorPayload,
  type ImageExportFinishedPayload,
  type ImageImportProgressPayload,
  type ImageImportErrorPayload,
  type ImageImportFinishedPayload
} from '../api/events'
import { formatBytes } from '../utils/format'
import type {
  ImageInfo,
  ImageSearchResult,
  ImageHistoryInfo
} from '../api/types'

/** 任务日志环形缓冲容量：超过则截断为最新 N 条，避免无界增长。 */
const TASK_LOG_KEEP = 20

/**
 * 镜像仓库（setup 风格）
 *
 * 与其他 store 保持一致的写法：state 用 ref，actions 直接以函数 return 出去。
 * runStoreAction 通过适配器对象桥接 ref，避免把 .value 内部细节扩散到 view 层。
 */
export const useImageStore = defineStore('image', () => {
  const images = ref<ImageInfo[]>([])
  const searchResults = ref<ImageSearchResult[]>([])
  const imageHistory = ref<ImageHistoryInfo[]>([])
  const loading = ref(false)
  const pulling = ref(false)
  const error = ref<string | null>(null)

  // runStoreAction 期望可读写的 LoadingErrorState；用 getter/setter 桥到 ref。
  const loadingState = {
    get loading() { return loading.value },
    set loading(v: boolean) { loading.value = v },
    get error() { return error.value },
    set error(v: string | null) { error.value = v }
  }

  const fetchImages = () =>
    runStoreAction(loadingState, '获取镜像', async () => {
      images.value = await imageApi.list()
    })

  const searchImages = (term: string) =>
    runStoreAction(loadingState, '搜索镜像', async () => {
      searchResults.value = await imageApi.search(term)
    })

  const clearSearchResults = () => {
    searchResults.value = []
  }

  const inspectImage = (id: string) =>
    runStoreAction(loadingState, '获取镜像详情', () => imageApi.inspect(id))

  const fetchImageHistory = (id: string) =>
    runStoreAction(loadingState, '获取镜像历史', async () => {
      imageHistory.value = await imageApi.history(id)
    })

  const removeImage = (id: string) =>
    runStoreAction(loadingState, '删除镜像', () => imageApi.remove(id), () => fetchImages())

  const pullImage = async (
    imageName: string,
    auth?: { username?: string; password?: string; serverAddress?: string }
  ) => {
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

    pulling.value = true
    error.value = null

    const unlistenList: UnlistenFn[] = []

    const cleanup = (status: 'success' | 'error', err?: string) => {
      unlistenList.forEach(fn => fn())

      setTimeout(() => {
        const hasOtherRunning = taskStore.tasks.some(t => t.id !== taskId && t.name.startsWith('拉取镜像:') && t.status === 'running')
        if (!hasOtherRunning) {
          pulling.value = false
        }
      }, 100)

      taskStore.updateTask(taskId, {
        status,
        progress: status === 'success' ? 100 : undefined,
        error: err
      })
    }

    try {
      const unlistenProgress = await listen<ImagePullProgressPayload>(EVT.imagePullProgress, (event) => {
        const { image, info: payload } = event.payload
        if (image !== targetImageName) return

        const logMsg = payload.status || payload.stream || ''

        let progress: number | undefined = undefined
        if (payload.progressDetail?.current && payload.progressDetail?.total) {
          progress = Math.round((payload.progressDetail.current / payload.progressDetail.total) * 100)
        }

        const task = taskStore.tasks.find(t => t.id === taskId)
        if (!task) return
        // 增量追加 + 原地截断：避免每 tick 重建整个数组（高 QPS 时显著降低 GC 压力）
        task.logs.push(logMsg)
        if (task.logs.length > TASK_LOG_KEEP) {
          task.logs.splice(0, task.logs.length - TASK_LOG_KEEP)
        }
        if (progress !== undefined && progress !== task.progress) {
          task.progress = progress
        }
      })
      unlistenList.push(unlistenProgress)

      const unlistenError = await listen<ImagePullErrorPayload>(EVT.imagePullError, (event) => {
        const { image, error: errMsg } = event.payload
        if (image !== targetImageName) return
        cleanup('error', errMsg)
      })
      unlistenList.push(unlistenError)

      const unlistenFinished = await listen<ImagePullFinishedPayload>(EVT.imagePullFinished, (event) => {
        const image = event.payload
        if (image !== targetImageName) return
        cleanup('success')
        fetchImages()
      })
      unlistenList.push(unlistenFinished)

      // 调用后端，传递 imageName 已经可能包含的登录凭证
      await imageApi.pull({
        image_name: imageName,
        username: auth?.username || null,
        password: auth?.password || null,
        server_address: auth?.serverAddress || null
      })
    } catch (err) {
      error.value = String(err)
      cleanup('error', String(err))
      throw err
    }
  }

  const pruneDanglingImages = () =>
    runStoreAction(loadingState, '清理虚悬镜像', async () => {
      const result = await imageApi.prune()
      await fetchImages()
      return result
    })

  const exportImage = async (imageId: string, imageName: string) => {
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

    const cleanup = (status: 'success' | 'error', err?: string) => {
      unlistenList.forEach(fn => fn())
      taskStore.updateTask(taskId, {
        status,
        progress: status === 'success' ? 100 : undefined,
        error: err
      })
    }

    try {
      const unlistenProgress = await listen<ImageExportProgressPayload>(EVT.imageExportProgress, (event) => {
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

      const unlistenError = await listen<ImageExportErrorPayload>(EVT.imageExportError, (event) => {
        const { image, error: errMsg } = event.payload
        if (image !== exportIdentifier) return
        cleanup('error', errMsg)
      })
      unlistenList.push(unlistenError)

      const unlistenFinished = await listen<ImageExportFinishedPayload>(EVT.imageExportFinished, (event) => {
        const image = event.payload
        if (image !== exportIdentifier) return
        cleanup('success')
      })
      unlistenList.push(unlistenFinished)

      await imageApi.export(exportIdentifier, path)
    } catch (err) {
      logError(`导出镜像失败: ${err}`).catch(() => {})
      cleanup('error', String(err))
      throw err
    }
  }

  const importImage = async (path: string, customTag?: string) => {
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
    const oldIds = new Set(images.value.map(img => img.id))

    const cleanup = async (status: 'success' | 'error', err?: string) => {
      unlistenList.forEach(fn => fn())
      taskStore.updateTask(taskId, {
        status,
        progress: status === 'success' ? 100 : undefined,
        error: err
      })
      if (status === 'success') {
        await fetchImages()

        if (customTag) {
          try {
            const newImage = images.value.find(img => !oldIds.has(img.id))
            if (newImage) {
              const parts = customTag.split(':')
              const repo = parts[0]
              const tag = parts[1] || 'latest'

              taskStore.updateTask(taskId, {
                logs: [`正在为新导入的镜像赋予标签: ${repo}:${tag}...`]
              })

              await imageApi.tag(newImage.id, repo, tag)

              await fetchImages()
              taskStore.updateTask(taskId, {
                logs: [`镜像打标签成功: ${repo}:${tag}`]
              })
            }
          } catch (tagErr) {
            logError(`打标签失败: ${tagErr}`).catch(() => {})
            taskStore.updateTask(taskId, {
              logs: [`自动打标签失败: ${tagErr}`]
            })
          }
        }
      }
    }

    try {
      const unlistenProgress = await listen<ImageImportProgressPayload>(EVT.imageImportProgress, (event) => {
        const { path: eventPath, status, stream, error: errMsg } = event.payload
        if (eventPath !== path) return

        const logMsg = stream || status || errMsg || ''
        const task = taskStore.tasks.find(t => t.id === taskId)
        if (!task) return
        // 增量追加 + 原地截断：避免每 tick 重建整个数组
        task.logs.push(logMsg)
        if (task.logs.length > TASK_LOG_KEEP) {
          task.logs.splice(0, task.logs.length - TASK_LOG_KEEP)
        }
      })
      unlistenList.push(unlistenProgress)

      const unlistenError = await listen<ImageImportErrorPayload>(EVT.imageImportError, (event) => {
        const { path: eventPath, error: errMsg } = event.payload
        if (eventPath !== path) return
        cleanup('error', errMsg)
      })
      unlistenList.push(unlistenError)

      const unlistenFinished = await listen<ImageImportFinishedPayload>(EVT.imageImportFinished, (event) => {
        const eventPath = event.payload
        if (eventPath !== path) return
        cleanup('success')
      })
      unlistenList.push(unlistenFinished)

      await imageApi.import(path)
    } catch (err) {
      logError(`流式导入镜像失败: ${err}`).catch(() => {})
      cleanup('error', String(err))
      throw err
    }
  }

  return {
    images,
    searchResults,
    imageHistory,
    loading,
    pulling,
    error,
    fetchImages,
    searchImages,
    clearSearchResults,
    inspectImage,
    fetchImageHistory,
    removeImage,
    pullImage,
    pruneDanglingImages,
    exportImage,
    importImage
  }
})
