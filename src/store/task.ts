import { defineStore } from 'pinia'

export interface Task {
  id: string
  name: string
  status: 'running' | 'success' | 'error'
  progress: number
  logs: string[]
  error?: string
}

export const useTaskStore = defineStore('task', {
  state: () => ({
    tasks: [] as Task[]
  }),
  actions: {
    addTask(task: Task) {
      this.tasks.unshift(task)
    },
    updateTask(id: string, updates: Partial<Task>) {
      const task = this.tasks.find(t => t.id === id)
      if (task) {
        Object.assign(task, updates)
      }
    },
    removeTask(id: string) {
      const index = this.tasks.findIndex(t => t.id === id)
      if (index !== -1) {
        this.tasks.splice(index, 1)
      }
    },
    clearFinishedTasks() {
      this.tasks = this.tasks.filter(t => t.status === 'running')
    }
  }
})
