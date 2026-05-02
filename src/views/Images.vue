<script setup lang="ts">
import { onMounted, h, computed, ref, watch } from 'vue'
import { useImageStore, type ImageInfo } from '../store/image'
import { 
  NDataTable, NTag, NSpace, NButton, NCard, 
  NText, NPopconfirm, NTooltip, NIcon, NStatistic, NGrid, NGi,
  NInput, NInputGroup, NDrawer, NDrawerContent, NScrollbar, useMessage
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { 
  CloudDownloadOutline,
  TrashOutline,
  SearchOutline
} from '@vicons/ionicons5'

const imageStore = useImageStore()
const message = useMessage()
const pullImageName = ref('')
const showPullDrawer = ref(false)
const scrollbarRef = ref<any>(null)

const totalSize = computed(() => {
  const bytes = imageStore.images.reduce((sum, img) => sum + img.size, 0)
  return formatBytes(bytes)
})

const imageCount = computed(() => imageStore.images.length)

function formatBytes(bytes: number, decimals = 2) {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

function formatDate(timestamp: number) {
  return new Date(timestamp * 1000).toLocaleString()
}

const columns: DataTableColumns<ImageInfo> = [
  {
    title: '镜像 ID',
    key: 'id',
    width: 150,
    render(row) {
      return row.id.split(':')[1]?.substring(0, 12) || row.id.substring(0, 12)
    }
  },
  {
    title: '仓库:标签',
    key: 'tags',
    minWidth: 250,
    render(row) {
      if (!row.tags || row.tags.length === 0) {
        return h(NTag, { type: 'info', bordered: false }, { default: () => '<none>:<none>' })
      }
      return h(NSpace, { size: 'small' }, {
        default: () => row.tags.map(tag => h(NTag, { type: 'info', bordered: false }, { default: () => tag }))
      })
    }
  },
  {
    title: '大小',
    key: 'size',
    width: 120,
    render(row) {
      return formatBytes(row.size)
    }
  },
  {
    title: '创建时间',
    key: 'created',
    width: 200,
    render(row) {
      return formatDate(row.created)
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    render(row) {
      return h(NPopconfirm, {
        onPositiveClick: () => handleDelete(row.id)
      }, {
        trigger: () => h(NTooltip, {}, {
          trigger: () => h(NButton, {
            circle: true,
            quaternary: true,
            type: 'error'
          }, { icon: () => h(NIcon, null, { default: () => h(TrashOutline) }) }),
          default: () => '删除镜像'
        }),
        default: () => '确定要删除此镜像吗？'
      })
    }
  }
]

async function handleDelete(id: string) {
  try {
    await imageStore.removeImage(id)
    message.success('镜像已删除')
  } catch (err) {
    message.error('删除镜像失败: ' + err)
  }
}

async function handlePull() {
  if (!pullImageName.value) {
    message.warning('请输入镜像名称')
    return
  }
  
  showPullDrawer.value = true
  try {
    await imageStore.pullImage(pullImageName.value)
    pullImageName.value = ''
  } catch (err) {
    message.error('启动拉取任务失败: ' + err)
  }
}

// 自动滚动到日志底部
watch(() => imageStore.pullLogs.length, () => {
  if (scrollbarRef.value) {
    setTimeout(() => {
      scrollbarRef.value.scrollTo({ position: 'bottom', silent: true })
    }, 100)
  }
})

onMounted(() => {
  imageStore.fetchImages()
})
</script>

<template>
  <div class="images-list">
    <n-space vertical size="large">
      <n-grid :cols="4" :x-gap="12">
        <n-gi :span="1">
          <n-card>
            <n-statistic label="镜像数量" :value="imageCount" />
          </n-card>
        </n-gi>
        <n-gi :span="1">
          <n-card>
            <n-statistic label="总占用空间" :value="totalSize" />
          </n-card>
        </n-gi>
        <n-gi :span="2">
          <n-card>
            <n-input-group>
              <n-input v-model:value="pullImageName" placeholder="输入镜像名称 (如: nginx:latest)" @keyup.enter="handlePull">
                <template #prefix>
                  <n-icon :component="SearchOutline" />
                </template>
              </n-input>
              <n-button type="primary" @click="handlePull" :loading="imageStore.pulling">
                <template #icon>
                  <n-icon :component="CloudDownloadOutline" />
                </template>
                拉取镜像
              </n-button>
            </n-input-group>
          </n-card>
        </n-gi>
      </n-grid>

      <n-card title="本地镜像列表">
        <template #header-extra>
          <n-button type="primary" :loading="imageStore.loading" @click="imageStore.fetchImages">
            刷新
          </n-button>
        </template>
        
        <n-text v-if="imageStore.error" type="error" style="display: block; margin-bottom: 12px;">
          错误: {{ imageStore.error }}
        </n-text>

        <n-data-table
          :columns="columns"
          :data="imageStore.images"
          :loading="imageStore.loading"
          :pagination="{ pageSize: 10 }"
        />
      </n-card>
    </n-space>

    <n-drawer v-model:show="showPullDrawer" :width="600" placement="right">
      <n-drawer-content title="拉取镜像进度" closable>
        <n-scrollbar ref="scrollbarRef" style="max-height: 100%">
          <div class="log-container">
            <div v-for="(log, index) in imageStore.pullLogs" :key="index" class="log-line">
              <n-text depth="3" v-if="log.id" style="margin-right: 8px">[{{ log.id }}]</n-text>
              <n-text>{{ log.status }}</n-text>
              <n-text depth="3" v-if="log.progress" style="margin-left: 8px">{{ log.progress }}</n-text>
              <n-text v-if="log.stream">{{ log.stream }}</n-text>
            </div>
            <div v-if="!imageStore.pulling && imageStore.pullLogs.length > 0" class="log-finished">
              <n-text type="success">拉取任务已结束</n-text>
            </div>
          </div>
        </n-scrollbar>
      </n-drawer-content>
    </n-drawer>
  </div>
</template>

<style scoped>
.images-list {
  padding: 24px;
}

.log-container {
  background-color: #1e1e1e;
  padding: 12px;
  border-radius: 4px;
  font-family: monospace;
  min-height: 100%;
}

.log-line {
  margin-bottom: 4px;
  word-break: break-all;
  color: #d4d4d4;
}

.log-finished {
  margin-top: 12px;
  border-top: 1px solid #333;
  padding-top: 8px;
  text-align: center;
}
</style>
