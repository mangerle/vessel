import { onUnmounted, ref } from 'vue'

/**
 * 通用的轮询 Hook，具备防止并发请求和自动清理功能
 * @param fn 轮询执行的任务函数（异步）
 * @param interval 轮询间隔（毫秒）
 * @param immediate 是否立即执行一次
 */
export function usePolling(fn: () => Promise<void>, interval: number, immediate = true) {
  const isPolling = ref(false)
  let timer: ReturnType<typeof setInterval> | null = null

  const poll = async () => {
    if (isPolling.value) return
    isPolling.value = true
    try {
      await fn()
    } catch (err) {
      console.error('Polling task failed:', err)
    } finally {
      isPolling.value = false
    }
  }

  const start = () => {
    stop()
    if (immediate) poll()
    timer = setInterval(poll, interval)
  }

  const stop = () => {
    if (timer) {
      clearInterval(timer)
      timer = null
    }
  }

  onUnmounted(stop)

  return {
    start,
    stop,
    isPolling
  }
}
