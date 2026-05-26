<template>
  <div class="resource-detail">
    <div v-if="loading" class="loading-overlay">
      <n-spin size="large"/>
    </div>
    <template v-else-if="item">
      <div class="detail-header">
        <div class="resource-info">
          <div class="name">{{ title }}</div>
          <div class="subtitle">{{ subtitle }}</div>
        </div>
        <div class="action-group">
          <slot name="actions"></slot>
        </div>
      </div>

      <div v-if="tabs && tabs.length > 0" class="tab-navigation">
        <SegmentedControl
            v-model="activeTab"
            :options="tabs"
        />
      </div>

      <div class="detail-content">
        <slot :name="activeTab"></slot>
        <slot></slot>
      </div>
    </template>
    <div v-else class="empty-state">
      <n-text depth="3">{{ emptyText || '请选择一个项目以查看详情' }}</n-text>
    </div>
  </div>
</template>

<script lang="ts" setup>
import {ref, watch} from 'vue'
import {NSpin, NText} from 'naive-ui'
import SegmentedControl from '../common/SegmentedControl.vue'

const props = defineProps<{
  item: any | null,
  loading: boolean,
  title: string,
  subtitle: string,
  emptyText?: string,
  tabs?: { label: string, value: string }[]
}>()

const activeTab = ref(props.tabs?.[0]?.value || '')

watch(() => props.item, () => {
  if (props.tabs && props.tabs.length > 0) {
    activeTab.value = props.tabs[0].value
  }
})
</script>

<style scoped>
.resource-detail {
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

.resource-info .name {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-title);
}

.resource-info .subtitle {
  font-size: 12px;
  color: var(--text-muted);
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
