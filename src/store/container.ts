import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface ContainerInfo {
  id: string
  name: string
  state: string
  image: string
  compose_project?: string
}

export interface PortMapping {
  private_port: number
  public_port?: number
  type_: string
  ip?: string
}

export interface MountInfo {
  source: string
  destination: string
  mode: string
  rw: boolean
}

export interface ContainerDetails {
  id: string
  name: string
  image: string
  image_id: string
  state: string
  status: string
  created: string
  env: string[]
  ports: PortMapping[]
  mounts: MountInfo[]
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
    },
    async startContainer(id: string) {
      this.loading = true
      this.error = null
      try {
        await invoke('start_container', { id })
        await this.fetchContainers()
      } catch (err) {
        console.error('启动容器失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    async stopContainer(id: string) {
      this.loading = true
      this.error = null
      try {
        await invoke('stop_container', { id })
        await this.fetchContainers()
      } catch (err) {
        console.error('停止容器失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    async restartContainer(id: string) {
      this.loading = true
      this.error = null
      try {
        await invoke('restart_container', { id })
        await this.fetchContainers()
      } catch (err) {
        console.error('重启容器失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    async removeContainer(id: string) {
      this.loading = true
      this.error = null
      try {
        await invoke('remove_container', { id })
        await this.fetchContainers()
      } catch (err) {
        console.error('删除容器失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    async pauseContainer(id: string) {
      this.loading = true
      this.error = null
      try {
        await invoke('pause_container', { id })
        await this.fetchContainers()
      } catch (err) {
        console.error('暂停容器失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    },
    async unpauseContainer(id: string) {
      this.loading = true
      this.error = null
      try {
        await invoke('unpause_container', { id })
        await this.fetchContainers()
      } catch (err) {
        console.error('恢复容器失败:', err)
        this.error = String(err)
        throw err
      } finally {
        this.loading = false
      }
    }
  }
})
