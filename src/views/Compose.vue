<script setup lang="ts">
import {computed, nextTick, onMounted, onUnmounted, ref} from 'vue'
import {useComposeStore} from '../store/compose'
import {useContainerStore} from '../store/container'
import {invoke} from '@tauri-apps/api/core'
import {listen} from '@tauri-apps/api/event'
import {
  NButton,
  NButtonGroup,
  NDescriptions,
  NDescriptionsItem,
  NDropdown,
  NGi,
  NGrid,
  NInput,
  NModal,
  NScrollbar,
  NSpace,
  NTag,
  NText,
  useMessage
} from 'naive-ui'
import {Terminal} from '@xterm/xterm'
import {FitAddon} from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import VChart from 'vue-echarts'
import {use} from 'echarts/core'
import {LineChart} from 'echarts/charts'
import {GridComponent, LegendComponent, TooltipComponent} from 'echarts/components'
import {CanvasRenderer} from 'echarts/renderers'
import ComposeProjectList from '../components/compose/ComposeProjectList.vue'
import ContainerDetail from '../components/compose/ContainerDetail.vue'
import {useContextMenu} from '../hooks/useContextMenu'

use([LineChart, GridComponent, TooltipComponent, LegendComponent, CanvasRenderer])

const composeStore = useComposeStore()
const containerStore = useContainerStore()
const message = useMessage()

// --- Selection State ---
const selectedId = ref<string | null>(null)
const selectedType = ref<'project' | 'container' | null>(null)
const selectedProject = ref<any>(null)
const containerDetails = ref<any>(null)
const loadingDetails = ref(false)

const onSelect = async (id: string) => {
  selectedId.value = id
  if (id.startsWith('project:')) {
    selectedType.value = 'project'
    const projectName = id.split(':')[1]
    selectedProject.value = composeStore.projects.find(p => p.name === projectName)
    if (selectedProject.value?.config_file) {
      await composeStore.fetchComposeFile(selectedProject.value.config_file)
    }
  } else {
    selectedType.value = 'container'
    await fetchDetails(id)
  }
}

// --- Context Menu ---
const {
  showDropdown: showMenu,
  x,
  y,
  currentOptions: menuOptions,
  currentTarget,
  handleContextMenu,
  onClickOutside: closeMenu
} = useContextMenu()

const handleMenuSelect = (key: string) => {
  const target = currentTarget.value
  closeMenu()

  const targetId = target?.id || (selectedType.value === 'container' ? selectedId.value : null)

  if (targetId) {
    // Handle Container Actions
    if (key === 'start') handleStart(targetId)
    else if (key === 'stop') handleStop(targetId)
    else if (key === 'restart') handleRestart(targetId)
    else if (key === 'delete') handleDelete(targetId)
    else if (key === 'terminal' || key === 'terminal_user') openTerminal(targetId)
    else if (key === 'terminal_root') openTerminal(targetId, 'root')
    else if (key === 'logs') {
      selectedId.value = targetId
      selectedType.value = 'container'
      fetchDetails(targetId)
    }
  }

  if (key === 'up' || key === 'down' || key === 'restart_project' || key === 'edit') {
    const project = (target && !target.id) ? target : selectedProject.value
    if (!project) return

    if (key === 'up') handleProjectUp(project)
    else if (key === 'down') handleProjectDown(project)
    else if (key === 'restart_project') handleProjectRestart(project)
    else if (key === 'edit') handleProjectEdit(project)
  }
}

const handleStart = async (id?: string) => {
  const targetId = typeof id === 'string' ? id : selectedId.value
  if (!targetId || (typeof id !== 'string' && selectedType.value !== 'container')) return
  try {
    await containerStore.startContainer(targetId)
    message.success('已启动容器')
    await composeStore.fetchProjects() // Refresh projects to update counts
    if (selectedId.value === targetId) await fetchDetails(targetId)
  } catch (e: any) {
    message.error('启动失败: ' + e)
  }
}

const handleStop = async (id?: string) => {
  const targetId = typeof id === 'string' ? id : selectedId.value
  if (!targetId || (typeof id !== 'string' && selectedType.value !== 'container')) return
  try {
    await containerStore.stopContainer(targetId)
    message.success('已停止容器')
    await composeStore.fetchProjects()
    if (selectedId.value === targetId) await fetchDetails(targetId)
  } catch (e: any) {
    message.error('停止失败: ' + e)
  }
}

const handleRestart = async (id?: string) => {
  const targetId = typeof id === 'string' ? id : selectedId.value
  if (!targetId || (typeof id !== 'string' && selectedType.value !== 'container')) return
  try {
    await containerStore.restartContainer(targetId)
    message.success('已重启容器')
    await composeStore.fetchProjects()
    if (selectedId.value === targetId) await fetchDetails(targetId)
  } catch (e: any) {
    message.error('重启失败: ' + e)
  }
}

const handleDelete = async (id?: string) => {
  const targetId = typeof id === 'string' ? id : selectedId.value
  if (!targetId || (typeof id !== 'string' && selectedType.value !== 'container')) return
  try {
    await containerStore.removeContainer(targetId)
    message.success('容器已删除')
    if (selectedId.value === targetId) {
      selectedId.value = null
      selectedType.value = null
    }
    await composeStore.fetchProjects()
  } catch (e: any) {
    message.error('删除失败: ' + e)
  }
}

const handleProjectUp = async (project?: any) => {
  const p = project?.working_dir ? project : selectedProject.value
  if (!p?.working_dir) return
  try {
    await composeStore.runComposeCommand(p.working_dir, ['up', '-d'])
    message.success('已发送 Compose Up 指令')
  } catch (e: any) {
    message.error('操作失败: ' + e)
  }
}

const handleProjectDown = async (project?: any) => {
  const p = project?.working_dir ? project : selectedProject.value
  if (!p?.working_dir) return
  try {
    await composeStore.runComposeCommand(p.working_dir, ['down'])
    message.success('已发送 Compose Down 指令')
  } catch (e: any) {
    message.error('操作失败: ' + e)
  }
}

const handleProjectRestart = async (project?: any) => {
  const p = project?.working_dir ? project : selectedProject.value
  if (!p?.working_dir) return
  try {
    await composeStore.runComposeCommand(p.working_dir, ['restart'])
    message.success('已发送 Compose Restart 指令')
  } catch (e: any) {
    message.error('操作失败: ' + e)
  }
}

const handleProjectEdit = async (project?: any) => {
  const p = project?.config_file ? project : selectedProject.value
  if (!p?.config_file) {
    message.warning('无法找到项目的配置文件路径')
    return
  }
  try {
    await composeStore.fetchComposeFile(p.config_file)
    message.success('已加载配置文件')
  } catch (e: any) {
    message.error('加载失败: ' + e)
  }
}

const handleSaveConfig = async () => {
  if (!selectedProject.value?.config_file) return
  try {
    await composeStore.saveComposeFile(selectedProject.value.config_file, composeStore.currentProjectFile)
    message.success('配置已保存')
  } catch (e: any) {
    message.error('保存失败: ' + e)
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

const handleTerminal = () => {
  if (selectedType.value === 'container' && selectedId.value) {
    openTerminal(selectedId.value)
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
      term.focus()
      
      term.onData(async (data) => {
        if (!terminalExecId) return
        const encoder = new TextEncoder()
        const bytes = Array.from(encoder.encode(data))
        try {
          await invoke('write_to_terminal', { execId: terminalExecId, data: bytes })
        } catch (e) {
          console.error('写入终端失败', e)
        }
      })

      term.onResize(async (size) => {
        if (!terminalExecId) return
        try {
          await invoke('resize_container_terminal', {
            execId: terminalExecId,
            height: size.rows,
            width: size.cols
          })
        } catch (e) {
          console.error('调整终端大小失败', e)
        }
      })

      try {
        terminalExecId = await invoke('create_container_terminal', {id, user})
        termUnlisten = await listen(`container-terminal-stdout-${terminalExecId}`, (event: any) => {
          const arr = new Uint8Array(event.payload)
          const str = new TextDecoder().decode(arr)
          term?.write(str)
        })

        // 发送一个回车以触发提示符显示（以防初始输出被错过）
        setTimeout(async () => {
          if (terminalExecId) {
            const encoder = new TextEncoder()
            const bytes = Array.from(encoder.encode('\n'))
            await invoke('write_to_terminal', { execId: terminalExecId, data: bytes })
          }
          fitAddon?.fit()
        }, 300)
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
          :containers="containerStore.containers"
          :projects="composeStore.projects"
          :selected-id="selectedId"
        @select="onSelect" 
        @contextmenu="handleContextMenu"
      />
    </div>
    <div class="detail-column floating-card">
      <ContainerDetail
          v-if="selectedType === 'container'"
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

      <div v-else-if="selectedType === 'project'" class="project-workspace">
        <div class="workspace-header">
          <div class="project-info">
            <h2 class="name">{{ selectedProject?.name }}</h2>
            <n-space>
              <n-tag :type="selectedProject?.status === 'running' ? 'success' : 'default'" round size="small">
                {{ selectedProject?.status === 'running' ? '运行中' : '已停止' }}
              </n-tag>
              <span class="stats">{{ selectedProject?.running_count }} / {{
                  selectedProject?.container_count
                }} 容器运行中</span>
            </n-space>
          </div>
          <div class="actions">
            <n-button-group round size="small">
              <n-button :loading="composeStore.executing" type="primary" @click="handleProjectUp">启动 (Up)</n-button>
              <n-button :loading="composeStore.executing" @click="handleProjectDown">停止 (Down)</n-button>
              <n-button :loading="composeStore.executing" @click="handleProjectRestart">重启</n-button>
              <n-button @click="handleSaveConfig">保存配置</n-button>
            </n-button-group>
          </div>
        </div>

        <div class="workspace-content">
          <div class="editor-container">
            <div class="editor-header">
              <span>docker-compose.yml</span>
              <span class="path">{{ selectedProject?.config_file }}</span>
            </div>
            <n-input
                v-model:value="composeStore.currentProjectFile"
                :autosize="{ minRows: 10 }"
                class="yaml-editor"
                placeholder="YAML 内容..."
                type="textarea"
            />
          </div>
          <div class="console-panel">
            <div class="console-header">执行输出</div>
            <n-scrollbar class="console-body">
              <div v-for="(line, idx) in composeStore.commandOutput" :key="idx" class="console-line">
                {{ line }}
              </div>
              <div v-if="composeStore.commandOutput.length === 0" class="empty-console">等待执行命令...</div>
            </n-scrollbar>
          </div>
        </div>
      </div>

      <div v-else class="empty-state">
        <n-text depth="3">请选择一个项目或容器以查看详情</n-text>
      </div>
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
  height: calc(100vh - 40px);
}

.list-column {
  width: 260px;
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
  height: calc(100vh - 180px);
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

.empty-state {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}

/* Project Workspace Styles */
.project-workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.workspace-header {
  padding: 16px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 0.5px solid var(--macos-border-color);
}

.workspace-header .name {
  margin: 0 0 4px 0;
  font-size: 18px;
  font-weight: 700;
}

.workspace-header .stats {
  font-size: 11px;
  color: #86868b;
}

.workspace-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  border-bottom: 0.5px solid var(--macos-border-color);
}

.editor-header {
  padding: 8px 16px;
  background: #f8f8f8;
  font-size: 12px;
  display: flex;
  justify-content: space-between;
  border-bottom: 0.5px solid var(--macos-border-color);
}

.editor-header .path {
  color: #86868b;
  font-family: monospace;
}

.yaml-editor {
  flex: 1;
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
}

.console-panel {
  height: 200px;
  background: #1e1e1e;
  display: flex;
  flex-direction: column;
}

.console-header {
  padding: 4px 16px;
  background: #333;
  color: #eee;
  font-size: 11px;
}

.console-body {
  flex: 1;
  padding: 8px 16px;
}

.console-line {
  color: #d4d4d4;
  font-family: monospace;
  font-size: 12px;
  white-space: pre-wrap;
  margin-bottom: 2px;
}

.empty-console {
  color: #666;
  font-style: italic;
  font-size: 12px;
  margin-top: 10px;
}
</style>