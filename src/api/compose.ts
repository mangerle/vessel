import { invoke } from '@tauri-apps/api/core'
import type { ComposeProject } from './types'

/**
 * Docker Compose 相关的 API 服务
 */
export const composeApi = {
  /**
   * 获取 Compose 项目列表
   */
  listProjects: () => invoke<ComposeProject[]>('list_compose_projects'),

  /**
   * 读取项目的 Compose 文件
   */
  readFile: (path: string, mode?: string, distro?: string) => 
    invoke<string>('read_compose_file', { path, mode, distro }),

  /**
   * 保存项目的 Compose 文件
   */
  writeFile: (path: string, content: string, mode?: string, distro?: string) => 
    invoke<void>('write_compose_file', { path, content, mode, distro }),

  /**
   * 运行 Compose 命令
   */
  runCommand: (projectDir: string, args: string[], mode?: string, distro?: string) => 
    invoke<void>('run_compose_command', { projectDir, args, mode, distro })
}
