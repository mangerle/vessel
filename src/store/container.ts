import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

interface ContainerInfo {
  id: string
  name: string
  state: string
  image: string
}

export const useContainerStore = defineStore('container', {
  state: () => ({
    containers: [] as ContainerInfo[],
    loading: false,
    error: null as string | null
  }),
  actions: {
    async fetchContainers() {
      this.loading = true
      this.error = null
      try {
        this.containers = await invoke<ContainerInfo[]>('list_local_containers')
      } catch (err) {
        console.error('获取容器失败:', err)
        this.error = String(err)
      } finally {
        this.loading = false
      }
    }
  }
})
