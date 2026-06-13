/**
 * 前后端事件名常量（前后端协议）
 *
 * 任何对事件名的修改必须前后端同步；新增事件也应在此处集中定义。
 * 后端对应常量见 `src-tauri/src/docker/utils.rs::events`。
 */

export const EVT = {
  // 容器统计/日志流（按容器 id 区分频道）
  containerStats: (id: string) => `container-stats-${id}`,
  containerLogs: (id: string) => `container-logs-${id}`,

  // 终端 exec 流
  containerTerminalStdout: (execId: string) => `container-terminal-stdout-${execId}`,
  containerTerminalExit: (execId: string) => `container-terminal-exit-${execId}`,

  // 镜像拉取/导出/导入进度
  imagePullProgress: 'image-pull-progress',
  imagePullError: 'image-pull-error',
  imagePullFinished: 'image-pull-finished',
  imageExportProgress: 'image-export-progress',
  imageExportFinished: 'image-export-finished',
  imageExportError: 'image-export-error',
  imageImportProgress: 'image-import-progress',
  imageImportFinished: 'image-import-finished',
  imageImportError: 'image-import-error',

  // Compose 命令执行
  composeCmdOutput: 'compose-cmd-output',
  composeCmdFinished: 'compose-cmd-finished',
  composeCmdError: 'compose-cmd-error',

  // 连接配置变更通知（后端 update_connection_config 末尾 emit）
  connectionUpdated: 'connection-updated',
} as const
