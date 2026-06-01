import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

/**
 * 容器性能统计数据 composable
 * 封装 CPU/内存/网络/IO 的 ECharts 数据流订阅与图表配置
 */
export function useContainerStats() {
  // --- 统计数据响应式数组 ---
  let statsUnlisten: UnlistenFn | null = null
  const cpuData = ref<{ time: string; value: number }[]>([])
  const memData = ref<{ time: string; value: number }[]>([])
  const netData = ref<{ time: string; rx: number; tx: number }[]>([])
  const ioData = ref<{ time: string; read: number; write: number }[]>([])

  // --- ECharts 公共配置 ---
  const commonChartOpts = {
    backgroundColor: 'transparent',
    tooltip: { 
      trigger: 'axis',
      backgroundColor: '#070a10',
      borderColor: 'rgba(255,255,255,0.08)',
      textStyle: { color: '#cbd5e1', fontSize: 10 }
    },
    grid: { top: 35, bottom: 20, left: 45, right: 15 },
    xAxis: { 
      type: 'category', 
      axisLine: { lineStyle: { color: 'rgba(255,255,255,0.05)' } },
      axisLabel: { color: '#64748b', fontSize: 9 }
    }
  }

  // --- 图表 computed 配置 ---
  const cpuOption = computed(() => ({
    ...commonChartOpts,
    xAxis: { ...commonChartOpts.xAxis, data: cpuData.value.map(d => d.time) },
    yAxis: { type: 'value', name: 'CPU %', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [{ data: cpuData.value.map(d => d.value), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#10b981' } }]
  }))

  const memOption = computed(() => ({
    ...commonChartOpts,
    xAxis: { ...commonChartOpts.xAxis, data: memData.value.map(d => d.time) },
    yAxis: { type: 'value', name: 'MB', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [{ data: memData.value.map(d => d.value), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#38bdf8' } }]
  }))

  const netOption = computed(() => ({
    ...commonChartOpts,
    legend: { data: ['Rx', 'Tx'], textStyle: { color: '#64748b', fontSize: 9 } },
    xAxis: { ...commonChartOpts.xAxis, data: netData.value.map(d => d.time) },
    yAxis: { type: 'value', name: 'KB/s', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [
      { name: 'Rx', data: netData.value.map(d => d.rx), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#f59e0b' } },
      { name: 'Tx', data: netData.value.map(d => d.tx), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#ef4444' } }
    ]
  }))

  const ioOption = computed(() => ({
    ...commonChartOpts,
    legend: { data: ['Read', 'Write'], textStyle: { color: '#64748b', fontSize: 9 } },
    xAxis: { ...commonChartOpts.xAxis, data: ioData.value.map(d => d.time) },
    yAxis: { type: 'value', name: 'KB/s', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [
      { name: 'Read', data: ioData.value.map(d => d.read), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#a855f7' } },
      { name: 'Write', data: ioData.value.map(d => d.write), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#ec4899' } }
    ]
  }))

  // --- 启动统计数据流 ---
  const startStatsStream = async (id: string) => {
    if (statsUnlisten) {
      statsUnlisten()
      statsUnlisten = null
    }
    cpuData.value = []
    memData.value = []
    netData.value = []
    ioData.value = []

    statsUnlisten = await listen(`container-stats-${id}`, (event: { payload: unknown }) => {
      const stats = event.payload as Record<string, any>
      const time = new Date().toLocaleTimeString()

      // CPU 使用率计算
      let cpuPercent = 0.0
      if (stats.cpu_stats && stats.precpu_stats) {
        const cpuDelta = stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage
        const systemDelta = stats.cpu_stats.system_cpu_usage - stats.precpu_stats.system_cpu_usage
        if (systemDelta > 0 && cpuDelta > 0) {
          cpuPercent = (cpuDelta / systemDelta) * (stats.cpu_stats.online_cpus || 1) * 100.0
        }
      }

      // 内存使用量
      let memUsage = 0
      if (stats.memory_stats) {
        memUsage = (stats.memory_stats.usage || 0) / (1024 * 1024)
      }

      // 网络流量
      let rx = 0; let tx = 0;
      if (stats.networks) {
        for (const key in stats.networks) {
          rx += stats.networks[key].rx_bytes || 0
          tx += stats.networks[key].tx_bytes || 0
        }
      }

      // 块设备 IO
      let read = 0; let write = 0;
      if (stats.blkio_stats && stats.blkio_stats.io_service_bytes_recursive) {
        for (const item of stats.blkio_stats.io_service_bytes_recursive) {
          if (item.op && item.op.toLowerCase() === 'read') read += item.value || 0
          if (item.op && item.op.toLowerCase() === 'write') write += item.value || 0
        }
      }

      cpuData.value.push({ time, value: parseFloat(cpuPercent.toFixed(2)) })
      memData.value.push({ time, value: parseFloat(memUsage.toFixed(2)) })
      netData.value.push({ time, rx: parseFloat((rx / 1024).toFixed(2)), tx: parseFloat((tx / 1024).toFixed(2)) })
      ioData.value.push({ time, read: parseFloat((read / 1024).toFixed(2)), write: parseFloat((write / 1024).toFixed(2)) })

      if (cpuData.value.length > 20) cpuData.value.shift()
      if (memData.value.length > 20) memData.value.shift()
      if (netData.value.length > 20) netData.value.shift()
      if (ioData.value.length > 20) ioData.value.shift()
    })

    try {
      await invoke('stream_container_stats', { id })
    } catch (e) {
      console.error('开始统计流失败', e)
    }
  }

  // --- 停止统计数据流 ---
  const stopStatsStream = () => {
    if (statsUnlisten) {
      statsUnlisten()
      statsUnlisten = null
    }
  }

  // --- 切换统计暂停/恢复 ---
  const handleToggleStats = (paused: boolean, currentId: string | null) => {
    if (paused) {
      stopStatsStream()
    } else {
      if (currentId) {
        startStatsStream(currentId)
      }
    }
  }

  // --- 重置统计数据 ---
  const handleResetStats = () => {
    cpuData.value = []
    memData.value = []
    netData.value = []
    ioData.value = []
  }

  return {
    cpuData,
    memData,
    netData,
    ioData,
    cpuOption,
    memOption,
    netOption,
    ioOption,
    startStatsStream,
    stopStatsStream,
    handleToggleStats,
    handleResetStats
  }
}
