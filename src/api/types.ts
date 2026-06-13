export interface ContainerInfo {
  id: string
  name: string
  state: string
  status?: string
  image: string
  compose_project?: string
  /**
   * Docker 原始 labels；当前后端 ContainerInfo 不导出此字段，
   * 但前端按 `'com.docker.compose.project' in labels` 兜底判定 compose 容器。
   */
  labels?: Record<string, string>
}

export interface PortMapping {
  private_port: number
  public_port?: number
  type_: string
  ip?: string
}

export interface MountInfo {
  source: string
  destination: string
  mode: string
  rw: boolean
}

export interface ComposeProject {
  name: string
  container_count: number
  running_count: number
  status: string
  working_dir?: string
  config_file?: string
}

export interface ContainerDetails {
  id: string
  name: string
  image: string
  image_id: string
  state: string
  status: string
  created: string
  env: string[]
  ports: PortMapping[]
  mounts: MountInfo[]
}

export interface ImageInfo {
  id: string
  tags: string[]
  size: number
  created: number
}

export interface ImageDetails extends Omit<ImageInfo, 'created'> {
  created: string
  architecture: string
  os: string
  env: string[]
  exposed_ports: string[]
  cmd: string[]
  entrypoint: string[]
  author?: string
  docker_version?: string
}

export interface ImageSearchResult {
  name: string
  description: string
  is_official: boolean
  star_count: number
}

export interface ImageHistoryInfo {
  id: string
  created: number
  created_by: string
  size: number
}

export interface PullProgress {
  status?: string
  progress?: string
  id?: string
  stream?: string
  error?: string
  progressDetail?: {
  current?: number
  total?: number
  }
  }

  export interface NetworkInfo {
  id: string
  name: string
  driver: string
  scope: string
  created: string
  }

  export interface ConnectedContainer {
  id: string
  name: string
  ipv4_address: string
  ipv6_address: string
  mac_address: string
  }

  export interface NetworkDetails {
  id: string
  name: string
  driver: string
  scope: string
  created: string
  internal: boolean
  attachable: boolean
  ingress: boolean
  subnet: string
  gateway: string
  containers: ConnectedContainer[]
  options: Record<string, string>
  labels: Record<string, string>
  }

  export interface VolumeInfo {
  name: string
  driver: string
  mountpoint: string
  created: string
  }

  export interface VolumeUser {
  container_id: string
  container_name: string
  source: string
  destination: string
  mode: string
  rw: boolean
  }

  /**
   * SSH 远端 Docker 环境诊断结果。
   * 与后端 `src-tauri/src/connection/ssh.rs::SshDiagnostic` 字段一一对应。
   * 任何字段新增/重命名都需要前后端同步。
   */
  export interface SshDiagnostic {
    /** SSH 凭据与网络是否可达 */
    ssh_ok: boolean
    ssh_error: string | null

    /** 当前 SSH 登录的用户 */
    current_user: string
    /** 用户所属的附加组列表 */
    groups: string[]
    /** 是否在 docker 组 */
    user_in_docker_group: boolean

    /** docker socket 路径 */
    docker_socket_path: string
    /** docker socket 权限串 */
    docker_socket_perms: string
    /** docker socket 属组 */
    docker_socket_group: string

    /** `docker ps` 不带 sudo 是否成功 */
    docker_works_without_sudo: boolean
    docker_error_without_sudo: string | null

    /** `sudo -n docker ps` 是否成功 */
    docker_works_with_sudo: boolean
    docker_error_with_sudo: string | null

    /** 远端环境其他错误 */
    remote_error: string | null

    /** 自动给出的修复建议 */
    recommendation: string
  }

  /**
   * Docker stats 流 payload 形状（与 bollard ContainerStatsResponse 字段子集对齐）。
   * 监听 `container-stats-<id>` 事件时使用本类型替代 `any`。
   */
  export interface ContainerStatsPayload {
    cpu_stats?: {
      cpu_usage?: { total_usage?: number; percpu_usage?: number[] }
      system_cpu_usage?: number
      online_cpus?: number
    }
    precpu_stats?: {
      cpu_usage?: { total_usage?: number }
      system_cpu_usage?: number
    }
    memory_stats?: { usage?: number }
    networks?: Record<string, { rx_bytes?: number; tx_bytes?: number }>
    blkio_stats?: {
      io_service_bytes_recursive?: Array<{ op?: string; value?: number }>
    }
  }

  /** 容器内文件 / 目录条目 */
  export interface ContainerFileInfo {
    name: string
    is_dir: boolean
    size: number
    mtime: number
    permissions: string
  }

  /** 容器 exec 进程 Top 标题 */
  export type ContainerTopTitles = string[]
  /** 容器 exec 进程 Top 数据行 */
  export type ContainerTopProcess = string[]

  /** 容器 exec 单次命令结果 */
  export interface ContainerExecResult {
    exit_code: number | null
    output: string
  }

