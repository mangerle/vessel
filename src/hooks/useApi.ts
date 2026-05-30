import { ref, shallowRef } from 'vue'
import { useMessage } from 'naive-ui'

/**
 * 通用的 API 处理 Hook，支持竞态保护和统一错误处理
 * @param apiFn API 请求函数
 * @param options 配置项
 */
export function useApi<T, Args extends any[]>(
  apiFn: (...args: Args) => Promise<T>,
  options: {
    onSuccess?: (data: T) => void
    onError?: (err: any) => void
    successMsg?: string
    shallow?: boolean
  } = {}
) {
  const data = options.shallow ? shallowRef<T | null>(null) : ref<T | null>(null)
  const loading = ref(false)
  const error = ref<any>(null)
  const message = useMessage()
  
  // 竞态保护：记录最后一次调用的 ID
  let lastCallId = 0

  const execute = async (...args: Args): Promise<T> => {
    const callId = ++lastCallId
    loading.value = true
    error.value = null

    try {
      const res = await apiFn(...args)
      
      // 只有最新的请求才能更新状态
      if (callId === lastCallId) {
        data.value = res as any
        if (options.successMsg) message.success(options.successMsg)
        options.onSuccess?.(res)
      }
      
      return res
    } catch (err: any) {
      if (callId === lastCallId) {
        error.value = err
        // 更好的错误处理，避免 [object Object]
        const errorMsg = typeof err === 'string' 
          ? err 
          : (err.message || err.error || (typeof err === 'object' ? JSON.stringify(err) : String(err)))
        
        message.error(errorMsg)
        options.onError?.(err)
      }
      throw err
    } finally {
      if (callId === lastCallId) {
        loading.value = false
      }
    }
  }

  return {
    data,
    loading,
    error,
    execute
  }
}
