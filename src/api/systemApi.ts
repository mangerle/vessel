import { invoke } from '@tauri-apps/api/core'
import { CMD } from './commands'

/**
 * 系统级 / 杂项 API 服务
 */

/** WSL 发行版列表 */
export const wslApi = {
  listDistros: () => invoke<string[]>(CMD.listWslDistros)
}

/** 文件系统打开（本地配置文件目录、日志目录） */
export const fsApi = {
  openConfigDir: () => invoke<void>(CMD.openConfigDir),
  openLogDir: () => invoke<void>(CMD.openLogDir)
}

/** 镜像单条操作（包含 `run_image` / `tag_image` 的特殊多参调用） */
export const imageOpsApi = {
  runImage: (params: {
    image: string
    name: string | null
    ports: string[]
    env: string[]
    restartPolicy: string
    binds: string[]
    tty: boolean
    openStdin: boolean
    cmd: string[] | null
    overwrite: boolean
  }) => invoke<string>(CMD.runImage, params),

  tagImage: (imageName: string, repo: string, tag: string) =>
    invoke<void>(CMD.tagImage, { imageName, repo, tag })
}
