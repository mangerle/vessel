import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { useSettingsStore } from './settings'
import { useContainerStore } from './container'

/**
 * Docker Compose 项目接口
 */
export interface ComposeProject {
  name: string
  container_count: number
  running_count: number
  status: string
  working_dir?: string
  config_file?: string
}

/**
 * Compose 项目仓库
 */
export const useComposeStore = defineStore('compose', {
  state: () => ({
    // 项目列表
    projects: [] as ComposeProject[],
    // 当前选中的项目配置文件内容
    currentProjectFile: '' as string,
    // 命令执行输出
    commandOutput: [] as string[],
    // 加载状态
    loading: false,
    // 正在执行命令
    executing: false,
    // 错误信息
    error: null as string | null
  }),
  actions: {
    /**
     * 获取 Compose 项目列表
     */
    async fetchProjects() {
      this.loading = true
      this.error = null
      try {
        this.projects = await invoke<ComposeProject[]>('list_compose_projects')
      } catch (err) {
        console.error('获取 Compose 项目失败:', err)
        this.error = String(err)
      } finally {
        this.loading = false
      }
    },

    /**
     * 读取项目的 Compose 文件
     */
    async fetchComposeFile(path: string) {
      this.loading = true
      this.error = null
      const settingsStore = useSettingsStore()
      try {
        this.currentProjectFile = await invoke<string>('read_compose_file', { 
          path,
          mode: settingsStore.connectionMode,
          distro: settingsStore.wslDistro
        })
      } catch (err) {
        console.error('读取 Compose 文件失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },

    /**
     * 保存项目的 Compose 文件
     */
    async saveComposeFile(path: string, content: string) {
      this.loading = true
      this.error = null
      const settingsStore = useSettingsStore()
      try {
        await invoke('write_compose_file', { 
          path, 
          content,
          mode: settingsStore.connectionMode,
          distro: settingsStore.wslDistro
        })
        this.currentProjectFile = content
      } catch (err) {
        console.error('保存 Compose 文件失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },

    /**
     * 运行 Compose 命令
     */
    async runComposeCommand(projectDir: string, args: string[]) {
      this.executing = true
      this.commandOutput = []
      this.error = null
      const settingsStore = useSettingsStore()

      const unlistenList: UnlistenFn[] = []

      const cleanup = () => {
        unlistenList.forEach(fn => fn())
        this.executing = false
      }

      try {
        const unlistenOutput = await listen<string>('compose-cmd-output', (event) => {
          this.commandOutput.push(event.payload)
          if (this.commandOutput.length > 1000) {
            this.commandOutput.shift()
          }
        })
        unlistenList.push(unlistenOutput)

        const unlistenFinished = await listen('compose-cmd-finished', () => {
          cleanup()
          const refresh = () => {
            this.fetchProjects()
            const containerStore = useContainerStore()
            containerStore.fetchContainers()
          }
          refresh()
          setTimeout(refresh, 400)
        })
        unlistenList.push(unlistenFinished)

        const unlistenError = await listen<string>('compose-cmd-error', (event) => {
          this.error = event.payload
          cleanup()
        })
        unlistenList.push(unlistenError)

        await invoke('run_compose_command', { 
          projectDir, 
          args,
          mode: settingsStore.connectionMode,
          distro: settingsStore.wslDistro
        })
      } catch (err) {
        console.error('执行 Compose 命令失败:', err)
        this.error = String(err)
        cleanup()
      }
    }
  }
})
