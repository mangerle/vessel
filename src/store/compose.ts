import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { error as logError } from '@tauri-apps/plugin-log'
import { useContainerStore } from './container'
import type { ComposeProject } from '../api/types'
import { composeApi } from '../api/compose'
import {
  EVT,
  type ComposeCmdOutputPayload,
  type ComposeCmdErrorPayload
} from '../api/events'

/** Compose 命令输出环形缓冲容量。原先用 shift() 是 O(n)，这里改为 splice 批量截断。 */
const COMPOSE_OUTPUT_KEEP = 1000

/**
 * Compose 项目仓库（setup 风格）
 */
export const useComposeStore = defineStore('compose', () => {
  // 项目列表
  const projects = ref<ComposeProject[]>([])
  // 当前选中的项目配置文件内容
  const currentProjectFile = ref<string>('')
  // 命令执行输出
  const commandOutput = ref<string[]>([])
  // 加载状态
  const loading = ref(false)
  // 正在执行命令
  const executing = ref(false)
  // 错误信息
  const error = ref<string | null>(null)

  /** 获取 Compose 项目列表 */
  const fetchProjects = async () => {
    loading.value = true
    error.value = null
    try {
      projects.value = await composeApi.listProjects()
    } catch (err) {
      logError(`获取 Compose 项目失败: ${err}`).catch(() => {})
      error.value = String(err)
    } finally {
      loading.value = false
    }
  }

  /** 读取项目的 Compose 文件 */
  const fetchComposeFile = async (path: string) => {
    loading.value = true
    error.value = null
    try {
      currentProjectFile.value = await composeApi.readFile(path)
    } catch (err) {
      logError(`读取 Compose 文件失败: ${err}`).catch(() => {})
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /** 保存项目的 Compose 文件 */
  const saveComposeFile = async (path: string, content: string) => {
    loading.value = true
    error.value = null
    try {
      await composeApi.writeFile(path, content)
      currentProjectFile.value = content
    } catch (err) {
      logError(`保存 Compose 文件失败: ${err}`).catch(() => {})
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /** 运行 Compose 命令 */
  const runComposeCommand = async (projectDir: string, args: string[]) => {
    executing.value = true
    commandOutput.value = []
    error.value = null

    const unlistenList: UnlistenFn[] = []

    const cleanup = () => {
      unlistenList.forEach(fn => fn())
      executing.value = false
    }

    try {
      const unlistenOutput = await listen<ComposeCmdOutputPayload>(EVT.composeCmdOutput, (event) => {
        commandOutput.value.push(event.payload)
        if (commandOutput.value.length > COMPOSE_OUTPUT_KEEP) {
          // 一次性裁掉超额头部，避免 N 次 O(n) shift()
          commandOutput.value.splice(0, commandOutput.value.length - COMPOSE_OUTPUT_KEEP)
        }
      })
      unlistenList.push(unlistenOutput)

      const unlistenFinished = await listen(EVT.composeCmdFinished, () => {
        cleanup()
        const refresh = () => {
          fetchProjects()
          const containerStore = useContainerStore()
          containerStore.fetchContainers()
        }
        refresh()
        setTimeout(refresh, 400)
      })
      unlistenList.push(unlistenFinished)

      const unlistenError = await listen<ComposeCmdErrorPayload | string>(EVT.composeCmdError, (event) => {
        // 后端有时直接 emit 字符串、有时 emit { error } 对象，做一次兼容收口
        const payload = event.payload as ComposeCmdErrorPayload | string
        error.value = typeof payload === 'string' ? payload : payload.error
        cleanup()
      })
      unlistenList.push(unlistenError)

      await composeApi.runCommand(projectDir, args)
    } catch (err) {
      logError(`执行 Compose 命令失败: ${err}`).catch(() => {})
      error.value = String(err)
      cleanup()
    }
  }

  return {
    projects,
    currentProjectFile,
    commandOutput,
    loading,
    executing,
    error,
    fetchProjects,
    fetchComposeFile,
    saveComposeFile,
    runComposeCommand
  }
})
