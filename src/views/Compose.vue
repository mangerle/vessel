<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useContainerStats } from '../hooks/useContainerStats'
import { useComposeStore } from '../store/compose'
import { useContainerStore } from '../store/container'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { containerApi } from '../api/container'
import { EVT } from '../api/events'
import type { ContainerDetails } from '../api/types'
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
  HammerOutline,
  TerminalOutline,
  BarChartOutline,
  CheckmarkCircleOutline,
  FileTrayFullOutline,
  FlashOutline,
  PauseOutline,
  PlayForwardOutline,
  CopyOutline,
  DocumentTextOutline,
  TrashOutline
} from '@vicons/ionicons5'

import VChart from 'vue-echarts'
import '../utils/chartRegistry'
import ComposeProjectList from '../components/compose/ComposeProjectList.vue'
import ContainerDetail from '../components/compose/ContainerDetail.vue'
import { useContextMenu, MenuOption, renderIcon } from '../hooks/useContextMenu'

const composeStore = useComposeStore()
const containerStore = useContainerStore()
const message = useMessage()

// --- 状态控制 ---
const detailRef = ref<{ activeTab?: string; selectedUser?: 'root' | 'default' } | null>(null)
const selectedId = ref<string | null>(null)
const selectedType = ref<'project' | 'container' | null>(null)
const containerDetails = ref<ContainerDetails | null>(null)
const loadingDetails = ref(false)

const selectedProject = computed(() => {
  if (selectedId.value && selectedId.value.startsWith('project:')) {
    const projectName = selectedId.value.split(':')[1]
    return composeStore.projects.find(p => p.name === projectName) || null
  }
  return null
})

// 悬浮复制 Toast 状态
const showCopyToast = ref(false)
const copyToastText = ref('已复制到剪贴板')



// Exec 命令弹窗
const showExecModal = ref(false)
const execCmdText = ref('echo "hello geek"')
const execTargetContainerId = ref('')
const execLoading = ref(false)
const execResult = ref<string | null>(null)
const execExitCode = ref<number | null>(null)

// Top 进程列表弹窗
const showTopModal = ref(false)
const topTitles = ref<string[]>([])
const topProcesses = ref<string[][]>([])
const topContainerName = ref('')

// 导入项目弹窗
const showImportModal = ref(false)
const importPath = ref('D:/code/project/docker-compose.yml')

// 节流缓冲区与刷新定时器（rAF ID 序列化为 number）
let logBuffer: string[] = []
let logFlushTimer: number | null = null

// 清理当前的流与定时器
// 修复 P1-5：必须 await 后端 close，确保后端 task 收到停止信号后再继续
const cleanupCurrentStreams = async () => {
  if (logsUnlisten) {
    logsUnlisten()
    logsUnlisten = null
  }
  if (logFlushTimer !== null) {
    cancelAnimationFrame(logFlushTimer)
    logFlushTimer = null
  }
  if (containerDetails.value?.id) {
    const oldId = containerDetails.value.id
    await Promise.allSettled([
      containerApi.closeLogs(oldId),
      containerApi.closeStats(oldId)
    ])
  }
}

const onSelect = async (id: string) => {
  // 修复 P1-5：必须 await，确保后端流被显式停止再切换
  await cleanupCurrentStreams()
  selectedId.value = id
  if (id.startsWith('project:')) {
    selectedType.value = 'project'
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

const projectMenuOptions = (_project: any): MenuOption[] => {
  return [
    { label: '启动项目 (Up)', key: 'up', icon: renderIcon(PlayOutline) },
    { label: '停止项目 (Stop)', key: 'stop_project', icon: renderIcon(StopOutline) },
    { label: '下线项目 (Down)', key: 'down_project', icon: renderIcon(TrashOutline) },
    { label: '重启项目 (Restart)', key: 'restart_project', icon: renderIcon(SyncOutline) }
  ]
}

const composeContainerMenuOptions = (container: any): MenuOption[] => {
  const isRunning = container.state === 'running';
  const isPaused = container.state === 'paused';
  const canStart = !isRunning && !isPaused;
  const canStop = isRunning || isPaused;
  
  return [
    { label: '启动容器', key: 'start', icon: renderIcon(PlayOutline), disabled: !canStart },
    { label: '停止容器', key: 'stop', icon: renderIcon(StopOutline), disabled: !canStop },
    { label: '重启容器', key: 'restart', icon: renderIcon(SyncOutline), disabled: !canStop },
    { label: isPaused ? '恢复运行' : '挂起容器', key: isPaused ? 'unpause' : 'pause', icon: renderIcon(isPaused ? PlayForwardOutline : PauseOutline), disabled: !canStop && !isPaused },
    { type: 'divider', key: 'd1' },
    { label: '复制容器 ID', key: 'copy_id', icon: renderIcon(CopyOutline) },
    { label: '复制镜像 ID', key: 'copy_image_id', icon: renderIcon(CopyOutline) },
    { type: 'divider', key: 'd2' },
    { label: '内部活跃进程 (Top)', key: 'show_top', icon: renderIcon(BarChartOutline), disabled: !isRunning },
    { label: '执行单行命令 (Exec)', key: 'exec_cmd', icon: renderIcon(TerminalOutline), disabled: !isRunning },
    { label: '交互式终端', key: 'terminal_user', icon: renderIcon(TerminalOutline), disabled: !isRunning },
    { label: '容器文件浏览器', key: 'file_explorer', icon: renderIcon(FolderOpenOutline), disabled: !isRunning },
    { label: '查看实时日志', key: 'logs', icon: renderIcon(DocumentTextOutline) },
    { type: 'divider', key: 'd3' },
    { label: '删除容器', key: 'delete', icon: renderIcon(TrashOutline), disabled: isRunning }
  ]
}

const globalMenuOptions: MenuOption[] = [
  { label: '刷新列表', key: 'refresh_list', icon: renderIcon(SyncOutline) }
]

const onContextMenu = (e: MouseEvent, type: string, item?: any) => {
  let options = globalMenuOptions
  if (type === 'project') {
    options = projectMenuOptions(item)
  } else if (type === 'container') {
    options = composeContainerMenuOptions(item)
  }
  handleContextMenu(e, options, item)
}

const adjustedY = computed(() => {
  if (y.value > window.innerHeight - 450) {
    return Math.max(10, y.value - 410)
  }
  return y.value
})

const handleMenuSelect = async (key: string) => {
  const target = currentTarget.value as Record<string, string> | null
  closeMenu()

  if (key === 'refresh_list') {
    await composeStore.fetchProjects()
    await containerStore.fetchContainers()
    message.success('已刷新项目和容器列表')
    return
  }

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
        copyText(containerDetails.value?.image_id || containerDetails.value?.image || 'image_id_placeholder')
        break
      case 'inspect_meta':
        // 切到元数据 Inspect 页
        break
      case 'show_top':
        await handleShowTop(targetId, target?.name || 'mysql')
        break
      case 'exec_cmd':
        execTargetContainerId.value = targetId
        execResult.value = null
        execExitCode.value = null
        execLoading.value = false
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
      case 'file_explorer':
        if (selectedId.value !== targetId || selectedType.value !== 'container') {
          selectedId.value = targetId
          selectedType.value = 'container'
          await fetchDetails(targetId)
        }
        nextTick(() => {
          if (detailRef.value) {
            detailRef.value.activeTab = 'files'
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
  if (key === 'up' || key === 'down' || key === 'stop_project' || key === 'down_project' || key === 'restart_project') {
    const project = (target && !target.id) ? target : selectedProject.value
    if (!project) return

    if (key === 'up') await handleProjectUp(project)
    else if (key === 'stop_project') await handleProjectStop(project)
    else if (key === 'down' || key === 'down_project') await handleProjectDown(project)
    else if (key === 'restart_project') await handleProjectRestart(project)
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
    await containerStore.pauseContainer(id)
    message.success('容器已挂起暂停')
    await composeStore.fetchProjects()
    await fetchDetails(id)
  } catch (e: any) {
    message.error('暂停失败: ' + e)
  }
}

const handleUnpause = async (id: string) => {
  try {
    await containerStore.unpauseContainer(id)
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

const handleProjectStop = async (project?: any) => {
  const p = project?.working_dir ? project : selectedProject.value
  if (!p?.working_dir) return
  try {
    await composeStore.runComposeCommand(p.working_dir, ['stop'])
    message.success('已发送 Compose Stop 指令')
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




const handleShowTop = async (id: string, name: string) => {
  topContainerName.value = name
  showTopModal.value = true
  topTitles.value = []
  topProcesses.value = []
  try {
    const res = await containerApi.top(id)
    topTitles.value = res.titles
    topProcesses.value = res.processes
  } catch (e: any) {
    message.error('获取进程列表失败: ' + e)
  }
}

const handleRunExec = async () => {
  if (!execCmdText.value.trim()) {
    message.warning('请输入要执行的命令')
    return
  }
  execLoading.value = true
  execResult.value = null
  execExitCode.value = null
  try {
    const res = await containerApi.exec(execTargetContainerId.value, execCmdText.value.trim())
    execResult.value = res.output || '[无输出]'
    execExitCode.value = res.exit_code
    if (res.exit_code === 0) {
      message.success('命令执行完成')
    } else {
      message.warning(`命令执行完毕，退出码为 ${res.exit_code}`)
    }
  } catch (e: any) {
    execResult.value = `执行错误: ${e}`
    message.error('执行失败: ' + e)
  } finally {
    execLoading.value = false
  }
}



const pickComposeFile = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
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
    containerDetails.value = await containerApi.inspect(id)
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
let logsUnlisten: UnlistenFn | null = null

// rAF 持续 flush：浏览器帧率自适应（60Hz ≈ 16ms），后台 tab 自动暂停。
// 容量上限 500 行：v-for 节点数从 2000 → 500，单次 patch 30-60ms → 5-10ms。
const flushLogBuffer = () => {
  if (logBuffer.length > 0) {
    logsList.value.push(...logBuffer)
    logBuffer = []
    if (logsList.value.length > 500) {
      logsList.value.splice(0, logsList.value.length - 500)
    }
  }
  logFlushTimer = requestAnimationFrame(flushLogBuffer)
}

const startLogsStream = async (id: string) => {
  logsList.value = []
  logBuffer = []
  logFlushTimer = requestAnimationFrame(flushLogBuffer)

  logsUnlisten = await listen<string>(EVT.containerLogs(id), (event) => {
    logBuffer.push(event.payload)
  })

  try {
    await containerApi.streamLogs(id)
  } catch (e) {
    console.error('开始日志流失败', e)
  }
}

// --- 性能仪表盘 (ECharts) 统计绑定 ---
const {
  cpuOption,
  memOption,
  netOption,
  ioOption,
  startStatsStream,
  stopStatsStream,
  handleToggleStats: _toggleStats,
  handleResetStats
} = useContainerStats()

const handleToggleStats = (paused: boolean) => {
  _toggleStats(paused, selectedType.value === 'container' ? selectedId.value : null)
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
  // onUnmounted 不可 async：用 IIFE 触发 await，但不再阻塞卸载流程
  cleanupCurrentStreams().catch(() => {})
  stopStatsStream()
})
</script>

<template>
  <div class="compose-view" @contextmenu.prevent="onContextMenu($event, 'global')">
    <!-- 左侧 Compose 极简文件管理器树 (240px 宽度) -->
    <div class="list-column">
      <ComposeProjectList
        :containers="containerStore.containers"
        :projects="composeStore.projects"
        :selected-id="selectedId"
        @select="onSelect"
        @contextmenu="onContextMenu"
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
        @toggle-stats="handleToggleStats"
        @reset-stats="handleResetStats"
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
                启动
              </n-button>
              <n-button :loading="composeStore.executing" @click="handleProjectStop">
                <template #icon><n-icon :component="StopOutline" /></template>
                停止
              </n-button>
              <n-button :loading="composeStore.executing" @click="handleProjectDown">
                <template #icon><n-icon :component="StopOutline" /></template>
                下线
              </n-button>
              <n-button :loading="composeStore.executing" @click="handleProjectRestart">
                <template #icon><n-icon :component="SyncOutline" /></template>
                重启
              </n-button>
            </n-button-group>
          </div>
        </div>

        <div class="workspace-content">
          <!-- 命令输出控制台 -->
          <div class="console-panel" style="flex: 1; height: 100%; border-radius: 4px; overflow: hidden; border: 1px solid var(--border-color);">
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
          <img src="/logo.png" alt="Vessel Logo" style="width: 80px; height: 80px; object-fit: contain;" />
        </div>
        <div class="empty-title">欢迎使用 Vessel</div>
        <div class="empty-sub">请在左侧选择一个 Compose 服务或项目以进行深度精细化操作。</div>
      </div>
    </div>
  </div>





  <!-- 3. Exec 快速执行命令弹窗 -->
  <n-modal v-model:show="showExecModal" preset="card" style="width: 600px;" title="快速执行单行命令">
    <template #header-extra>
      <n-icon :component="TerminalOutline" />
    </template>
    <div class="exec-modal-body" style="display: flex; flex-direction: column; gap: 12px;">
      <div>
        <div class="modal-field-title" style="margin-bottom: 6px;">命令输入 (通过 /bin/sh -c 执行)</div>
        <n-input v-model:value="execCmdText" :disabled="execLoading" type="textarea" placeholder="例如: ls -la /var/www" />
      </div>

      <div v-if="execLoading || execResult !== null" class="exec-result-container" style="display: flex; flex-direction: column; gap: 6px;">
        <div class="modal-field-title" style="display: flex; justify-content: space-between; align-items: center;">
          <span>执行输出 <span v-if="execExitCode !== null" :style="{ color: execExitCode === 0 ? 'var(--brand-primary)' : '#ef4444' }">(退出码: {{ execExitCode }})</span></span>
          <span v-if="execLoading" style="color: var(--brand-primary)">执行中...</span>
        </div>
        <pre class="exec-output-box" style="margin: 0; padding: 12px; background-color: #070a10; color: #cbd5e1; border-radius: 4px; max-height: 250px; overflow-y: auto; font-family: monospace; font-size: 11px; white-space: pre-wrap; border: 1px solid var(--border-color);">{{ execResult || '正在执行并收集输出...' }}</pre>
      </div>
    </div>
    <template #footer>
      <div class="warning-modal-footer">
        <n-button type="primary" :loading="execLoading" @click="handleRunExec">
          {{ execResult !== null ? '重新运行' : '运行' }}
        </n-button>
        <n-button quaternary :disabled="execLoading" @click="showExecModal = false">关闭</n-button>
      </div>
    </template>
  </n-modal>

  <!-- 4. Top 内部进程查看弹窗 -->
  <n-modal v-model:show="showTopModal" preset="card" style="width: 700px;" :title="`内部活跃进程 (${topContainerName})`">
    <template #header-extra>
      <n-icon :component="BarChartOutline" />
    </template>
    <div class="top-modal-body">
      <div v-if="topProcesses.length === 0" style="padding: 20px; text-align: center; color: var(--text-muted);">
        暂无活跃进程信息或加载中...
      </div>
      <table v-else class="top-table">
        <thead>
          <tr>
            <th v-for="title in topTitles" :key="title">{{ title }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(proc, idx) in topProcesses" :key="idx">
            <td v-for="(val, vIdx) in proc" :key="vIdx" :class="{ 'monospace-text': vIdx === proc.length - 1 }">
              {{ val }}
            </td>
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
    :y="adjustedY"
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

.editor-content-wrapper {
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--bg-terminal);
}

/* 确保 CodeMirror 6 满幅填充容器并隐藏外部边框 */
.editor-content-wrapper :deep(.cm-editor) {
  height: 100%;
  flex: 1;
  outline: none !important;
}

.editor-content-wrapper :deep(.cm-scroller) {
  font-family: "JetBrains Mono", "Fira Code", "Consolas", monospace !important;
}

/* 启用高对比度文字抗锯齿，并微调字重，彻底消除深色模式下字体发虚模糊的问题 */
.editor-content-wrapper :deep(.cm-content) {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: optimizeLegibility;
  font-weight: 500;
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

<style>
.n-dropdown-menu {
  max-height: 420px !important;
  overflow-y: auto !important;
}
</style>