<template>
  <div class="compose-project-list" @contextmenu.prevent.stop="$emit('contextmenu', $event, 'global')">
    <!-- 顶部 40px 高度的一键导入按钮 -->
    <div class="import-header">
      <button class="import-btn" @click="handleImportProject">
        <n-icon :component="AddOutline" />
        导入现有项目
      </button>
    </div>

    <!-- 极简树结构列表 -->
    <n-scrollbar class="list-container">
      <div v-for="project in tree" :key="project.name" class="project-group">
        <!-- 项目节点 -->
        <div
          class="project-node"
          :class="{ active: selectedId === 'project:' + project.name }"
          @click="handleSelect('project:' + project.name)"
          @contextmenu.prevent.stop="$emit('contextmenu', $event, 'project', project)"
        >
          <span 
            class="tree-arrow" 
            :class="{ expanded: expandedProjects.includes(project.name) }"
            @click.stop="toggleProject(project.name)"
          >
            <n-icon :component="ChevronForwardOutline" />
          </span>
          <n-icon :component="CubeOutline" class="node-icon" />
          <span class="node-label">{{ project.name }}</span>
        </div>

        <!-- 子容器服务列表 -->
        <div v-if="expandedProjects.includes(project.name)" class="container-sub-tree">
          <div
            v-for="container in project.containers"
            :key="container.id"
            class="container-node"
            :class="{ 
              active: selectedId === container.id,
              'is-up': container.state === 'running',
              'is-down': container.state !== 'running'
            }"
            @click="handleSelect(container.id)"
            @contextmenu.prevent.stop="$emit('contextmenu', $event, 'container', container)"
          >
            <span class="status-dot" :class="container.state"></span>
            <span class="node-label">{{ container.name }}</span>
          </div>
        </div>
      </div>
    </n-scrollbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { NScrollbar, NIcon } from 'naive-ui'
import { 
  CubeOutline, 
  AddOutline,
  ChevronForwardOutline
} from '@vicons/ionicons5'

const props = defineProps<{
  projects: any[]
  containers: any[]
  selectedId: string | null
}>()

const emit = defineEmits(['select', 'contextmenu', 'import'])

const expandedProjects = ref<string[]>([])

const toggleProject = (name: string) => {
  const index = expandedProjects.value.indexOf(name)
  if (index > -1) {
    expandedProjects.value.splice(index, 1)
  } else {
    expandedProjects.value.push(name)
  }
}

const tree = computed(() => {
  return props.projects.map(p => ({
    ...p,
    containers: props.containers.filter(c => c.compose_project === p.name)
  }))
})

// 默认展开所有项目
watch(() => props.projects, (newProjects) => {
  if (expandedProjects.value.length === 0 && newProjects.length > 0) {
    expandedProjects.value = newProjects.map(p => p.name)
  }
}, { immediate: true })

const handleSelect = (id: string) => {
  emit('select', id)
}

const handleImportProject = () => {
  emit('import')
}
</script>

<style scoped>
.compose-project-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  user-select: none;
  background-color: var(--bg-sidebar);
}

.import-header {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.import-btn {
  width: 100%;
  height: 24px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-body);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.import-btn:hover {
  background-color: var(--bg-active);
  color: var(--text-title);
  border-color: var(--brand-primary);
}

.list-container {
  flex: 1;
  padding: 4px 0;
}

.project-group {
  margin-top: 6px;
}

/* 节点样式 */
.project-node,
.container-node {
  height: 32px;
  display: flex;
  align-items: center;
  margin: 2px 8px;
  padding: 0 8px;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-body);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 13px;
  background-color: rgba(128, 128, 128, 0.06); /* 默认淡色背景显出成行 */
}

.project-node:hover,
.container-node:hover {
  background-color: rgba(128, 128, 128, 0.12); /* 悬浮时加重背景色 */
  color: var(--text-title);
}

.project-node.active,
.container-node.active {
  background-color: var(--bg-active) !important;
  color: var(--text-title);
  font-weight: 600;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

/* 树箭头 */
.tree-arrow {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 9px;
  width: 14px;
  height: 14px;
  color: var(--text-muted);
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  margin-right: 4px;
  border-radius: 3px;
}

.tree-arrow:hover {
  background-color: var(--bg-hover);
  color: var(--text-title);
}

.tree-arrow.expanded {
  transform: rotate(90deg);
}

.node-icon {
  font-size: 13px;
  margin-right: 8px;
  display: flex;
  align-items: center;
  color: var(--text-muted);
}

.project-node.active .node-icon {
  color: var(--brand-primary);
}

.node-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  font-weight: 500;
}

/* 子树 */
.container-sub-tree {
  display: flex;
  flex-direction: column;
  margin-top: 2px;
  margin-bottom: 4px;
}

.container-node {
  padding-left: 36px;
}

/* 状态指示小圆点 */
.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  margin-right: 8px;
  background-color: #8e8e93; /* 经典的已停止灰色 */
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.status-dot.running,
.status-dot.up {
  background-color: #34c759; /* 经典的健康运行绿色 */
  box-shadow: 0 0 4px rgba(52, 199, 89, 0.6);
}

.status-dot.exited,
.status-dot.stopped,
.status-dot.created {
  background-color: #8e8e93; /* 经典的已停止灰色 */
}
</style>
