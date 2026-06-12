import { invoke } from '@tauri-apps/api/core'
import type { ComposeProject } from './types'

/**
 * Docker Compose 相关的 API 服务
 *
 * 注意：所有 compose 相关命令现在统一从后端全局活动连接配置读取引擎信息，
 * 不再需要前端显式传入 mode / distro 等参数。
 */
export const composeApi = {
  /**
   * 获取 Compose 项目列表
   */
  listProjects: () => invoke<ComposeProject[]>('list_compose_projects'),

  /**
   * 读取项目的 Compose 文件
   */
  readFile: (path: string) => invoke<string>('read_compose_file', { path }),

  /**
   * 保存项目的 Compose 文件
   */
  writeFile: (path: string, content: string) =>
    invoke<void>('write_compose_file', { path, content }),

  /**
   * 运行 Compose 命令
   */
  runCommand: (projectDir: string, args: string[]) =>
    invoke<void>('run_compose_command', { projectDir, args })
}
