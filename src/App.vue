<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { exit } from '@tauri-apps/plugin-process'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from './store/settings'
import SingleInstanceListener from './components/common/SingleInstanceListener.vue'
import StartupUpdater from './components/common/StartupUpdater.vue'
import { 
  NConfigProvider, 
  NMessageProvider, 
  NDialogProvider, 
  NNotificationProvider, 
  NGlobalStyle, 
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
  await settingsStore.loadSettings()
  
  // 初始化后端连接上下文
  try {
    await invoke('update_connection_config', { 
      mode: settingsStore.connectionMode, 
      distro: settingsStore.wslDistro 
    })
  } catch (e) {
    console.error('初始化后端配置失败:', e)
  }
  
  // 监听窗口关闭事件
  const appWindow = getCurrentWindow()
  await appWindow.onCloseRequested(async (_event) => {
    if (!settingsStore.closeToTray) {
      await appWindow.destroy()
      await exit(0)
    }
  })
})
</script>

<template>
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN" :theme="currentTheme" :theme-overrides="themeOverrides">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <n-global-style />
          <router-view />
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
</style>

