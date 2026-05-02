import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

/**
 * Docker Compose 项目接口
 */
interface ComposeProject {
  name: string
  container_count: number
  running_count: number
  status: string
}

/**
 * Compose 项目仓库
 */
export const useComposeStore = defineStore('compose', {
  state: () => ({
    // 项目列表
    projects: [] as ComposeProject[],
    // 加载状态
    loading: false,
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
    }
  }
})
