<script setup lang="ts">
import { onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { exit } from '@tauri-apps/plugin-process'
import { useSettingsStore } from './store/settings'
import { 
  NConfigProvider, 
  NMessageProvider, 
  NDialogProvider, 
  NNotificationProvider, 
  NGlobalStyle, 
  zhCN, 
  dateZhCN,
  GlobalThemeOverrides
} from 'naive-ui'

const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#007AFF',
    primaryColorHover: '#007AFF',
    primaryColorPressed: '#007AFF',
    borderRadius: '8px'
  },
  Menu: {
    itemColorActive: '#007AFF',
    itemColorActiveHover: '#007AFF',
    itemTextColorActive: '#FFFFFF',
    itemTextColorActiveHover: '#FFFFFF',
    itemIconColorActive: '#FFFFFF',
    itemIconColorActiveHover: '#FFFFFF',
    itemHeight: '40px',
    borderRadius: '6px'
  },
  Dropdown: {
    optionColorHover: '#007AFF',
    optionColorActive: '#007AFF',
    optionTextColorHover: '#FFFFFF',
    optionTextColorActive: '#FFFFFF',
    optionIconColorHover: '#FFFFFF',
    optionIconColorActive: '#FFFFFF',
    padding: '4px',
    borderRadius: '6px'
  }
}

const settingsStore = useSettingsStore()

onMounted(async () => {
  await settingsStore.loadSettings()
  
  // 监听窗口关闭事件
  const appWindow = getCurrentWindow()
  await appWindow.onCloseRequested(async (_event) => {
    if (!settingsStore.closeToTray) {
      // 如果没有开启“最小化到托盘”，则直接退出程序
      // 先销毁窗口，确保 WebView2 资源释放，避免 1412 错误
      await appWindow.destroy()
      await exit(0)
    }
  })
})
</script>

<template>
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN" :theme-overrides="themeOverrides">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <n-global-style />
          <router-view />
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
body {
  margin: 0;
  font-family: v-sans, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  overflow: hidden;
  background-color: #F5F5F7;
}

/* 移除全局布局中可能存在的灰色叠加 */
.n-menu-item-content--selected::before {
  background-color: #007AFF !important;
  opacity: 1 !important;
}

.n-menu-item-content--selected:hover::before {
  background-color: #007AFF !important;
  opacity: 1 !important;
}

/* 修复右键菜单的外边距和圆角，使其更像 macOS */
.n-dropdown-option-body {
  margin: 2px 4px !important;
}
</style>
