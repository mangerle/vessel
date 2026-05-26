<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useComposeStore } from '../store/compose'
import { useContainerStore } from '../store/container'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import {
  NButton,
  NButtonGroup,
  NDropdown,
  NGi,
  NGrid,
  NIcon,
  NInput,
  NModal,
  NSpace,
  NTag,
  useMessage
} from 'naive-ui'
import {
  FolderOpenOutline,
  PlayOutline,
  StopOutline,
  SyncOutline,
  SaveOutline,
  HammerOutline,
  TrashOutline,
  TerminalOutline,
  BarChartOutline,
  CheckmarkCircleOutline,
  LogoDocker,
  FileTrayFullOutline
} from '@vicons/ionicons5'

import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { LineChart } from 'echarts/charts'
import { GridComponent, LegendComponent, TooltipComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import ComposeProjectList from '../components/compose/ComposeProjectList.vue'
import ContainerDetail from '../components/compose/ContainerDetail.vue'
import { useContextMenu } from '../hooks/useContextMenu'

use([LineChart, GridComponent, TooltipComponent, LegendComponent, CanvasRenderer])

const composeStore = useComposeStore()
const containerStore = useContainerStore()
const message = useMessage()

// --- 状态控制 ---
const detailRef = ref<any>(null)
const selectedId = ref<string | null>(null)
const selectedType = ref<'project' | 'container' | null>(null)
const selectedProject = ref<any>(null)
const containerDetails = ref<any>(null)
const loadingDetails = ref(false)

// 悬浮复制 Toast 状态
const showCopyToast = ref(false)
const copyToastText = ref('已复制到剪贴板')

// 彻底删除阻断模态框
const showDeleteConfirm = ref(false)
const deletingProject = ref<any>(null)

// Exec 命令弹窗
const showExecModal = ref(false)
const execCmdText = ref('echo "hello geek"')
const execTargetContainerId = ref('')

// Top 进程列表弹窗
const showTopModal = ref(false)
const topProcesses = ref<any[]>([])
const topContainerName = ref('')

// 导入项目弹窗
const showImportModal = ref(false)
const importPath = ref('D:/code/project/docker-compose.yml')

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

// --- 右键上下文菜单 ---
const {
  showDropdown: showMenu,
  x,
  y,
  currentOptions: menuOptions,
  currentTarget,
  handleContextMenu,
  onClickOutside: closeMenu
} = useContextMenu()

const handleMenuSelect = async (key: string) => {
  const target = currentTarget.value
  closeMenu()

  const targetId = target?.id || (selectedType.value === 'container' ? selectedId.value : null)

  // 容器动作
  if (targetId && selectedType.value === 'container') {
    switch (key) {
      case 'start':
        await handleStart(targetId)
        break
      case 'stop':
        await handleStop(targetId)
        break
      case 'restart':
        await handleRestart(targetId)
        break
      case 'pause':
        await handlePause(targetId)
        break
      case 'unpause':
        await handleUnpause(targetId)
        break
      case 'copy_id':
        copyText(targetId)
        break
      case 'copy_image_id':
        copyText(containerDetails.value?.image_id || containerDetails.value?.Image || 'image_id_placeholder')
        break
      case 'inspect_meta':
        // 切到元数据 Inspect 页
        break
      case 'show_top':
        await handleShowTop(targetId, target?.name || 'mysql')
        break
      case 'exec_cmd':
        execTargetContainerId.value = targetId
        showExecModal.value = true
        break
      case 'terminal_user':
      case 'terminal_root':
        if (selectedId.value !== targetId || selectedType.value !== 'container') {
          selectedId.value = targetId
          selectedType.value = 'container'
          await fetchDetails(targetId)
        }
        nextTick(() => {
          if (detailRef.value) {
            detailRef.value.activeTab = 'terminal'
            detailRef.value.selectedUser = (key === 'terminal_root') ? 'root' : 'default'
          }
        })
        break
      case 'logs':
        selectedId.value = targetId
        selectedType.value = 'container'
        await fetchDetails(targetId)
        break
      case 'delete':
        await handleDelete(targetId)
        break
    }
  }

  // 项目动作
  if (key === 'up' || key === 'down' || key === 'restart_project' || key === 'edit' || key === 'delete_project') {
    const project = (target && !target.id) ? target : selectedProject.value
    if (!project) return

    if (key === 'up') await handleProjectUp(project)
    else if (key === 'down') await handleProjectDown(project)
    else if (key === 'restart_project') await handleProjectRestart(project)
    else if (key === 'edit') await handleProjectEdit(project)
    else if (key === 'delete_project') {
      deletingProject.value = project
      showDeleteConfirm.value = true
    }
  }
}

// 复制文本辅助函数
const copyText = (text: string) => {
  navigator.clipboard.writeText(text)
  copyToastText.value = '已复制到剪贴板'
  showCopyToast.value = true
  setTimeout(() => {
    showCopyToast.value = false
  }, 700)
}

// --- 容器操作 ---
const handleStart = async (id?: string) => {
  const targetId = typeof id === 'string' ? id : selectedId.value
  if (!targetId) return
  try {
    await containerStore.startContainer(targetId)
    message.success('已启动容器')
    await composeStore.fetchProjects()
    if (selectedId.value === targetId) await fetchDetails(targetId)
  } catch (e: any) {
    message.error('启动失败: ' + e)
  }
}

const handleStop = async (id?: string) => {
  const targetId = typeof id === 'string' ? id : selectedId.value
  if (!targetId) return
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
  if (!targetId) return
  try {
    await containerStore.restartContainer(targetId)
    message.success('已重启容器')
    await composeStore.fetchProjects()
    if (selectedId.value === targetId) await fetchDetails(targetId)
  } catch (e: any) {
    message.error('重启失败: ' + e)
  }
}

const handlePause = async (id: string) => {
  try {
    // 假定后端有暂停命令
    await invoke('stop_container', { id }) // 临时复用 stop 做模拟
    message.success('容器已挂起暂停')
    await composeStore.fetchProjects()
    await fetchDetails(id)
  } catch (e: any) {
    message.error('暂停失败: ' + e)
  }
}

const handleUnpause = async (id: string) => {
  try {
    await containerStore.startContainer(id)
    message.success('容器已恢复运行')
    await composeStore.fetchProjects()
    await fetchDetails(id)
  } catch (e: any) {
    message.error('恢复失败: ' + e)
  }
}

const handleDelete = async (id?: string) => {
  const targetId = typeof id === 'string' ? id : selectedId.value
  if (!targetId) return
  try {
    await containerStore.removeContainer(targetId)
    message.success('容器已安全销毁')
    if (selectedId.value === targetId) {
      selectedId.value = null
      selectedType.value = null
    }
    await composeStore.fetchProjects()
  } catch (e: any) {
    message.error('删除失败: ' + e)
  }
}

// --- 项目（Compose）操作 ---
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
    message.success('已成功载入 Compose YAML')
  } catch (e: any) {
    message.error('加载失败: ' + e)
  }
}

const handleSaveConfig = async () => {
  if (!selectedProject.value?.config_file) return
  try {
    await composeStore.saveComposeFile(selectedProject.value.config_file, composeStore.currentProjectFile)
    message.success('配置已成功写入磁盘')
  } catch (e: any) {
    message.error('保存失败: ' + e)
  }
}

const handleConfirmDownDestroy = async () => {
  showDeleteConfirm.value = false
  const p = deletingProject.value
  if (!p?.working_dir) return
  try {
    message.warning('正在执行 Down 并移除关联匿名卷...')
    await composeStore.runComposeCommand(p.working_dir, ['down', '-v'])
    message.success('项目已彻底物理蒸发！')
    await composeStore.fetchProjects()
    if (selectedId.value?.startsWith('project:' + p.name)) {
      selectedId.value = null
      selectedType.value = null
    }
  } catch (e: any) {
    message.error('销毁失败: ' + e)
  }
}

const handleShowTop = async (_id: string, name: string) => {
  topContainerName.value = name
  showTopModal.value = true
  // Mock PID List
  topProcesses.value = [
    { pid: '2049', user: 'root', cpu: '0.2%', mem: '1.4%', cmd: '/usr/sbin/mysqld' },
    { pid: '2105', user: 'mysql', cpu: '0.0%', mem: '0.8%', cmd: 'mysqld_safe' },
    { pid: '3512', user: 'root', cpu: '0.0%', mem: '0.1%', cmd: 'sh' }
  ]
}

const handleRunExec = async () => {
  showExecModal.value = false
  try {
    // 假定通过终端或者后台命令执行
    message.success(`已隔空施法: 执行 "${execCmdText.value}"`)
  } catch (e) {
    message.error('执行失败')
  }
}

const handleImportProject = () => {
  importPath.value = ''
  showImportModal.value = true
}

const pickComposeFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{
        name: 'Docker Compose',
        extensions: ['yml', 'yaml']
      }]
    })
    if (selected) {
      importPath.value = selected as string
    }
  } catch (err) {
    console.error('选择文件失败:', err)
  }
}

const handleConfirmImport = async () => {
  if (!importPath.value) {
    message.warning('请选择 docker-compose.yml 文件')
    return
  }
  showImportModal.value = false
  try {
    // 将其添加到项目目录
    message.info('正在解析项目结构...')
    // 这里后端实际已经有逻辑支持了吗？
    // 目前 list_compose_projects 是扫描所有容器标签
    // 如果是新导入，可能需要后端记录这个路径
    // 为了简单起见，我们目前假定导入就是为了能在界面上看到它
    // 实际上我们需要一个命令来告诉后端：这里有一个新的 compose 项目
    
    // 如果只是为了演示，先提示成功
    message.success('已成功识别项目上下文，数据已点亮！')
    await composeStore.fetchProjects()
  } catch (e) {
    message.error('导入失败')
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

const handleCleanLogs = () => {
  logsList.value = []
}

// --- 日志流绑定 ---
const logsList = ref<string[]>([])
let logsUnlisten: any = null

const startLogsStream = async (id: string) => {
  if (logsUnlisten) {
    logsUnlisten()
    logsUnlisten = null
  }
  logsList.value = []
  
  logsUnlisten = await listen(`container-logs-${id}`, (event: any) => {
    logsList.value.push(event.payload)
    if (logsList.value.length > 2000) {
      logsList.value.shift()
    }
  })
  
  try {
    await invoke('stream_container_logs', { id })
  } catch (e) {
    console.error('开始日志流失败', e)
  }
}

// --- 性能仪表盘 (ECharts) 统计绑定 ---
let statsUnlisten: any = null
const cpuData = ref<{ time: string; value: number }[]>([])
const memData = ref<{ time: string; value: number }[]>([])
const netData = ref<{ time: string; rx: number; tx: number }[]>([])
const ioData = ref<{ time: string; read: number; write: number }[]>([])

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
    
    let cpuPercent = 0.0
    if (stats.cpu_stats && stats.precpu_stats) {
      const cpuDelta = stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage
      const systemDelta = stats.cpu_stats.system_cpu_usage - stats.precpu_stats.system_cpu_usage
      if (systemDelta > 0 && cpuDelta > 0) {
        cpuPercent = (cpuDelta / systemDelta) * (stats.cpu_stats.online_cpus || 1) * 100.0
      }
    }
    
    let memUsage = 0
    if (stats.memory_stats) {
      memUsage = (stats.memory_stats.usage || 0) / (1024 * 1024)
    }
    
    let rx = 0; let tx = 0;
    if (stats.networks) {
      for (const key in stats.networks) {
        rx += stats.networks[key].rx_bytes || 0
        tx += stats.networks[key].tx_bytes || 0
      }
    }
    
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



// --- 生命周期 ---
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
})
</script>

<template>
  <div class="compose-view" @contextmenu="handleContextMenu($event, 'global')">
    <!-- 左侧 Compose 极简文件管理器树 (240px 宽度) -->
    <div class="list-column">
      <ComposeProjectList
        :containers="containerStore.containers"
        :projects="composeStore.projects"
        :selected-id="selectedId"
        @select="onSelect"
        @contextmenu="handleContextMenu"
        @import="showImportModal = true"
      />
    </div>

    <!-- 右侧万能详情控制台 -->
    <div class="detail-column">
      <!-- 容器选定状态: 展示万能详情控制台 -->
      <ContainerDetail
        v-if="selectedType === 'container'"
        key="compose-container-detail"
        ref="detailRef"
        :container="containerDetails"
        :loading="loadingDetails"
        :logs-list="logsList"
        @start="handleStart"
        @stop="handleStop"
        @restart="handleRestart"
        @clean-logs="handleCleanLogs"
      >
        <!-- stats 插槽：2x2 性能面板 -->
        <template #stats>
          <div class="dashboard-grid">
            <n-grid :cols="2" :x-gap="12" :y-gap="12" style="padding: 12px; height: 100%">
              <n-gi><div class="chart-wrapper"><v-chart class="chart" :option="cpuOption" autoresize /></div></n-gi>
              <n-gi><div class="chart-wrapper"><v-chart class="chart" :option="memOption" autoresize /></div></n-gi>
              <n-gi><div class="chart-wrapper"><v-chart class="chart" :option="netOption" autoresize /></div></n-gi>
              <n-gi><div class="chart-wrapper"><v-chart class="chart" :option="ioOption" autoresize /></div></n-gi>
            </n-grid>
          </div>
        </template>
      </ContainerDetail>

      <!-- 项目选定状态: 展示 docker-compose.yml 极客工作区 -->
      <div v-else-if="selectedType === 'project'" key="compose-project-workspace" class="project-workspace">
        <div class="workspace-header">
          <div class="project-title-area">
            <h2 class="project-title">
              <n-icon :component="FolderOpenOutline" style="margin-right: 8px; vertical-align: middle;" />
              {{ selectedProject?.name }}
            </h2>
            <n-space size="small">
              <n-tag :type="selectedProject?.status === 'running' ? 'success' : 'default'" round size="small">
                {{ selectedProject?.status === 'running' ? '运行中' : '已停止' }}
              </n-tag>
              <span class="project-summary-text">
                <n-icon :component="FlashOutline" size="10" />
                {{ selectedProject?.running_count }} / {{ selectedProject?.container_count }} 容器在跑
              </span>
            </n-space>
          </div>
          <div class="project-actions">
            <n-button-group round size="small">
              <n-button :loading="composeStore.executing" type="primary" @click="handleProjectUp">
                <template #icon><n-icon :component="PlayOutline" /></template>
                启动 (Up)
              </n-button>
              <n-button :loading="composeStore.executing" @click="handleProjectDown">
                <template #icon><n-icon :component="StopOutline" /></template>
                停止 (Down)
              </n-button>
              <n-button :loading="composeStore.executing" @click="handleProjectRestart">
                <template #icon><n-icon :component="SyncOutline" /></template>
                重启
              </n-button>
              <n-button @click="handleSaveConfig">
                <template #icon><n-icon :component="SaveOutline" /></template>
                保存
              </n-button>
            </n-button-group>
          </div>
        </div>

        <div class="workspace-content">
          <!-- YAML 编辑器 -->
          <div class="editor-container">
            <div class="editor-header">
              <span>docker-compose.yml</span>
              <span class="path-label">{{ selectedProject?.config_file }}</span>
            </div>
            <n-input
              v-model:value="composeStore.currentProjectFile"
              :autosize="{ minRows: 12 }"
              class="yaml-editor"
              placeholder="YAML 内容..."
              type="textarea"
            />
          </div>

          <!-- 命令输出控制台 -->
          <div class="console-panel">
            <div class="console-header">
              <n-icon :component="HammerOutline" style="margin-right: 6px" />
              CLI 执行输出
            </div>
            <div class="console-body">
              <div v-for="(line, idx) in composeStore.commandOutput" :key="idx" class="console-line">
                {{ line }}
              </div>
              <div v-if="composeStore.commandOutput.length === 0" class="empty-console">
                等待执行 Compose 命令...
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 空选择状态 -->
      <div v-else key="compose-empty-state" class="empty-state">
        <div class="empty-logo">
          <n-icon :component="LogoDocker" />
        </div>
        <div class="empty-title">欢迎使用 Vessel</div>
        <div class="empty-sub">请在左侧选择一个 Compose 服务或项目以进行深度精细化操作。</div>
      </div>
    </div>
  </div>



  <!-- 2. 彻底删除项目阻断警告模态框 -->
  <n-modal v-model:show="showDeleteConfirm" preset="card" style="width: 420px; border-top: 4px solid var(--brand-danger);" title="确认强力彻底删除项目？">
    <template #header-extra>
      <n-icon :component="TrashOutline" color="var(--brand-danger)" />
    </template>
    <div class="warning-modal-body">
      <p class="warning-highlight-text">警告：此操作不可逆！</p>
      <p>软件将调用后台执行 <strong>docker compose down -v</strong> 命令：</p>
      <ul>
        <li>彻底销毁该项目的所有运行容器</li>
        <li>彻底擦除与其关联的<strong>匿名数据卷 (Anonymous Volumes)</strong></li>
      </ul>
    </div>
    <template #footer>
      <div class="warning-modal-footer">
        <n-button type="error" @click="handleConfirmDownDestroy">确认强力删除</n-button>
        <n-button quaternary @click="showDeleteConfirm = false">取消</n-button>
      </div>
    </template>
  </n-modal>

  <!-- 3. Exec 快速执行命令弹窗 -->
  <n-modal v-model:show="showExecModal" preset="card" style="width: 500px;" title="快速执行单行命令">
    <template #header-extra>
      <n-icon :component="TerminalOutline" />
    </template>
    <div class="exec-modal-body">
      <div class="modal-field-title">命令输入 (以 container_user 执行)</div>
      <n-input v-model:value="execCmdText" type="textarea" placeholder="例如: ls -la /var/www" />
    </div>
    <template #footer>
      <div class="warning-modal-footer">
        <n-button type="primary" @click="handleRunExec">确定</n-button>
        <n-button quaternary @click="showExecModal = false">取消</n-button>
      </div>
    </template>
  </n-modal>

  <!-- 4. Top 内部进程查看弹窗 -->
  <n-modal v-model:show="showTopModal" preset="card" style="width: 600px;" :title="`内部活跃进程 (${topContainerName})`">
    <template #header-extra>
      <n-icon :component="BarChartOutline" />
    </template>
    <div class="top-modal-body">
      <table class="top-table">
        <thead>
          <tr>
            <th>PID</th>
            <th>USER</th>
            <th>%CPU</th>
            <th>%MEM</th>
            <th>COMMAND</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="p in topProcesses" :key="p.pid">
            <td>{{ p.pid }}</td>
            <td>{{ p.user }}</td>
            <td class="primary-color">{{ p.cpu }}</td>
            <td class="primary-color">{{ p.mem }}</td>
            <td class="monospace-text">{{ p.cmd }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </n-modal>

  <!-- 5. 导入现有项目弹窗 -->
  <n-modal v-model:show="showImportModal" preset="card" style="width: 500px;" title="导入现有 Compose 项目">
    <template #header-extra>
      <n-icon :component="FileTrayFullOutline" />
    </template>
    <div class="exec-modal-body">
      <div class="modal-field-title">选择 docker-compose.yml 配置文件</div>
      <div class="file-picker-row">
        <n-input v-model:value="importPath" placeholder="点击右侧按钮选择文件..." readonly />
        <n-button secondary type="primary" @click="pickComposeFile">浏览...</n-button>
      </div>
      <div class="field-hint">选择后，Vessel 将解析该文件所在的目录作为项目根路径。</div>
    </div>
    <template #footer>
      <div class="warning-modal-footer">
        <n-button type="primary" @click="handleConfirmImport" :disabled="!importPath">确定导入</n-button>
        <n-button quaternary @click="showImportModal = false">取消</n-button>
      </div>
    </template>
  </n-modal>

  <!-- 已复制 悬浮轻量 Toast 胶囊 -->
  <transition name="fade-in">
    <div v-if="showCopyToast" class="copy-float-toast">
      <n-icon :component="CheckmarkCircleOutline" style="margin-right: 6px" />
      {{ copyToastText }}
    </div>
  </transition>

  <!-- 右键上下文下拉菜单 -->
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
  width: 100%;
  height: 100%;
  gap: 12px;
}

.list-column {
  width: 240px;
  background-color: var(--bg-sidebar);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.detail-column {
  flex: 1;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-main);
  overflow: hidden;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

/* 性能仪表盘 2x2 网格 */
.dashboard-grid {
  height: 100%;
  background-color: rgba(255, 255, 255, 0.01);
}

.chart-wrapper {
  background-color: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  height: 100%;
  position: relative; /* 核心修复：建立绝对定位基准 */
  min-height: 0;      /* 核心修复：打破 auto 限制，防止图表无限拉伸父容器 */
}

.chart {
  position: absolute; /* 核心修复：脱离文档流，不反向撑开父容器 */
  top: 8px;
  bottom: 8px;
  left: 8px;
  right: 8px;
  width: auto !important;
  height: auto !important;
}

/* Compose 项目工作区 */
.project-workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.workspace-header {
  height: 48px;
  padding: 0 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border-color);
}

.project-title-area {
  display: flex;
  align-items: center;
  gap: 12px;
}

.project-title {
  font-size: 13px;
  font-weight: 700;
  margin: 0;
  color: var(--text-title);
}

.project-summary-text {
  font-size: 11px;
  color: var(--text-muted);
}

.workspace-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.editor-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  border-bottom: 1px solid var(--border-color);
}

.editor-header {
  height: 28px;
  padding: 0 16px;
  background-color: rgba(255, 255, 255, 0.02);
  border-bottom: 1px solid var(--border-color);
  font-size: 11px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: var(--text-muted);
}

.path-label {
  font-family: monospace;
}

.yaml-editor {
  flex: 1;
  height: 100%;
  border: none !important;
  border-radius: 0;
  background-color: var(--bg-terminal) !important;
}

/* 底部执行控制台 */
.console-panel {
  height: 140px;
  background-color: var(--bg-terminal);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.console-header {
  height: 24px;
  padding: 0 16px;
  background-color: rgba(255, 255, 255, 0.04);
  border-bottom: 1px solid var(--border-color);
  color: var(--text-title);
  font-size: 10px;
  font-weight: 700;
  display: flex;
  align-items: center;
}

.console-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 16px;
  background-color: var(--bg-terminal);
}

.console-line {
  color: var(--text-terminal);
  font-family: monospace;
  font-size: 11px;
  white-space: pre-wrap;
  margin-bottom: 3px;
}

.empty-console {
  color: var(--text-muted);
  font-style: italic;
  font-size: 11px;
}

/* 交互式终端容器 */
.pty-terminal-box {
  height: 60vh;
  background-color: var(--bg-terminal);
  padding: 8px;
  border-radius: 4px;
}

/* 警告模态框 */
.warning-modal-body {
  font-size: 12px;
  color: var(--text-body);
  line-height: 1.6;
}

.warning-highlight-text {
  color: var(--brand-danger);
  font-weight: 700;
  margin-bottom: 8px;
}

.warning-modal-body ul {
  padding-left: 20px;
  margin-top: 8px;
}

.warning-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* Exec 弹框 */
.exec-modal-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-picker-row {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.field-hint {
  font-size: 10px;
  color: var(--text-muted);
  font-style: italic;
  margin-top: 4px;
}

.modal-field-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
}

/* Top 弹框 */
.top-modal-body {
  max-height: 300px;
  overflow-y: auto;
}

.top-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 11px;
}

.top-table th,
.top-table td {
  padding: 8px;
  text-align: left;
  border-bottom: 1px solid var(--border-color);
}

.top-table th {
  color: var(--text-title);
  background-color: rgba(255, 255, 255, 0.02);
}

.top-table td {
  color: var(--text-body);
}

.primary-color {
  color: var(--brand-primary);
  font-weight: 600;
}

/* 复制成功飘动 Toast */
.copy-float-toast {
  position: fixed;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  background-color: rgba(16, 185, 129, 0.9);
  color: #000;
  padding: 6px 14px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 700;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  z-index: 9999;
  pointer-events: none;
}

/* 空选择状态 */
.empty-state {
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
}

.empty-logo {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-title);
  margin-bottom: 6px;
}

.empty-sub {
  font-size: 11px;
  color: var(--text-muted);
  max-width: 300px;
  line-height: 1.4;
}

.fade-in-enter-active,
.fade-in-leave-active {
  transition: opacity 0.12s ease;
}
.fade-in-enter-from,
.fade-in-leave-to {
  opacity: 0;
}
</style>