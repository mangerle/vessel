<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useSettingsStore } from '../store/settings'
import { invoke } from '@tauri-apps/api/core'
import {
  NSelect,
  NSwitch,
  NInput,
  NInputNumber,
  NRadio,
  NRadioGroup,
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
  SaveOutline
} from '@vicons/ionicons5'

const settingsStore = useSettingsStore()
const message = useMessage()

const activeTab = ref<string>('general')

// 页面分类小页签
const tabs = [
  { label: '基础常规', value: 'general', icon: DesktopOutline },
  { label: 'Docker 引擎', value: 'docker', icon: LogoDocker },
  { label: '镜像仓库', value: 'registries', icon: GlobeOutline },
  { label: '账户凭证', value: 'credentials', icon: ShieldCheckmarkOutline },
  { label: '数据备份', value: 'backup', icon: SaveOutline }
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
  connectionMode: 'wsl',
  wslDistro: 'Ubuntu',
  sshHost: '192.168.1.105',
  sshPort: 22,
  sshUser: 'root',
  sshPassword: 'my_ssh_root_password',
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
    connectionMode: settingsStore.connectionMode,
    wslDistro: settingsStore.wslDistro || 'Ubuntu',
    sshHost: settingsStore.sshHost || '192.168.1.105',
    sshPort: settingsStore.sshPort,
    sshUser: settingsStore.sshUser || 'root',
    sshPassword: settingsStore.sshPassword || '',
    registries: settingsStore.registries.map(r => ({ ...r }))
  }
}

// 探测配置是否被篡改过 (Dirty 检测)
const isDirty = computed(() => {
  return (
    draft.value.theme !== settingsStore.theme ||
    draft.value.autoStart !== settingsStore.autoStart ||
    draft.value.closeToTray !== settingsStore.closeToTray ||
    draft.value.refreshInterval !== settingsStore.refreshInterval ||
    JSON.stringify(draft.value.visibleMenus) !== JSON.stringify(settingsStore.visibleMenus) ||
    draft.value.connectionMode !== settingsStore.connectionMode ||
    draft.value.wslDistro !== (settingsStore.wslDistro || 'Ubuntu') ||
    draft.value.sshHost !== (settingsStore.sshHost || '192.168.1.105') ||
    draft.value.sshPort !== settingsStore.sshPort ||
    draft.value.sshUser !== (settingsStore.sshUser || 'root') ||
    draft.value.sshPassword !== (settingsStore.sshPassword || '') ||
    JSON.stringify(draft.value.registries) !== JSON.stringify(settingsStore.registries)
  )
})

const handleSave = async () => {
  // 1. 同步给后端 Rust 环境 (核心：两栖执行环境切换)
  try {
    await invoke('update_connection_config', { 
      mode: draft.value.connectionMode, 
      distro: draft.value.wslDistro 
    })
  } catch (e) {
    console.error('后端配置同步失败:', e)
  }

  // 2. 保存到 store 状态中并自动持久化
  settingsStore.theme = draft.value.theme as any
  settingsStore.closeToTray = draft.value.closeToTray
  settingsStore.refreshInterval = draft.value.refreshInterval
  settingsStore.visibleMenus = [...draft.value.visibleMenus]
  settingsStore.connectionMode = draft.value.connectionMode as any
  settingsStore.wslDistro = draft.value.wslDistro
  settingsStore.sshHost = draft.value.sshHost
  settingsStore.sshPort = draft.value.sshPort
  settingsStore.sshUser = draft.value.sshUser
  settingsStore.sshPassword = draft.value.sshPassword
  settingsStore.registries = draft.value.registries.map(r => ({ ...r }))
  
  await settingsStore.setAutoStart(draft.value.autoStart) // 会触发自启动插件并保存
  settingsStore.saveSettings()
  
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
      if (!draft.value.wslDistro || !list.includes(draft.value.wslDistro)) {
        draft.value.wslDistro = list[0]
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
const handleResetFactory = () => {
  localStorage.clear()
  message.error('本地缓存已擦除！恢复出厂设置中...')
  setTimeout(() => {
    window.location.reload()
  }, 1000)
}

onMounted(async () => {
  await settingsStore.loadSettings()
  syncDraftFromStore()
  await handleRefreshWsl(true) // 冷启动静默探测
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
            <div class="section-title">Docker 引擎两栖通信设置</div>

            <div class="form-row">
              <div class="row-label-area">
                <div class="row-title">引擎连接模式</div>
                <div class="row-desc">选择通过本地 WSL 管道侧载拉起还是通过 SSH 连接远程主机 Docker。</div>
              </div>
              <div class="row-value-area">
                <n-radio-group v-model:value="draft.connectionMode" name="conn-mode">
                  <n-space>
                    <n-radio value="wsl">本地 WSL 管道侧载</n-radio>
                    <n-radio value="ssh">远程 SSH 密码连接</n-radio>
                  </n-space>
                </n-radio-group>
              </div>
            </div>

            <!-- 本地 WSL 管道侧载模式配置 -->
            <transition name="fade-in">
              <div v-if="draft.connectionMode === 'wsl'" class="nested-mode-card">
                <div class="nested-title">🐧 本地 WSL 管道侧载配置</div>
                <div class="form-row borderless">
                  <div class="row-label-area">
                    <div class="row-title">WSL Linux 分发版</div>
                    <div class="row-desc">选择安装了 Docker 守护进程的默认 WSL 系统。</div>
                  </div>
                  <div class="row-value-area flex-row">
                    <n-select v-model:value="draft.wslDistro" :options="wslOptions" class="select-field select-wsl" size="small" />
                    <button class="form-action-btn" @click="() => handleRefreshWsl()">🔄 重新扫描</button>
                  </div>
                </div>
              </div>
            </transition>

            <!-- 远程 SSH 密码连接模式配置 -->
            <transition name="fade-in">
              <div v-if="draft.connectionMode === 'ssh'" class="nested-mode-card">
                <div class="nested-title">🌐 远程 SSH 密码连接配置</div>
                
                <div class="grid-form-fields">
                  <div class="field-item">
                    <span class="field-label">主机地址 (Host)</span>
                    <n-input v-model:value="draft.sshHost" size="small" placeholder="例如: 192.168.1.105" />
                  </div>
                  <div class="field-item">
                    <span class="field-label">端口 (Port)</span>
                    <n-input-number v-model:value="draft.sshPort" size="small" :min="1" :max="65535" />
                  </div>
                  <div class="field-item">
                    <span class="field-label">用户名 (Username)</span>
                    <n-input v-model:value="draft.sshUser" size="small" placeholder="root" />
                  </div>
                  <div class="field-item">
                    <span class="field-label">登录密码 (Password)</span>
                    <n-input v-model:value="draft.sshPassword" type="password" size="small" placeholder="my_ssh_root_password" />
                  </div>
                </div>
              </div>
            </transition>

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
              <button class="form-action-btn border-btn" @click="openAddModal">
                ➕ 新增私有仓/加速器
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
                        class="action-delete-btn" 
                        @click="handleDeleteRegistry(reg.id)"
                        title="删除该镜像仓库"
                      >
                        🗑️ 删除
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
        </n-scrollbar>
      </div>
    </div>

    <!-- 底部即改即生效控制栏 (绝对布局贴底) -->
    <div class="settings-bottom-actions">
      <button 
        class="save-btn" 
        :class="{ 'breath-green': isDirty }"
        :disabled="!isDirty" 
        @click="handleSave"
      >
        💾 保存配置
      </button>
      <button 
        class="cancel-btn" 
        :disabled="!isDirty" 
        @click="handleCancel"
      >
        ↩️ 取消
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
  padding-bottom: 56px; /* 留出底部保存按钮空间 */
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
  bottom: 0;
  left: 0;
  width: 100%;
  height: 48px;
  background-color: var(--bg-sidebar);
  border-top: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  padding: 0 20px;
  gap: 12px;
  z-index: 10;
}

.save-btn {
  height: 28px;
  padding: 0 16px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background-color: rgba(255,255,255,0.03);
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  outline: none;
  transition: all 0.2s ease;
}

.save-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.save-btn.breath-green {
  background-color: var(--brand-primary);
  border-color: var(--brand-primary);
  color: #000;
  box-shadow: 0 0 12px rgba(16, 185, 129, 0.2);
  animation: green-breath-anim 2s infinite ease-in-out;
}

@keyframes green-breath-anim {
  0% { box-shadow: 0 0 4px rgba(16, 185, 129, 0.2); }
  50% { box-shadow: 0 0 14px rgba(16, 185, 129, 0.5); }
  100% { box-shadow: 0 0 4px rgba(16, 185, 129, 0.2); }
}

.cancel-btn {
  height: 28px;
  padding: 0 16px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background-color: rgba(255,255,255,0.03);
  color: var(--text-body);
  font-size: 11px;
  cursor: pointer;
  outline: none;
  transition: all 0.15s ease;
}
.cancel-btn:hover:not(:disabled) {
  background-color: var(--bg-active);
  color: var(--text-title);
}
.cancel-btn:disabled {
  opacity: 0.5;
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
</style>
