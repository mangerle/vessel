import { ref } from 'vue'
import { useMessage } from 'naive-ui'

/**
 * 通用的 API 处理 Hook
 * @param apiFn API 请求函数
 * @param options 配置项
 */
export function useApi<T, Args extends any[]>(
  apiFn: (...args: Args) => Promise<T>,
  options: {
    onSuccess?: (data: T) => void
    onError?: (err: any) => void
    successMsg?: string
  } = {}
) {
  const data = ref<T | null>(null)
  const loading = ref(false)
  const error = ref<any>(null)
  const message = useMessage()

  const execute = async (...args: Args): Promise<T> => {
    loading.value = true
    error.value = null
    try {
      const res = await apiFn(...args)
      data.value = res
      if (options.successMsg) {
        message.success(options.successMsg)
      }
      options.onSuccess?.(res)
      return res
    } catch (err) {
      error.value = err
      message.error(String(err))
      options.onError?.(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    data,
    loading,
    error,
    execute
  }
}
