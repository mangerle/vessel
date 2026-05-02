<script setup lang="ts">
import { onMounted, h, computed } from 'vue'
import { useNetworkStore } from '../store/network'
import { 
  NDataTable, NSpace, NButton, NCard, 
  NText, NPopconfirm, NTooltip, NIcon, NStatistic, NGrid, NGi
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { TrashOutline } from '@vicons/ionicons5'

/**
 * Docker 网络信息接口
 */
interface NetworkInfo {
  id: string
  name: string
  driver: string
  scope: string
  created: string
}

const networkStore = useNetworkStore()

// 统计网络总数
const totalCount = computed(() => networkStore.networks.length)

// 表格列定义
const columns: DataTableColumns<NetworkInfo> = [
  {
    title: 'ID',
    key: 'id',
    width: 120,
    render(row) {
      return row.id.substring(0, 12)
    }
  },
  {
    title: '名称',
    key: 'name',
    minWidth: 150
  },
  {
    title: '驱动',
    key: 'driver',
    width: 100
  },
  {
    title: '范围',
    key: 'scope',
    width: 100
  },
  {
    title: '创建时间',
    key: 'created',
    minWidth: 180,
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
        onPositiveClick: () => networkStore.removeNetwork(row.id)
      }, {
        trigger: () => h(NTooltip, {}, {
          trigger: () => h(NButton, {
            circle: true,
            quaternary: true,
            type: 'error'
          }, { icon: () => h(NIcon, null, { default: () => h(TrashOutline) }) }),
          default: () => '删除网络'
        }),
        default: () => '确定要删除此网络吗？'
      })
    }
  }
]

onMounted(() => {
  networkStore.fetchNetworks()
})
</script>

<template>
  <div class="network-list">
    <n-space vertical size="large">
      <n-grid :cols="4" :x-gap="12">
        <n-gi>
          <n-card>
            <n-statistic label="总网络数" :value="totalCount" />
          </n-card>
        </n-gi>
      </n-grid>

      <n-card title="Docker 网络列表">
        <template #header-extra>
          <n-space>
            <n-popconfirm @positive-click="networkStore.pruneNetworks">
              <template #trigger>
                <n-button type="warning" ghost>清理未使用</n-button>
              </template>
              确定要清理所有未使用的网络吗？
            </n-popconfirm>
            <n-button type="primary" :loading="networkStore.loading" @click="networkStore.fetchNetworks">
              刷新
            </n-button>
          </n-space>
        </template>
        
        <n-text v-if="networkStore.error" type="error" style="display: block; margin-bottom: 12px;">
          错误: {{ networkStore.error }}
        </n-text>

        <n-data-table
          :columns="columns"
          :data="networkStore.networks"
          :loading="networkStore.loading"
          :pagination="{ pageSize: 10 }"
        />
      </n-card>
    </n-space>
  </div>
</template>

<style scoped>
.network-list {
  padding: 24px;
}
</style>
