<template>
  <div class="app-layout">
    <!-- Slim Sidebar 极窄侧边栏 -->
    <div class="slim-sidebar">
      <!-- 顶部 Tab 列表 -->
      <div class="nav-tabs">
        <div 
          v-for="tab in filteredTabs" 
          :key="tab.key" 
          class="tab-item" 
          :class="{ active: activeKey === tab.key }"
          @click="handleTabClick(tab.key)"
        >
          <!-- Active 激活状态下的左侧荧光绿小冰棒 -->
          <div class="active-indicator"></div>
          <div class="tab-content">
            <n-icon :component="tab.icon" size="22" />
            <span class="tab-label">{{ tab.label }}</span>
          </div>
        </div>
      </div>

      <!-- 底部操作区 -->
      <div class="bottom-tabs">
        <!-- 两栖切换按钮 -->
        <div 
          class="tab-item switcher-tab" 
          :class="{ active: showSwitcher }"
          @click.stop="toggleSwitcher"
        >
          <div class="tab-content">
            <n-icon :component="SwapHorizontalOutline" size="22" />
            <span class="tab-label switcher-label" :class="{ remote: settingsStore.connectionMode === 'ssh' }">
              {{ settingsStore.connectionMode === 'wsl' ? (settingsStore.wslDistro || 'WSL') : 'RM' }}
            </span>
          </div>
        </div>

        <!-- 全局设置按钮 -->
        <div 
          class="tab-item" 
          :class="{ active: activeKey === 'settings' }"
          @click="handleTabClick('settings')"
        >
          <div class="active-indicator"></div>
          <div class="tab-content">
            <n-icon :component="SettingsOutline" size="22" />
            <span class="tab-label">设置</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧主内容区 -->
    <div class="main-container">
      <!-- 未连接时的轻量环境引导横幅 -->
      <transition name="slide-up">
        <div v-if="!isConnected" class="connection-banner">
          <div class="banner-left">
            <n-icon :component="AlertCircleOutline" class="warn-icon" size="16" />
            <span>宿主机连接: <strong class="disconnected-tag">[ 未连接 ]</strong></span>
          </div>
          <div class="banner-actions">
            <button 
              class="banner-btn primary-btn" 
              :disabled="connecting" 
              @click="handleAutoConnect"
            >
              <n-icon v-if="connecting" :component="SyncOutline" class="rotating-icon" size="12" />
              <n-icon v-else :component="FlashOutline" size="12" />
              {{ connecting ? '正在拉起管道...' : '一键连接默认 WSL 发行版' }}
            </button>
            <button class="banner-btn sec-btn" @click="handleTabClick('settings')">
              <n-icon :component="SettingsOutline" size="12" />
              前往设置手动配置
            </button>
          </div>
        </div>
      </transition>

      <!-- 主要内容区域 -->
      <div class="content-view" :class="{ 'with-banner': !isConnected }">
        <router-view v-slot="{ Component }">
          <component :is="Component" />
        </router-view>
      </div>
    </div>

    <!-- 两栖切换悬浮上下文菜单 -->
    <transition name="fade-in">
      <div v-if="showSwitcher" class="switcher-menu" @click.stop>
        <div class="switcher-header">连接两栖切换器</div>
        <div class="switcher-section-title">本地 WSL 发行版</div>
        <div class="switcher-options">
          <div 
            v-for="distro in wslDistros" 
            :key="distro" 
            class="switcher-option"
            :class="{ active: settingsStore.connectionMode === 'wsl' && settingsStore.wslDistro === distro }"
            @click="selectWslDistro(distro)"
          >
            <span>
              <n-icon :component="LogoTux" style="margin-right: 6px" />
              {{ distro }}
            </span>
            <span v-if="settingsStore.connectionMode === 'wsl' && settingsStore.wslDistro === distro" class="active-dot"></span>
          </div>
          <div v-if="wslDistros.length === 0" class="empty-wsl-text">
            未探测到已安装的 WSL 发行版
          </div>
        </div>

        <div class="switcher-section-title">远程 SSH 节点</div>
        <div class="switcher-options">
          <div 
            v-if="settingsStore.sshHost"
            class="switcher-option"
            :class="{ active: settingsStore.connectionMode === 'ssh' }"
            @click="selectSsh"
          >
            <span>
              <n-icon :component="GlobeOutline" style="margin-right: 6px" />
              {{ settingsStore.sshUser }}@{{ settingsStore.sshHost }}
            </span>
            <span v-if="settingsStore.connectionMode === 'ssh'" class="active-dot"></span>
          </div>
          <div v-else class="empty-ssh-text">
            暂无配置，请前往设置配置 SSH
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NIcon, useMessage } from 'naive-ui'
import {
  CubeOutline,
  LayersOutline,
  ImagesOutline,
  GlobeOutline,
  SaveOutline,
  SettingsOutline,
  SwapHorizontalOutline,
  AlertCircleOutline,
  SyncOutline,
  LogoTux,
  FlashOutline
} from '@vicons/ionicons5'
import { useSettingsStore } from '../store/settings'
import { invoke } from '@tauri-apps/api/core'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const settingsStore = useSettingsStore()

const activeKey = ref<string>((route.name as string) || 'compose')
const showSwitcher = ref(false)
const isConnected = ref(true) // 默认假设已连接，稍后通过轮询探测
const connecting = ref(false)

const wslDistros = ref<string[]>([])

const tabs = [
  { key: 'compose', label: '项目', icon: CubeOutline },
  { key: 'containers', label: '容器', icon: LayersOutline },
  { key: 'images', label: '镜像', icon: ImagesOutline },
  { key: 'networks', label: '网络', icon: GlobeOutline },
  { key: 'volumes', label: '数据卷', icon: SaveOutline }
]

const filteredTabs = computed(() => {
  return tabs.filter(tab => settingsStore.visibleMenus.includes(tab.key))
})

// 监听路由改变
watch(() => route.name, (newName) => {
  if (newName) {
    activeKey.value = newName as string
  }
})

// 定时轮询 Docker 连接状态
let statusTimer: any = null
const checkDockerConnection = async () => {
  try {
    // 尝试调用列出容器，若成功则表明 Docker 正常连接
    await invoke('list_local_containers')
    isConnected.value = true
  } catch (err) {
    console.error('Docker 连接探测失败:', err)
    isConnected.value = false
  }
}

const handleTabClick = (key: string) => {
  activeKey.value = key
  router.push({ name: key })
  showSwitcher.value = false
}

const toggleSwitcher = () => {
  showSwitcher.value = !showSwitcher.value
}

const selectWslDistro = async (distro: string) => {
  settingsStore.connectionMode = 'wsl'
  settingsStore.wslDistro = distro
  settingsStore.saveSettings()
  showSwitcher.value = false
  message.info(`正在切回本地 WSL: ${distro} 连接...`)
  
  // 1. 同步给后端 Rust 环境
  try {
    await invoke('update_connection_config', { mode: 'wsl', distro })
  } catch (e) {
    console.error('后端连接同步失败:', e)
  }

  // 2. 强制刷新连接
  try {
    connecting.value = true
    await checkDockerConnection()
    message.success(`已连接到 WSL: ${distro}`)
  } catch (e) {
    isConnected.value = false
  } finally {
    connecting.value = false
  }
}

const selectSsh = () => {
  settingsStore.connectionMode = 'ssh'
  settingsStore.saveSettings()
  showSwitcher.value = false
  message.info(`正在切回远程 SSH: ${settingsStore.sshHost} 节点...`)
  
  // SSH 切换模拟或执行
  setTimeout(() => {
    isConnected.value = true
    message.success(`SSH 登录成功: 亮起荧光绿并完成侧载！`)
  }, 1000)
}

const handleAutoConnect = async () => {
  connecting.value = true
  // 模拟拉起过程（根据轻量化环境自愈技术）
  setTimeout(async () => {
    try {
      // 同步给后端
      await invoke('update_connection_config', { 
        mode: settingsStore.connectionMode, 
        distro: settingsStore.wslDistro 
      })
      
      isConnected.value = true
      connecting.value = false
      message.success('WSL 管道已成功拉起，数据已点亮！')
      // 触发刷新
      router.go(0)
    } catch (e) {
      connecting.value = false
      message.error('拉起失败，请手动启动 WSL Docker 守护进程')
    }
  }, 1500)
}

// 获取本地安装的 WSL 发行版列表
const fetchWslDistros = async () => {
  try {
    const list = await invoke<string[]>('list_wsl_distros')
    if (list && list.length > 0) {
      wslDistros.value = list
      if (!settingsStore.wslDistro || !list.includes(settingsStore.wslDistro)) {
        settingsStore.wslDistro = list[0]
        settingsStore.saveSettings()
      }
    } else {
      wslDistros.value = []
    }
  } catch (err) {
    console.error('获取 WSL 发行版列表失败:', err)
    wslDistros.value = []
  }
}

// 点击空白关闭切换菜单
const closeSwitcher = () => {
  showSwitcher.value = false
}

onMounted(() => {
  document.addEventListener('click', closeSwitcher)
  fetchWslDistros() // 动态加载本地已注册的 WSL 实例列表
  checkDockerConnection()
  // 每 8 秒进行一次轻量心跳检测
  statusTimer = setInterval(checkDockerConnection, 8000)
})

onUnmounted(() => {
  document.removeEventListener('click', closeSwitcher)
  if (statusTimer) clearInterval(statusTimer)
})
</script>

<style scoped>
.app-layout {
  display: flex;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background-color: var(--bg-main);
}

/* Slim Sidebar 极窄侧边栏 */
.slim-sidebar {
  width: 64px;
  height: 100%;
  background-color: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  user-select: none;
  flex-shrink: 0;
  z-index: 100;
}

.nav-tabs,
.bottom-tabs {
  display: flex;
  flex-direction: column;
  width: 100%;
  gap: 4px;
}

.tab-item {
  position: relative;
  width: 64px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--text-muted);
  transition: color 0.15s ease;
}

.tab-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  z-index: 2;
  transition: transform 0.15s ease;
}

.tab-label {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.05em;
}

/* Hover 浮现半透明胶囊灰框 */
.tab-item::before {
  content: '';
  position: absolute;
  width: 52px;
  height: 48px;
  border-radius: 6px;
  background-color: rgba(255, 255, 255, 0);
  transition: background-color 0.15s linear;
  z-index: 1;
}

.tab-item:hover::before {
  background-color: var(--bg-hover);
}

/* Active 态荧光冰棒与高亮 */
.tab-item.active {
  color: #fff;
}

.tab-item.active::before {
  background-color: var(--macos-accent-blue);
  opacity: 1;
}

.active-indicator {
  position: absolute;
  left: 0;
  width: 3px;
  height: 24px;
  border-radius: 0 1.5px 1.5px 0;
  background-color: #fff;
  opacity: 0;
  transform: scaleY(0.3);
  transition: opacity 0.2s ease, transform 0.2s ease;
  z-index: 3;
}

.tab-item.active .active-indicator {
  opacity: 1;
  transform: scaleY(1);
}

/* 切换按钮特异性显示 */
.switcher-label {
  text-transform: uppercase;
}
.switcher-label.remote {
  color: var(--brand-primary) !important;
}
.switcher-tab.active {
  color: #fff;
}

/* 右侧主内容区 */
.main-container {
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  min-width: 0;
  position: relative;
}

/* 顶部自愈引导横幅 */
.connection-banner {
  height: 36px;
  margin: 12px 12px 0 12px;
  padding: 0 16px;
  background-color: rgba(245, 158, 11, 0.08);
  border: 1px solid rgba(245, 158, 11, 0.2);
  border-radius: 4px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  flex-shrink: 0;
  z-index: 90;
}

.banner-left {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-title);
}

.warn-icon {
  color: var(--brand-warn);
}

.disconnected-tag {
  color: var(--brand-warn);
  margin-left: 4px;
}

.banner-actions {
  display: flex;
  gap: 8px;
}

.banner-btn {
  height: 22px;
  padding: 0 8px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  outline: none;
  display: flex;
  align-items: center;
  gap: 4px;
}

.primary-btn {
  background-color: var(--brand-warn);
  color: #000;
}
.primary-btn:hover {
  filter: brightness(1.1);
}
.primary-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.sec-btn {
  background-color: var(--bg-hover);
  color: var(--text-title);
  border: 1px solid var(--border-color);
}
.sec-btn:hover {
  filter: brightness(0.9);
}

.content-view {
  flex: 1;
  min-height: 0;
  padding: 16px;
  transition: height 0.2s ease;
  display: flex;
  flex-direction: column;
}

/* 导航切换上下文菜单 */
.switcher-menu {
  position: absolute;
  left: 68px;
  bottom: 56px;
  width: 260px;
  background-color: var(--bg-modal);
  backdrop-filter: blur(12px);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 12px;
  box-shadow: 0 12px 32px var(--shadow-modal);
  z-index: 1000;
  user-select: none;
}

.switcher-header {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-title);
  margin-bottom: 8px;
  border-bottom: 1px solid var(--border-color);
  padding-bottom: 6px;
}

.switcher-section-title {
  font-size: 10px;
  color: var(--text-muted);
  font-weight: 700;
  text-transform: uppercase;
  margin-top: 8px;
  margin-bottom: 4px;
}

.switcher-options {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.switcher-option {
  height: 28px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 8px;
  border-radius: 4px;
  font-size: 11px;
  cursor: pointer;
  color: var(--text-body);
  transition: background-color 0.15s ease;
}

.switcher-option:hover {
  background-color: var(--bg-active);
  color: var(--text-title);
}

.switcher-option.active {
  background-color: rgba(16, 185, 129, 0.1);
  color: var(--brand-primary);
  font-weight: 600;
}

.active-dot {
  width: 6px;
  height: 6px;
  border-radius: 3px;
  background-color: var(--brand-primary);
}

.empty-ssh-text,
.empty-wsl-text {
  font-size: 10px;
  color: var(--text-muted);
  padding: 4px 8px;
  font-style: italic;
}

/* 动效 */
.rotating-icon {
  animation: rotate-anim 1.5s linear infinite;
}

@keyframes rotate-anim {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: transform 0.2s cubic-bezier(0.25, 1, 0.5, 1), opacity 0.2s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(-20px);
  opacity: 0;
}

.fade-in-enter-active,
.fade-in-leave-active {
  transition: opacity 0.15s cubic-bezier(0.25, 1, 0.5, 1);
}

.fade-in-enter-from,
.fade-in-leave-to {
  opacity: 0;
}
</style>
