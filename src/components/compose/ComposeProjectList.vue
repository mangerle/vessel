<template>
  <div class="compose-project-list" @contextmenu="$emit('contextmenu', $event, 'global')">
    <div class="search-header">
      <n-input v-model:value="searchQuery" clearable placeholder="搜索项目或容器..." round size="small">
        <template #prefix>
          <n-icon><SearchOutline /></n-icon>
        </template>
      </n-input>
    </div>
    <n-scrollbar class="list-container">
      <div v-for="project in filteredTree" :key="project.name" class="project-group">
        <div
            :class="{ active: selectedId === 'project:' + project.name }"
            class="project-header"
            @click="handleSelect('project:' + project.name)"
            @contextmenu.stop="$emit('contextmenu', $event, 'project', project)"
        >
          <n-icon :class="{ expanded: expandedProjects.includes(project.name) }" class="chevron"
                  @click.stop="toggleProject(project.name)">
            <ChevronForwardOutline/>
          </n-icon>
          <n-icon class="folder-icon">
            <CubeOutline/>
          </n-icon>
          <div class="project-info">
            <span class="name">{{ project.name }}</span>
            <span class="count">{{ project.running_count }}/{{ project.container_count }}</span>
          </div>
        </div>

        <div v-if="expandedProjects.includes(project.name)" class="container-list">
          <div
              v-for="container in project.containers"
              :key="container.id"
              :class="{ active: selectedId === container.id }"
              class="container-item"
              @click="handleSelect(container.id)"
              @contextmenu.stop="$emit('contextmenu', $event, 'container', container)"
          >
            <div :style="{ backgroundColor: container.state === 'running' ? 'var(--macos-success-green)' : 'var(--macos-border-color)' }"
                 class="status-indicator"></div>
            <div class="item-body">
              <div class="item-name">{{ container.name }}</div>
              <div class="item-sub">{{ container.image }}</div>
            </div>
          </div>
        </div>
      </div>
    </n-scrollbar>
  </div>
</template>

<script setup lang="ts">
import {computed, ref} from 'vue'
import {NIcon, NInput, NScrollbar} from 'naive-ui'
import {ChevronForwardOutline, CubeOutline, SearchOutline} from '@vicons/ionicons5'

const props = defineProps<{
  projects: any[],
  containers: any[],
  selectedId: string | null
}>()

const emit = defineEmits(['select', 'contextmenu'])

const searchQuery = ref('')
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

const filteredTree = computed(() => {
  if (!searchQuery.value) return tree.value
  const q = searchQuery.value.toLowerCase()
  return tree.value.filter(p =>
      p.name.toLowerCase().includes(q) ||
      p.containers.some((c: any) => c.name.toLowerCase().includes(q) || c.image.toLowerCase().includes(q))
  )
})

const handleSelect = (id: string) => {
  emit('select', id)
}

// 默认展开所有项目
if (expandedProjects.value.length === 0 && props.projects.length > 0) {
  expandedProjects.value = props.projects.map(p => p.name)
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

.project-header {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  cursor: pointer;
  color: #1d1d1f;
  transition: background-color 0.2s;
}

.project-header:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.project-header.active {
  background-color: rgba(0, 122, 255, 0.1);
}

.chevron {
  font-size: 14px;
  margin-right: 4px;
  transition: transform 0.2s;
  color: #86868b;
}

.chevron.expanded {
  transform: rotate(90deg);
}

.folder-icon {
  font-size: 18px;
  margin-right: 8px;
  color: var(--macos-accent-blue);
}

.project-info {
  display: flex;
  justify-content: space-between;
  flex: 1;
  font-weight: 600;
  font-size: 13px;
}

.project-info .count {
  font-weight: normal;
  color: #86868b;
  font-size: 11px;
}

.container-list {
  padding-left: 20px;
}
.container-item {
  height: 54px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  cursor: pointer;
  transition: background-color 0.2s;
  position: relative;
  border-radius: 6px;
  margin: 2px 8px;
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
  height: 32px;
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
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.item-sub {
  font-size: 11px;
  opacity: 0.7;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.active .item-sub {
  color: white;
}
</style>
