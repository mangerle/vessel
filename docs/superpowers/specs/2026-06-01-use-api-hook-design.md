# useApi Hook 设计规格

## 目标
实现一个通用的 Vue 3 Composition API Hook，用于标准化 API 处理逻辑，减少重复代码，并统一错误处理体验。

## 架构
该 Hook 封装了异步请求的生命周期管理。

### 核心功能
1.  **状态追踪**：自动维护 `loading` (boolean), `data` (T | null), `error` (any) 状态。
2.  **错误处理**：捕获异常并通过 Naive UI 的 `message.error` 进行通知。
3.  **成功反馈**：可选通过 `successMsg` 提供 `message.success` 提示。
4.  **生命周期钩子**：提供 `onSuccess` 和 `onError` 回调函数。
5.  **类型安全**：全 TypeScript 支持，利用泛型确保输入参数和返回结果的类型正确。

### 接口定义
```typescript
export function useApi<T, Args extends any[]>(
  apiFn: (...args: Args) => Promise<T>,
  options?: {
    onSuccess?: (data: T) => void
    onError?: (err: any) => void
    successMsg?: string
  }
): {
  data: Ref<T | null>
  loading: Ref<boolean>
  error: Ref<any>
  execute: (...args: Args) => Promise<T>
}
```

## 错误处理与反馈
- 错误信息将转换为字符串并显示。
- 即使发生错误，`loading` 状态也会在 `finally` 块中重置。

## 测试策略
- 验证在调用 `execute` 时 `loading` 变为 `true`。
- 验证成功后 `data` 更新且 `onSuccess` 被触发。
- 验证失败后 `error` 更新且 `message.error` 被调用。
