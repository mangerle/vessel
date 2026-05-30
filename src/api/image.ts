import { invoke } from '@tauri-apps/api/core'
import type { ImageInfo, ImageDetails, ImageSearchResult, ImageHistoryInfo } from './types'

/**
 * 镜像拉取的参数接口
 */
export interface PullParams {
  imageName: string
  username?: string | null
  password?: string | null
  serverAddress?: string | null
}

/**
 * 镜像相关的 API 服务
 */
export const imageApi = {
  /**
   * 获取镜像列表
   */
  list: () => invoke<ImageInfo[]>('list_images'),

  /**
   * 获取镜像详情
   */
  inspect: (id: string) => invoke<ImageDetails>('inspect_image', { id }),

  /**
   * 删除镜像
   */
  remove: (id: string) => invoke<void>('remove_image', { id }),

  /**
   * 搜索镜像
   */
  search: (term: string) => invoke<ImageSearchResult[]>('search_images', { term }),

  /**
   * 拉取镜像
   */
  pull: (params: PullParams) => invoke<void>('pull_image', params as any),

  /**
   * 清理虚悬镜像
   */
  prune: () => invoke<{ deleted_count: number; space_reclaimed: number }>('prune_images'),

  /**
   * 获取镜像历史
   */
  history: (id: string) => invoke<ImageHistoryInfo[]>('get_image_history', { id }),

  /**
   * 导出镜像
   */
  export: (imageIdOrName: string, path: string) => invoke<void>('export_image', { imageIdOrName, path }),

  /**
   * 导入镜像
   */
  import: (path: string) => invoke<void>('import_image', { path }),

  /**
   * 为镜像打标签
   */
  tag: (imageName: string, repo: string, tag: string) => invoke<void>('tag_image', { imageName, repo, tag })
}
