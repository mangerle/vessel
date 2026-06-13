import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface Task {
  id: string
  name: string
  status: 'running' | 'success' | 'error'
  progress: number
  logs: string[]
  error?: string
}

/**
 * 后台任务仓库（setup 风格）
 *
 * 用于跟踪镜像 pull / export / import 等长任务的进度与日志。
 */
export const useTaskStore = defineStore('task', () => {
  const tasks = ref<Task[]>([])

  const addTask = (task: Task) => {
    tasks.value.unshift(task)
  }

  const updateTask = (id: string, updates: Partial<Task>) => {
    const task = tasks.value.find(t => t.id === id)
    if (task) {
      Object.assign(task, updates)
    }
  }

  const removeTask = (id: string) => {
    const index = tasks.value.findIndex(t => t.id === id)
    if (index !== -1) {
      tasks.value.splice(index, 1)
    }
  }

  const clearFinishedTasks = () => {
    tasks.value = tasks.value.filter(t => t.status === 'running')
  }

  return {
    tasks,
    addTask,
    updateTask,
    removeTask,
    clearFinishedTasks
  }
})
