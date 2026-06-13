import { shallowRef, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { containerApi } from '../api/container'
import { EVT } from '../api/events'
import type { ContainerStatsPayload } from '../api/types'

/**
 * 容器性能统计数据 composable
 * 封装 CPU/内存/网络/IO 的 ECharts 数据流订阅与图表配置
 *
 * 性能要点：4 个数据数组合并为单个 shallowRef 整体替换，单次事件仅触发 1 次
 * 响应式更新（而非 4 次独立 ref mutation + 4 次 computed 重建）。
 */
export function useContainerStats() {
  // --- 统计数据响应式数组：合并为单 shallowRef，整体替换 1 次响应 ---
  interface StatsSnapshot {
    cpu: { time: string; value: number }[]
    mem: { time: string; value: number }[]
    net: { time: string; rx: number; tx: number }[]
    io: { time: string; read: number; write: number }[]
  }
  let statsUnlisten: UnlistenFn | null = null
  const stats = shallowRef<StatsSnapshot>({
    cpu: [],
    mem: [],
    net: [],
    io: []
  })
  // 计算属性侧单独暴露 4 个数组的派生，避免外部直接 mutation
  const cpuData = computed(() => stats.value.cpu)
  const memData = computed(() => stats.value.mem)
  const netData = computed(() => stats.value.net)
  const ioData = computed(() => stats.value.io)

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
    xAxis: { ...commonChartOpts.xAxis, data: stats.value.cpu.map(d => d.time) },
    yAxis: { type: 'value', name: 'CPU %', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [{ data: stats.value.cpu.map(d => d.value), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#10b981' } }]
  }))

  const memOption = computed(() => ({
    ...commonChartOpts,
    xAxis: { ...commonChartOpts.xAxis, data: stats.value.mem.map(d => d.time) },
    yAxis: { type: 'value', name: 'MB', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [{ data: stats.value.mem.map(d => d.value), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#38bdf8' } }]
  }))

  const netOption = computed(() => ({
    ...commonChartOpts,
    legend: { data: ['Rx', 'Tx'], textStyle: { color: '#64748b', fontSize: 9 } },
    xAxis: { ...commonChartOpts.xAxis, data: stats.value.net.map(d => d.time) },
    yAxis: { type: 'value', name: 'KB/s', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [
      { name: 'Rx', data: stats.value.net.map(d => d.rx), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#f59e0b' } },
      { name: 'Tx', data: stats.value.net.map(d => d.tx), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#ef4444' } }
    ]
  }))

  const ioOption = computed(() => ({
    ...commonChartOpts,
    legend: { data: ['Read', 'Write'], textStyle: { color: '#64748b', fontSize: 9 } },
    xAxis: { ...commonChartOpts.xAxis, data: stats.value.io.map(d => d.time) },
    yAxis: { type: 'value', name: 'KB/s', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [
      { name: 'Read', data: stats.value.io.map(d => d.read), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#a855f7' } },
      { name: 'Write', data: stats.value.io.map(d => d.write), type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#ec4899' } }
    ]
  }))

  // --- 启动统计数据流 ---
  const startStatsStream = async (id: string) => {
    if (statsUnlisten) {
      statsUnlisten()
      statsUnlisten = null
    }
    // 切换 selectedId 时清空旧数据
    stats.value = { cpu: [], mem: [], net: [], io: [] }

    statsUnlisten = await listen<ContainerStatsPayload>(EVT.containerStats(id), (event) => {
      const payload = event.payload
      const time = new Date().toLocaleTimeString()

      // CPU 使用率计算
      let cpuPercent = 0.0
      const cpuNow = payload.cpu_stats?.cpu_usage?.total_usage ?? 0
      const cpuPre = payload.precpu_stats?.cpu_usage?.total_usage ?? 0
      const sysNow = payload.cpu_stats?.system_cpu_usage ?? 0
      const sysPre = payload.precpu_stats?.system_cpu_usage ?? 0
      const cpuDelta = cpuNow - cpuPre
      const systemDelta = sysNow - sysPre
      if (systemDelta > 0 && cpuDelta > 0) {
        cpuPercent = (cpuDelta / systemDelta) * (payload.cpu_stats?.online_cpus ?? 1) * 100.0
      }

      // 内存使用量
      const memUsage = (payload.memory_stats?.usage ?? 0) / (1024 * 1024)

      // 网络流量
      let rx = 0
      let tx = 0
      if (payload.networks) {
        for (const key in payload.networks) {
          const net = payload.networks[key]
          rx += net.rx_bytes ?? 0
          tx += net.tx_bytes ?? 0
        }
      }

      // 块设备 IO
      let read = 0
      let write = 0
      for (const item of payload.blkio_stats?.io_service_bytes_recursive ?? []) {
        const op = item.op?.toLowerCase()
        if (op === 'read') read += item.value ?? 0
        else if (op === 'write') write += item.value ?? 0
      }

      // 整体替换 stats：1 次响应（4 个数组 spread + slice(-20)）
      const prev = stats.value
      stats.value = {
        cpu: [...prev.cpu, { time, value: parseFloat(cpuPercent.toFixed(2)) }].slice(-20),
        mem: [...prev.mem, { time, value: parseFloat(memUsage.toFixed(2)) }].slice(-20),
        net: [...prev.net, { time, rx: parseFloat((rx / 1024).toFixed(2)), tx: parseFloat((tx / 1024).toFixed(2)) }].slice(-20),
        io: [...prev.io, { time, read: parseFloat((read / 1024).toFixed(2)), write: parseFloat((write / 1024).toFixed(2)) }].slice(-20)
      }
    })

    try {
      await containerApi.streamStats(id)
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
    stats.value = { cpu: [], mem: [], net: [], io: [] }
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
