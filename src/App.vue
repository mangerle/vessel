<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useSettingsStore } from './store/settings'
import { bootstrapConnection } from './utils/connectionSwitcher'
import { EVT } from './api/events'
import type { ConnectionConfigPayload } from './api/connection'
import SingleInstanceListener from './components/common/SingleInstanceListener.vue'
import StartupUpdater from './components/common/StartupUpdater.vue'
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  NGlobalStyle,
  NSpin,
  zhCN,
  dateZhCN,
  GlobalThemeOverrides,
  darkTheme,
  lightTheme
} from 'naive-ui'

const settingsStore = useSettingsStore()

const currentTheme = computed(() => {
  return settingsStore.theme === 'light-apple' ? lightTheme : darkTheme
})

const themeOverrides = computed<GlobalThemeOverrides>(() => {
  const isDeepBlack = settingsStore.theme === 'deep-black'
  const isLightApple = settingsStore.theme === 'light-apple'
  
  let primary = '#4ade80'
  let bgMain = '#181818'
  let bgSidebar = '#121212'
  let bgActive = '#2d2d2d'
  let textTitle = '#ffffff'
  let textBody = '#cccccc'
  let border = 'rgba(255, 255, 255, 0.06)'

  if (isDeepBlack) {
    primary = '#10b981'
    bgMain = '#0b0f19'
    bgSidebar = '#070a10'
    bgActive = '#1e293b'
    textTitle = '#f8fafc'
    textBody = '#cbd5e1'
    border = 'rgba(255, 255, 255, 0.04)'
  } else if (isLightApple) {
    primary = '#0071e3'
    bgMain = '#ffffff'
    bgSidebar = '#f5f5f7'
    bgActive = '#e8e8ed'
    textTitle = '#1d1d1f'
    textBody = '#424245'
    border = 'rgba(0, 0, 0, 0.1)'
  }
  
  return {
    common: {
      primaryColor: primary,
      primaryColorHover: primary,
      primaryColorPressed: primary,
      bodyColor: bgMain,
      cardColor: bgMain,
      textColor1: textTitle,
      textColor2: textBody,
      borderRadius: '4px',
      borderColor: border,
      inputColor: bgSidebar,
      inputColorDisabled: bgSidebar,
      buttonColor2: bgActive
    },
    Card: {
      color: bgMain,
      borderColor: border,
      titleTextColor: textTitle,
      textColor: textBody
    },
    Menu: {
      itemColorActive: bgActive,
      itemColorActiveHover: bgActive,
      itemTextColorActive: isLightApple ? primary : textTitle,
      itemTextColorActiveHover: isLightApple ? primary : textTitle,
      itemIconColorActive: isLightApple ? primary : textTitle,
      itemIconColorActiveHover: isLightApple ? primary : textTitle,
      itemHeight: '40px',
      borderRadius: '4px'
    },
    Dropdown: {
      color: bgMain,
      optionColorHover: isLightApple ? '#007AFF' : '#007AFF',
      optionColorActive: isLightApple ? '#007AFF' : '#007AFF',
      optionTextColorHover: '#ffffff',
      optionTextColorActive: '#ffffff',
      optionIconColorHover: '#ffffff',
      optionIconColorActive: '#ffffff',
      padding: '4px',
      borderRadius: '4px'
    },
    List: {
      color: 'transparent',
      borderColor: border
    },
    Descriptions: {
      color: bgSidebar,
      borderColor: border,
      titleTextColor: textTitle,
      thColor: bgSidebar,
      tdColor: 'transparent'
    },
    Input: {
      color: bgSidebar,
      colorFocus: bgSidebar,
      textColor: textTitle,
      border: `1px solid ${border}`,
      borderFocus: `1px solid ${primary}`,
      borderHover: `1px solid ${primary}`
    },
    Tabs: {
      tabTextColorActiveLine: primary,
      tabTextColorHoverLine: primary,
      tabTextColorActiveBar: primary,
      tabTextColorHoverBar: primary,
      barColor: primary
    }
  }
})

onMounted(async () => {
  // 监听后端 connection-updated 事件，校正 activeConnectionId
  // 解决多窗口/托盘切换场景下前后端 active 状态失同步
  listen<ConnectionConfigPayload>(EVT.connectionUpdated, (event) => {
    settingsStore.applyBackendConfig(event.payload)
  })

  // 启动期串行化：loadSettings → updateConfig(active) → ping → preloadAll
  // 全程 connectionReady=false 由模板里全屏 Loading 遮罩，杜绝
  // 「fetch 已发飞 + updateConfig 关 client」造成的 SendRequest/Connect race。
  try {
    await bootstrapConnection()
  } catch (e) {
    console.error('启动期连接初始化失败:', e)
  }

  // 窗口关闭事件由后端 lib.rs 的 on_window_event 统一拦截（关闭即隐藏到托盘）；
  // 真正的「真正退出」由托盘菜单或单实例插件触发，不再在前端重复注册
})
</script>

<template>
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN" :theme="currentTheme" :theme-overrides="themeOverrides">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <n-global-style />
          <!-- 启动期 connectionReady=false 时全屏 Loading，杜绝早于连接就绪的 fetch 落空 -->
          <div v-if="!settingsStore.connectionReady" class="bootstrap-mask">
            <div class="bootstrap-card">
              <n-spin :size="36" />
              <div class="bootstrap-title">正在初始化连接</div>
              <div class="bootstrap-sub">正在启用激活的 Docker 引擎...</div>
            </div>
          </div>
          <router-view v-else />
          <SingleInstanceListener />
          <StartupUpdater />
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
body {
  margin: 0;
  font-family: "Segoe UI Variable Text", "Segoe UI", "Inter", system-ui, -apple-system, sans-serif;
  overflow: hidden;
  background-color: var(--bg-main);
  color: var(--text-body);
}

/* 覆盖 Naive UI 某些组件默认样式以匹配工业风 */
.n-layout {
  background-color: var(--bg-main) !important;
}

.n-layout-sider {
  background-color: var(--bg-sidebar) !important;
  border-right: 1px solid var(--border-color) !important;
}

/* 启动期全屏 Loading 遮罩 */
.bootstrap-mask {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-main, #181818);
}

.bootstrap-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 32px 48px;
  border: 1px solid var(--border-color, rgba(255,255,255,0.06));
  border-radius: 12px;
  background-color: var(--bg-sidebar, #121212);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
}

.bootstrap-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-title, #ffffff);
  letter-spacing: 0.5px;
}

.bootstrap-sub {
  font-size: 11px;
  color: var(--text-body, #cccccc);
}
</style>

