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
            <span class="tab-label switcher-label" :class="{ remote: currentConnectionMode === 'ssh' }">
              {{ currentConnectionShortName }}
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

    <!-- 引擎切换悬浮上下文菜单 -->
    <transition name="fade-in">
      <div v-if="showSwitcher" class="switcher-menu" @click.stop>
        <div class="switcher-header">切换连接引擎</div>
        <div class="switcher-options">
          <div 
            v-for="conn in settingsStore.connections" 
            :key="conn.id" 
            class="switcher-option"
            :class="{ active: settingsStore.activeConnectionId === conn.id }"
            @click="selectConnection(conn)"
          >
            <span>
              <n-icon v-if="conn.type === 'wsl'" :component="LogoTux" style="margin-right: 6px" />
              <n-icon v-else-if="conn.type === 'ssh'" :component="GlobeOutline" style="margin-right: 6px" />
              <n-icon v-else :component="LogoDocker" style="margin-right: 6px" />
              {{ conn.name }}
            </span>
            <span v-if="settingsStore.activeConnectionId === conn.id" class="active-dot"></span>
          </div>
          <div v-if="settingsStore.connections.length === 0" class="empty-wsl-text">
            未配置任何连接引擎
          </div>
        </div>
      </div>
    </transition>

    <!-- 切换连接进度 modal：connectionSwitching=true 时强制锁定 UI -->
    <n-modal
      :show="settingsStore.connectionSwitching"
      :mask-closable="false"
      :closable="false"
      preset="card"
      style="width: 360px"
      :show-header="false"
    >
      <div class="switching-modal-body">
        <n-icon :component="SyncOutline" size="36" class="rotating-icon" style="color: var(--brand-primary)" />
        <div class="switching-title">正在切换到</div>
        <div class="switching-target">{{ settingsStore.switchingTargetName || '...' }}</div>
        <div class="switching-sub">正在重建 Docker 连接通道，请稍候...</div>
      </div>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NIcon, NModal, useMessage } from 'naive-ui'
import { connectionApi } from '../api/connectionApi'
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
  LogoDocker,
  FlashOutline
} from '@vicons/ionicons5'
import { useSettingsStore } from '../store/settings'
import { switchConnection, refreshAllStores } from '../utils/connectionSwitcher'

import { usePolling } from '../utils/polling'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const settingsStore = useSettingsStore()

const activeKey = ref<string>((route.name as string) || 'compose')
const showSwitcher = ref(false)
const isConnected = ref(true) // 默认假设已连接，稍后通过轮询探测
const connecting = ref(false)

const currentConnectionShortName = computed(() => {
  const activeConn = settingsStore.connections.find(c => c.id === settingsStore.activeConnectionId)
  if (!activeConn) return 'DKP'
  if (activeConn.type === 'wsl') return activeConn.wslDistro || 'WSL'
  if (activeConn.type === 'ssh') return 'SSH'
  return 'DKP'
})

const currentConnectionMode = computed(() => {
  const activeConn = settingsStore.connections.find(c => c.id === settingsStore.activeConnectionId)
  return activeConn?.type || 'desktop'
})

const selectConnection = async (conn: { id: string; name: string; type: string; wslDistro?: string }) => {
  showSwitcher.value = false
  if (settingsStore.activeConnectionId === conn.id) return

  // 统一走 switchConnection：清列表 → 弹 modal → updateConfig → ping → preloadAll
  // 弹窗与列表清理已由 connectionSwitcher 内部驱动 store 完成
  isConnected.value = true
  try {
    await switchConnection(conn.id)
    // 切换成功后再落盘 active：避免 ping 失败时把无效 active 写入磁盘
    await settingsStore.saveSettings()
    message.success(`已连接到: ${conn.name}`)
  } catch (e: any) {
    isConnected.value = false
    message.error(`连接 ${conn.name} 失败: ${e}`)
  }
}

const handleAutoConnect = async () => {
  connecting.value = true
  try {
    const config = settingsStore.getActiveConnectionConfig()
    await connectionApi.updateConfig(config)
    await connectionApi.ping()
    isConnected.value = true
    // 重连成功后再补一次预拉，让首屏列表立即填充
    await refreshAllStores()
    message.success('数据管道已成功拉起，数据已点亮！')
  } catch (e) {
    message.error('拉起失败: ' + e)
  } finally {
    connecting.value = false
  }
}

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


// 数据预拉已由启动期 bootstrapConnection / 切换期 switchConnection 全权负责，
// 心跳只关心连通性，不再重复触发首屏预拉，避免重复 IPC 风暴。
let wasDisconnected = false
const { start: startHeartbeat, stop: stopHeartbeat } = usePolling(async () => {
  try {
    await connectionApi.ping()
    if (wasDisconnected) {
      // 断线 → 重连过渡：补一次预拉，让 UI 列表跟上
      wasDisconnected = false
      refreshAllStores().catch(() => {})
    }
    isConnected.value = true
  } catch (err) {
    console.error('Docker 连接探测失败:', err)
    isConnected.value = false
    wasDisconnected = true
  }
}, 8000)


const handleTabClick = (key: string) => {
  activeKey.value = key
  router.push({ name: key })
  showSwitcher.value = false
}

const toggleSwitcher = () => {
  showSwitcher.value = !showSwitcher.value
}

// 点击空白关闭切换菜单
const closeSwitcher = () => {
  showSwitcher.value = false
}

onMounted(() => {
  document.addEventListener('click', closeSwitcher)
  startHeartbeat()
})

onUnmounted(() => {
  document.removeEventListener('click', closeSwitcher)
  stopHeartbeat()
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

/* 切换连接 modal：居中卡片 + 旋转图标 */
.switching-modal-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 20px 12px 8px;
}
.switching-title {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-muted);
  letter-spacing: 0.5px;
}
.switching-target {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-title);
}
.switching-sub {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 4px;
}
</style>
