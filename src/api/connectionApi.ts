import { invoke } from '@tauri-apps/api/core'
import { CMD } from './commands'
import type { ConnectionConfigPayload } from './connection'
import type { SshDiagnostic } from './types'

/**
 * 连接管理 API 服务
 * 后端命令集中在 `src-tauri/src/connection/`。
 */
export const connectionApi = {
  /**
   * 同步完整连接配置到后端（模式 + 主机 + 端口 + 用户 + 密码 + sudo）
   */
  updateConfig: (config: ConnectionConfigPayload) =>
    invoke<void>(CMD.updateConnectionConfig, { config }),

  /**
   * 探测当前活动连接是否连通
   */
  ping: () => invoke<void>(CMD.pingDocker),

  /**
   * SSH 远端 Docker 环境诊断
   */
  diagnoseSsh: (config: ConnectionConfigPayload) =>
    invoke<SshDiagnostic>(CMD.diagnoseSshConnection, { config })
}
