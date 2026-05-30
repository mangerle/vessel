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
