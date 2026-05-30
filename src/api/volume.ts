import { invoke } from '@tauri-apps/api/core'
import type { VolumeInfo, VolumeUser } from './types'

/**
 * 获取数据卷列表
 */
export async function listVolumes(): Promise<VolumeInfo[]> {
  return await invoke<VolumeInfo[]>('list_volumes')
}

/**
 * 获取使用特定卷的容器
 * @param name 卷名称
 */
export async function listVolumeContainers(name: string): Promise<VolumeUser[]> {
  return await invoke<VolumeUser[]>('list_volume_containers', { name })
}

/**
 * 在文件管理器中打开卷路径
 * @param path 卷路径
 */
export async function openVolumePath(path: string): Promise<void> {
  await invoke('open_volume_path', { path })
}

/**
 * 删除数据卷
 * @param name 卷名称
 */
export async function removeVolume(name: string): Promise<void> {
  await invoke('remove_volume', { name })
}

/**
 * 清理未使用的数据卷
 */
export async function pruneVolumes(): Promise<void> {
  await invoke('prune_volumes')
}
