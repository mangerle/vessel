<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, h, nextTick } from 'vue'
import { useComposeStore } from '../store/compose'
import { useContainerStore } from '../store/container'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { 
  NLayout, NLayoutSider, NLayoutContent, NTree, NDropdown,
  NTabs, NTabPane, NButton, NSpace, NDescriptions, NDescriptionsItem,
  NSpin, useMessage, NModal, NCard, NText, NGrid, NGi, NScrollbar
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

const composeStore = useComposeStore()
const containerStore = useContainerStore()
const message = useMessage()

// --- Tree Data Construction ---
const treeData = computed(() => {
  return composeStore.projects.map(project => {
    const projectContainers = containerStore.containers.filter(c => c.compose_project === project.name)
    return {
      key: `project-${project.name}`,
      label: project.name,
      isProject: true,
      isRunning: project.running_count > 0,
      children: projectContainers.map(c => ({
        key: c.id,
        label: c.name,
        isProject: false,
        isRunning: c.state === 'running',
        container: c
      }))
    }
  })
})

const renderPrefix = ({ option }: { option: any }) => {
  return h('div', {
    style: {
      width: '8px',
      height: '8px',
      borderRadius: '50%',
      backgroundColor: option.isRunning ? '#18a058' : '#d9d9d9',
      display: 'inline-block',
      marginRight: '8px'
    }
  })
}

// --- Selection & Details ---
const selectedContainerId = ref<string | null>(null)
const containerDetails = ref<any>(null)
const loadingDetails = ref(false)

const handleSelect = async (_keys: string[], options: any[]) => {
  if (options.length > 0 && !options[0].isProject) {
    const id = options[0].key
    selectedContainerId.value = id
    await fetchDetails(id)
  } else {
    selectedContainerId.value = null
    containerDetails.value = null
    if (logsUnlisten) logsUnlisten()
    if (statsUnlisten) statsUnlisten()
  }
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

// --- Context Menu ---
const showDropdown = ref(false)
const dropdownX = ref(0)
const dropdownY = ref(0)
const rightClickedContainer = ref<any>(null)

const handleContextMenu = (e: MouseEvent, option: any) => {
  if (option.isProject) return
  e.preventDefault()
  showDropdown.value = false
  nextTick(() => {
    showDropdown.value = true
    dropdownX.value = e.clientX
    dropdownY.value = e.clientY
    rightClickedContainer.value = option.container
  })
}

const nodeProps = ({ option }: { option: any }) => {
  return {
    onContextmenu(e: MouseEvent) {
      handleContextMenu(e, option)
    }
  }
}

const onClickOutside = () => {
  showDropdown.value = false
}

const contextMenuOptions = [
  { label: '重启容器', key: 'restart' },
  { label: '暂停/停止容器', key: 'stop' },
  { label: '复制容器ID', key: 'copy_id' },
  { label: '复制镜像ID', key: 'copy_image_id' },
  { 
    label: '创建终端', 
    key: 'terminal',
    children: [
      { label: '作为容器用户', key: 'term_user' },
      { label: '作为 Root 用户', key: 'term_root' }
    ]
  },
  { label: '删除', key: 'delete' }
]

const handleMenuSelect = async (key: string) => {
  showDropdown.value = false
  const c = rightClickedContainer.value
  if (!c) return
  
  try {
    switch (key) {
      case 'restart':
        await containerStore.restartContainer(c.id)
        message.success('已发送重启指令')
        break
      case 'stop':
        await containerStore.stopContainer(c.id)
        message.success('已发送停止指令')
        break
      case 'copy_id':
        await navigator.clipboard.writeText(c.id)
        message.success('已复制容器ID')
        break
      case 'copy_image_id':
        await navigator.clipboard.writeText(c.image)
        message.success('已复制镜像')
        break
      case 'term_user':
        openTerminal(c.id, undefined)
        break
      case 'term_root':
        openTerminal(c.id, 'root')
        break
      case 'delete':
        await containerStore.removeContainer(c.id)
        message.success('已发送删除指令')
        break
    }
    await loadData()
  } catch (e: any) {
    message.error('操作失败: ' + e)
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
  <n-layout has-sider style="height: 100vh; margin: -24px;">
    <!-- 左侧 Compose 列表 -->
    <n-layout-sider bordered width="320" style="padding: 16px;">
      <n-space vertical size="large">
        <n-space justify="space-between" align="center">
          <n-text strong>Compose 项目与容器</n-text>
          <n-button size="small" @click="loadData">刷新</n-button>
        </n-space>
        <n-tree
          block-line
          :data="treeData"
          :render-prefix="renderPrefix"
          expand-on-click
          selectable
          @update:selected-keys="handleSelect"
          :node-props="nodeProps"
          style="max-height: calc(100vh - 100px); overflow-y: auto;"
        />
      </n-space>
    </n-layout-sider>

    <!-- 右侧详情区 -->
    <n-layout-content style="padding: 24px; background: #fafafa;">
      <div v-if="loadingDetails" style="display: flex; justify-content: center; margin-top: 100px;">
        <n-spin size="large" />
      </div>
      <div v-else-if="!selectedContainerId" style="display: flex; justify-content: center; margin-top: 100px;">
        <n-text depth="3">请在左侧点击一个容器以查看详细信息</n-text>
      </div>
      <div v-else-if="containerDetails">
        
        <!-- 容器头部信息 -->
        <n-card style="margin-bottom: 16px;">
          <n-space justify="space-between" align="center">
            <n-space vertical :size="4">
              <n-text strong style="font-size: 20px;">{{ containerDetails.name }}</n-text>
              <n-text depth="3">
                ID: {{ containerDetails.id.substring(0, 12) }} | 镜像: {{ containerDetails.image }}
              </n-text>
            </n-space>
            <n-space>
              <n-button @click="containerStore.restartContainer(selectedContainerId)">重启</n-button>
              <n-button @click="containerStore.stopContainer(selectedContainerId)">停止</n-button>
              <n-button type="primary" @click="openTerminal(selectedContainerId)">终端</n-button>
            </n-space>
          </n-space>
        </n-card>

        <!-- 多功能标签页 -->
        <n-card>
          <n-tabs type="line" animated>
            <n-tab-pane name="logs" tab="日志">
              <n-scrollbar ref="logScrollRef" style="height: calc(100vh - 350px); background: #1e1e1e; border-radius: 4px; padding: 12px;">
                <div style="color: #d4d4d4; font-family: monospace; white-space: pre-wrap; word-break: break-all;">
                  <div v-for="(log, idx) in logsList" :key="idx">{{ log }}</div>
                </div>
              </n-scrollbar>
            </n-tab-pane>
            
            <n-tab-pane name="settings" tab="设置">
              <n-scrollbar style="height: calc(100vh - 350px);">
                <n-descriptions bordered :column="1" size="small">
                  <n-descriptions-item label="环境变量">
                    <n-space vertical :size="4">
                      <n-text v-for="env in containerDetails.env" :key="env" style="font-family: monospace;">{{ env }}</n-text>
                    </n-space>
                  </n-descriptions-item>
                  <n-descriptions-item label="端口映射">
                    <n-space vertical :size="4">
                      <n-text v-for="p in containerDetails.ports" :key="p.private_port">
                        {{ p.public_port ? `${p.ip || '0.0.0.0'}:${p.public_port} -> ` : '' }}{{ p.private_port }}/{{ p.type_ }}
                      </n-text>
                    </n-space>
                  </n-descriptions-item>
                  <n-descriptions-item label="数据卷">
                    <n-space vertical :size="4">
                      <n-text v-for="m in containerDetails.mounts" :key="m.destination">
                        {{ m.source }} -> {{ m.destination }} ({{ m.mode }})
                      </n-text>
                    </n-space>
                  </n-descriptions-item>
                </n-descriptions>
              </n-scrollbar>
            </n-tab-pane>
            
            <n-tab-pane name="dashboard" tab="仪表盘">
              <n-scrollbar style="height: calc(100vh - 350px);">
                <n-grid :cols="2" :x-gap="12" :y-gap="12">
                  <n-gi>
                    <n-card title="CPU 使用率" size="small" bordered>
                      <v-chart class="chart" :option="cpuOption" autoresize style="height: 250px" />
                    </n-card>
                  </n-gi>
                  <n-gi>
                    <n-card title="内存使用量" size="small" bordered>
                      <v-chart class="chart" :option="memOption" autoresize style="height: 250px" />
                    </n-card>
                  </n-gi>
                  <n-gi>
                    <n-card title="网络 I/O" size="small" bordered>
                      <v-chart class="chart" :option="netOption" autoresize style="height: 250px" />
                    </n-card>
                  </n-gi>
                  <n-gi>
                    <n-card title="磁盘 I/O" size="small" bordered>
                      <v-chart class="chart" :option="ioOption" autoresize style="height: 250px" />
                    </n-card>
                  </n-gi>
                </n-grid>
              </n-scrollbar>
            </n-tab-pane>
          </n-tabs>
        </n-card>
      </div>
    </n-layout-content>
  </n-layout>

  <!-- 上下文菜单 -->
  <n-dropdown
    placement="bottom-start"
    trigger="manual"
    :x="dropdownX"
    :y="dropdownY"
    :options="contextMenuOptions"
    :show="showDropdown"
    :on-clickoutside="onClickOutside"
    @select="handleMenuSelect"
  />

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
</template>

<style scoped>
/* Ensure echarts resize works correctly */
.chart {
  width: 100%;
}
</style>