<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick } from 'vue'
import { useComposeStore } from '../store/compose'
import { useContainerStore } from '../store/container'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { 
  NDescriptions, NDescriptionsItem,
  useMessage, NModal, NGrid, NGi, NScrollbar, NDropdown
} from 'naive-ui'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { LineChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

use([LineChart, GridComponent, TooltipComponent, LegendComponent, CanvasRenderer])

import ComposeProjectList from '../components/compose/ComposeProjectList.vue'
import ContainerDetail from '../components/compose/ContainerDetail.vue'
import { useContextMenu } from '../hooks/useContextMenu'

const composeStore = useComposeStore()
const containerStore = useContainerStore()
const message = useMessage()

// --- Context Menu ---
const {
  showDropdown: showMenu,
  x,
  y,
  currentOptions: menuOptions,
  handleContextMenu,
  onClickOutside: closeMenu
} = useContextMenu()

const handleMenuSelect = (key: string) => {
  closeMenu()
  if (!selectedContainerId.value) return

  switch (key) {
    case 'restart':
      handleRestart()
      break
    case 'stop':
      handleStop()
      break
    case 'terminal':
      handleTerminal()
      break
    case 'logs':
      // Switch to logs tab if possible, or just log for now
      message.info('查看日志: ' + selectedContainerId.value)
      break
  }
}

// --- Selection & Details ---
const selectedContainerId = ref<string | null>(null)
const containerDetails = ref<any>(null)
const loadingDetails = ref(false)

const onSelect = async (id: string) => {
  selectedContainerId.value = id
  await fetchDetails(id)
}

const fetchDetails = async (id: string) => {
  loadingDetails.value = true
  try {
    containerDetails.value = await invoke('inspect_container', { id })
    await startLogsStream(id)
    await startStatsStream(id)
  } catch (e: any) {
    message.error('获取详情失败: ' + e)
  } finally {
    loadingDetails.value = false
  }
}

const handleRestart = async () => {
  if (!selectedContainerId.value) return
  try {
    await containerStore.restartContainer(selectedContainerId.value)
    message.success('已发送重启指令')
  } catch (e: any) {
    message.error('操作失败: ' + e)
  }
}

const handleStop = async () => {
  if (!selectedContainerId.value) return
  try {
    await containerStore.stopContainer(selectedContainerId.value)
    message.success('已发送停止指令')
  } catch (e: any) {
    message.error('操作失败: ' + e)
  }
}

const handleTerminal = () => {
  if (selectedContainerId.value) {
    openTerminal(selectedContainerId.value)
  }
}

// --- Logs Stream ---
const logsList = ref<string[]>([])
let logsUnlisten: any = null
const logScrollRef = ref<any>(null)

const startLogsStream = async (id: string) => {
  if (logsUnlisten) {
    logsUnlisten()
    logsUnlisten = null
  }
  logsList.value = []
  
  logsUnlisten = await listen(`container-logs-${id}`, (event: any) => {
    logsList.value.push(event.payload)
    if (logsList.value.length > 1000) {
      logsList.value.shift()
    }
    nextTick(() => {
      if (logScrollRef.value) {
        logScrollRef.value.scrollTo({ position: 'bottom' })
      }
    })
  })
  
  try {
    await invoke('stream_container_logs', { id })
  } catch (e) {
    console.error('开始日志流失败', e)
  }
}

// --- Stats Stream (Dashboard) ---
let statsUnlisten: any = null
const cpuData = ref<{ time: string, value: number }[]>([])
const memData = ref<{ time: string, value: number }[]>([])
const netData = ref<{ time: string, rx: number, tx: number }[]>([])
const ioData = ref<{ time: string, read: number, write: number }[]>([])

const commonChartOpts = {
  tooltip: { trigger: 'axis' },
  xAxis: { type: 'category' }
}

const cpuOption = computed(() => ({
  ...commonChartOpts,
  xAxis: { ...commonChartOpts.xAxis, data: cpuData.value.map(d => d.time) },
  yAxis: { type: 'value', name: 'CPU %' },
  series: [{ data: cpuData.value.map(d => d.value), type: 'line', smooth: true }]
}))

const memOption = computed(() => ({
  ...commonChartOpts,
  xAxis: { ...commonChartOpts.xAxis, data: memData.value.map(d => d.time) },
  yAxis: { type: 'value', name: 'MB' },
  series: [{ data: memData.value.map(d => d.value), type: 'line', smooth: true }]
}))

const netOption = computed(() => ({
  ...commonChartOpts,
  legend: { data: ['Rx (KB)', 'Tx (KB)'] },
  xAxis: { ...commonChartOpts.xAxis, data: netData.value.map(d => d.time) },
  yAxis: { type: 'value', name: 'KB/s' },
  series: [
    { name: 'Rx (KB)', data: netData.value.map(d => d.rx), type: 'line', smooth: true },
    { name: 'Tx (KB)', data: netData.value.map(d => d.tx), type: 'line', smooth: true }
  ]
}))

const ioOption = computed(() => ({
  ...commonChartOpts,
  legend: { data: ['Read (KB)', 'Write (KB)'] },
  xAxis: { ...commonChartOpts.xAxis, data: ioData.value.map(d => d.time) },
  yAxis: { type: 'value', name: 'KB/s' },
  series: [
    { name: 'Read (KB)', data: ioData.value.map(d => d.read), type: 'line', smooth: true },
    { name: 'Write (KB)', data: ioData.value.map(d => d.write), type: 'line', smooth: true }
  ]
}))

const startStatsStream = async (id: string) => {
  if (statsUnlisten) {
    statsUnlisten()
    statsUnlisten = null
  }
  cpuData.value = []
  memData.value = []
  netData.value = []
  ioData.value = []
  
  statsUnlisten = await listen(`container-stats-${id}`, (event: any) => {
    const stats = event.payload
    const time = new Date().toLocaleTimeString()
    
    // CPU
    let cpuPercent = 0.0
    if (stats.cpu_stats && stats.precpu_stats) {
      const cpuDelta = stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage
      const systemDelta = stats.cpu_stats.system_cpu_usage - stats.precpu_stats.system_cpu_usage
      if (systemDelta > 0 && cpuDelta > 0) {
        cpuPercent = (cpuDelta / systemDelta) * (stats.cpu_stats.online_cpus || 1) * 100.0
      }
    }
    
    // Memory
    let memUsage = 0
    if (stats.memory_stats) {
      memUsage = (stats.memory_stats.usage || 0) / (1024 * 1024)
    }
    
    // Network
    let rx = 0; let tx = 0;
    if (stats.networks) {
      for (const key in stats.networks) {
        rx += stats.networks[key].rx_bytes || 0
        tx += stats.networks[key].tx_bytes || 0
      }
    }
    
    // IO
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

// --- Terminal Modal ---
const showTerminal = ref(false)
const terminalRef = ref<HTMLElement | null>(null)
let terminalExecId = ''
let term: Terminal | null = null
let fitAddon: FitAddon | null = null
let termUnlisten: any = null

const openTerminal = async (id: string | null, user?: string) => {
  if (!id) return
  showTerminal.value = true
  nextTick(async () => {
    if (terminalRef.value) {
      term = new Terminal({ cursorBlink: true, theme: { background: '#1e1e1e' } })
      fitAddon = new FitAddon()
      term.loadAddon(fitAddon)
      term.open(terminalRef.value)
      fitAddon.fit()
      
      term.onData(async (data) => {
        const encoder = new TextEncoder()
        const bytes = Array.from(encoder.encode(data))
        try {
          await invoke('write_to_terminal', { exec_id: terminalExecId, data: bytes })
        } catch (e) {
          console.error('写入终端失败', e)
        }
      })

      try {
        terminalExecId = await invoke('create_container_terminal', { id, user })
        termUnlisten = await listen(`container-terminal-stdout-${terminalExecId}`, (event: any) => {
          const arr = new Uint8Array(event.payload)
          const str = new TextDecoder().decode(arr)
          term?.write(str)
        })
      } catch (e: any) {
        term.write(`\r\nError: ${e}\r\n`)
      }
    }
  })
}

const closeTerminal = () => {
  showTerminal.value = false
  if (term) {
    term.dispose()
    term = null
  }
  if (termUnlisten) {
    termUnlisten()
    termUnlisten = null
  }
}

// --- Lifecycle ---
const loadData = async () => {
  await composeStore.fetchProjects()
  await containerStore.fetchContainers()
}

onMounted(() => {
  loadData()
})

onUnmounted(() => {
  if (logsUnlisten) logsUnlisten()
  if (statsUnlisten) statsUnlisten()
  closeTerminal()
})
</script>

<template>
  <div class="compose-view" @contextmenu="handleContextMenu($event, 'global')">
    <div class="list-column floating-card">
      <ComposeProjectList 
        :items="containerStore.containers" 
        :selected-id="selectedContainerId" 
        @select="onSelect" 
        @contextmenu="handleContextMenu"
      />
    </div>
    <div class="detail-column floating-card">
      <ContainerDetail 
        :container="containerDetails" 
        :loading="loadingDetails"
        @restart="handleRestart"
        @stop="handleStop"
        @terminal="handleTerminal"
      >
        <template #overview>
          <n-scrollbar class="tab-pane-content">
            <n-descriptions bordered :column="1" size="small" style="padding: 16px" v-if="containerDetails">
              <n-descriptions-item label="镜像">
                {{ containerDetails.image }}
              </n-descriptions-item>
              <n-descriptions-item label="端口映射">
                <div v-for="p in containerDetails.ports" :key="p.private_port">
                  {{ p.public_port ? `${p.ip || '0.0.0.0'}:${p.public_port} -> ` : '' }}{{ p.private_port }}/{{ p.type_ }}
                </div>
              </n-descriptions-item>
              <n-descriptions-item label="挂载卷">
                <div v-for="m in containerDetails.mounts" :key="m.destination">
                  {{ m.source }} -> {{ m.destination }} ({{ m.mode }})
                </div>
              </n-descriptions-item>
            </n-descriptions>
          </n-scrollbar>
        </template>
        <template #logs>
          <n-scrollbar ref="logScrollRef" class="tab-pane-content log-window">
            <div class="log-line" v-for="(log, idx) in logsList" :key="idx">{{ log }}</div>
          </n-scrollbar>
        </template>
        <template #stats>
          <n-scrollbar class="tab-pane-content dashboard-grid">
            <n-grid :cols="2" :x-gap="12" :y-gap="12" style="padding: 16px">
              <n-gi><v-chart class="chart" :option="cpuOption" autoresize /></n-gi>
              <n-gi><v-chart class="chart" :option="memOption" autoresize /></n-gi>
              <n-gi><v-chart class="chart" :option="netOption" autoresize /></n-gi>
              <n-gi><v-chart class="chart" :option="ioOption" autoresize /></n-gi>
            </n-grid>
          </n-scrollbar>
        </template>
        <template #settings>
          <n-scrollbar class="tab-pane-content">
            <n-descriptions bordered :column="1" size="small" style="padding: 16px" title="环境变量" v-if="containerDetails">
              <n-descriptions-item v-for="env in containerDetails.env" :key="env">
                <code style="font-size: 12px">{{ env }}</code>
              </n-descriptions-item>
            </n-descriptions>
          </n-scrollbar>
        </template>
      </ContainerDetail>
    </div>
  </div>

  <!-- 终端弹窗 -->
  <n-modal 
    v-model:show="showTerminal" 
    title="交互式终端" 
    preset="card" 
    style="width: 80vw; max-width: 1200px;" 
    @after-leave="closeTerminal"
  >
    <div ref="terminalRef" style="height: 60vh; width: 100%; background: #1e1e1e; padding: 8px; border-radius: 4px;"></div>
  </n-modal>

  <!-- 右键菜单 -->
  <n-dropdown
    placement="bottom-start"
    trigger="manual"
    :x="x"
    :y="y"
    :options="menuOptions"
    :show="showMenu"
    :on-clickoutside="closeMenu"
    @select="handleMenuSelect"
  />
</template>

<style scoped>
.compose-view {
  display: flex;
  gap: 16px;
  height: calc(100vh - 64px - 32px); /* 64px header, 32px total padding (16*2) */
}

.list-column {
  width: 320px;
  flex-shrink: 0;
}

.detail-column {
  flex: 1;
  min-width: 0;
}

.floating-card {
  background-color: var(--macos-card-bg-light);
  border-radius: var(--macos-radius);
  border: 1px solid var(--macos-border-color);
  box-shadow: var(--macos-shadow);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.tab-pane-content {
  height: calc(100vh - 64px - 32px - 140px);
}

.log-window {
  background-color: #1e1e1e;
  color: #d4d4d4;
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  padding: 12px;
}

.log-line {
  white-space: pre-wrap;
  word-break: break-all;
  margin-bottom: 2px;
}

.chart {
  height: 250px;
  width: 100%;
}
</style>