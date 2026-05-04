<template>
  <div class="settings-view">
    <div class="header">
      <h1>设置</h1>
    </div>
    
    <div class="settings-content">
      <n-card title="通用设置" size="medium">
        <n-list>
          <n-list-item>
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-label">开机自启动</div>
                <div class="setting-description">系统登录时自动启动 Vessel</div>
              </div>
              <n-switch v-model:value="settingsStore.autoStart" @update:value="handleAutoStartChange" />
            </div>
          </n-list-item>

          <n-list-item>
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-label">退出行为</div>
                <div class="setting-description">点击关闭按钮时最小化到系统托盘</div>
              </div>
              <n-switch v-model:value="settingsStore.closeToTray" @update:value="handleCloseToTrayChange" />
            </div>
          </n-list-item>
          
          <n-list-item>
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-label">关于 Vessel</div>
                <div class="setting-description">版本 0.1.0</div>
              </div>
            </div>
          </n-list-item>
        </n-list>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { NCard, NList, NListItem, NSwitch } from 'naive-ui'
import { useSettingsStore } from '../store/settings'

const settingsStore = useSettingsStore()

const handleAutoStartChange = (value: boolean) => {
  settingsStore.setAutoStart(value)
}

const handleCloseToTrayChange = (value: boolean) => {
  settingsStore.setCloseToTray(value)
}

onMounted(async () => {
  await settingsStore.loadSettings()
})
</script>

<style scoped>
.settings-view {
  max-width: 800px;
  margin: 0 auto;
}

.header {
  margin-bottom: 24px;
}

.header h1 {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
}

.settings-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
}

.setting-description {
  font-size: 12px;
  color: #888;
}
</style>
