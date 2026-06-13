/**
 * 连接生命周期协调器：启动 + 切换两个流程的唯一入口。
 *
 * 抽到 utils 而非 settings store，是为了规避「settings store 反向 import 业务 store」
 * 造成的循环依赖；同时让 store 层保持纯数据语义。
 *
 * 启动流程 bootstrap()：
 *   loadSettings → updateConfig(active) → ping → preloadAll → connectionReady=true
 *   全程 connectionReady=false 由 App.vue 全屏 Loading 遮罩，根除启动期 fetch 与
 *   updateConfig 之间的 race（旧版日志里 SendRequest/Connect 错误的根因）。
 *
 * 切换流程 switchTo(connId)：
 *   先 clearAll 立即清空所有列表（避免短暂残留旧连接数据），
 *   再 connectionSwitching=true 弹切换 modal，
 *   updateConfig → ping → preloadAll → connectionSwitching=false。
 */
import { useSettingsStore } from '../store/settings'
import { useComposeStore } from '../store/compose'
import { useContainerStore } from '../store/container'
import { useImageStore } from '../store/image'
import { useNetworkStore } from '../store/network'
import { useVolumeStore } from '../store/volume'
import { connectionApi } from '../api/connectionApi'
import { error as logError, info as logInfo } from '@tauri-apps/plugin-log'

/** 立即清空所有业务 store 列表，避免切换瞬间 UI 残留旧连接数据 */
const clearAllStores = () => {
  useComposeStore().projects = []
  useContainerStore().containers = []
  useImageStore().images = []
  useNetworkStore().networks = []
  useVolumeStore().volumes = []
}

/** 触发所有业务 store 并发预拉数据。失败仅记日志，不阻塞流程 */
const preloadAll = async () => {
  await Promise.allSettled([
    useComposeStore().fetchProjects(),
    useContainerStore().fetchContainers(),
    useImageStore().fetchImages(),
    useNetworkStore().fetchNetworks(),
    useVolumeStore().fetchVolumes()
  ])
}

/**
 * 软件启动时调用。串行执行：加载本地配置 → 下发后端激活连接 → ping 探活 → 预拉数据。
 *
 * 任意一步失败均尽量推进到底（连接未就绪也允许进入主界面，但 connectionReady=true 仅在
 * ping 成功时才置位；ping 失败时由顶部「未连接」横幅提示用户）。
 *
 * @returns ping 是否成功
 */
export const bootstrapConnection = async (): Promise<boolean> => {
  const settingsStore = useSettingsStore()
  let pingOk = false

  try {
    await settingsStore.loadSettings()
  } catch (e) {
    logError(`启动期加载本地配置失败: ${e}`).catch(() => {})
  }

  try {
    const config = settingsStore.getActiveConnectionConfig()
    logInfo(`启动期下发激活连接: ${config.name} (${config.mode})`).catch(() => {})
    await connectionApi.updateConfig(config)
  } catch (e) {
    logError(`启动期同步后端连接配置失败: ${e}`).catch(() => {})
  }

  try {
    await connectionApi.ping()
    pingOk = true
  } catch (e) {
    logError(`启动期 ping 当前连接失败: ${e}`).catch(() => {})
  }

  if (pingOk) {
    await preloadAll()
  }

  // 无论 ping 成功与否，都允许进入主界面：
  // ping 成功 → 数据已预拉，首屏直接可见；
  // ping 失败 → 顶部横幅提示「未连接」，用户可点击「前往设置」。
  settingsStore.connectionReady = true
  return pingOk
}

/**
 * 切换到指定 connectionId 的连接。返回 ping 是否成功。
 *
 * 前置：调用方应已把目标 connection 加入 settingsStore.connections（如有必要先 saveSettings）。
 * 流程：activeConnectionId 切换 → clearAllStores → connectionSwitching=true →
 *      updateConfig → ping → preloadAll → connectionSwitching=false。
 */
export const switchConnection = async (connId: string): Promise<boolean> => {
  const settingsStore = useSettingsStore()
  const target = settingsStore.connections.find(c => c.id === connId)
  if (!target) {
    throw new Error(`未找到连接: ${connId}`)
  }
  settingsStore.activeConnectionId = connId

  // 立即清空各列表：避免「切换后 UI 还显示旧连接的容器/镜像」
  clearAllStores()

  settingsStore.switchingTargetName = target.name
  settingsStore.connectionSwitching = true

  try {
    const config = settingsStore.getActiveConnectionConfig()
    logInfo(`切换连接: ${config.name} (${config.mode})`).catch(() => {})
    await connectionApi.updateConfig(config)
    await connectionApi.ping()
    await preloadAll()
    return true
  } catch (e) {
    logError(`切换连接失败 (${target.name}): ${e}`).catch(() => {})
    throw e
  } finally {
    settingsStore.connectionSwitching = false
    settingsStore.switchingTargetName = ''
  }
}

/** 仅触发预加载，由心跳路径在「断线 → 重连」过渡时按需调用 */
export const refreshAllStores = preloadAll
