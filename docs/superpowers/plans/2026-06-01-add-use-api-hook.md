# add useApi hook 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 实现 `src/hooks/useApi.ts`，提供标准化的 API 调用状态管理和错误提示逻辑。

**架构：** 使用 Vue 3 Composition API 管理响应式状态，集成 Naive UI `useMessage`。

**技术栈：** Vue 3, TypeScript, Naive UI.

---

### 任务 1：创建 `src/hooks/useApi.ts`

**文件：**
- 创建：`src/hooks/useApi.ts`

- [ ] **步骤 1：实现 `useApi` Hook**

```typescript
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
      data.value = res as any
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
```

- [ ] **步骤 2：Commit 变更**

```bash
git add src/hooks/useApi.ts
git commit -m "feat(frontend): add useApi hook for standardized API handling"
```
