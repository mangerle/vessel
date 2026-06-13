/**
 * Tauri 命令名常量（前端协议层）
 *
 * 与后端 `src-tauri/src/docker/*::#[tauri::command]` 函数名一一对应。
 * 所有 invoke 调用必须引用本文件中的常量，禁止在 view / store 层
 * 直接硬编码命令名字符串。
 *
 * 命名规则：动词_名词，全小写，下划线分隔。
 */

export const CMD = {
  // 容器
  listLocalContainers: 'list_local_containers',
  startContainer: 'start_container',
  stopContainer: 'stop_container',
  restartContainer: 'restart_container',
  removeContainer: 'remove_container',
  renameContainer: 'rename_container',
  commitContainer: 'commit_container',
  inspectContainer: 'inspect_container',
  pauseContainer: 'pause_container',
  unpauseContainer: 'unpause_container',
  topContainer: 'top_container',
  execContainer: 'exec_container',
  streamContainerStats: 'stream_container_stats',
  closeContainerStats: 'close_container_stats',
  streamContainerLogs: 'stream_container_logs',
  closeContainerLogs: 'close_container_logs',
  createContainerTerminal: 'create_container_terminal',
  closeContainerTerminal: 'close_container_terminal',
  writeToTerminal: 'write_to_terminal',
  resizeContainerTerminal: 'resize_container_terminal',

  // 容器内文件系统
  listContainerFiles: 'list_container_files',
  downloadFileFromContainer: 'download_file_from_container',
  uploadFileToContainer: 'upload_file_to_container',
  deleteContainerFile: 'delete_container_file',
  createContainerFile: 'create_container_file',
  renameContainerFile: 'rename_container_file',
  readContainerTextFile: 'read_container_text_file',
  writeContainerTextFile: 'write_container_text_file',

  // 镜像
  listImages: 'list_images',
  inspectImage: 'inspect_image',
  removeImage: 'remove_image',
  searchImages: 'search_images',
  getImageHistory: 'get_image_history',
  pullImage: 'pull_image',
  runImage: 'run_image',
  exportImage: 'export_image',
  importImage: 'import_image',
  tagImage: 'tag_image',
  pruneImages: 'prune_images',

  // Compose
  listComposeProjects: 'list_compose_projects',
  readComposeFile: 'read_compose_file',
  writeComposeFile: 'write_compose_file',
  runComposeCommand: 'run_compose_command',

  // 网络
  listNetworks: 'list_networks',
  getNetworkDetails: 'get_network_details',
  removeNetwork: 'remove_network',
  pruneNetworks: 'prune_networks',
  disconnectNetwork: 'disconnect_network',

  // 卷
  listVolumes: 'list_volumes',
  listVolumeContainers: 'list_volume_containers',
  removeVolume: 'remove_volume',
  openVolumePath: 'open_volume_path',
  pruneVolumes: 'prune_volumes',

  // WSL / 系统
  listWslDistros: 'list_wsl_distros',
  openConfigDir: 'open_config_dir',
  openLogDir: 'open_log_dir',

  // 连接
  updateConnectionConfig: 'update_connection_config',
  pingDocker: 'ping_docker',
  diagnoseSshConnection: 'diagnose_ssh_connection',
} as const

export type TauriCommand = typeof CMD[keyof typeof CMD]
