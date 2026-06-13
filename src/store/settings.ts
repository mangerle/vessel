import { defineStore } from 'pinia'
import { ref, watch, computed } from 'vue'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
import { Store } from '@tauri-apps/plugin-store'
import { emptyWslConfig, matchesConnectionConfig, toConnectionConfig } from '../api/connection'
import type { ConnectionConfigPayload } from '../api/connection'

// 声明全局惰性 Store 实例
let storeInstance: Store | null = null
const getStore = async () => {
  if (!storeInstance) {
    storeInstance = await Store.load('settings.json')
  }
  return storeInstance
}

export interface Registry {
  id: string
  name: string
  url: string
  username?: string
  password?: string
  isDefault?: boolean
}

export interface DockerConnection {
  id: string
  name: string
  type: 'wsl' | 'ssh' | 'desktop'
  wslDistro?: string
  sshHost?: string
  sshPort?: number
  sshUser?: string
  sshPassword?: string
  /** SSH 模式下是否使用 sudo 提升权限调用 docker */
  useSudo?: boolean
}

export type { ConnectionConfigPayload }

export const useSettingsStore = defineStore('settings', () => {
  const autoStart = ref(false)
  const closeToTray = ref(true)
  const theme = ref<'deep-black' | 'zed-gray' | 'light-apple'>('deep-black')
  const refreshInterval = ref(3) // 默认 3 秒
  const visibleMenus = ref<string[]>(['compose', 'containers', 'images', 'networks', 'volumes'])

  // 旧字段（connectionMode/wslDistro/sshHost/sshPort/sshUser/sshPassword）
  // 改为 computed 派生自 activeConnection，向后兼容旧消费方（如 Volumes.vue）
  // 的同时消除「双源 + 双向 watch」架构硬伤。

  // 多引擎连接配置（单一数据源）
  const connections = ref<DockerConnection[]>([
    {
      id: 'conn_default_desktop',
      name: 'Docker Desktop',
      type: 'desktop'
    },
    {
      id: 'conn_default_wsl',
      name: 'WSL (Ubuntu)',
      type: 'wsl',
      wslDistro: 'Ubuntu'
    }
  ])
  const activeConnectionId = ref<string>('conn_default_desktop')

  const activeConnection = computed(() => {
    return connections.value.find(c => c.id === activeConnectionId.value) || connections.value[0]
  })

  // 旧字段 computed 派生（消除双向 watch 回环，单一数据源 = activeConnection）
  const connectionMode = computed<'wsl' | 'ssh' | 'desktop'>(() =>
    (activeConnection.value?.type as 'wsl' | 'ssh' | 'desktop') || 'wsl'
  )
  const wslDistro = computed(() => activeConnection.value?.wslDistro || '')
  const sshHost = computed(() => activeConnection.value?.sshHost || '')
  const sshPort = computed(() => activeConnection.value?.sshPort ?? 22)
  const sshUser = computed(() => activeConnection.value?.sshUser || '')
  const sshPassword = computed(() => activeConnection.value?.sshPassword || '')

  // 镜像仓库配置列表，默认包含宿主机环境
  const registries = ref<Registry[]>([
    {
      id: 'default',
      name: '使用宿主机环境',
      url: '',
      username: '',
      password: '',
      isDefault: true
    }
  ])
  const currentRegistryId = ref('default')
  
  // 监听主题变化，更新 HTML 属性
  watch(theme, (newTheme) => {
    document.documentElement.setAttribute('data-theme', newTheme)
  }, { immediate: true })

  // 异步加载本地配置文件中的设置
  const loadSettings = async () => {
    try {
      autoStart.value = await isEnabled()

      const store = await getStore()
      const hasSavedTheme = await store.get<string>('theme')
      if (hasSavedTheme !== null) {
        theme.value = hasSavedTheme as any
        closeToTray.value = (await store.get<boolean>('closeToTray')) ?? true
        refreshInterval.value = (await store.get<number>('refreshInterval')) ?? 3
        visibleMenus.value = (await store.get<string[]>('visibleMenus')) ?? ['compose', 'containers', 'images', 'networks', 'volumes']

        // 读取旧字段到临时局部变量，仅用于一次性迁移到 connections[]
        const legacyMode = (await store.get<'wsl' | 'ssh' | 'desktop'>('connectionMode')) ?? 'wsl'
        const legacyDistro = (await store.get<string>('wslDistro')) ?? ''
        const legacyHost = (await store.get<string>('sshHost')) ?? ''
        const legacyPort = (await store.get<number>('sshPort')) ?? 22
        const legacyUser = (await store.get<string>('sshUser')) ?? ''
        const legacyPassword = (await store.get<string>('sshPassword')) ?? ''

        // 加载多连接引擎配置
        const savedConnections = await store.get<DockerConnection[]>('connections')
        const savedActiveConnectionId = await store.get<string>('activeConnectionId')
        if (savedConnections && Array.isArray(savedConnections) && savedConnections.length > 0) {
          connections.value = savedConnections
          activeConnectionId.value = savedActiveConnectionId || savedConnections[0].id
        } else {
          // 一次性从旧字段迁移到 connections[]
          if (legacyMode === 'wsl') {
            connections.value = [
              {
                id: 'conn_default_wsl',
                name: `WSL (${legacyDistro || 'Ubuntu'})`,
                type: 'wsl',
                wslDistro: legacyDistro || 'Ubuntu'
              },
              {
                id: 'conn_default_desktop',
                name: 'Docker Desktop (命名管道)',
                type: 'desktop'
              }
            ]
            activeConnectionId.value = 'conn_default_wsl'
          } else {
            connections.value = [
              {
                id: 'conn_default_ssh',
                name: `SSH (${legacyUser || 'root'}@${legacyHost || 'localhost'})`,
                type: 'ssh',
                sshHost: legacyHost,
                sshPort: legacyPort,
                sshUser: legacyUser,
                sshPassword: legacyPassword
              },
              {
                id: 'conn_default_desktop',
                name: 'Docker Desktop (命名管道)',
                type: 'desktop'
              }
            ]
            activeConnectionId.value = 'conn_default_ssh'
          }
        }

        const savedRegistries = await store.get<Registry[]>('registries')
        if (savedRegistries && Array.isArray(savedRegistries)) {
          registries.value = savedRegistries
        }
        currentRegistryId.value = (await store.get<string>('currentRegistryId')) ?? 'default'
      } else {
        // 若本地没有配置文件，则将当前的默认值保存落盘
        await saveSettings()
      }
    } catch (e) {
      console.error('从 settings.json 加载配置失败:', e)
    }
  }
  
  const setAutoStart = async (value: boolean) => {
    try {
      if (value) {
        await enable()
      } else {
        await disable()
      }
      autoStart.value = value
      await saveSettings()
    } catch (e) {
      console.error('设置自启动失败:', e)
    }
  }

  const setCloseToTray = async (value: boolean) => {
    closeToTray.value = value
    await saveSettings()
  }

  const setTheme = async (value: 'deep-black' | 'zed-gray' | 'light-apple') => {
    theme.value = value
    await saveSettings()
  }
  
  // 异步将当前的内存设置保存到本地物理 json 文件中
  // 写入采用「只写变更 key」策略：首次保存全部字段，之后仅持久化与上次不同的字段
  const lastSavedSnapshot = ref<Record<string, unknown> | null>(null)
  const deepEqual = (a: unknown, b: unknown): boolean => {
    if (a === b) return true
    if (a === null || b === null) return false
    if (typeof a !== typeof b) return false
    if (typeof a !== 'object') return false
    if (Array.isArray(a) !== Array.isArray(b)) return false
    if (Array.isArray(a) && Array.isArray(b)) {
      if (a.length !== b.length) return false
      for (let i = 0; i < a.length; i++) {
        if (!deepEqual(a[i], b[i])) return false
      }
      return true
    }
    const ka = Object.keys(a as object)
    const kb = Object.keys(b as object)
    if (ka.length !== kb.length) return false
    for (const k of ka) {
      if (!deepEqual((a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k])) {
        return false
      }
    }
    return true
  }

  const saveSettings = async () => {
    try {
      const store = await getStore()
      // 待持久化的键值：不再写 legacy 字段（connectionMode/wslDistro/sshHost/sshPort/sshUser/sshPassword），
      // 旧版本兼容由 loadSettings 一次性迁移路径承担。
      const snapshot: Record<string, unknown> = {
        autoStart: autoStart.value,
        closeToTray: closeToTray.value,
        theme: theme.value,
        refreshInterval: refreshInterval.value,
        visibleMenus: visibleMenus.value,
        connections: connections.value,
        activeConnectionId: activeConnectionId.value,
        registries: registries.value,
        currentRegistryId: currentRegistryId.value
      }
      // 仅 set 与上次不同的 key，减少 IPC 次数
      if (lastSavedSnapshot.value) {
        for (const [k, v] of Object.entries(snapshot)) {
          if (!deepEqual(v, lastSavedSnapshot.value[k])) {
            await store.set(k, v)
          }
        }
      } else {
        for (const [k, v] of Object.entries(snapshot)) {
          await store.set(k, v)
        }
      }
      lastSavedSnapshot.value = snapshot
      await store.save()
    } catch (e) {
      console.error('保存配置到 settings.json 失败:', e)
    }
  }
  
  // 恢复出厂设置，直接清空本地配置文件
  const resetSettings = async () => {
    try {
      const store = await getStore()
      await store.clear()
      await store.save()
    } catch (e) {
      console.error('清除 settings.json 失败:', e)
    }
  }

  /**
   * 取得当前活动连接的 ConnectionConfig 形状（用于 invoke update_connection_config）。
   * 如果没有任何连接，则回退到 WSL 默认值。
   */
  const getActiveConnectionConfig = (): ConnectionConfigPayload => {
    const conn = connections.value.find(c => c.id === activeConnectionId.value)
      || connections.value[0]
    if (!conn) {
      return emptyWslConfig()
    }
    return toConnectionConfig(conn)
  }

  /**
   * 应用后端推送的 ConnectionConfig（来自 connection-updated 事件）：
   * 在 connections[] 中按 (mode + 关键字段) 匹配 id，校正 activeConnectionId。
   * 用于多窗口/托盘切换场景下前后端 active 状态同步。
   * 不修改 connections[] 内容（用户的未保存编辑由 Settings.vue 路径负责）。
   */
  const applyBackendConfig = (cfg: ConnectionConfigPayload): void => {
    const matched = connections.value.find(c => matchesConnectionConfig(c, cfg))
    if (matched) {
      activeConnectionId.value = matched.id
    }
  }

  return {
    autoStart,
    closeToTray,
    theme,
    refreshInterval,
    visibleMenus,
    connectionMode,
    wslDistro,
    sshHost,
    sshPort,
    sshUser,
    sshPassword,
    connections,
    activeConnectionId,
    activeConnection,
    registries,
    currentRegistryId,
    loadSettings,
    setAutoStart,
    setCloseToTray,
    setTheme,
    saveSettings,
    resetSettings,
    getActiveConnectionConfig,
    applyBackendConfig
  }
})

