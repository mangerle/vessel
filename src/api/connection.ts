import type { DockerConnection } from '../store/settings'

/**
 * 与 Rust 端 `ConnectionConfig` 形状完全一致的前端序列化结果。
 * 用于 invoke `update_connection_config` / `diagnose_ssh_connection`。
 */
export interface ConnectionConfigPayload {
  mode: 'wsl' | 'ssh' | 'desktop'
  name: string
  wsl_distro: string | null
  ssh_host: string | null
  ssh_port: number | null
  ssh_user: string | null
  ssh_password: string | null
  use_sudo: boolean
}

/** SSH 默认端口，避免在前端散落 22 这个魔法数字 */
export const DEFAULT_SSH_PORT = 22

/**
 * 把前端的 DockerConnection 序列化为后端 ConnectionConfig 形状。
 * 未填写的可选字段统一置 null。
 */
export const toConnectionConfig = (conn: DockerConnection): ConnectionConfigPayload => {
  return {
    mode: conn.type,
    name: conn.name,
    wsl_distro: conn.type === 'wsl' ? (conn.wslDistro ?? null) : null,
    ssh_host: conn.type === 'ssh' ? (conn.sshHost ?? null) : null,
    ssh_port: conn.type === 'ssh' ? (conn.sshPort ?? DEFAULT_SSH_PORT) : null,
    ssh_user: conn.type === 'ssh' ? (conn.sshUser ?? null) : null,
    ssh_password: conn.type === 'ssh' ? (conn.sshPassword ?? null) : null,
    use_sudo: conn.type === 'ssh' ? (conn.useSudo ?? false) : false
  }
}

/**
 * 构造一个完全为空的 WSL 默认配置（用于没有任何连接时的兜底）。
 */
export const emptyWslConfig = (): ConnectionConfigPayload => ({
  mode: 'wsl',
  name: 'WSL',
  wsl_distro: null,
  ssh_host: null,
  ssh_port: null,
  ssh_user: null,
  ssh_password: null,
  use_sudo: false
})

/**
 * 把后端 ConnectionConfig 投影为 DockerConnection-shape 的可识别特征，
 * 用于在前端 connections[] 中按 (mode + 关键字段) 匹配。
 */
export const matchesConnectionConfig = (
  conn: DockerConnection,
  cfg: ConnectionConfigPayload
): boolean => {
  if (conn.type !== cfg.mode) return false
  if (cfg.mode === 'wsl') {
    return (conn.wslDistro ?? null) === cfg.wsl_distro
  }
  if (cfg.mode === 'ssh') {
    return (
      (conn.sshHost ?? null) === cfg.ssh_host &&
      (conn.sshPort ?? DEFAULT_SSH_PORT) === (cfg.ssh_port ?? DEFAULT_SSH_PORT) &&
      (conn.sshUser ?? null) === cfg.ssh_user
    )
  }
  // desktop
  return cfg.mode === 'desktop'
}
