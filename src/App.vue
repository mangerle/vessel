<script setup lang="ts">
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
