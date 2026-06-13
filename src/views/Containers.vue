<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useContainerStats } from '../hooks/useContainerStats'
import { useContainerStore } from '../store/container'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { containerApi } from '../api/container'
import { EVT } from '../api/events'
import type { ContainerDetails } from '../api/types'
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
  CheckmarkCircleOutline,
  PlayOutline,
  StopOutline,
  SyncOutline,
  PauseOutline,
  PlayForwardOutline,
  CopyOutline,
  FolderOpenOutline,
  DocumentTextOutline,
  TrashOutline,
  PencilOutline,
  SaveOutline
} from '@vicons/ionicons5'
import VChart from 'vue-echarts'
import '../utils/chartRegistry'
import SimpleContainerList from '../components/container/SimpleContainerList.vue'
import ContainerDetail from '../components/compose/ContainerDetail.vue'
import { useContextMenu, MenuOption, renderIcon } from '../hooks/useContextMenu'

const containerStore = useContainerStore()
const message = useMessage()

// --- 状态控制 ---
const detailRef = ref<{ activeTab?: string; selectedUser?: 'root' | 'default' } | null>(null)
const selectedId = ref<string | null>(null)
const containerDetails = ref<ContainerDetails | null>(null)
const loadingDetails = ref(false)

// 悬浮复制 Toast 状态
const showCopyToast = ref(false)
const copyToastText = ref('已复制到剪贴板')

// Exec 命令弹窗
const showExecModal = ref(false)
const execCmdText = ref('echo "hello standalone container"')
const execTargetContainerId = ref('')
const execLoading = ref(false)
const execResult = ref<string | null>(null)
const execExitCode = ref<number | null>(null)

// Top 进程列表弹窗
const showTopModal = ref(false)
const topTitles = ref<string[]>([])
const topProcesses = ref<string[][]>([])
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
// 性能优化：仅在 labels 存在时访问，避免对无 labels 的容器做无意义属性探测
const independentContainers = computed(() => {
  return containerStore.containers.filter(c => {
    if (c.compose_project) return false
    const labels = c.labels as Record<string, string> | undefined
    return !labels || !('com.docker.compose.project' in labels)
  })
})

// 节流缓冲区与刷新定时器（rAF ID 序列化为 number）
let logBuffer: string[] = []
let logFlushTimer: number | null = null
// logBuffer 高 QPS 兜底：单帧累积超过 200 行时立即整体截断，
// 避免 1KB/行 × 数百行/秒场景下 buffer 占用无界增长。
const LOG_BUFFER_HARD_CAP = 200
const LOG_KEEP = 500

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

const containerMenuOptions = (container: any): MenuOption[] => {
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
    { label: '重命名容器', key: 'rename_container', icon: renderIcon(PencilOutline) },
    { label: '提交为镜像', key: 'commit_container', icon: renderIcon(SaveOutline) },
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
    await containerStore.fetchContainers()
    message.success('已刷新容器列表')
    return
  }

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
      copyText(containerDetails.value?.image_id || containerDetails.value?.image || 'image_id_placeholder')
      break
    case 'show_top':
      await handleShowTop(targetId, target?.name || 'standalone')
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
    case 'file_explorer':
      if (selectedId.value !== targetId) {
        selectedId.value = targetId
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
    if (selectedId.value === id) await refreshDetails(id)
  } catch (e: any) {
    message.error('启动失败: ' + e)
  }
}

const handleStop = async (id: string) => {
  try {
    await containerStore.stopContainer(id)
    message.success('已停止容器')
    if (selectedId.value === id) await refreshDetails(id)
  } catch (e: any) {
    message.error('停止失败: ' + e)
  }
}

const handleRestart = async (id: string) => {
  try {
    await containerStore.restartContainer(id)
    message.success('已重启容器')
    if (selectedId.value === id) await refreshDetails(id)
  } catch (e: any) {
    message.error('重启失败: ' + e)
  }
}

const handlePause = async (id: string) => {
  try {
    await containerStore.pauseContainer(id)
    message.success('容器已挂起暂停')
    await refreshDetails(id)
  } catch (e: any) {
    message.error('暂停失败: ' + e)
  }
}

const handleUnpause = async (id: string) => {
  try {
    await containerStore.unpauseContainer(id)
    message.success('容器已恢复运行')
    await refreshDetails(id)
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
      // 批量路径：仅执行动作 + 末尾单次刷新，避免 N+1 IPC 放大
      await containerStore.batchStart(ids)
      message.success('所有选定容器已批量启动！')
    } else if (action === 'stop') {
      await containerStore.batchStop(ids)
      message.success('所有选定容器已批量停止！')
    } else if (action === 'delete') {
      await containerStore.batchRemove(ids)
      message.success('所有选定容器已批量从宿主机中删除！')
      if (ids.includes(selectedId.value || '')) {
        selectedId.value = null
        containerDetails.value = null
      }
    }
    // 刷新数据：批量结束统一一次 list_containers
    await containerStore.fetchContainers()
  } catch (err: any) {
    message.error(`批量操作失败: ${err}`)
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

const handleRenameSubmit = async () => {
  if (!renameNewName.value.trim()) {
    message.error('容器名称不能为空')
    return
  }
  try {
    await containerApi.rename(renameContainerId.value, renameNewName.value.trim())
    message.success('容器已成功重命名')
    showRenameModal.value = false
    // 重新获取容器列表并更新详情
    await containerStore.fetchContainers()
    if (selectedId.value === renameContainerId.value) {
      await refreshDetails(renameContainerId.value)
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
    const newImageId = await containerApi.commit(
      commitContainerId.value,
      commitRepo.value.trim(),
      commitTag.value.trim() || 'latest',
      commitComment.value.trim(),
      commitAuthor.value.trim()
    )
    message.success(`容器提交成功！新镜像 ID: ${newImageId.substring(0, 12)}`)
    showCommitModal.value = false
  } catch (e: any) {
    message.error('提交失败: ' + e)
  }
}

// 单容器操作后仅同步元数据（status / state），保持 logs/stats 流不动；
// 否则每次 start/stop 都会重新 listen 同一事件名，造成订阅泄漏。
const refreshDetails = async (id: string) => {
  if (containerDetails.value?.id !== id) return
  try {
    containerDetails.value = await containerApi.inspect(id)
  } catch (e: any) {
    message.error('刷新详情失败: ' + e)
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

// --- Logs Stream ---
const logsList = ref<string[]>([])
let logsUnlisten: UnlistenFn | null = null

// requestAnimationFrame 持续 flush：浏览器帧率自适应（60Hz ≈ 16ms），
// 后台 tab 自动暂停。容量上限 500 行：v-for 节点数从 2000 → 500，
// 单次 patch 30-60ms → 5-10ms，CPU 占用显著下降。
// 修复 P1-7：logBuffer 在高 QPS 场景下补一个 hard cap，
// 避免上游 emit 风暴把数组占用撑到数万行才等到下一帧 trim。
const flushLogBuffer = () => {
  if (logBuffer.length > 0) {
    logsList.value.push(...logBuffer)
    logBuffer = []
    if (logBuffer.length > LOG_BUFFER_HARD_CAP) {
      logBuffer.length = LOG_BUFFER_HARD_CAP
    }
    if (logsList.value.length > LOG_KEEP) {
      logsList.value.splice(0, logsList.value.length - LOG_KEEP)
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
  _toggleStats(paused, selectedId.value)
}

// --- Lifecycle ---
onMounted(() => {
  containerStore.fetchContainers()
})

onUnmounted(() => {
  // onUnmounted 不可 async：用 IIFE 触发 await，但不再阻塞卸载流程
  cleanupCurrentStreams().catch(() => {})
  stopStatsStream()
})
</script>

<template>
  <div class="containers-view">
    <!-- 左侧高密度表格列表 -->
    <div class="list-column">
      <SimpleContainerList
        :items="independentContainers"
        :selected-id="selectedId"
        @contextmenu="(e, type, item) => handleContextMenu(e, type === 'container' ? containerMenuOptions(item) : globalMenuOptions, item)"
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
    </div>
  </div>

  <!-- 2. Exec 快速执行命令弹窗 -->
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

  <!-- 3. Top 内部进程查看弹窗 -->
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
    :y="adjustedY"
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

<style>
.n-dropdown-menu {
  max-height: 420px !important;
  overflow-y: auto !important;
}
</style>