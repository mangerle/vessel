import { invoke } from '@tauri-apps/api/core'
import type { VolumeInfo, VolumeUser } from './types'

/** 卷内文件项（与后端 `VolumeFileEntry` 一一对应） */
export interface VolumeFileEntry {
  name: string
  is_dir: boolean
  path: string
}

/**
 * 数据卷相关的 API 服务
 */
export const volumeApi = {
  /**
   * 获取数据卷列表
   */
  list: () => invoke<VolumeInfo[]>('list_volumes'),

  /**
   * 获取使用特定卷的容器
   * @param name 卷名称
   */
  listContainers: (name: string) => invoke<VolumeUser[]>('list_volume_containers', { name }),

  /**
   * 在文件管理器中打开卷路径
   * @param path 卷路径
   */
  openPath: (path: string) => invoke<void>('open_volume_path', { path }),

  /**
   * 删除数据卷
   * @param name 卷名称
   */
  remove: (name: string) => invoke<void>('remove_volume', { name }),

  /**
   * 清理未使用的数据卷
   */
  prune: () => invoke<void>('prune_volumes'),

  // ---------- 卷文件浏览器（修复 P0-7 / P0-17） ----------

  /** 列出卷内目录文件（按当前连接模式自动分派 Desktop/WSL/SSH） */
  listFiles: (volume: string, path: string) =>
    invoke<VolumeFileEntry[]>('list_volume_files', { volume, path }),

  /** 读取卷内文本文件 */
  readTextFile: (volume: string, path: string) =>
    invoke<string>('read_volume_text_file', { volume, path }),

  /** 写入卷内文本文件 */
  writeTextFile: (volume: string, path: string, content: string) =>
    invoke<void>('write_volume_text_file', { volume, path, content })
}
