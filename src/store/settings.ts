import { defineStore } from 'pinia'
import { ref, watch, computed } from 'vue'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
import { Store } from '@tauri-apps/plugin-store'
import { error as logError, warn as logWarn } from '@tauri-apps/plugin-log'
import { emptyWslConfig, matchesConnectionConfig, toConnectionConfig, DEFAULT_SSH_PORT } from '../api/connection'
import { secretsApi, sshPasswordKey, registryPasswordKey } from '../api/secrets'
import { safeParseField, settingsFieldSchemas } from './settingsSchema'
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

  // 连接生命周期状态（启动 / 切换）
  // connectionReady=false 时由 App.vue 全屏 Loading 遮罩主界面，根除启动期 fetch race。
  // connectionSwitching=true 时由 MainLayout 弹出居中 modal，期间禁止旧连接的请求落入新连接。
  const connectionReady = ref(false)
  const connectionSwitching = ref(false)
  const switchingTargetName = ref('')

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
  const sshPort = computed(() => activeConnection.value?.sshPort ?? DEFAULT_SSH_PORT)
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
  // 修复 P1-8：每个字段先经 zod schema 校验，校验失败回退默认值并打 console.warn，
  // 避免用户手动改坏的 settings.json 直接污染 store 状态。
  const loadSettings = async () => {
    try {
      autoStart.value = await isEnabled()

      const store = await getStore()
      const hasSavedTheme = await store.get<string>('theme')
      if (hasSavedTheme !== null) {
        theme.value = safeParseField(settingsFieldSchemas.theme, hasSavedTheme, theme.value, 'theme')
        closeToTray.value = safeParseField(
          settingsFieldSchemas.closeToTray,
          await store.get<boolean>('closeToTray'),
          true,
          'closeToTray'
        )
        refreshInterval.value = safeParseField(
          settingsFieldSchemas.refreshInterval,
          await store.get<number>('refreshInterval'),
          3,
          'refreshInterval'
        )
        visibleMenus.value = safeParseField(
          settingsFieldSchemas.visibleMenus,
          await store.get<string[]>('visibleMenus'),
          ['compose', 'containers', 'images', 'networks', 'volumes'],
          'visibleMenus'
        )

        // 读取旧字段到临时局部变量，仅用于一次性迁移到 connections[]
        const legacyMode = safeParseField(
          settingsFieldSchemas.connectionMode,
          await store.get<'wsl' | 'ssh' | 'desktop'>('connectionMode'),
          'wsl',
          'connectionMode'
        )
        const legacyDistro = safeParseField(
          settingsFieldSchemas.wslDistro,
          await store.get<string>('wslDistro'),
          '',
          'wslDistro'
        )
        const legacyHost = safeParseField(
          settingsFieldSchemas.sshHost,
          await store.get<string>('sshHost'),
          '',
          'sshHost'
        )
        const legacyPort = safeParseField(
          settingsFieldSchemas.sshPort,
          await store.get<number>('sshPort'),
          DEFAULT_SSH_PORT,
          'sshPort'
        )
        const legacyUser = safeParseField(
          settingsFieldSchemas.sshUser,
          await store.get<string>('sshUser'),
          '',
          'sshUser'
        )
        const legacyPassword = safeParseField(
          settingsFieldSchemas.sshPassword,
          await store.get<string>('sshPassword'),
          '',
          'sshPassword'
        )

        // 加载多连接引擎配置（schema 校验失败 → 回退到 legacy 迁移分支）
        const savedConnectionsRaw = await store.get<DockerConnection[]>('connections')
        const parsedConnections = savedConnectionsRaw === null
          ? null
          : safeParseField(
              settingsFieldSchemas.connections,
              savedConnectionsRaw,
              [] as DockerConnection[],
              'connections'
            )
        const savedActiveConnectionId = safeParseField(
          settingsFieldSchemas.activeConnectionId,
          await store.get<string>('activeConnectionId'),
          '',
          'activeConnectionId'
        )
        if (parsedConnections && parsedConnections.length > 0) {
          connections.value = parsedConnections as DockerConnection[]
          activeConnectionId.value = savedActiveConnectionId || (parsedConnections[0] as DockerConnection).id
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
            // 修复 P0-3：legacy 迁移路径同样要把密码搬到 keyring
            if (legacyPassword) {
              try {
                await secretsApi.set(sshPasswordKey('conn_default_ssh'), legacyPassword)
              } catch (e) {
                logWarn(`legacy SSH 密码迁移到 keyring 失败: ${e}`).catch(() => {})
              }
            }
          }
        }

        // 修复 P0-3：connections / registries 加载完成后，密码统一从 keyring 回填到内存
        for (const c of connections.value) {
          if (c.type === 'ssh' && !c.sshPassword) {
            try {
              const pw = await secretsApi.get(sshPasswordKey(c.id))
              if (pw) c.sshPassword = pw
            } catch (e) {
              logWarn(`SSH 密码从 keyring 回填失败（${c.id}）: ${e}`).catch(() => {})
            }
          }
        }

        const savedRegistriesRaw = await store.get<Registry[]>('registries')
        if (savedRegistriesRaw !== null) {
          registries.value = safeParseField(
            settingsFieldSchemas.registries,
            savedRegistriesRaw,
            registries.value,
            'registries'
          ) as Registry[]
        }
        // 修复 P0-3：仓库密码从 keyring 回填
        for (const r of registries.value) {
          if (!r.password) {
            try {
              const pw = await secretsApi.get(registryPasswordKey(r.id))
              if (pw) r.password = pw
            } catch (e) {
              logWarn(`仓库密码从 keyring 回填失败（${r.id}）: ${e}`).catch(() => {})
            }
          }
        }
        currentRegistryId.value = safeParseField(
          settingsFieldSchemas.currentRegistryId,
          await store.get<string>('currentRegistryId'),
          'default',
          'currentRegistryId'
        )
      } else {
        // 若本地没有配置文件，则将当前的默认值保存落盘
        await saveSettings()
      }
    } catch (e) {
      logError(`从 settings.json 加载配置失败: ${e}`).catch(() => {})
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
      logError(`设置自启动失败: ${e}`).catch(() => {})
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
  // 修复 P1-1：lastSavedSnapshot 必须保存「深拷贝」而非引用，
  // 否则下一次保存时 deepEqual 在 a === b 顶层短路，增量同步完全失效。
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

  // 浅克隆 + 数组/对象引用断开：仅在 saveSettings 内使用，避免污染 store 原引用
  const cloneSnapshot = (src: Record<string, unknown>): Record<string, unknown> => {
    const out: Record<string, unknown> = {}
    for (const [k, v] of Object.entries(src)) {
      if (Array.isArray(v)) {
        out[k] = v.map(item => (item && typeof item === 'object' ? { ...item } : item))
      } else if (v && typeof v === 'object') {
        out[k] = { ...(v as Record<string, unknown>) }
      } else {
        out[k] = v
      }
    }
    return out
  }

  const saveSettings = async () => {
    try {
      const store = await getStore()
      // 修复 P0-3：把 connections / registries 中的密码字段移到 OS Keychain，
      // 落盘版本是脱敏副本（密码字段置 undefined），settings.json 不再含明文。
      // 写入 keyring 失败不阻塞 settings 落盘——但会打日志，避免 UI 卡死。
      const sanitizedConnections = await Promise.all(
        connections.value.map(async (c) => {
          if (c.type === 'ssh' && c.sshPassword) {
            try {
              await secretsApi.set(sshPasswordKey(c.id), c.sshPassword)
            } catch (e) {
              logWarn(`SSH 密码写入 keyring 失败（${c.id}）: ${e}`).catch(() => {})
            }
          }
          // 移除密码字段
          const { sshPassword: _omit, ...rest } = c
          return rest
        })
      )
      const sanitizedRegistries = await Promise.all(
        registries.value.map(async (r) => {
          if (r.password) {
            try {
              await secretsApi.set(registryPasswordKey(r.id), r.password)
            } catch (e) {
              logWarn(`仓库密码写入 keyring 失败（${r.id}）: ${e}`).catch(() => {})
            }
          }
          const { password: _omit, ...rest } = r
          return rest
        })
      )

      // 待持久化的键值：不再写 legacy 字段（connectionMode/wslDistro/sshHost/sshPort/sshUser/sshPassword），
      // 旧版本兼容由 loadSettings 一次性迁移路径承担。
      const snapshot: Record<string, unknown> = {
        autoStart: autoStart.value,
        closeToTray: closeToTray.value,
        theme: theme.value,
        refreshInterval: refreshInterval.value,
        visibleMenus: visibleMenus.value,
        connections: sanitizedConnections,
        activeConnectionId: activeConnectionId.value,
        registries: sanitizedRegistries,
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
      // 修复 P1-1：必须存深拷贝，store 中 connections/registries 是被同一引用持有的响应式数组，
      // 直接存 snapshot 会让 lastSavedSnapshot 与下一次快照顶层 ===，deepEqual 失效。
      lastSavedSnapshot.value = cloneSnapshot(snapshot)
      await store.save()
    } catch (e) {
      logError(`保存配置到 settings.json 失败: ${e}`).catch(() => {})
    }
  }

  // 恢复出厂设置，直接清空本地配置文件
  const resetSettings = async () => {
    try {
      const store = await getStore()
      await store.clear()
      await store.save()
    } catch (e) {
      logError(`清除 settings.json 失败: ${e}`).catch(() => {})
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
   * 在 connections[] 中**按 name 精确匹配** id，校正 activeConnectionId。
   *
   * 修复 P0-14 / P0-16：原实现按 (mode + 关键字段) 模糊匹配，
   * 多个同类型 connection（两个 desktop、或两个同主机但密码不同的 SSH）
   * 都会命中第一个，导致用户主动选 A 时 UI 反向刷成 B。
   *
   * fallback：name 缺失或不匹配时退到旧的字段匹配，保证 legacy 兼容。
   * 不修改 connections[] 内容（用户的未保存编辑由 Settings.vue 路径负责）。
   */
  const applyBackendConfig = (cfg: ConnectionConfigPayload): void => {
    if (cfg.name) {
      const byName = connections.value.find(c => c.name === cfg.name)
      if (byName) {
        activeConnectionId.value = byName.id
        return
      }
    }
    const matched = connections.value.find(c => matchesConnectionConfig(c, cfg))
    if (matched) {
      activeConnectionId.value = matched.id
    }
  }

  /**
   * Settings.vue 「保存」按钮的统一入口（仅做内存与持久化，不触发后端连接切换）。
   *
   * 后端 update_connection_config + ping + 各 store 数据刷新统一交由
   * utils/connectionSwitcher.switchTo 外部协调，避免 store 反向 import 其他 store。
   * 调用方在 applyDraft 前后自行决定是否要触发连接切换。
   */
  const applyDraft = async (draft: {
    theme: 'deep-black' | 'zed-gray' | 'light-apple'
    autoStart: boolean
    closeToTray: boolean
    refreshInterval: number
    visibleMenus: string[]
    connections: DockerConnection[]
    activeConnectionId: string
    registries: Registry[]
  }): Promise<void> => {
    // 1. 写入 store 状态
    theme.value = draft.theme
    closeToTray.value = draft.closeToTray
    refreshInterval.value = draft.refreshInterval
    visibleMenus.value = [...draft.visibleMenus]
    connections.value = draft.connections.map(c => ({ ...c }))
    activeConnectionId.value = draft.activeConnectionId
    registries.value = draft.registries.map(r => ({ ...r }))

    // 2. autostart 与持久化
    await setAutoStart(draft.autoStart)
    await saveSettings()
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
    connectionReady,
    connectionSwitching,
    switchingTargetName,
    loadSettings,
    setAutoStart,
    setCloseToTray,
    setTheme,
    saveSettings,
    resetSettings,
    getActiveConnectionConfig,
    applyBackendConfig,
    applyDraft
  }
})

