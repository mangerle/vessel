<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { 
  NPageHeader, NCard, NGrid, NGi,
  NSpace, NBreadcrumb, NBreadcrumbItem
} from 'naive-ui'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent
} from 'echarts/components'

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent
])

const route = useRoute()
const router = useRouter()
const containerId = route.params.id as string

const cpuData = ref<any[]>([])
const memData = ref<any[]>([])
const timestamps = ref<string[]>([])

const cpuOption = ref({
  title: { text: 'CPU 使用率 (%)' },
  tooltip: { trigger: 'axis' },
  xAxis: { type: 'category', data: timestamps.value },
  yAxis: { type: 'value' },
  series: [{
    name: 'CPU',
    type: 'line',
    data: cpuData.value,
    smooth: true,
    showSymbol: false,
    areaStyle: {}
  }]
})

const memOption = ref({
  title: { text: '内存使用 (MB)' },
  tooltip: { trigger: 'axis' },
  xAxis: { type: 'category', data: timestamps.value },
  yAxis: { type: 'value' },
  series: [{
    name: 'Memory',
    type: 'line',
    data: memData.value,
    smooth: true,
    showSymbol: false,
    areaStyle: { color: '#82ca9d' },
    itemStyle: { color: '#82ca9d' }
  }]
})

let unlisten: (() => void) | null = null

const calculateCpuPercent = (stats: any) => {
  const cpuDelta = stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage
  const systemDelta = stats.cpu_stats.system_cpu_usage - stats.precpu_stats.system_cpu_usage
  const onlineCpus = stats.cpu_stats.online_cpus || stats.cpu_stats.cpu_usage.percpu_usage?.length || 1
  
  if (systemDelta > 0 && cpuDelta > 0) {
    return (cpuDelta / systemDelta) * onlineCpus * 100
  }
  return 0
}

onMounted(async () => {
  // 启动后端监控流
  await invoke('stream_container_stats', { id: containerId })

  // 监听监控数据
  unlisten = await listen(`container-stats-${containerId}`, (event: any) => {
    const stats = event.payload
    const now = new Date().toLocaleTimeString()
    
    // 计算 CPU
    const cpuPercent = calculateCpuPercent(stats)
    // 计算内存 (Bytes -> MB)
    const memUsage = stats.memory_stats.usage / 1024 / 1024

    timestamps.value.push(now)
    cpuData.value.push(cpuPercent.toFixed(2))
    memData.value.push(memUsage.toFixed(2))

    // 保持最近 20 条数据
    if (timestamps.value.length > 20) {
      timestamps.value.shift()
      cpuData.value.shift()
      memData.value.shift()
    }

    // 手动触发响应式更新，并确保数据数组被替换以触发 ECharts 内部监听
    cpuOption.value = { 
      ...cpuOption.value,
      xAxis: { ...cpuOption.value.xAxis, data: [...timestamps.value] },
      series: [{ ...cpuOption.value.series[0], data: [...cpuData.value] }]
    }
    memOption.value = { 
      ...memOption.value,
      xAxis: { ...memOption.value.xAxis, data: [...timestamps.value] },
      series: [{ ...memOption.value.series[0], data: [...memData.value] }]
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
  invoke('close_container_stats', { id: containerId }).catch(() => {})
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
      </n-grid>
    </n-space>
  </div>
</template>

<style scoped>
.container-detail {
  padding: 24px;
}
.chart {
  height: 400px;
}
</style>
