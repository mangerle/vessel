<script setup lang="ts">
import { onMounted, h, computed } from 'vue'
import { useContainerStore } from '../store/container'
import { useRouter } from 'vue-router'
import { 
  NDataTable, NTag, NSpace, NButton, NCard, 
  NText, NPopconfirm, NTooltip, NIcon, NStatistic, NGrid, NGi
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { 
  PlayOutline, 
  PauseOutline, 
  RefreshOutline, 
  TrashOutline,
  InformationCircleOutline
} from '@vicons/ionicons5'

interface ContainerInfo {
  id: string
  name: string
  state: string
  image: string
}

const containerStore = useContainerStore()
const router = useRouter()

const runningCount = computed(() => containerStore.containers.filter(c => c.state === 'running').length)
const totalCount = computed(() => containerStore.containers.length)

const columns: DataTableColumns<ContainerInfo> = [
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
    title: '镜像',
    key: 'image',
    minWidth: 200
  },
  {
    title: '状态',
    key: 'state',
    width: 100,
    render(row) {
      const type = row.state === 'running' ? 'success' : 'warning'
      return h(NTag, { type, bordered: false }, { default: () => row.state })
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 280,
    render(row) {
      return h(NSpace, {}, {
        default: () => [
          // 启动/停止按钮
          row.state === 'running' 
            ? h(NTooltip, {}, {
                trigger: () => h(NButton, {
                  circle: true,
                  quaternary: true,
                  type: 'warning',
                  onClick: () => containerStore.stopContainer(row.id)
                }, { icon: () => h(NIcon, null, { default: () => h(PauseOutline) }) }),
                default: () => '停止'
              })
            : h(NTooltip, {}, {
                trigger: () => h(NButton, {
                  circle: true,
                  quaternary: true,
                  type: 'success',
                  onClick: () => containerStore.startContainer(row.id)
                }, { icon: () => h(NIcon, null, { default: () => h(PlayOutline) }) }),
                default: () => '启动'
              }),
          
          // 重启按钮
          h(NTooltip, {}, {
            trigger: () => h(NButton, {
              circle: true,
              quaternary: true,
              type: 'info',
              onClick: () => containerStore.restartContainer(row.id)
            }, { icon: () => h(NIcon, null, { default: () => h(RefreshOutline) }) }),
            default: () => '重启'
          }),

          // 详情按钮
          h(NTooltip, {}, {
            trigger: () => h(NButton, {
              circle: true,
              quaternary: true,
              onClick: () => router.push({ name: 'container-detail', params: { id: row.id } })
            }, { icon: () => h(NIcon, null, { default: () => h(InformationCircleOutline) }) }),
            default: () => '详情'
          }),

          // 删除按钮
          h(NPopconfirm, {
            onPositiveClick: () => containerStore.removeContainer(row.id)
          }, {
            trigger: () => h(NTooltip, {}, {
              trigger: () => h(NButton, {
                circle: true,
                quaternary: true,
                type: 'error'
              }, { icon: () => h(NIcon, null, { default: () => h(TrashOutline) }) }),
              default: () => '删除'
            }),
            default: () => '确定要删除此容器吗？'
          })
        ]
      })
    }
  }
]

onMounted(() => {
  containerStore.fetchContainers()
})
</script>

<template>
  <div class="container-list">
    <n-space vertical size="large">
      <n-grid :cols="4" :x-gap="12">
        <n-gi>
          <n-card>
            <n-statistic label="运行中" :value="runningCount">
              <template #suffix>
                / {{ totalCount }}
              </template>
            </n-statistic>
          </n-card>
        </n-gi>
      </n-grid>

      <n-card title="本地容器列表">
        <template #header-extra>
          <n-button type="primary" :loading="containerStore.loading" @click="containerStore.fetchContainers">
            刷新
          </n-button>
        </template>
        
        <n-text v-if="containerStore.error" type="error" style="display: block; margin-bottom: 12px;">
          错误: {{ containerStore.error }}
        </n-text>

        <n-data-table
          :columns="columns"
          :data="containerStore.containers"
          :loading="containerStore.loading"
          :pagination="{ pageSize: 10 }"
        />
      </n-card>
    </n-space>
  </div>
</template>

<style scoped>
.container-list {
  padding: 24px;
}
</style>
