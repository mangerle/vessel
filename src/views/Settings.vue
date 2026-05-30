<script setup lang="ts">
import { ref, shallowRef, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useSettingsStore } from '../store/settings'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import {
  NSelect,
  NSwitch,
  NInput,
  NInputNumber,
  NCheckbox,
  NCheckboxGroup,
  NIcon,
  NModal,
  NButton,
  NSpace,
  useMessage
} from 'naive-ui'
import {
  DesktopOutline,
  LogoDocker,
  GlobeOutline,
  ShieldCheckmarkOutline,
  SaveOutline,
  FlashOutline,
  TrashOutline,
  AddOutline,
  InformationCircleOutline,
  SparklesOutline,
  SyncOutline
} from '@vicons/ionicons5'

const settingsStore = useSettingsStore()
const message = useMessage()
const route = useRoute()

const activeTab = ref<string>('general')

// 页面分类小页签
const tabs = [
  { label: '基础常规', value: 'general', icon: DesktopOutline },
  { label: 'Docker 引擎', value: 'docker', icon: LogoDocker },
  { label: '镜像仓库', value: 'registries', icon: GlobeOutline },
  { label: '账户凭证', value: 'credentials', icon: ShieldCheckmarkOutline },
  { label: '数据备份', value: 'backup', icon: SaveOutline },
  { label: '关于 Vessel', value: 'about', icon: InformationCircleOutline }
]

// 菜单显示项
const menuOptions = [
  { label: '项目 (Compose)', value: 'compose' },
  { label: '容器 (Containers)', value: 'containers' },
  { label: '镜像 (Images)', value: 'images' },
  { label: '网络 (Networks)', value: 'networks' },
  { label: '数据卷 (Volumes)', value: 'volumes' }
]

// 主题备选项
const themeOptions = [
  { label: '深邃暗黑 (Deep Black)', value: 'deep-black' },
  { label: 'Zed 同款冷灰 (Zed Gray)', value: 'zed-gray' },
  { label: '苹果亮白 (Light Apple)', value: 'light-apple' }
]

// 本地 WSL 分发版备选项
const wslOptions = ref<{ label: string; value: string }[]>([])

// 新增镜像仓库弹窗控制与临时表单
const showAddModal = ref(false)
const newRegistry = ref({
  name: '',
  url: '',
  username: '',
  password: ''
})

const openAddModal = () => {
  newRegistry.value = {
    name: '',
    url: '',
    username: '',
    password: ''
  }
  showAddModal.value = true
}

const handleAddRegistry = () => {
  if (!newRegistry.value.name.trim()) {
    message.warning('请输入仓库名称')
    return
  }
  
  // 生成随机 ID
  const id = 'reg_' + Math.random().toString(36).substring(2, 11)
  draft.value.registries.push({
    id,
    name: newRegistry.value.name.trim(),
    url: newRegistry.value.url.trim(),
    username: newRegistry.value.username.trim(),
    password: newRegistry.value.password.trim(),
    isDefault: false
  })
  
  showAddModal.value = false
  message.success('已添加到草稿列表，请点击保存配置使其落盘')
}

const handleDeleteRegistry = (id: string) => {
  const idx = draft.value.registries.findIndex(r => r.id === id)
  if (idx !== -1) {
    if (draft.value.registries[idx].isDefault) {
      message.error('默认环境不能删除')
      return
    }
    draft.value.registries.splice(idx, 1)
    message.info('已从列表中移除，请点击保存配置使其落盘')
  }
}

// 草稿配置 (Draft Settings) 实现即改即生效
const draft = ref({
  theme: 'deep-black',
  autoStart: false,
  closeToTray: true,
  refreshInterval: 3,
  visibleMenus: ['compose', 'containers', 'images', 'networks', 'volumes'],
  connections: [] as any[],
  activeConnectionId: '',
  registries: [] as any[]
})

// 初始化草稿
const syncDraftFromStore = () => {
  draft.value = {
    theme: settingsStore.theme,
    autoStart: settingsStore.autoStart,
    closeToTray: settingsStore.closeToTray,
    refreshInterval: settingsStore.refreshInterval,
    visibleMenus: [...settingsStore.visibleMenus],
    connections: settingsStore.connections.map(c => ({ ...c })),
    activeConnectionId: settingsStore.activeConnectionId,
    registries: settingsStore.registries.map(r => ({ ...r }))
  }
}

// 规整连接列表，过滤 undefined 属性以确保 JSON 序列化对比一致
const cleanConnections = (conns: any[]) => {
  return conns.map(c => {
    const clean: any = { id: c.id, name: c.name, type: c.type }
    if (c.wslDistro !== undefined && c.wslDistro !== null) clean.wslDistro = c.wslDistro
    if (c.sshHost !== undefined && c.sshHost !== null) clean.sshHost = c.sshHost
    if (c.sshPort !== undefined && c.sshPort !== null) clean.sshPort = c.sshPort
    if (c.sshUser !== undefined && c.sshUser !== null) clean.sshUser = c.sshUser
    if (c.sshPassword !== undefined && c.sshPassword !== null) clean.sshPassword = c.sshPassword
    return clean
  })
}

// 探测配置是否被篡改过 (Dirty 检测)
const isDirty = computed(() => {
  return (
    draft.value.theme !== settingsStore.theme ||
    draft.value.autoStart !== settingsStore.autoStart ||
    draft.value.closeToTray !== settingsStore.closeToTray ||
    draft.value.refreshInterval !== settingsStore.refreshInterval ||
    JSON.stringify(draft.value.visibleMenus) !== JSON.stringify(settingsStore.visibleMenus) ||
    JSON.stringify(cleanConnections(draft.value.connections)) !== JSON.stringify(cleanConnections(settingsStore.connections)) ||
    draft.value.activeConnectionId !== settingsStore.activeConnectionId ||
    JSON.stringify(draft.value.registries) !== JSON.stringify(settingsStore.registries)
  )
})

const handleSave = async () => {
  // 1. 同步给后端 Rust 环境 (核心：连接引擎切换)
  const activeConn = draft.value.connections.find(c => c.id === draft.value.activeConnectionId)
  if (activeConn) {
    try {
      await invoke('update_connection_config', { 
        mode: activeConn.type, 
        distro: activeConn.wslDistro || null 
      })
    } catch (e) {
      console.error('后端配置同步失败:', e)
    }
  }

  // 2. 保存到 store 状态中并自动持久化
  settingsStore.theme = draft.value.theme as any
  settingsStore.closeToTray = draft.value.closeToTray
  settingsStore.refreshInterval = draft.value.refreshInterval
  settingsStore.visibleMenus = [...draft.value.visibleMenus]
  settingsStore.connections = draft.value.connections.map(c => ({ ...c }))
  settingsStore.activeConnectionId = draft.value.activeConnectionId
  settingsStore.registries = draft.value.registries.map(r => ({ ...r }))
  
  await settingsStore.setAutoStart(draft.value.autoStart) // 会触发自启动插件并保存
  await settingsStore.saveSettings()
  
  // 3. 重新同步草稿，使 isDirty 变为 false，按钮自动退去高亮
  syncDraftFromStore()
  
  message.success('配置已成功落盘，系统通信管道已重载！')
}

const handleCancel = () => {
  syncDraftFromStore()
  message.info('已丢弃所有内存修改')
}

// 刷新 WSL 列表（冷启动探测）
const handleRefreshWsl = async (silent = false) => {
  try {
    if (!silent) message.warning('正在扫描宿主机可用 Linux 发行版...')
    const list = await invoke<string[]>('list_wsl_distros')
    if (list && list.length > 0) {
      wslOptions.value = list.map(distro => ({
        label: distro,
        value: distro
      }))
      
      // 更新当前选中的 WSL 连接的默认分发版
      const activeConn = draft.value.connections.find(c => c.id === draft.value.activeConnectionId)
      if (activeConn && activeConn.type === 'wsl') {
        if (!activeConn.wslDistro || !list.includes(activeConn.wslDistro)) {
          activeConn.wslDistro = list[0]
        }
      }
      if (!silent) message.success('扫描完毕！已重新装载 WSL 分发版列表。')
    } else {
      wslOptions.value = []
      if (!silent) message.info('未探测到已安装的 WSL 发行版。')
    }
  } catch (err) {
    console.error('扫描分发版失败:', err)
    if (!silent) message.error('扫描分发版失败')
  }
}

// 新增连接弹窗状态与临时表单
const showAddConnModal = ref(false)
const newConnection = ref({
  name: '',
  type: 'desktop' as 'wsl' | 'ssh' | 'desktop',
  wslDistro: '',
  sshHost: '192.168.1.105',
  sshPort: 22,
  sshUser: 'root',
  sshPassword: ''
})

const openAddConnModal = () => {
  newConnection.value = {
    name: '',
    type: 'desktop',
    wslDistro: wslOptions.value[0]?.value || 'Ubuntu',
    sshHost: '192.168.1.105',
    sshPort: 22,
    sshUser: 'root',
    sshPassword: ''
  }
  showAddConnModal.value = true
}

const handleAddConnection = () => {
  if (!newConnection.value.name.trim()) {
    message.warning('请输入连接名称')
    return
  }
  if (newConnection.value.type === 'ssh') {
    if (!newConnection.value.sshHost.trim()) {
      message.warning('请输入 SSH 主机地址')
      return
    }
    if (!newConnection.value.sshUser.trim()) {
      message.warning('请输入 SSH 用户名')
      return
    }
  }
  
  const id = 'conn_' + Math.random().toString(36).substring(2, 11)
  draft.value.connections.push({
    id,
    name: newConnection.value.name.trim(),
    type: newConnection.value.type,
    wslDistro: newConnection.value.type === 'wsl' ? newConnection.value.wslDistro : undefined,
    sshHost: newConnection.value.type === 'ssh' ? newConnection.value.sshHost.trim() : undefined,
    sshPort: newConnection.value.type === 'ssh' ? newConnection.value.sshPort : undefined,
    sshUser: newConnection.value.type === 'ssh' ? newConnection.value.sshUser.trim() : undefined,
    sshPassword: newConnection.value.type === 'ssh' ? newConnection.value.sshPassword.trim() : undefined
  })

  // 默认激活新增的连接
  draft.value.activeConnectionId = id
  
  showAddConnModal.value = false
  message.success('已添加到草稿列表，请点击保存配置使其落盘')
}

const handleDeleteConnection = (id: string) => {
  const idx = draft.value.connections.findIndex(c => c.id === id)
  if (idx !== -1) {
    if (draft.value.connections[idx].id === draft.value.activeConnectionId) {
      message.error('当前活动中的连接引擎不能删除，请先切换到其他引擎')
      return
    }
    draft.value.connections.splice(idx, 1)
    message.info('已从列表中移除，请点击保存配置使其落盘')
  }
}

const handleSelectConnection = (id: string) => {
  draft.value.activeConnectionId = id
}

// 打开本地配置文件目录
const handleOpenConfigDir = async () => {
  try {
    await invoke('open_config_dir')
    message.success('已打开本地配置文件所在目录')
  } catch (e: any) {
    message.error('打开目录失败: ' + e)
  }
}

// 恢复出厂设置
const handleResetFactory = async () => {
  localStorage.clear()
  try {
    await settingsStore.resetSettings()
    message.error('本地物理配置已擦除！恢复出厂设置中...')
  } catch (err) {
    message.error('清除物理配置文件失败，仅清理本地缓存。')
  }
  setTimeout(() => {
    window.location.reload()
  }, 1000)
}

// 关于及检查更新相关状态
const appVersion = ref('v0.1.0')
const checkingUpdate = ref(false)
const showUpdateModal = ref(false)
const updateProgress = ref(0)
const updateStep = ref<'idle' | 'found' | 'downloading' | 'ready'>('idle')
const updateError = ref('')

// 当前获取到的更新对象
const activeUpdate = shallowRef<any>(null)
const updateInfo = ref({
  version: '',
  date: '',
  body: ''
})

// 将 Markdown 文本格式化清洗为干净的纯文本
const cleanMarkdown = (text: string) => {
  if (!text) return ''
  return text
    .replace(/^(?:##|###)\s+(.+)$/gm, '$1:')
    .replace(/^\*\s+/gm, '- ')
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    // 抹去带外包圆括号的 Commit 链接，如 " ([80fec30](https://...))" -> ""
    .replace(/\s*\(\[([^\]]+)\]\([^)]+\)\)/g, '')
    // 兜底：若有普通 Markdown 链接则只保留文字
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/\n{3,}/g, '\n\n')
}

// 真正检查更新
const handleCheckUpdate = async () => {
  if (checkingUpdate.value) return
  checkingUpdate.value = true
  updateError.value = ''
  
  try {
    const updateResult = await check()
    if (updateResult) {
      activeUpdate.value = updateResult
      updateInfo.value = {
        version: updateResult.version,
        date: updateResult.date || '',
        body: updateResult.body || '包含多引擎管理与稳定性修复'
      }
      updateStep.value = 'found'
      showUpdateModal.value = true
    } else {
      message.info('当前已是最新版本')
    }
  } catch (e: any) {
    console.error('检查更新失败:', e)
    message.error('检查更新失败: ' + e)
    updateError.value = String(e)
  } finally {
    checkingUpdate.value = false
  }
}

// 真正执行下载与安装
const handleStartDownload = async () => {
  if (!activeUpdate.value || updateStep.value === 'downloading') return
  updateStep.value = 'downloading'
  updateProgress.value = 0
  
  try {
    let downloadedBytes = 0
    await activeUpdate.value.downloadAndInstall((event: any) => {
      switch (event.event) {
        case 'Started':
          console.log('开始下载更新...')
          break
        case 'Progress':
          if (event.data && event.data.chunkLength) {
            downloadedBytes += event.data.chunkLength
            // 如果含有完整的长度，则算百分比
            if (event.data.contentLength) {
              updateProgress.value = Math.round((downloadedBytes / event.data.contentLength) * 100)
            } else {
              if (updateProgress.value < 95) {
                updateProgress.value += 5
              }
            }
          }
          break
        case 'Finished':
          console.log('下载完成并成功安装更新')
          updateProgress.value = 100
          updateStep.value = 'ready'
          break
      }
    })
  } catch (e: any) {
    console.error('下载安装更新失败:', e)
    message.error('更新下载失败: ' + e)
    updateStep.value = 'found'
  }
}

// 立即重启应用
const handleRelaunch = async () => {
  try {
    await relaunch()
  } catch (e) {
    console.error('重启应用失败:', e)
    message.error('重启应用失败，请手动重启')
  }
}

onMounted(async () => {
  await settingsStore.loadSettings()
  syncDraftFromStore()
  await handleRefreshWsl(true) // 冷启动静默探测
  
  // 动态拉取当前真实版本号
  try {
    appVersion.value = 'v' + await getVersion()
  } catch (e) {
    console.error('获取应用版本号失败:', e)
  }

  // 检查是否从启动更新提醒跳转而来
  if (route.query.triggerUpdate === 'true') {
    handleCheckUpdate()
    // 切换到关于页签以展示更新信息
    activeTab.value = 'about'
  }
})
</script>

<template>
  <div class="settings-view">
    <div class="settings-layout-box">
      <!-- 左侧分类小页签 (宽度 160px) -->
      <div class="settings-sidebar-tabs">
        <div 
          v-for="t in tabs" 
          :key="t.value" 
          class="settings-tab-node"
          :class="{ active: activeTab === t.value }"
          @click="activeTab = t.value"
        >
          <n-icon :component="t.icon" size="14" />
          <span>{{ t.label }}</span>
        </div>
      </div>

      <!-- 右侧网格配置表单 -->
      <div class="settings-form-content">
        <n-scrollbar style="height: 100%; padding-right: 12px">
          <!-- 💻 基础常规 (General) -->
          <div v-show="activeTab === 'general'" class="form-section">
            <div class="section-title">基础常规常规配置</div>
            
            <div class="form-row">
              <div class="row-label-area">
                <div class="row-title">全局界面主题</div>
                <div class="row-desc">选择系统的视觉主题配色，支持一帧内秒级无缝变色。</div>
              </div>
              <div class="row-value-area">
                <n-select v-model:value="draft.theme" :options="themeOptions" class="select-field" size="small" />
              </div>
            </div>

            <div class="form-row">
              <div class="row-label-area">
                <div class="row-title">自启动行为</div>
                <div class="row-desc">系统开机并完成用户登录时，自动在后台侧载拉起 Vessel。</div>
              </div>
              <div class="row-value-area">
                <n-switch v-model:value="draft.autoStart" />
              </div>
            </div>

            <div class="form-row">
              <div class="row-label-area">
                <div class="row-title">窗口关闭行为</div>
                <div class="row-desc">点击窗口右上角关闭按钮时，隐藏到系统托盘而不退出程序。</div>
              </div>
              <div class="row-value-area">
                <n-switch v-model:value="draft.closeToTray" />
              </div>
            </div>

            <div class="form-row">
              <div class="row-label-area">
                <div class="row-title">侧边栏功能菜单</div>
                <div class="row-desc">选择显示在侧边栏的功能菜单项，未选中的项将被隐藏。</div>
              </div>
              <div class="row-value-area wide">
                <n-checkbox-group v-model:value="draft.visibleMenus">
                  <div class="checkbox-grid">
                    <n-checkbox v-for="opt in menuOptions" :key="opt.value" :value="opt.value" :label="opt.label" />
                  </div>
                </n-checkbox-group>
              </div>
            </div>
          </div>

          <!-- 🐳 Docker 引擎 (Docker Engine) -->
          <div v-show="activeTab === 'docker'" class="form-section">
            <div class="section-title flex-between">
              <span>Docker 引擎连接设置</span>
              <button class="form-action-btn border-btn flex-center-btn" @click="openAddConnModal">
                <n-icon :component="AddOutline" style="margin-right: 4px;" />
                新增连接引擎
              </button>
            </div>

            <!-- 连接引擎列表 -->
            <div class="registries-table-box">
              <table class="geek-settings-table">
                <thead>
                  <tr>
                    <th>连接名称</th>
                    <th>连接类型</th>
                    <th>配置信息</th>
                    <th style="width: 140px; text-align: center;">操作</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="conn in draft.connections" :key="conn.id" :class="{ 'active-row': draft.activeConnectionId === conn.id }">
                    <td>
                      <span class="active-indicator-dot" v-if="draft.activeConnectionId === conn.id"></span>
                      <strong>{{ conn.name }}</strong>
                    </td>
                    <td>
                      <span class="type-tag" :class="conn.type">{{ conn.type.toUpperCase() }}</span>
                    </td>
                    <td class="monospace">
                      <span v-if="conn.type === 'wsl'">分发版: {{ conn.wslDistro || '-' }}</span>
                      <span v-else-if="conn.type === 'ssh'">{{ conn.sshUser }}@{{ conn.sshHost }}:{{ conn.sshPort }}</span>
                      <span v-else>默认本地命名管道</span>
                    </td>
                    <td style="text-align: center;">
                      <n-space justify="center" :size="4">
                        <button 
                          v-if="draft.activeConnectionId !== conn.id"
                          class="action-activate-btn flex-center-btn"
                          @click="handleSelectConnection(conn.id)"
                        >
                          <n-icon :component="FlashOutline" style="margin-right: 2px;" />
                          激活
                        </button>
                        <button 
                          v-if="draft.activeConnectionId !== conn.id"
                          class="action-delete-btn flex-center-btn" 
                          @click="handleDeleteConnection(conn.id)"
                        >
                          <n-icon :component="TrashOutline" style="margin-right: 2px;" />
                          删除
                        </button>
                        <span v-else class="active-tag">活动中</span>
                      </n-space>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>

            <!-- 数据自动刷新周期（全局） -->
            <div class="form-row">
              <div class="row-label-area">
                <div class="row-title">数据自动刷新周期</div>
                <div class="row-desc">控制容器状态轮询及 CPU/内存等性能流心跳刷新步长。</div>
              </div>
              <div class="row-value-area flex-row align-center">
                <n-input-number v-model:value="draft.refreshInterval" size="small" :min="1" :max="60" style="width: 100px" />
                <span class="input-unit">秒</span>
              </div>
            </div>
          </div>

          <!-- 🌐 镜像仓库 (Registries) -->
          <div v-show="activeTab === 'registries'" class="form-section">
            <div class="section-title flex-between">
              <span>镜像仓库加速器与私有 Harbor 列表</span>
              <button class="form-action-btn border-btn flex-center-btn" @click="openAddModal">
                <n-icon :component="AddOutline" style="margin-right: 4px;" />
                新增私有仓/加速器
              </button>
            </div>
            <div class="registries-table-box">
              <table class="geek-settings-table">
                <thead>
                  <tr>
                    <th>仓库名称</th>
                    <th>Harbor URL</th>
                    <th>用户名</th>
                    <th>密码 (明文)</th>
                    <th style="width: 80px; text-align: center;">操作</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="reg in draft.registries" :key="reg.id">
                    <td><strong>{{ reg.name }}</strong></td>
                    <td class="monospace">{{ reg.url || '-' }}</td>
                    <td>{{ reg.username || '-' }}</td>
                    <td class="monospace masked">{{ reg.password ? '••••••••' : '-' }}</td>
                    <td style="text-align: center;">
                      <button 
                        v-if="!reg.isDefault" 
                        class="action-delete-btn flex-center-btn" 
                        @click="handleDeleteRegistry(reg.id)"
                        title="删除该镜像仓库"
                      >
                        <n-icon :component="TrashOutline" style="margin-right: 2px;" />
                        删除
                      </button>
                      <span v-else class="system-tag">系统默认</span>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- 🛡️ 账户凭证 (Credentials) -->
          <div v-show="activeTab === 'credentials'" class="form-section">
            <div class="section-title">机密存储安全凭证管理</div>
            <div class="credentials-card">
              <p>为了极客在开发调试过程中的透明配置性，软件现阶段使用本地 TOML 纯文本明文存储 SSH 与 Harbor 密钥。</p>
              <p>系统已为下一阶段接入 Win32 硬件特征隐形锁（AES-256-GCM）一机一密加密提供了平滑过渡策略。</p>
              
              <button class="danger-border-btn" @click="syncDraftFromStore">
                🛡️ 擦除并重置全部机密凭证
              </button>
            </div>
          </div>

          <!-- 💾 数据备份 (Backup & Reset) -->
          <div v-show="activeTab === 'backup'" class="form-section">
            <div class="section-title">数据备份与出厂重置</div>
            
            <div class="form-row">
              <div class="row-label-area">
                <div class="row-title">本地纯文本配置文件</div>
                <div class="row-desc">在资源管理器中直接定位并修改 config.toml 与 registries.toml。</div>
              </div>
              <div class="row-value-area">
                <button class="form-action-btn border-btn" @click="handleOpenConfigDir">
                  📂 打开配置文件目录
                </button>
              </div>
            </div>

            <div class="form-row">
              <div class="row-label-area">
                <div class="row-title">恢复出厂设置</div>
                <div class="row-desc">一键彻底擦除本地所有存储状态、两栖 SSH 节点，重置应用并重启。</div>
              </div>
              <div class="row-value-area">
                <button class="danger-solid-btn" @click="handleResetFactory">
                  💥 恢复出厂设置
                </button>
              </div>
            </div>
          </div>

          <!-- ℹ️ 关于 Vessel (About) -->
          <div v-show="activeTab === 'about'" class="form-section about-section">
            <div class="about-logo-wrapper">
              <div class="about-logo-icon-bg">
                <img src="/logo.png" alt="Vessel Logo" class="about-logo-img" style="width: 52px; height: 52px; object-fit: contain;" />
              </div>
              <div class="about-app-name">Vessel</div>
              <div class="about-app-version">{{ appVersion }}</div>
            </div>

            <div class="about-description-card">
              <p>Vessel 是一款面向现代化打造的<strong>轻量级 Docker 桌面容器管理客户端</strong>。</p>
              <p>系统支持本地 WSL 管道侧载直连、远程 SSH 密码多节点调度，全面升级了引擎多连接切换与全通道加签安全升级技术。</p>
            </div>

            <div class="about-action-row">
              <button 
                class="form-action-btn border-btn flex-center-btn update-check-btn" 
                :disabled="checkingUpdate"
                @click="handleCheckUpdate"
              >
                <n-icon :component="checkingUpdate ? SyncOutline : SparklesOutline" :class="{ 'rotating-icon': checkingUpdate }" style="margin-right: 6px;" />
                {{ checkingUpdate ? '正在检查更新...' : '检查更新' }}
              </button>
            </div>

            <div class="about-copyright">
              Copyright &copy; 2026 Vessel Dev Team. All Rights Reserved.
            </div>
          </div>
        </n-scrollbar>
      </div>
    </div>

    <!-- 底部即改即生效控制栏 (绝对布局贴底) -->
    <div class="settings-bottom-actions">
      <button 
        class="cancel-btn" 
        :disabled="!isDirty" 
        @click="handleCancel"
      >
        取消
      </button>
      <button 
        class="save-btn" 
        :class="{ 'breath-green': isDirty }"
        :disabled="!isDirty" 
        @click="handleSave"
      >
        保存
      </button>
    </div>
  </div>

  <!-- 新增仓库弹窗 -->
  <n-modal
    v-model:show="showAddModal"
    preset="card"
    title="➕ 新增镜像仓库"
    style="width: 450px"
  >
    <div class="add-registry-form">
      <div class="form-field-item">
        <span class="field-label">仓库名称 *</span>
        <n-input v-model:value="newRegistry.name" placeholder="例如: 腾讯云镜像源" size="small" />
      </div>
      <div class="form-field-item">
        <span class="field-label">仓库 URL * (不带协议前缀，如: harbor.sk-tech.com)</span>
        <n-input v-model:value="newRegistry.url" placeholder="例如: harbor.sk-tech.com" size="small" />
      </div>
      <div class="form-field-item">
        <span class="field-label">用户名 (可选，私有仓鉴权使用)</span>
        <n-input v-model:value="newRegistry.username" placeholder="请输入用户名" size="small" />
      </div>
      <div class="form-field-item">
        <span class="field-label">密码 / Token (可选)</span>
        <n-input v-model:value="newRegistry.password" type="password" placeholder="请输入密码" size="small" />
      </div>
      
      <div class="form-modal-actions">
        <n-button type="primary" size="small" @click="handleAddRegistry">确定添加</n-button>
        <n-button size="small" @click="showAddModal = false">取消</n-button>
      </div>
    </div>
  </n-modal>

  <!-- 新增连接引擎弹窗 -->
  <n-modal
    v-model:show="showAddConnModal"
    preset="card"
    title="新增连接引擎"
    style="width: 450px"
  >
    <div class="add-registry-form">
      <div class="form-field-item">
        <span class="field-label">连接名称 *</span>
        <n-input v-model:value="newConnection.name" placeholder="例如: 本地 WSL Ubuntu" size="small" />
      </div>
      <div class="form-field-item">
        <span class="field-label">连接类型 *</span>
        <n-select 
          v-model:value="newConnection.type" 
          :options="[
            { label: '本地 WSL 管道侧载', value: 'wsl' },
            { label: '远程 SSH 密码连接', value: 'ssh' },
            { label: 'Docker Desktop (本地默认)', value: 'desktop' }
          ]" 
          size="small" 
        />
      </div>
      
      <!-- 如果选了 WSL 类型，直接在新增时提供默认发行版选择 -->
      <div v-if="newConnection.type === 'wsl'" class="form-field-item">
        <span class="field-label">默认 WSL 发行版</span>
        <n-select v-model:value="newConnection.wslDistro" :options="wslOptions" size="small" />
      </div>

      <!-- 如果选了 SSH 类型，提供主机地址、端口、用户名和密码输入 -->
      <div v-if="newConnection.type === 'ssh'" class="grid-form-fields" style="margin-top: 4px;">
        <div class="field-item">
          <span class="field-label">主机地址 (Host) *</span>
          <n-input v-model:value="newConnection.sshHost" size="small" placeholder="例如: 192.168.1.105" />
        </div>
        <div class="field-item">
          <span class="field-label">端口 (Port) *</span>
          <n-input-number v-model:value="newConnection.sshPort" size="small" :min="1" :max="65535" />
        </div>
        <div class="field-item">
          <span class="field-label">用户名 (Username) *</span>
          <n-input v-model:value="newConnection.sshUser" size="small" placeholder="root" />
        </div>
        <div class="field-item">
          <span class="field-label">登录密码 (Password)</span>
          <n-input v-model:value="newConnection.sshPassword" type="password" size="small" placeholder="密码" />
        </div>
      </div>

      <div class="form-modal-actions">
        <n-button type="primary" size="small" @click="handleAddConnection">确定添加</n-button>
        <n-button size="small" @click="showAddConnModal = false">取消</n-button>
      </div>
    </div>
  </n-modal>

  <!-- 软件更新弹窗 -->
  <n-modal
    v-model:show="showUpdateModal"
    preset="card"
    title="🚀 发现新版本更新"
    style="width: 450px"
    :closable="updateStep !== 'downloading'"
    :mask-closable="updateStep !== 'downloading'"
  >
    <div class="add-registry-form">
      <div class="update-info-header">
        <div class="update-version-label">新版本: <strong>{{ updateInfo.version }}</strong></div>
        <div class="update-date-label" v-if="updateInfo.date">发布时间: {{ updateInfo.date }}</div>
      </div>
      
      <div class="update-logs-box">
        <div class="update-logs-title">更新日志:</div>
        <div class="update-logs-content" style="white-space: pre-wrap; font-size: 11px; line-height: 1.5; color: var(--text-body);">{{ cleanMarkdown(updateInfo.body) }}</div>
      </div>

      <!-- 进度条（当正在下载或准备就绪时展示） -->
      <div v-if="updateStep === 'downloading' || updateStep === 'ready'" class="update-progress-area">
        <div class="progress-bar-container">
          <div class="progress-bar-fill" :style="{ width: updateProgress + '%' }"></div>
        </div>
        <div class="progress-text-row">
          <span class="progress-status-text">
            {{ updateStep === 'ready' ? '🎉 安装成功！等待重启' : '正在下载更新包...' }}
          </span>
          <span class="progress-percent-text">{{ updateProgress }}%</span>
        </div>
      </div>

      <div class="form-modal-actions">
        <n-button 
          v-if="updateStep === 'found'" 
          type="primary" 
          size="small" 
          @click="handleStartDownload"
        >
          立即下载并升级
        </n-button>
        <n-button 
          v-else-if="updateStep === 'downloading'" 
          type="primary" 
          size="small" 
          disabled
        >
          正在下载中...
        </n-button>
        <n-button 
          v-else-if="updateStep === 'ready'" 
          type="success" 
          size="small" 
          @click="handleRelaunch"
        >
          立即重启应用
        </n-button>

        <n-button 
          v-if="updateStep === 'found'" 
          size="small" 
          @click="showUpdateModal = false"
        >
          以后再说
        </n-button>
      </div>
    </div>
  </n-modal>
</template>

<style scoped>
.settings-view {
  width: 100%;
  height: 100%;
  position: relative;
  display: flex;
  flex-direction: column;
  user-select: none;
}

.settings-layout-box {
  display: flex;
  flex: 1;
  min-height: 0;
  gap: 16px;
  padding-bottom: 76px; /* 增加内边距防止浮动底部栏遮挡内容 */
}

/* 左侧页签 160px */
.settings-sidebar-tabs {
  width: 160px;
  background-color: var(--bg-sidebar);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
}

.settings-tab-node {
  height: 32px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-body);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}
.settings-tab-node:hover {
  background-color: rgba(255,255,255,0.02);
  color: var(--text-title);
}
.settings-tab-node.active {
  background-color: var(--bg-active);
  color: var(--text-title);
  font-weight: 700;
}

/* 右侧配置表单 */
.settings-form-content {
  flex: 1;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-main);
  padding: 20px;
  overflow: hidden;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-title {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-title);
  border-bottom: 1px solid var(--border-color);
  padding-bottom: 8px;
  text-transform: uppercase;
}

.form-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border-color);
}
.form-row.borderless {
  border-bottom: none;
  padding-bottom: 0;
}

.row-label-area {
  flex: 1;
  padding-right: 20px;
}

.row-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-title);
  margin-bottom: 3px;
}

.row-desc {
  font-size: 10px;
  color: var(--text-muted);
  line-height: 1.4;
}

.row-value-area {
  width: 220px;
  display: flex;
  justify-content: flex-end;
}

.row-value-area.flex-row {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.row-value-area.align-center {
  align-items: center;
}

.row-value-area.wide {
  width: 320px;
}

.checkbox-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  width: 100%;
}

.select-field {
  width: 100%;
}

.input-unit {
  font-size: 10px;
  color: var(--text-muted);
  margin-left: 6px;
}

/* WSL/SSH 嵌套卡片 */
.nested-mode-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 12px;
  margin-bottom: 6px;
}

.nested-title {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  margin-bottom: 10px;
}

.select-wsl {
  width: 130px;
}

.form-action-btn {
  height: 24px;
  padding: 0 10px;
  background-color: rgba(255,255,255,0.04);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-body);
  font-size: 10px;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.form-action-btn:hover {
  background-color: var(--bg-active);
  color: var(--text-title);
}

.form-action-btn.border-btn {
  border-color: var(--brand-primary);
  color: var(--brand-primary);
}
.form-action-btn.border-btn:hover {
  background-color: rgba(16, 185, 129, 0.1);
}

.grid-form-fields {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.field-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field-label {
  font-size: 9px;
  font-weight: 700;
  color: var(--text-muted);
}

/* Registries */
.registries-table-box {
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
}

.geek-settings-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 11px;
}
.geek-settings-table th,
.geek-settings-table td {
  padding: 8px 12px;
  text-align: left;
}
.geek-settings-table th {
  background-color: rgba(255,255,255,0.02);
  color: var(--text-title);
  border-bottom: 1px solid var(--border-color);
}
.geek-settings-table td {
  border-bottom: 1px solid var(--border-color);
  color: var(--text-body);
}
.geek-settings-table tr:last-child td {
  border-bottom: none;
}

.monospace {
  font-family: monospace;
}
.monospace.masked {
  letter-spacing: 2px;
  color: var(--text-muted);
}

/* Credentials */
.credentials-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 16px;
  font-size: 11px;
  line-height: 1.6;
  color: var(--text-body);
}

.danger-border-btn {
  margin-top: 12px;
  height: 28px;
  background: transparent;
  border: 1px solid var(--brand-danger);
  color: var(--brand-danger);
  border-radius: 4px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.danger-border-btn:hover {
  background-color: rgba(239, 68, 68, 0.1);
}

.danger-solid-btn {
  height: 28px;
  background-color: var(--brand-danger);
  border: none;
  color: #000;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.danger-solid-btn:hover {
  filter: brightness(1.1);
}

/* 底部即改即生效控制栏 (呼吸绿) */
.settings-bottom-actions {
  position: absolute;
  bottom: 12px;
  left: 12px;
  right: 12px;
  height: 52px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 0 20px;
  gap: 12px;
  z-index: 10;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
}

.save-btn {
  height: 30px;
  padding: 0 16px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background-color: rgba(0, 0, 0, 0.03);
  color: var(--text-body);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  outline: none;
  transition: all 0.2s ease;
}

.save-btn:hover:not(:disabled) {
  background-color: rgba(0, 0, 0, 0.06);
  border-color: var(--text-muted);
}

.save-btn:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}

.save-btn.breath-green {
  background-color: var(--brand-primary);
  border-color: var(--brand-primary);
  color: #fff;
  font-weight: 700;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.2);
  animation: green-breath-anim 2.5s infinite ease-in-out;
}

@keyframes green-breath-anim {
  0% { box-shadow: 0 0 4px rgba(16, 185, 129, 0.2); }
  50% { box-shadow: 0 0 14px rgba(16, 185, 129, 0.5); }
  100% { box-shadow: 0 0 4px rgba(16, 185, 129, 0.2); }
}

.cancel-btn {
  height: 30px;
  padding: 0 16px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background-color: transparent;
  color: var(--text-body);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  outline: none;
  transition: all 0.2s ease;
}

.cancel-btn:hover:not(:disabled) {
  background-color: rgba(0, 0, 0, 0.03);
  border-color: var(--text-muted);
}

.cancel-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.fade-in-enter-active,
.fade-in-leave-active {
  transition: opacity 0.12s ease;
}
.fade-in-enter-from,
.fade-in-leave-to {
  opacity: 0;
}

.flex-between {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.action-delete-btn {
  background: transparent;
  border: 1px solid var(--brand-danger);
  color: var(--brand-danger);
  padding: 2px 6px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 10px;
  transition: all 0.15s ease;
  outline: none;
}
.action-delete-btn:hover {
  background-color: rgba(239, 68, 68, 0.15);
}
.system-tag {
  font-size: 10px;
  color: var(--text-muted);
  background-color: rgba(255, 255, 255, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
}
.add-registry-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.form-field-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.form-modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 12px;
}

.active-row {
  background-color: rgba(16, 185, 129, 0.02);
}
.active-indicator-dot {
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: var(--brand-primary);
  margin-right: 6px;
  box-shadow: 0 0 8px var(--brand-primary);
}
.type-tag {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
}
.type-tag.wsl {
  background-color: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}
.type-tag.ssh {
  background-color: rgba(16, 185, 129, 0.1);
  color: #10b981;
}
.type-tag.desktop {
  background-color: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}
.action-activate-btn {
  background: transparent;
  border: 1px solid var(--brand-primary);
  color: var(--brand-primary);
  padding: 2px 6px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 10px;
  transition: all 0.15s ease;
  outline: none;
}
.action-activate-btn:hover {
  background-color: rgba(16, 185, 129, 0.15);
}
.active-tag {
  font-size: 10px;
  color: var(--brand-primary);
  font-weight: 700;
}
.flex-center-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

/* About Section styles */
.about-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 30px 10px 10px 10px;
  gap: 20px;
}

.about-logo-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.about-logo-icon-bg {
  width: 72px;
  height: 72px;
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.1) 0%, rgba(59, 130, 246, 0.1) 100%);
  border: 1px solid rgba(16, 185, 129, 0.2);
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 8px 24px rgba(16, 185, 129, 0.15), inset 0 2px 4px rgba(255, 255, 255, 0.05);
  transition: all 0.3s ease;
}
.about-logo-icon-bg:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 28px rgba(16, 185, 129, 0.25), inset 0 2px 4px rgba(255, 255, 255, 0.1);
  border-color: rgba(16, 185, 129, 0.4);
}

.about-logo-img {
  filter: drop-shadow(0 2px 8px rgba(16, 185, 129, 0.4));
}

.about-logo-icon {
  color: var(--brand-primary);
  filter: drop-shadow(0 2px 8px rgba(16, 185, 129, 0.4));
}

.about-app-name {
  font-size: 22px;
  font-weight: 800;
  color: var(--text-title);
  letter-spacing: 1px;
  background: linear-gradient(120deg, var(--text-title) 30%, var(--brand-primary) 90%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.about-app-version {
  font-size: 11px;
  color: var(--text-muted);
  font-family: monospace;
  background-color: rgba(255, 255, 255, 0.04);
  padding: 2px 8px;
  border-radius: 10px;
  border: 1px solid var(--border-color);
}

.about-description-card {
  max-width: 460px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 16px;
  font-size: 11px;
  line-height: 1.6;
  color: var(--text-body);
  text-align: center;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
}

.about-description-card p {
  margin: 0 0 8px 0;
}
.about-description-card p:last-child {
  margin-bottom: 0;
}

.about-action-row {
  display: flex;
  justify-content: center;
  margin-top: 10px;
}

.update-check-btn {
  padding: 6px 18px !important;
  height: 30px !important;
  font-size: 11px !important;
  font-weight: bold;
}

.about-copyright {
  font-size: 9px;
  color: var(--text-muted);
  margin-top: 15px;
}

.rotating-icon {
  animation: spin 1.5s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Update Modal & Progress styles */
.update-info-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 12px;
}

.update-version-label {
  font-size: 12px;
  color: var(--text-title);
}

.update-date-label {
  font-size: 10px;
  color: var(--text-muted);
}

.update-logs-box {
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 12px;
  margin-bottom: 16px;
  max-height: 150px;
  overflow-y: auto;
}

.update-logs-title {
  font-size: 10px;
  font-weight: bold;
  color: var(--text-title);
  margin-bottom: 6px;
}

.update-logs-content {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.update-progress-area {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
  padding: 12px;
  background-color: rgba(16, 185, 129, 0.02);
  border: 1px solid rgba(16, 185, 129, 0.1);
  border-radius: 4px;
}

.progress-bar-container {
  width: 100%;
  height: 6px;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
  overflow: hidden;
  position: relative;
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #10b981 0%, #3b82f6 100%);
  box-shadow: 0 0 8px rgba(16, 185, 129, 0.5);
  border-radius: 3px;
  transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.progress-text-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 10px;
}

.progress-status-text {
  color: var(--text-body);
}

.progress-percent-text {
  font-weight: bold;
  color: var(--brand-primary);
  font-family: monospace;
}
</style>
