import { invoke } from '@tauri-apps/api/core'
import { CMD } from './commands'
import type { ContainerFileInfo } from './types'

/**
 * 容器内文件系统 API 服务
 * 后端命令集中在 `src-tauri/src/docker/fs.rs`。
 */
export const containerFsApi = {
  listFiles: (id: string, path: string) =>
    invoke<ContainerFileInfo[]>(CMD.listContainerFiles, { id, path }),

  download: (id: string, containerPath: string, localPath: string) =>
    invoke<void>(CMD.downloadFileFromContainer, { id, containerPath, localPath }),

  upload: (id: string, localPath: string, containerDir: string) =>
    invoke<void>(CMD.uploadFileToContainer, { id, localPath, containerDir }),

  delete: (id: string, path: string) =>
    invoke<void>(CMD.deleteContainerFile, { id, path }),

  create: (id: string, path: string, isDir: boolean) =>
    invoke<void>(CMD.createContainerFile, { id, path, isDir }),

  rename: (id: string, src: string, dest: string) =>
    invoke<void>(CMD.renameContainerFile, { id, src, dest }),

  readText: (id: string, path: string) =>
    invoke<string>(CMD.readContainerTextFile, { id, path }),

  writeText: (id: string, path: string, content: string) =>
    invoke<void>(CMD.writeContainerTextFile, { id, path, content })
}
