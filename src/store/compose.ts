import {defineStore} from 'pinia'
import {invoke} from '@tauri-apps/api/core'
import {listen} from '@tauri-apps/api/event'

/**
 * Docker Compose 项目接口
 */
export interface ComposeProject {
  name: string
  container_count: number
  running_count: number
  status: string
    working_dir?: string
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
          try {
              this.currentProjectFile = await invoke<string>('read_compose_file', {path})
          } catch (err) {
              console.error('读取 Compose 文件失败:', err)
              throw err
          }
      },

      /**
       * 保存项目的 Compose 文件
       */
      async saveComposeFile(path: string, content: string) {
          try {
              await invoke('write_compose_file', {path, content})
              this.currentProjectFile = content
          } catch (err) {
              console.error('保存 Compose 文件失败:', err)
              throw err
          }
      },

      /**
       * 运行 Compose 命令
       */
      async runComposeCommand(projectDir: string, args: string[]) {
          this.executing = true
          this.commandOutput = []

          const unlisten = await listen('compose-cmd-output', (event: any) => {
              this.commandOutput.push(event.payload)
              if (this.commandOutput.length > 1000) {
                  this.commandOutput.shift()
              }
          })

          try {
              await invoke('run_compose_command', {projectDir, args})
          } catch (err) {
              console.error('执行 Compose 命令失败:', err)
              this.error = String(err)
          } finally {
              // 命令通常是异步启动的，后端负责通过事件报告结束（虽然目前后端实现只是简单的 spawn）
              // 实际上我们需要某种方式知道命令何时结束，但目前先简单处理
              setTimeout(() => {
                  this.executing = false
                  unlisten()
              }, 2000)
          }
    }
  }
})
