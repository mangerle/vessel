<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { containerApi } from '../api/container'
import { useContainerStats } from '../hooks/useContainerStats'
import {
  NPageHeader, NCard, NGrid, NGi,
  NSpace, NBreadcrumb, NBreadcrumbItem
} from 'naive-ui'
import VChart from 'vue-echarts'
import '../utils/chartRegistry'

const route = useRoute()
const router = useRouter()
const containerId = route.params.id as string

// 复用统一的 stats composable，避免与 useContainerStats / Containers.vue 路径
// 的解析逻辑（CPU 占用、内存换算、Rx/Tx、blkio）重复实现。
const {
  cpuOption,
  memOption,
  netOption,
  ioOption,
  startStatsStream,
  stopStatsStream
} = useContainerStats()

onMounted(async () => {
  await startStatsStream(containerId)
})

onUnmounted(() => {
  stopStatsStream()
  containerApi.closeStats(containerId).catch(() => {})
})
</script>

<template>
  <div class="container-detail">
    <n-space vertical size="large">
      <n-page-header @back="router.push({ name: 'containers' })">
        <template #header>
          <n-breadcrumb>
            <n-breadcrumb-item @click="router.push({ name: 'containers' })">容器</n-breadcrumb-item>
            <n-breadcrumb-item>详情</n-breadcrumb-item>
          </n-breadcrumb>
        </template>
        <template #title>
          容器详情: {{ containerId.substring(0, 12) }}
        </template>
      </n-page-header>

      <n-grid :cols="2" :x-gap="12" :y-gap="12">
        <n-gi>
          <n-card>
            <v-chart class="chart" :option="cpuOption" autoresize />
          </n-card>
        </n-gi>
        <n-gi>
          <n-card>
            <v-chart class="chart" :option="memOption" autoresize />
          </n-card>
        </n-gi>
        <n-gi>
          <n-card>
            <v-chart class="chart" :option="netOption" autoresize />
          </n-card>
        </n-gi>
        <n-gi>
          <n-card>
            <v-chart class="chart" :option="ioOption" autoresize />
          </n-card>
        </n-gi>
      </n-grid>
    </n-space>
  </div>
</template>

<style scoped>
.container-detail {
  padding: 24px;
}
.chart {
  height: 320px;
}
</style>
