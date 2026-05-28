<template>
  <div class="simple-container-list" @contextmenu="$emit('contextmenu', $event, 'global')">
    <!-- 顶部 40px 工具栏 -->
    <div class="list-toolbar">
      <!-- 搜索框 -->
      <input 
        v-model="searchQuery" 
        class="search-input" 
        placeholder="输入名称/ID过滤..." 
      />

      <!-- 状态快筛 -->
      <div class="filter-dropdown-wrapper">
        <button class="toolbar-btn" @click.stop="showFilterMenu = !showFilterMenu">
          <n-icon :component="FilterOutline" />
          状态
          <n-icon :component="ChevronDownOutline" size="10" />
        </button>
        <transition name="fade-in">
          <div v-if="showFilterMenu" class="dropdown-popover filter-popover" @click.stop>
            <div class="popover-item" :class="{ active: statusFilter === 'all' }" @click="selectFilter('all')">全部</div>
            <div class="popover-item" :class="{ active: statusFilter === 'running' }" @click="selectFilter('running')">运行中</div>
            <div class="popover-item" :class="{ active: statusFilter === 'stopped' }" @click="selectFilter('stopped')">已停止</div>
          </div>
        </transition>
      </div>

      <!-- 批量操作 -->
      <div class="batch-dropdown-wrapper">
        <button 
          class="toolbar-btn batch-btn" 
          :disabled="selectedIds.length === 0" 
          @click.stop="selectedIds.length > 0 && (showBatchMenu = !showBatchMenu)"
        >
          <n-icon :component="LayersOutline" />
          批量 ({{ selectedIds.length }})
          <n-icon :component="ChevronDownOutline" size="10" />
        </button>
        <transition name="fade-in">
          <div v-if="showBatchMenu" class="dropdown-popover batch-popover" @click.stop>
            <div class="popover-item primary" @click="triggerBatch('start')">
              <n-icon :component="PlayOutline" />
              批量启动
            </div>
            <div class="popover-item warn" @click="triggerBatch('stop')">
              <n-icon :component="StopOutline" />
              批量停止
            </div>
            <div class="popover-item danger" @click="triggerBatch('delete')">
              <n-icon :component="TrashOutline" />
              批量删除
            </div>
          </div>
        </transition>
      </div>
    </div>

    <!-- 高密度列表 (每行 32px) -->
    <n-scrollbar class="list-container">
      <!-- 全选栏 -->
      <div class="list-header-row" v-if="filteredItems.length > 0">
        <label class="checkbox-container">
          <input 
            type="checkbox" 
            :checked="isAllSelected" 
            :indeterminate="isPartiallySelected"
            @change="toggleSelectAll" 
          />
          <span class="checkmark"></span>
        </label>
        <span class="header-label">全选 (当前显示 {{ filteredItems.length }} 个)</span>
      </div>

      <div
        v-for="item in filteredItems"
        :key="item.id"
        class="container-item"
        :class="{ 
          active: selectedId === item.id,
          'is-up': (item.state || item.status) === 'running',
          'is-down': (item.state || item.status) !== 'running'
        }"
        @click="handleSelect(item.id)"
        @contextmenu.stop="$emit('contextmenu', $event, 'container', item)"
      >
        <!-- 复选框 -->
        <label class="checkbox-container" @click.stop>
          <input 
            type="checkbox" 
            v-model="selectedIds" 
            :value="item.id" 
          />
          <span class="checkmark"></span>
        </label>

        <!-- 🟢/🔴 状态小圆点 -->
        <span class="status-dot" :class="item.state || item.status"></span>

        <!-- 容器名称 (11px 纯净文本) -->
        <span class="item-name" :title="item.name">{{ item.name }}</span>
      </div>

      <div v-if="filteredItems.length === 0" class="empty-list">
        无匹配的独立容器
      </div>
    </n-scrollbar>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { NScrollbar, NIcon } from 'naive-ui'
import { 
  FilterOutline, 
  LayersOutline, 
  PlayOutline, 
  StopOutline, 
  TrashOutline,
  ChevronDownOutline
} from '@vicons/ionicons5'

const props = defineProps<{
  items: any[]
  selectedId: string | null
}>()

const emit = defineEmits(['select', 'contextmenu', 'batch'])

const searchQuery = ref('')
const statusFilter = ref<'all' | 'running' | 'stopped'>('all')
const selectedIds = ref<string[]>([])

const showFilterMenu = ref(false)
const showBatchMenu = ref(false)

// 逆向过滤掉 compose 容器的逻辑，由父组件传来的 items 已经逆向过滤了
// 在这里仅进行前端的 search & status 快筛
const filteredItems = computed(() => {
  let list = props.items
  
  if (statusFilter.value === 'running') {
    list = list.filter(i => (i.state || i.status) === 'running')
  } else if (statusFilter.value === 'stopped') {
    list = list.filter(i => (i.state || i.status) !== 'running')
  }
  
  if (!searchQuery.value) return list
  const q = searchQuery.value.toLowerCase()
  return list.filter(i => i.name.toLowerCase().includes(q) || i.id.toLowerCase().includes(q))
})

const isAllSelected = computed(() => {
  return filteredItems.value.length > 0 && filteredItems.value.every(item => selectedIds.value.includes(item.id))
})

const isPartiallySelected = computed(() => {
  const selectedCount = filteredItems.value.filter(item => selectedIds.value.includes(item.id)).length
  return selectedCount > 0 && selectedCount < filteredItems.value.length
})

const toggleSelectAll = (e: Event) => {
  const checked = (e.target as HTMLInputElement).checked
  if (checked) {
    filteredItems.value.forEach(item => {
      if (!selectedIds.value.includes(item.id)) {
        selectedIds.value.push(item.id)
      }
    })
  } else {
    filteredItems.value.forEach(item => {
      const idx = selectedIds.value.indexOf(item.id)
      if (idx > -1) {
        selectedIds.value.splice(idx, 1)
      }
    })
  }
}

const selectFilter = (val: 'all' | 'running' | 'stopped') => {
  statusFilter.value = val
  showFilterMenu.value = false
}

const triggerBatch = (action: 'start' | 'stop' | 'delete') => {
  emit('batch', { action, ids: [...selectedIds.value] })
  selectedIds.value = [] // 执行后重置
  showBatchMenu.value = false
}

const handleSelect = (id: string) => {
  emit('select', id)
}

const closeMenus = () => {
  showFilterMenu.value = false
  showBatchMenu.value = false
}

onMounted(() => {
  document.addEventListener('click', closeMenus)
})

onUnmounted(() => {
  document.removeEventListener('click', closeMenus)
})
</script>

<style scoped>
.simple-container-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  user-select: none;
  background-color: var(--bg-sidebar);
}

/* 顶部高密工具栏 */
.list-toolbar {
  height: 40px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  padding: 0 8px;
  gap: 4px;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  min-width: 0;
  height: 24px;
  background-color: var(--bg-main);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-title);
  font-size: 10px;
  padding: 0 6px;
  outline: none;
}
.search-input::placeholder {
  color: var(--text-muted);
}

.filter-dropdown-wrapper,
.batch-dropdown-wrapper {
  position: relative;
.toolbar-btn {
  height: 24px;
  background-color: rgba(255,255,255,0.03);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-body);
  font-size: 10px;
  padding: 0 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
}
  white-space: nowrap;
  outline: none;
}
.toolbar-btn:hover {
  background-color: var(--bg-active);
  color: var(--text-title);
}
.toolbar-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.dropdown-popover {
  position: absolute;
  top: 28px;
  background-color: var(--bg-main);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 4px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  box-shadow: 0 8px 24px rgba(0,0,0,0.5);
  z-index: 1000;
}

.filter-popover {
  left: 0;
  width: 90px;
}

.batch-popover {
  right: 0;
  width: 100px;
}

.popover-item {
  height: 24px;
  display: flex;
  align-items: center;
  padding: 0 8px;
  font-size: 10px;
  border-radius: 3px;
  cursor: pointer;
  color: var(--text-body);
  transition: background-color 0.15s ease;
}

.popover-item:hover {
  background-color: var(--bg-active);
  color: var(--text-title);
}

.popover-item.active {
  background-color: rgba(255,255,255,0.06);
  font-weight: 600;
}

.popover-item.primary:hover { color: var(--brand-primary); }
.popover-item.warn:hover { color: var(--brand-warn); }
.popover-item.danger:hover { color: var(--brand-danger); }

.list-container {
  flex: 1;
}

/* 全选和每行 */
.list-header-row {
  height: 28px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
  background-color: rgba(255,255,255,0.01);
}

.header-label {
  font-size: 10px;
  color: var(--text-muted);
  font-weight: 700;
  margin-left: 8px;
}

.container-item {
  height: 32px;
  display: flex;
  align-items: center;
  margin: 2px 8px;
  padding: 0 8px;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-body);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 11px;
  background-color: rgba(128, 128, 128, 0.06); /* 默认淡色背景显出成行 */
}

.container-item:hover {
  background-color: rgba(128, 128, 128, 0.12); /* 悬浮时加重背景色 */
  color: var(--text-title);
}

.container-item.active {
  background-color: var(--bg-active) !important;
  color: var(--text-title);
  font-weight: 600;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

/* 圆点 */
.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 3px;
  margin-left: 8px;
  margin-right: 8px;
  background-color: #64748b;
}
.status-dot.running, .status-dot.up {
  background-color: #10b981;
}
.status-dot.exited, .status-dot.stopped {
  background-color: #64748b;
}

.item-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.empty-list {
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
  font-style: italic;
}

/* Custom Checkbox */
.checkbox-container {
  display: block;
  position: relative;
  width: 14px;
  height: 14px;
  cursor: pointer;
}

.checkbox-container input {
  position: absolute;
  opacity: 0;
  cursor: pointer;
  height: 0;
  width: 0;
}

.checkmark {
  position: absolute;
  top: 0;
  left: 0;
  height: 14px;
  width: 14px;
  background-color: rgba(255,255,255,0.05);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  transition: all 0.15s ease;
}

.checkbox-container:hover input ~ .checkmark {
  background-color: rgba(255,255,255,0.1);
  border-color: var(--brand-primary);
}

.checkbox-container input:checked ~ .checkmark {
  background-color: var(--brand-primary);
  border-color: var(--brand-primary);
}

.checkbox-container input:indeterminate ~ .checkmark {
  background-color: var(--brand-primary);
  border-color: var(--brand-primary);
}

.checkmark:after {
  content: "";
  position: absolute;
  display: none;
}

.checkbox-container input:checked ~ .checkmark:after {
  display: block;
}

.checkbox-container input:indeterminate ~ .checkmark:after {
  display: block;
  left: 3px;
  top: 6px;
  width: 8px;
  height: 2px;
  background: #000;
}

.checkbox-container .checkmark:after {
  left: 4.5px;
  top: 1.5px;
  width: 3px;
  height: 7px;
  border: solid #000;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.fade-in-enter-active,
.fade-in-leave-active {
  transition: opacity 0.12s ease;
}
.fade-in-enter-from,
.fade-in-leave-to {
  opacity: 0;
}
</style>
