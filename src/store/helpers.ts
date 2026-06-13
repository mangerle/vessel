/**
 * Pinia store 通用 action 模板
 *
 * container/network/volume/image 等 store 都重复了相同的「loading + 错误捕获 + 末尾刷新」骨架，
 * 这里抽到一个泛型 helper：调用方只需传入有 loading/error 字段的 store 引用、
 * 一个动作名（用于错误日志）、动作函数、可选的刷新函数即可。
 *
 * 设计取舍：
 * - 不强制 store 必须使用某个基类，只要求实现 LoadingError 形状（多数 store 已天然满足）。
 * - refresh 接受 `() => Promise<void>` 或 `null`，避免每个 store 自己 `if (refresh)` 分支。
 * - 错误重新抛出，让 view 层可继续捕获 toast，避免静默吞错。
 */

import { error as logError } from '@tauri-apps/plugin-log'

export interface LoadingErrorState {
  loading: boolean
  error: string | null
}

/**
 * 通用 store action 包装器：
 * 1. 进入时置 loading=true、清空 error；
 * 2. 执行 actionFn 并保留其返回值；
 * 3. 若提供 refresh，则在动作成功后调用之；
 * 4. 出错时记录 error 字段并通过 tauri-plugin-log 落盘后端日志，再向上抛出；
 * 5. 无论成功失败 finally 关闭 loading。
 */
export async function runStoreAction<T>(
  state: LoadingErrorState,
  actionName: string,
  actionFn: () => Promise<T>,
  refresh: (() => Promise<void>) | null = null
): Promise<T> {
  state.loading = true
  state.error = null
  try {
    const result = await actionFn()
    if (refresh) {
      await refresh()
    }
    return result
  } catch (err) {
    // 走 tauri-plugin-log 而非 console.error：
    // attachConsole 虽然能桥接，但显式调用更贴合错误路径语义，便于后端按 level 过滤。
    logError(`${actionName}失败: ${err}`).catch(() => {})
    state.error = String(err)
    throw err
  } finally {
    state.loading = false
  }
}
