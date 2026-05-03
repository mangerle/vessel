<template>
  <div class="simple-container-list" @contextmenu="$emit('contextmenu', $event, 'global')">
    <div class="search-header">
      <n-input v-model:value="searchQuery" clearable placeholder="搜索独立容器..." round size="small">
        <template #prefix>
          <n-icon>
            <SearchOutline/>
          </n-icon>
        </template>
      </n-input>
    </div>
    <n-scrollbar class="list-container">
      <div
          v-for="item in filteredItems"
          :key="item.id"
          :class="{ active: selectedId === item.id }"
          class="container-item"
          @click="handleSelect(item.id)"
          @contextmenu.stop="$emit('contextmenu', $event, 'container', item)"
      >
        <div :style="{ backgroundColor: item.state === 'running' ? 'var(--macos-success-green)' : 'var(--macos-border-color)' }"
             class="status-indicator"></div>
        <div class="item-body">
          <div class="item-name">{{ item.name }}</div>
          <div class="item-sub">{{ item.image }}</div>
        </div>
      </div>
      <div v-if="filteredItems.length === 0" class="empty-list">
        <n-text depth="3">无匹配的独立容器</n-text>
      </div>
    </n-scrollbar>
  </div>
</template>

<script lang="ts" setup>
import {computed, ref} from 'vue'
import {NIcon, NInput, NScrollbar, NText} from 'naive-ui'
import {SearchOutline} from '@vicons/ionicons5'

const props = defineProps<{
  items: any[],
  selectedId: string | null
}>()

const emit = defineEmits(['select', 'contextmenu'])

const searchQuery = ref('')

const filteredItems = computed(() => {
  if (!searchQuery.value) return props.items
  const q = searchQuery.value.toLowerCase()
  return props.items.filter(i => i.name.toLowerCase().includes(q) || i.image.toLowerCase().includes(q))
})

const handleSelect = (id: string) => {
  emit('select', id)
}
</script>

<style scoped>
.simple-container-list {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.search-header {
  padding: 16px;
  border-bottom: 0.5px solid var(--macos-border-color);
}

.list-container {
  flex: 1;
}

.container-item {
  height: 64px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  cursor: pointer;
  transition: background-color 0.2s;
  position: relative;
  margin: 2px 8px;
  border-radius: 8px;
}

.container-item:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.container-item.active {
  background-color: var(--macos-accent-blue);
  color: white;
}

.status-indicator {
  width: 4px;
  height: 40px;
  border-radius: 2px;
  margin-right: 12px;
}

.item-body {
  display: flex;
  flex-direction: column;
  justify-content: center;
  overflow: hidden;
}

.item-name {
  font-weight: 600;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-sub {
  font-size: 12px;
  opacity: 0.7;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-list {
  padding: 32px;
  text-align: center;
}
</style>
