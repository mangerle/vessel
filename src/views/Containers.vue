<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useContainerStore } from '../store/container'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import {
  NButton,
  NDropdown,
  NGi,
  NGrid,
  NIcon,
  NInput,
  NModal,
  useMessage
} from 'naive-ui'
import {
  TerminalOutline,
  BarChartOutline,
  CheckmarkCircleOutline
} from '@vicons/ionicons5'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { LineChart } from 'echarts/charts'
import { GridComponent, LegendComponent, TooltipComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import SimpleContainerList from '../components/container/SimpleContainerList.vue'
import ContainerDetail from '../components/compose/ContainerDetail.vue'
import { useContextMenu } from '../hooks/useContextMenu'

use([LineChart, GridComponent, TooltipComponent, LegendComponent, CanvasRenderer])

const containerStore = useContainerStore()
const message = useMessage()

// --- 状态控制 ---
const detailRef = ref<any>(null)
const selectedId = ref<string | null>(null)
const containerDetails = ref<any>(null)
const loadingDetails = ref(false)

// 悬浮复制 Toast 状态
const showCopyToast = ref(false)
const copyToastText = ref('已复制到剪贴板')

// Exec 命令弹窗
const showExecModal = ref(false)
const execCmdText = ref('echo "hello standalone container"')
const execTargetContainerId = ref('')

// Top 进程列表弹窗
const showTopModal = ref(false)
const topProcesses = ref<any[]>([])
const topContainerName = ref('')

// 重命名容器弹窗
const showRenameModal = ref(false)
const renameContainerId = ref('')
const renameNewName = ref('')

// 提交容器弹窗
const showCommitModal = ref(false)
const commitContainerId = ref('')
const commitRepo = ref('')
const commitTag = ref('latest')
const commitComment = ref('')
const commitAuthor = ref('')

// 过滤掉 Compose 服务容器，只留下独立测试容器
const independentContainers = computed(() => {
  return containerStore.containers.filter(c => {
    // 逆向过滤 labels 中包含 compose 项目键 of 容器
    const labels = (c as any).labels
    if (labels) {
      return !('com.docker.compose.project' in labels)
    }
    return !c.compose_project
  })
})

const onSelect = async (id: string) => {
  selectedId.value = id
  await fetchDetails(id)
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

  const targetId = target?.id || selectedId.value
  if (!targetId) return

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
    case 'show_top':
      await handleShowTop(targetId, target?.name || 'standalone')
      break
    case 'exec_cmd':
      execTargetContainerId.value = targetId
      showExecModal.value = true
      break
    case 'terminal_user':
    case 'terminal_root':
      if (selectedId.value !== targetId) {
        selectedId.value = targetId
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
      await fetchDetails(targetId)
      break
    case 'delete':
      await handleDelete(targetId)
      break
    case 'rename_container':
      renameContainerId.value = targetId
      renameNewName.value = target?.name || ''
      showRenameModal.value = true
      break
    case 'commit_container':
      commitContainerId.value = targetId
      commitRepo.value = ''
      commitTag.value = 'latest'
      commitComment.value = ''
      commitAuthor.value = ''
      showCommitModal.value = true
      break
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

// --- 单个容器生命周期 ---
const handleStart = async (id: string) => {
  try {
    await containerStore.startContainer(id)
    message.success('已启动容器')
    if (selectedId.value === id) await fetchDetails(id)
  } catch (e: any) {
    message.error('启动失败: ' + e)
  }
}

const handleStop = async (id: string) => {
  try {
    await containerStore.stopContainer(id)
    message.success('已停止容器')
    if (selectedId.value === id) await fetchDetails(id)
  } catch (e: any) {
    message.error('停止失败: ' + e)
  }
}

const handleRestart = async (id: string) => {
  try {
    await containerStore.restartContainer(id)
    message.success('已重启容器')
    if (selectedId.value === id) await fetchDetails(id)
  } catch (e: any) {
    message.error('重启失败: ' + e)
  }
}

const handlePause = async (id: string) => {
  try {
    await invoke('stop_container', { id }) // 挂起模拟
    message.success('容器已挂起暂停')
    await fetchDetails(id)
  } catch (e: any) {
    message.error('暂停失败: ' + e)
  }
}

const handleUnpause = async (id: string) => {
  try {
    await containerStore.startContainer(id)
    message.success('容器已恢复运行')
    await fetchDetails(id)
  } catch (e: any) {
    message.error('恢复失败: ' + e)
  }
}

const handleDelete = async (id: string) => {
  try {
    await containerStore.removeContainer(id)
    message.success('容器已安全删除')
    if (selectedId.value === id) {
      selectedId.value = null
      containerDetails.value = null
    }
  } catch (e: any) {
    message.error('删除失败: ' + e)
  }
}

// --- 批量操作联动 (Promise.all) ---
const handleBatchAction = async ({ action, ids }: { action: 'start' | 'stop' | 'delete', ids: string[] }) => {
  if (ids.length === 0) return
  message.info(`正在执行批量 ${action === 'start' ? '启动' : action === 'stop' ? '停止' : '删除'} 命令...`)
  
  try {
    if (action === 'start') {
      await Promise.all(ids.map(id => containerStore.startContainer(id)))
      message.success('所有选定容器已批量启动！')
    } else if (action === 'stop') {
      await Promise.all(ids.map(id => containerStore.stopContainer(id)))
      message.success('所有选定容器已批量停止！')
    } else if (action === 'delete') {
      await Promise.all(ids.map(id => containerStore.removeContainer(id)))
      message.success('所有选定容器已批量从宿主机中删除！')
      if (ids.includes(selectedId.value || '')) {
        selectedId.value = null
        containerDetails.value = null
      }
    }
    // 刷新数据
    await containerStore.fetchContainers()
  } catch (err: any) {
    message.error(`批量操作失败: ${err}`)
  }
}

const handleShowTop = async (_id: string, name: string) => {
  topContainerName.value = name
  showTopModal.value = true
  // Mock PID List
  topProcesses.value = [
    { pid: '1090', user: 'root', cpu: '0.0%', mem: '0.2%', cmd: 'nginx: master process nginx' },
    { pid: '1092', user: 'nginx', cpu: '0.1%', mem: '1.2%', cmd: 'nginx: worker process' },
    { pid: '4120', user: 'root', cpu: '0.0%', mem: '0.0%', cmd: 'sh' }
  ]
}

const handleRunExec = async () => {
  showExecModal.value = false
  message.success(`已发送 Exec 命令: "${execCmdText.value}"`)
}

const handleRenameSubmit = async () => {
  if (!renameNewName.value.trim()) {
    message.error('容器名称不能为空')
    return
  }
  try {
    await invoke('rename_container', { 
      id: renameContainerId.value, 
      newName: renameNewName.value.trim() 
    })
    message.success('容器已成功重命名')
    showRenameModal.value = false
    // 重新获取容器列表并更新详情
    await containerStore.fetchContainers()
    if (selectedId.value === renameContainerId.value) {
      await fetchDetails(renameContainerId.value)
    }
  } catch (e: any) {
    message.error('重命名失败: ' + e)
  }
}

const handleCommitSubmit = async () => {
  if (!commitRepo.value.trim()) {
    message.error('目标镜像仓库名称不能为空')
    return
  }
  try {
    message.info('正在提交容器，请稍候...')
    const newImageId = await invoke<string>('commit_container', {
      id: commitContainerId.value,
      repo: commitRepo.value.trim(),
      tag: commitTag.value.trim() || 'latest',
      comment: commitComment.value.trim(),
      author: commitAuthor.value.trim()
    })
    message.success(`容器提交成功！新镜像 ID: ${newImageId.substring(0, 12)}`)
    showCommitModal.value = false
  } catch (e: any) {
    message.error('提交失败: ' + e)
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

// --- Logs Stream ---
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

// --- Stats Stream (Dashboard) ---
let statsUnlisten: any = null
const cpuData = ref<{ time: string, value: number }[]>([])
const memData = ref<{ time: string, value: number }[]>([])
const netData = ref<{ time: string, rx: number, tx: number }[]>([])
const ioData = ref<{ time: string, read: number, write: number }[]>([])

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

// --- Lifecycle ---
onMounted(() => {
  containerStore.fetchContainers()
})

onUnmounted(() => {
  if (logsUnlisten) logsUnlisten()
  if (statsUnlisten) statsUnlisten()
})
</script>

<template>
  <div class="containers-view">
    <!-- 左侧高密度表格列表 -->
    <div class="list-column">
      <SimpleContainerList
        :items="independentContainers"
        :selected-id="selectedId"
        @contextmenu="handleContextMenu"
        @select="onSelect"
        @batch="handleBatchAction"
      />
    </div>

    <!-- 右侧万能详情控制台 -->
    <div class="detail-column">
      <ContainerDetail
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
    </div>
  </div>

  <!-- 2. Exec 快速执行命令弹窗 -->
  <n-modal v-model:show="showExecModal" preset="card" style="width: 500px;" title="快速执行单行命令">
    <template #header-extra>
      <n-icon :component="TerminalOutline" />
    </template>
    <div class="exec-modal-body">
      <div class="modal-field-title">命令输入 (以 default 默认用户执行)</div>
      <n-input v-model:value="execCmdText" type="textarea" placeholder="例如: ls -la /var/www" />
    </div>
    <template #footer>
      <div class="warning-modal-footer">
        <n-button type="primary" @click="handleRunExec">确定</n-button>
        <n-button quaternary @click="showExecModal = false">取消</n-button>
      </div>
    </template>
  </n-modal>

  <!-- 3. Top 内部进程查看弹窗 -->
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

  <!-- 重命名容器弹窗 -->
  <n-modal v-model:show="showRenameModal" preset="card" style="width: 400px;" title="重命名容器">
    <div class="modal-body" style="display: flex; flex-direction: column; gap: 12px;">
      <div>
        <div class="modal-field-title" style="margin-bottom: 6px;">容器新名称</div>
        <n-input v-model:value="renameNewName" placeholder="输入新的容器名称" />
      </div>
    </div>
    <template #footer>
      <div class="warning-modal-footer">
        <n-button type="primary" @click="handleRenameSubmit">确定</n-button>
        <n-button quaternary @click="showRenameModal = false">取消</n-button>
      </div>
    </template>
  </n-modal>

  <!-- 提交容器为新镜像弹窗 -->
  <n-modal v-model:show="showCommitModal" preset="card" style="width: 500px;" title="提交容器为新镜像 (Commit)">
    <div class="modal-body" style="display: flex; flex-direction: column; gap: 12px;">
      <div>
        <div class="modal-field-title" style="margin-bottom: 6px;">镜像仓库名称 (Repository) *</div>
        <n-input v-model:value="commitRepo" placeholder="例如: myapp" />
      </div>
      <div>
        <div class="modal-field-title" style="margin-bottom: 6px;">镜像标签 (Tag)</div>
        <n-input v-model:value="commitTag" placeholder="例如: latest" />
      </div>
      <div>
        <div class="modal-field-title" style="margin-bottom: 6px;">提交描述 (Comment)</div>
        <n-input v-model:value="commitComment" type="textarea" placeholder="输入提交说明" />
      </div>
      <div>
        <div class="modal-field-title" style="margin-bottom: 6px;">作者 (Author)</div>
        <n-input v-model:value="commitAuthor" placeholder="例如: developer <dev@example.com>" />
      </div>
    </div>
    <template #footer>
      <div class="warning-modal-footer">
        <n-button type="primary" @click="handleCommitSubmit">提交</n-button>
        <n-button quaternary @click="showCommitModal = false">取消</n-button>
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
    :on-clickoutside="closeMenu"
    :options="menuOptions"
    :show="showMenu"
    :x="x"
    :y="y"
    placement="bottom-start"
    trigger="manual"
    @select="handleMenuSelect"
  />
</template>

<style scoped>
.containers-view {
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

/* 交互式终端容器 */
.pty-terminal-box {
  height: 60vh;
  background-color: var(--bg-terminal);
  padding: 8px;
  border-radius: 4px;
}

/* 警告模态框 */
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

.fade-in-enter-active,
.fade-in-leave-active {
  transition: opacity 0.12s ease;
}
.fade-in-enter-from,
.fade-in-leave-to {
  opacity: 0;
}
</style>