import { defineStore } from 'pinia'
import { ref, watch, computed } from 'vue'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
import { Store } from '@tauri-apps/plugin-store'
import { emptyWslConfig, toConnectionConfig } from '../api/connection'
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
  const connectionMode = ref<'wsl' | 'ssh' | 'desktop'>('wsl')
  const wslDistro = ref('')
  const sshHost = ref('')
  const sshPort = ref(22)
  const sshUser = ref('')
  const sshPassword = ref('')

  // 多引擎连接配置
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

  // 当 activeConnection 改变时，同步旧字段以维护向下兼容
  watch(activeConnection, (newVal) => {
    if (newVal) {
      connectionMode.value = newVal.type as any
      wslDistro.value = newVal.wslDistro || ''
      sshHost.value = newVal.sshHost || ''
      sshPort.value = newVal.sshPort || 22
      sshUser.value = newVal.sshUser || ''
      sshPassword.value = newVal.sshPassword || ''
    }
  }, { immediate: true, deep: true })

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
        connectionMode.value = (await store.get<'wsl' | 'ssh' | 'desktop'>('connectionMode')) ?? 'wsl'
        wslDistro.value = (await store.get<string>('wslDistro')) ?? ''
        sshHost.value = (await store.get<string>('sshHost')) ?? ''
        sshPort.value = (await store.get<number>('sshPort')) ?? 22
        sshUser.value = (await store.get<string>('sshUser')) ?? ''
        sshPassword.value = (await store.get<string>('sshPassword')) ?? ''
        
        // 加载多连接引擎配置
        const savedConnections = await store.get<DockerConnection[]>('connections')
        const savedActiveConnectionId = await store.get<string>('activeConnectionId')
        if (savedConnections && Array.isArray(savedConnections) && savedConnections.length > 0) {
          connections.value = savedConnections
          activeConnectionId.value = savedActiveConnectionId || savedConnections[0].id
        } else {
          // 兼容旧配置转换
          if (connectionMode.value === 'wsl') {
            connections.value = [
              {
                id: 'conn_default_wsl',
                name: `WSL (${wslDistro.value || 'Ubuntu'})`,
                type: 'wsl',
                wslDistro: wslDistro.value || 'Ubuntu'
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
                name: `SSH (${sshUser.value || 'root'}@${sshHost.value || 'localhost'})`,
                type: 'ssh',
                sshHost: sshHost.value,
                sshPort: sshPort.value,
                sshUser: sshUser.value,
                sshPassword: sshPassword.value
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
  const saveSettings = async () => {
    try {
      const store = await getStore()
      await store.set('autoStart', autoStart.value)
      await store.set('closeToTray', closeToTray.value)
      await store.set('theme', theme.value)
      await store.set('refreshInterval', refreshInterval.value)
      await store.set('visibleMenus', visibleMenus.value)
      await store.set('connectionMode', connectionMode.value)
      await store.set('wslDistro', wslDistro.value)
      await store.set('sshHost', sshHost.value)
      await store.set('sshPort', sshPort.value)
      await store.set('sshUser', sshUser.value)
      await store.set('sshPassword', sshPassword.value)
      await store.set('connections', connections.value)
      await store.set('activeConnectionId', activeConnectionId.value)
      await store.set('registries', registries.value)
      await store.set('currentRegistryId', currentRegistryId.value)
      
      // 执行物理落盘
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
    getActiveConnectionConfig
  }
})

