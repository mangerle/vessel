export interface ContainerInfo {
  id: string
  name: string
  state: string
  image: string
  compose_project?: string
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

