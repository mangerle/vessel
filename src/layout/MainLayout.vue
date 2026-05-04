<template>
  <n-layout has-sider position="absolute" style="height: 100vh">
    <n-layout-sider
        bordered
        collapse-mode="width"
        :collapsed-width="56"
        :width="200"
        :collapsed="collapsed"
        show-trigger
        @collapse="collapsed = true"
        @expand="collapsed = false"
    >
      <div class="logo">
      </div>
      <n-menu
          :collapsed="collapsed"
          :collapsed-width="56"
          :collapsed-icon-size="22"
          :options="menuOptions"
          :value="activeKey"
          @update:value="handleMenuClick"
      />
      
      <div class="bottom-section">
        <div 
          class="settings-item" 
          :class="{ active: activeKey === 'settings', collapsed }"
          @click="handleMenuClick('settings')"
        >
          <n-icon :component="SettingsOutline" size="22" />
          <span v-if="!collapsed">设置</span>
        </div>

        <div v-if="!collapsed && taskStore.tasks.length > 0" class="task-section">
          <div class="task-header">
            <span>后台任务</span>
            <n-button quaternary circle size="tiny" @click="taskStore.clearFinishedTasks">
              <template #icon><n-icon :component="TrashOutline" /></template>
            </n-button>
          </div>
          <n-scrollbar style="max-height: 200px">
            <div v-for="task in taskStore.tasks" :key="task.id" class="task-item">
              <div class="task-info">
                <span class="task-name" :title="task.name">{{ task.name }}</span>
                <span class="task-status" :class="task.status">
                  <n-icon v-if="task.status === 'running'" class="rotating" :component="SyncOutline" />
                  <n-icon v-else-if="task.status === 'success'" :component="CheckmarkCircleOutline" />
                  <n-icon v-else :component="AlertCircleOutline" />
                </span>
              </div>
              <n-progress
                type="line"
                :percentage="task.progress"
                :show-indicator="false"
                :status="task.status === 'error' ? 'error' : (task.status === 'success' ? 'success' : 'info')"
                :height="4"
                processing
              />
            </div>
          </n-scrollbar>
        </div>
      </div>
    </n-layout-sider>
    <n-layout>
      <n-layout-content content-style="padding: 20px; background-color: var(--macos-bg-light);">
        <router-view/>
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<script setup lang="ts">
import {Component, h, ref, watch} from 'vue'
import type {MenuOption} from 'naive-ui'
import {NIcon, NLayout, NLayoutContent, NLayoutSider, NMenu, NScrollbar, NProgress, NButton} from 'naive-ui'
import {useRoute, useRouter} from 'vue-router'
import {
  CubeOutline, 
  GlobeOutline, 
  ImagesOutline, 
  LayersOutline, 
  SaveOutline, 
  SyncOutline, 
  CheckmarkCircleOutline, 
  AlertCircleOutline,
  TrashOutline,
  SettingsOutline
} from '@vicons/ionicons5'
import {useTaskStore} from '../store/task'

const router = useRouter()
const route = useRoute()
const collapsed = ref(false)
const taskStore = useTaskStore()

const activeKey = ref<string>(route.name as string || 'compose')

// 监听路由变化更新激活的菜单项
watch(() => route.name, (newName) => {
  if (newName) {
    activeKey.value = newName as string
  }
})

function renderIcon(icon: Component) {
  return () => h(NIcon, null, {default: () => h(icon)})
}

const menuOptions: MenuOption[] = [
  {
    label: 'Compose',
    key: 'compose',
    icon: renderIcon(CubeOutline)
  },
  {
    label: '容器',
    key: 'containers',
    icon: renderIcon(LayersOutline)
  },
  {
    label: '镜像',
    key: 'images',
    icon: renderIcon(ImagesOutline)
  },
  {
    label: '网络',
    key: 'networks',
    icon: renderIcon(GlobeOutline)
  },
  {
    label: '卷',
    key: 'volumes',
    icon: renderIcon(SaveOutline)
  }
]

function handleMenuClick(key: string) {
  activeKey.value = key
  router.push({name: key})
}
</script>

<style scoped>
.logo {
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  white-space: nowrap;
}

.sidebar-menu {
  margin-bottom: 50px; /* Space for settings button */
}

.bottom-section {
  position: absolute;
  bottom: 0;
  width: 100%;
  border-top: 1px solid var(--macos-border-color);
  background: rgba(255, 255, 255, 0.5);
  backdrop-filter: blur(10px);
}

.settings-item {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  cursor: pointer;
  gap: 12px;
  transition: all 0.3s;
  color: rgb(51, 54, 57);
}

.settings-item:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.settings-item.active {
  color: #18a058;
  background-color: rgba(24, 160, 88, 0.1);
}

.settings-item.collapsed {
  justify-content: center;
  padding: 12px 0;
}

.task-section {
  width: 100%;
  border-top: 1px solid var(--macos-border-color);
  padding: 12px 0;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 12px 8px;
  font-size: 11px;
  font-weight: 600;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.task-item {
  padding: 8px 12px;
}

.task-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.task-name {
  font-size: 12px;
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-status {
  font-size: 14px;
  display: flex;
  align-items: center;
}

.task-status.running { color: #007AFF; }
.task-status.success { color: #34C759; }
.task-status.error { color: #FF3B30; }

.rotating {
  animation: rotate 2s linear infinite;
}

@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.empty-tasks {
  padding: 12px;
  text-align: center;
  font-size: 11px;
  color: #bbb;
}
</style>
