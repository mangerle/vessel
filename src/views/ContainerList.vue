<script setup lang="ts">
import { onMounted, h } from 'vue'
import { useContainerStore } from '../store/container'
import { NDataTable, NTag, NSpace, NButton, NCard, NText } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'

interface ContainerInfo {
  id: string
  name: string
  state: string
  image: string
}

const containerStore = useContainerStore()

const columns: DataTableColumns<ContainerInfo> = [
  {
    title: 'ID',
    key: 'id',
    render(row) {
      return row.id.substring(0, 12)
    }
  },
  {
    title: '名称',
    key: 'name'
  },
  {
    title: '镜像',
    key: 'image'
  },
  {
    title: '状态',
    key: 'state',
    render(row) {
      const type = row.state === 'running' ? 'success' : 'warning'
      return h(NTag, { type, bordered: false }, { default: () => row.state })
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
