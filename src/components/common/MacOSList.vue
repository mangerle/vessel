<template>
  <div class="resource-list" @contextmenu="$emit('contextmenu', $event, 'global')">
    <div class="search-header">
      <n-input v-model:value="searchQuery" :placeholder="placeholder" clearable round size="small">
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
          :key="item[idKey]"
          :class="{ active: selectedId === item[idKey] }"
          class="resource-item"
          @click="handleSelect(item[idKey])"
          @contextmenu.stop="$emit('contextmenu', $event, 'item', item)"
      >
        <div class="item-body">
          <div class="item-name">{{ renderName(item) }}</div>
          <div class="item-sub">{{ renderSub(item) }}</div>
        </div>
      </div>
      <div v-if="filteredItems.length === 0" class="empty-list">
        <n-text depth="3">无匹配项</n-text>
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
  selectedId: string | null,
  placeholder: string,
  idKey: string,
  renderName: (item: any) => string,
  renderSub: (item: any) => string,
  searchFields: string[]
}>()

const emit = defineEmits(['select', 'contextmenu'])

const searchQuery = ref('')

const filteredItems = computed(() => {
  if (!searchQuery.value) return props.items
  const q = searchQuery.value.toLowerCase()
  return props.items.filter(i =>
      props.searchFields.some(field => {
        const val = i[field]
        if (Array.isArray(val)) {
          return val.some(v => v.toLowerCase().includes(q))
        }
        return val?.toString().toLowerCase().includes(q)
      })
  )
})

const handleSelect = (id: string) => {
  emit('select', id)
}
</script>

<style scoped>
.resource-list {
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

.resource-item {
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

.resource-item:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.resource-item.active {
  background-color: var(--macos-accent-blue);
  color: white;
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
