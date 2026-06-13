import { invoke } from '@tauri-apps/api/core'
import { CMD } from './commands'
import type { ImageInfo, ImageDetails, ImageSearchResult, ImageHistoryInfo } from './types'

/**
 * 镜像拉取的参数接口
 * 字段名与后端 `src-tauri/src/docker/image.rs::pull_image` 形参（snake_case）保持一致。
 */
export interface PullParams extends Record<string, unknown> {
  image_name: string
  username?: string | null
  password?: string | null
  server_address?: string | null
}

/**
 * 镜像相关的 API 服务
 */
export const imageApi = {
  /**
   * 获取镜像列表
   */
  list: () => invoke<ImageInfo[]>(CMD.listImages),

  /**
   * 获取镜像详情
   */
  inspect: (id: string) => invoke<ImageDetails>(CMD.inspectImage, { id }),

  /**
   * 删除镜像
   */
  remove: (id: string) => invoke<void>(CMD.removeImage, { id }),

  /**
   * 搜索镜像
   */
  search: (term: string) => invoke<ImageSearchResult[]>(CMD.searchImages, { term }),

  /**
   * 拉取镜像
   */
  pull: (params: PullParams) => invoke<void>(CMD.pullImage, params),

  /**
   * 清理虚悬镜像
   */
  prune: () => invoke<{ deleted_count: number; space_reclaimed: number }>(CMD.pruneImages),

  /**
   * 获取镜像历史
   */
  history: (id: string) => invoke<ImageHistoryInfo[]>(CMD.getImageHistory, { id }),

  /**
   * 导出镜像
   */
  export: (imageIdOrName: string, path: string) =>
    invoke<void>(CMD.exportImage, { imageIdOrName, path }),

  /**
   * 导入镜像
   */
  import: (path: string) => invoke<void>(CMD.importImage, { path }),

  /**
   * 为镜像打标签
   */
  tag: (imageName: string, repo: string, tag: string) =>
    invoke<void>(CMD.tagImage, { imageName, repo, tag })
}
