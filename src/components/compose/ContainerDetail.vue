<template>
  <div class="container-detail">
    <div v-if="loading" class="loading-overlay">
      <n-spin size="large" />
    </div>
    <template v-else-if="container">
      <div class="detail-header">
        <div class="container-info">
          <div class="name">{{ container.name }}</div>
          <div class="id-badge">{{ container.id.substring(0, 12) }}</div>
        </div>
        <div class="action-group">
          <n-button quaternary circle size="small" @click="$emit('restart')">
            <template #icon><n-icon><RefreshOutline /></n-icon></template>
          </n-button>
          <n-button quaternary circle size="small" @click="$emit('stop')">
            <template #icon><n-icon><StopOutline /></n-icon></template>
          </n-button>
          <n-button type="primary" size="small" round @click="$emit('terminal')">终端</n-button>
        </div>
      </div>

      <div class="tab-navigation">
        <SegmentedControl 
          v-model="activeTab" 
          :options="[
            { label: '概览', value: 'overview' },
            { label: '日志', value: 'logs' },
            { label: '仪表盘', value: 'stats' },
            { label: '设置', value: 'settings' }
          ]" 
        />
      </div>

      <div class="detail-content">
        <slot :name="activeTab"></slot>
      </div>
    </template>
    <div v-else class="empty-state">
      <n-text depth="3">请选择一个容器以查看详情</n-text>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NButton, NIcon, NSpin, NText } from 'naive-ui'
import { RefreshOutline, StopOutline } from '@vicons/ionicons5'
import SegmentedControl from '../common/SegmentedControl.vue'

const props = defineProps<{
  container: any | null,
  loading: boolean
}>()

const emit = defineEmits(['restart', 'stop', 'terminal'])

const activeTab = ref('overview')
</script>

<style scoped>
.container-detail {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
}
.loading-overlay, .empty-state {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}
.detail-header {
  padding: 20px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.container-info .name {
  font-size: 20px;
  font-weight: 700;
  color: #1d1d1f;
}
.container-info .id-badge {
  font-size: 12px;
  color: #86868b;
  font-family: monospace;
}
.action-group {
  display: flex;
  gap: 8px;
}
.tab-navigation {
  padding: 0 24px 16px;
  border-bottom: 0.5px solid var(--macos-border-color);
}
.detail-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
