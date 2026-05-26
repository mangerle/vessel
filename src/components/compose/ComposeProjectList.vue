<template>
  <div class="compose-project-list" @contextmenu="$emit('contextmenu', $event, 'global')">
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
          @contextmenu.stop="$emit('contextmenu', $event, 'project', project)"
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
            @contextmenu.stop="$emit('contextmenu', $event, 'container', container)"
          >
            <n-icon :component="SettingsOutline" class="node-icon service-icon" />
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
  SettingsOutline, 
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
}

.project-group {
  margin-top: 4px;
}

/* 节点样式 */
.project-node,
.container-node {
  height: 28px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  cursor: pointer;
  color: var(--text-body);
  transition: all 0.15s ease;
  font-size: 11px;
}

.project-node:hover,
.container-node:hover {
  background-color: rgba(255, 255, 255, 0.02);
  color: var(--text-title);
}

.project-node.active {
  background-color: var(--macos-accent-blue) !important;
  color: #fff;
  font-weight: 600;
}

.container-node.active.is-up {
  background-color: #10b981 !important;
  color: #fff;
  font-weight: 600;
}

.container-node.active.is-down {
  background-color: #64748b !important;
  color: #fff;
  font-weight: 600;
}

/* 树箭头 */
.tree-arrow {
  display: inline-block;
  font-size: 8px;
  width: 12px;
  color: var(--text-muted);
  transition: transform 0.12s ease;
  margin-right: 4px;
}

.tree-arrow.expanded {
  transform: rotate(90deg);
}

.node-icon {
  font-size: 12px;
  margin-right: 6px;
  display: flex;
  align-items: center;
}

.node-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.container-sub-tree {
  display: flex;
  flex-direction: column;
}

.container-node {
  padding-left: 20px;
}

.container-node.is-up .service-icon {
  color: #10b981;
  opacity: 1;
}

.container-node.is-down .service-icon {
  color: #64748b;
  opacity: 0.6;
}

.container-node.active .service-icon {
  color: #fff !important;
  opacity: 1;
}

.service-icon {
  font-size: 10px;
  transition: color 0.2s ease;
}
</style>
