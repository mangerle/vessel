<script setup lang="ts">
import { onMounted, h, computed } from 'vue'
import { useComposeStore } from '../store/compose'
import { 
  NDataTable, NTag, NSpace, NCard, 
  NText, NStatistic, NGrid, NGi, NButton
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'

/**
 * Docker Compose 项目接口
 */
interface ComposeProject {
  name: string
  container_count: number
  running_count: number
  status: string
}

const composeStore = useComposeStore()

// 统计项目数量
const totalProjects = computed(() => composeStore.projects.length)
// 活跃项目数量（有容器在运行）
const activeProjects = computed(() => composeStore.projects.filter(p => p.running_count > 0).length)

// 表格列定义
const columns: DataTableColumns<ComposeProject> = [
  {
    title: '项目名称',
    key: 'name',
    minWidth: 200
  },
  {
    title: '容器数量 (运行/总数)',
    key: 'containers',
    render(row) {
      return `${row.running_count} / ${row.container_count}`
    }
  },
  {
    title: '状态',
    key: 'status',
    render(row) {
      const type = row.running_count > 0 ? 'success' : 'warning'
      return h(NTag, { type, bordered: false }, { default: () => row.status })
    }
  }
]

onMounted(() => {
  composeStore.fetchProjects()
})
</script>

<template>
  <div class="compose-list">
    <n-space vertical size="large">
      <n-grid :cols="4" :x-gap="12">
        <n-gi>
          <n-card>
            <n-statistic label="活跃项目" :value="activeProjects">
              <template #suffix>
                / {{ totalProjects }}
              </template>
            </n-statistic>
          </n-card>
        </n-gi>
      </n-grid>

      <n-card title="Docker Compose 项目">
        <template #header-extra>
          <n-button type="primary" :loading="composeStore.loading" @click="composeStore.fetchProjects">
            刷新
          </n-button>
        </template>
        
        <n-text v-if="composeStore.error" type="error" style="display: block; margin-bottom: 12px;">
          错误: {{ composeStore.error }}
        </n-text>

        <n-data-table
          :columns="columns"
          :data="composeStore.projects"
          :loading="composeStore.loading"
          :pagination="{ pageSize: 10 }"
        />
      </n-card>
    </n-space>
  </div>
</template>

<style scoped>
.compose-list {
  padding: 24px;
}
</style>
