<template>
  <div class="compose-project-list">
    <div class="search-header">
      <n-input v-model:value="searchQuery" placeholder="搜索容器..." size="small" round clearable>
        <template #prefix>
          <n-icon><SearchOutline /></n-icon>
        </template>
      </n-input>
    </div>
    <n-scrollbar class="list-container">
      <div 
        v-for="item in filteredItems" 
        :key="item.id" 
        class="container-item"
        :class="{ active: selectedId === item.id }"
        @click="handleSelect(item.id)"
      >
        <div class="status-indicator" :style="{ backgroundColor: item.state === 'running' ? 'var(--macos-success-green)' : 'var(--macos-border-color)' }"></div>
        <div class="item-body">
          <div class="item-name">{{ item.name }}</div>
          <div class="item-sub">{{ item.image }}</div>
        </div>
      </div>
    </n-scrollbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NInput, NIcon, NScrollbar } from 'naive-ui'
import { SearchOutline } from '@vicons/ionicons5'

const props = defineProps<{
  items: any[],
  selectedId: string | null
}>()

const emit = defineEmits(['select'])

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
.compose-project-list {
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
}
.container-item:hover {
  background-color: rgba(0, 0, 0, 0.02);
}
.container-item.active {
  background-color: rgba(0, 122, 255, 0.1);
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
  color: #1d1d1f;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.item-sub {
  font-size: 12px;
  color: #86868b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
