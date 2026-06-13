import { shallowRef, computed, markRaw } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { containerApi } from '../api/container'
import { EVT } from '../api/events'
import type { ContainerStatsPayload } from '../api/types'

/**
 * 容器性能统计数据 composable
 * 封装 CPU/内存/网络/IO 的 ECharts 数据流订阅与图表配置
 *
 * 性能要点：
 * 1. 4 个数据数组合并为单个 shallowRef 整体替换，单次事件仅触发 1 次响应式更新
 * 2. 修复 P0-11：把 xAxis.data / series.data 预拆为单独数组，每次事件只 map 一次（而非 8 次）；
 *    commonChartOpts 用 markRaw 包裹避免响应式代理；computed 仅做对象组装
 */
export function useContainerStats() {
  // 修复 P0-11：把每个图表的 4 列数据预拆，事件回调一次性算好；
  // computed 不再在 hot path 反复 .map 同一个数组。
  interface PreparedSnapshot {
    cpuTimes: string[]
    cpuValues: number[]
    memTimes: string[]
    memValues: number[]
    netTimes: string[]
    netRx: number[]
    netTx: number[]
    ioTimes: string[]
    ioRead: number[]
    ioWrite: number[]
  }
  const empty = (): PreparedSnapshot => ({
    cpuTimes: [], cpuValues: [],
    memTimes: [], memValues: [],
    netTimes: [], netRx: [], netTx: [],
    ioTimes: [], ioRead: [], ioWrite: []
  })
  let statsUnlisten: UnlistenFn | null = null
  const stats = shallowRef<PreparedSnapshot>(empty())

  // --- ECharts 公共配置 ---
  // markRaw：避免 Vue 把这个对象代理化，computed 中 spread 时不会触发依赖收集
  const commonChartOpts = markRaw({
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
  })

  // --- 图表 computed 配置 ---
  // 修复 P0-11：直接复用 stats.value 中已切片好的数组，无 .map 调用
  const cpuOption = computed(() => ({
    ...commonChartOpts,
    xAxis: { ...commonChartOpts.xAxis, data: stats.value.cpuTimes },
    yAxis: { type: 'value', name: 'CPU %', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [{ data: stats.value.cpuValues, type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#10b981' } }]
  }))

  const memOption = computed(() => ({
    ...commonChartOpts,
    xAxis: { ...commonChartOpts.xAxis, data: stats.value.memTimes },
    yAxis: { type: 'value', name: 'MB', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [{ data: stats.value.memValues, type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#38bdf8' } }]
  }))

  const netOption = computed(() => ({
    ...commonChartOpts,
    legend: { data: ['Rx', 'Tx'], textStyle: { color: '#64748b', fontSize: 9 } },
    xAxis: { ...commonChartOpts.xAxis, data: stats.value.netTimes },
    yAxis: { type: 'value', name: 'KB/s', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [
      { name: 'Rx', data: stats.value.netRx, type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#f59e0b' } },
      { name: 'Tx', data: stats.value.netTx, type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#ef4444' } }
    ]
  }))

  const ioOption = computed(() => ({
    ...commonChartOpts,
    legend: { data: ['Read', 'Write'], textStyle: { color: '#64748b', fontSize: 9 } },
    xAxis: { ...commonChartOpts.xAxis, data: stats.value.ioTimes },
    yAxis: { type: 'value', name: 'KB/s', splitLine: { lineStyle: { color: 'rgba(255,255,255,0.02)' } } },
    series: [
      { name: 'Read', data: stats.value.ioRead, type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#a855f7' } },
      { name: 'Write', data: stats.value.ioWrite, type: 'line', smooth: true, showSymbol: false, itemStyle: { color: '#ec4899' } }
    ]
  }))

  /** 在数组尾部 append 一个值，并保留最近 N 个；新数组返回（shallowRef 整体替换） */
  const appendKeep = (arr: number[], v: number, keep: number): number[] => {
    if (arr.length < keep) return [...arr, v]
    const next = arr.slice(arr.length - keep + 1)
    next.push(v)
    return next
  }
  const appendKeepStr = (arr: string[], v: string, keep: number): string[] => {
    if (arr.length < keep) return [...arr, v]
    const next = arr.slice(arr.length - keep + 1)
    next.push(v)
    return next
  }
  const KEEP = 20

  // --- 启动统计数据流 ---
  const startStatsStream = async (id: string) => {
    if (statsUnlisten) {
      statsUnlisten()
      statsUnlisten = null
    }
    // 切换 selectedId 时清空旧数据
    stats.value = empty()

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
        for (const net of Object.values(payload.networks)) {
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

      const cpu2 = parseFloat(cpuPercent.toFixed(2))
      const mem2 = parseFloat(memUsage.toFixed(2))
      const rx2 = parseFloat((rx / 1024).toFixed(2))
      const tx2 = parseFloat((tx / 1024).toFixed(2))
      const read2 = parseFloat((read / 1024).toFixed(2))
      const write2 = parseFloat((write / 1024).toFixed(2))

      const prev = stats.value
      stats.value = {
        cpuTimes: appendKeepStr(prev.cpuTimes, time, KEEP),
        cpuValues: appendKeep(prev.cpuValues, cpu2, KEEP),
        memTimes: appendKeepStr(prev.memTimes, time, KEEP),
        memValues: appendKeep(prev.memValues, mem2, KEEP),
        netTimes: appendKeepStr(prev.netTimes, time, KEEP),
        netRx: appendKeep(prev.netRx, rx2, KEEP),
        netTx: appendKeep(prev.netTx, tx2, KEEP),
        ioTimes: appendKeepStr(prev.ioTimes, time, KEEP),
        ioRead: appendKeep(prev.ioRead, read2, KEEP),
        ioWrite: appendKeep(prev.ioWrite, write2, KEEP)
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
    stats.value = empty()
  }

  return {
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
