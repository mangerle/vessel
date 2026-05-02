<script setup lang="ts">
import { onMounted, h, computed } from 'vue'
import { useVolumeStore } from '../store/volume'
import { 
  NDataTable, NSpace, NButton, NCard, 
  NText, NPopconfirm, NTooltip, NIcon, NStatistic, NGrid, NGi
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { TrashOutline } from '@vicons/ionicons5'

/**
 * Docker 数据卷接口
 */
interface VolumeInfo {
  name: string
  driver: string
  mountpoint: string
  created: string
}

const volumeStore = useVolumeStore()

// 统计卷总数
const totalCount = computed(() => volumeStore.volumes.length)

// 表格列定义
const columns: DataTableColumns<VolumeInfo> = [
  {
    title: '名称',
    key: 'name',
    minWidth: 200
  },
  {
    title: '驱动',
    key: 'driver',
    width: 100
  },
  {
    title: '挂载点',
    key: 'mountpoint',
    minWidth: 250
  },
  {
    title: '创建时间',
    key: 'created',
    width: 180,
    render(row) {
      return row.created ? new Date(row.created).toLocaleString() : '未知'
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    render(row) {
      return h(NPopconfirm, {
        onPositiveClick: () => volumeStore.removeVolume(row.name)
      }, {
        trigger: () => h(NTooltip, {}, {
          trigger: () => h(NButton, {
            circle: true,
            quaternary: true,
            type: 'error'
          }, { icon: () => h(NIcon, null, { default: () => h(TrashOutline) }) }),
          default: () => '删除卷'
        }),
        default: () => '确定要删除此卷吗？'
      })
    }
  }
]

onMounted(() => {
  volumeStore.fetchVolumes()
})
</script>

<template>
  <div class="volume-list">
    <n-space vertical size="large">
      <n-grid :cols="4" :x-gap="12">
        <n-gi>
          <n-card>
            <n-statistic label="总数据卷" :value="totalCount" />
          </n-card>
        </n-gi>
      </n-grid>

      <n-card title="Docker 数据卷列表">
        <template #header-extra>
          <n-space>
            <n-popconfirm @positive-click="volumeStore.pruneVolumes">
              <template #trigger>
                <n-button type="warning" ghost>清理未使用</n-button>
              </template>
              确定要清理所有未使用的卷吗？
            </n-popconfirm>
            <n-button type="primary" :loading="volumeStore.loading" @click="volumeStore.fetchVolumes">
              刷新
            </n-button>
          </n-space>
        </template>
        
        <n-text v-if="volumeStore.error" type="error" style="display: block; margin-bottom: 12px;">
          错误: {{ volumeStore.error }}
        </n-text>

        <n-data-table
          :columns="columns"
          :data="volumeStore.volumes"
          :loading="volumeStore.loading"
          :pagination="{ pageSize: 10 }"
        />
      </n-card>
    </n-space>
  </div>
</template>

<style scoped>
.volume-list {
  padding: 24px;
}
</style>
