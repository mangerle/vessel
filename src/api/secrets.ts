import { invoke } from '@tauri-apps/api/core'
import { CMD } from './commands'

/**
 * OS 凭据库 API（修复 P0-3）
 *
 * 把 SSH/Registry 密码从 settings.json 移到 OS Keychain：
 * - Windows: Credential Manager
 * - macOS:   Keychain
 * - Linux:   Secret Service / kwallet
 *
 * key 命名约定：
 * - SSH 密码： `conn:<connectionId>:sshPassword`
 * - Registry 密码：`registry:<registryId>:password`
 */
export const secretsApi = {
  set: (key: string, value: string) => invoke<void>(CMD.setSecret, { key, value }),
  get: (key: string) => invoke<string | null>(CMD.getSecret, { key }),
  remove: (key: string) => invoke<void>(CMD.deleteSecret, { key })
}

/** 构造连接 SSH 密码的 keyring key */
export const sshPasswordKey = (connectionId: string) =>
  `conn:${connectionId}:sshPassword`

/** 构造仓库密码的 keyring key */
export const registryPasswordKey = (registryId: string) =>
  `registry:${registryId}:password`
