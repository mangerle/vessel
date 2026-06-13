<template>
  <div class="container-detail">
    <!-- Loading 遮罩 -->
    <div v-if="loading" class="loading-overlay">
      <n-spin size="large" />
    </div>

    <template v-else-if="container">
      <!-- 顶层双行控制栏 (固定高 72px) -->
      <div class="control-header-wrapper" key="active-detail-header">
        <!-- 行 1: 周期观测 Tab 栏 (高 32px) -->
        <div class="tab-line-1">
          <div 
            v-for="t in tabs" 
            :key="t.value" 
            class="obs-tab" 
            :class="{ active: activeTab === t.value }"
            @click="activeTab = t.value"
          >
            <n-icon :component="t.icon" size="14" class="tab-icon" />
            <span>{{ t.label }}</span>
            <div class="tab-indicator"></div>
          </div>
        </div>

        <!-- 行 2: 元数据与生命周期按钮 (高 40px) -->
        <div class="meta-line-2">
          <!-- 左侧元数据区 -->
          <div class="meta-left">
            <span class="status-dot" :class="container.status || container.State?.Status"></span>
            <span class="container-name">{{ container.name || container.Name?.replace('/', '') }}</span>
            <span class="container-id">{{ (container.id || container.Id || '').substring(0, 8) }}</span>
            <div class="vertical-divider"></div>
            <span class="meta-badge image-badge" :title="container.image || container.Config?.Image">
              <n-icon :component="DiscOutline" size="12" />
              {{ (container.image || container.Config?.Image || '').split('@')[0] }}
            </span>
            <span class="meta-badge project-badge">
              <n-icon :component="FlashOutline" size="12" />
              {{ container.compose_project || 'Standalone' }}
            </span>
          </div>

          <!-- 右侧动作按钮区 -->
          <div class="meta-right">
            <!-- 启动 / 停止 按钮 -->
            <button 
              class="action-btn"
              :class="isUp ? 'stop-border' : 'start-border'"
              @click="togglePower"
            >
              <n-icon :component="isUp ? StopOutline : PlayOutline" size="14" />
              {{ isUp ? '停止' : '启动' }}
            </button>

            <!-- 终端 组合分裂按钮 (Split Button) - 原位切换 -->
            <div class="split-btn-group">
              <button 
                class="split-main-btn" 
                :class="{ active: activeTab === 'terminal' }"
                @click="triggerTerminal"
              >
                <n-icon :component="TerminalOutline" size="14" />
                终端{{ selectedUser === 'root' ? ' (root)' : '' }}
              </button>
              <button 
                class="split-arrow-btn" 
                :class="{ highlighted: selectedUser === 'root' }" 
                @click.stop="showUserMenu = !showUserMenu"
              >
                <n-icon :component="ChevronDownOutline" size="12" />
              </button>
              <!-- 身份下拉菜单 -->
              <transition name="fade-in">
                <div v-if="showUserMenu" class="user-dropdown-menu">
                  <div 
                    class="user-option" 
                    :class="{ active: selectedUser === 'default' }"
                    @click="selectUser('default')"
                  >
                    <n-icon :component="PersonOutline" size="14" />
                    default (默认用户)
                  </div>
                  <div 
                    class="user-option" 
                    :class="{ active: selectedUser === 'root' }"
                    @click="selectUser('root')"
                  >
                    <n-icon :component="PersonOutline" size="14" />
                    root (超级管理员)
                  </div>
                </div>
              </transition>
            </div>
          </div>
        </div>
      </div>

      <!-- 下方主内容区 -->
      <div class="obs-content" key="active-detail-content">
        <!-- 1. 运行日志 (三栏像素级布局) -->
        <div v-show="activeTab === 'logs'" class="logs-pane">
          <!-- 左栏: 等宽文本流大宽区 -->
          <div 
            class="logs-text-area" 
            :class="{ 'word-wrap': wordWrap }" 
            ref="logTextRef" 
            @scroll="onLogScroll"
            @contextmenu.prevent.stop="handleLogsContext"
          >
            <div v-for="(log, idx) in logsList" :key="idx" class="log-line">
              {{ log }}
            </div>
            <div v-if="logsList.length === 0" class="empty-logs-text">
              等待容器日志输出...
            </div>
          </div>

          <!-- 中栏: 独立自定义垂直滚动条 -->
          <div 
            class="custom-scrollbar-track" 
            @mouseenter="scrollHover = true" 
            @mouseleave="scrollHover = false"
            @mousedown="onScrollTrackMouseDown"
          >
            <div 
              class="custom-scrollbar-thumb" 
              :style="{ height: thumbHeight + 'px', top: thumbTop + 'px', opacity: scrollHover || isDragging ? 0.8 : 0.3 }"
              @mousedown.stop="onScrollThumbMouseDown"
            ></div>
          </div>

          <!-- 右栏: 极窄飞梭动作纽扣控制带 -->
          <div class="shuttle-controls">
            <button class="shuttle-btn" title="上移" @mousedown="startScroll('up')" @mouseup="stopScroll" @mouseleave="stopScroll">
              <n-icon :component="ChevronUpOutline" />
            </button>
            <button class="shuttle-btn" title="下移" @mousedown="startScroll('down')" @mouseup="stopScroll" @mouseleave="stopScroll">
              <n-icon :component="ChevronDownOutline" />
            </button>
            <button class="shuttle-btn" :class="{ active: wordWrap }" title="自动换行" @click="wordWrap = !wordWrap">
              <n-icon :component="ReturnDownBackOutline" />
            </button>
            <button class="shuttle-btn" :class="{ active: tailFollow }" title="锚定末尾" @click="toggleTailFollow">
              <n-icon :component="PinOutline" />
            </button>
            <button class="shuttle-btn danger-btn" title="清空缓冲区" @click="$emit('clean-logs')">
              <n-icon :component="TrashOutline" />
            </button>
          </div>
        </div>

        <!-- 2. 原位交互式终端 PTY (新增强力特性) -->
        <div v-show="activeTab === 'terminal'" class="terminal-pane" @contextmenu.prevent.stop="handleTerminalContext">
          <div ref="terminalRef" class="pty-terminal-container"></div>
        </div>

        <!-- 3. 性能仪表盘 -->
        <div v-show="activeTab === 'stats'" class="stats-pane" @contextmenu.prevent.stop="handleStatsContext">
          <slot name="stats"></slot>
        </div>

        <!-- 4. 元数据详情 -->
        <div v-show="activeTab === 'inspect'" class="inspect-pane" @contextmenu.prevent.stop="handleInspectContext">
          <n-scrollbar style="height: 100%">
            <div class="inspect-grid">
              <div class="inspect-section-title">端口映射</div>
              <div class="inspect-card">
                <div v-if="ports.length > 0" class="port-list">
                  <div v-for="p in ports" :key="p.private_port" class="meta-row">
                    <span class="meta-key">宿主机</span>
                    <span class="meta-val highlight-val">{{ p.public_port ? `${p.ip || '0.0.0.0'}:${p.public_port}` : '-' }}</span>
                    <span class="meta-arrow">➔</span>
                    <span class="meta-key">容器内</span>
                    <span class="meta-val">{{ p.private_port }}/{{ p.type_ }}</span>
                  </div>
                </div>
                <div v-else class="empty-meta-text">无端口映射</div>
              </div>

              <div class="inspect-section-title">挂载卷</div>
              <div class="inspect-card">
                <div v-if="mounts.length > 0" class="mount-list">
                  <div v-for="m in mounts" :key="m.destination" class="mount-row">
                    <div class="mount-path"><span class="path-tag">源</span> {{ m.source }}</div>
                    <div class="mount-path"><span class="path-tag dest">宿</span> {{ m.destination }} <span class="mode-tag">({{ m.mode }})</span></div>
                  </div>
                </div>
                <div v-else class="empty-meta-text">无数据卷挂载</div>
              </div>

              <div class="inspect-section-title">环境变量</div>
              <div class="inspect-card">
                <div v-if="envs.length > 0" class="env-list">
                  <div v-for="env in envs" :key="env.key" class="env-row">
                    <span class="env-key">{{ env.key }}</span>
                    <span class="env-equal">=</span>
                    <span 
                      class="env-val" 
                      :class="{ masked: env.masked }"
                      @click="env.masked = !env.masked"
                      title="点击切换显示/遮罩"
                    >
                      {{ env.masked ? '••••••••' : env.value }}
                    </span>
                  </div>
                </div>
                <div v-else class="empty-meta-text">无环境变量</div>
              </div>
            </div>
          </n-scrollbar>
        </div>

        <!-- 5. 文件浏览 -->
        <div v-show="activeTab === 'files'" class="files-pane">
          <ContainerFileBrowser :container-id="container.id" :container-status="isUp" :active="activeTab === 'files'" />
        </div>
      </div>
    </template>

    <div v-else class="empty-state" key="empty-state-pane">
      <div class="empty-logo">
        <img src="/logo.png" alt="Vessel Logo" style="width: 80px; height: 80px; object-fit: contain;" />
      </div>
      <div class="empty-title">欢迎使用 Vessel</div>
      <div class="empty-sub">请在左侧选择一个项目或容器，开始高效微服务管控</div>
    </div>

    <!-- 右键菜单 -->
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="x"
      :y="y"
      :options="currentOptions"
      :show="showDropdown"
      :on-clickoutside="onClickOutside"
      @select="handleMenuSelect"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { NSpin, NScrollbar, NIcon, NDropdown, useMessage } from 'naive-ui'
import { 
  PlayOutline, 
  StopOutline, 
  TerminalOutline, 
  DocumentTextOutline, 
  BarChartOutline, 
  ClipboardOutline,
  TrashOutline,
  PersonOutline,
  ChevronUpOutline,
  ChevronDownOutline,
  ReturnDownBackOutline,
  PinOutline,
  DiscOutline,
  FlashOutline,
  FolderOutline,
  CopyOutline,
  RefreshOutline,
  PauseOutline
} from '@vicons/ionicons5'
import { useSettingsStore } from '../../store/settings'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { terminalApi } from '../../api/terminalApi'
import { EVT } from '../../api/events'
import ContainerFileBrowser from './ContainerFileBrowser.vue'
import { useContextMenu, renderIcon, MenuOption } from '../../hooks/useContextMenu'

const props = defineProps<{
  container: any | null
  loading: boolean
  logsList: string[]
}>()

const message = useMessage()
const settingsStore = useSettingsStore()
const emit = defineEmits(['start', 'stop', 'restart', 'clean-logs', 'toggle-stats', 'reset-stats'])

// 右键菜单支持
const { 
  showDropdown, 
  x, 
  y, 
  currentOptions,
  handleContextMenu, 
  onClickOutside 
} = useContextMenu()

const selectedLogText = ref('')
const selectedTerminalText = ref('')

const logMenuOptions = computed((): MenuOption[] => [
  {
    label: '复制选中文本',
    key: 'copy_selected_logs',
    icon: renderIcon(CopyOutline)
  },
  {
    label: '复制全部日志',
    key: 'copy_all_logs',
    icon: renderIcon(CopyOutline)
  },
  {
    label: wordWrap.value ? '关闭自动换行' : '开启自动换行',
    key: 'toggle_wrap',
    icon: renderIcon(ReturnDownBackOutline)
  },
  {
    label: tailFollow.value ? '取消锚定末尾' : '开启锚定末尾',
    key: 'toggle_follow',
    icon: renderIcon(PinOutline)
  },
  {
    type: 'divider',
    key: 'd1'
  },
  {
    label: '清空显示缓冲区',
    key: 'clear_logs',
    icon: renderIcon(TrashOutline)
  }
])

const terminalMenuOptions: MenuOption[] = [
  {
    label: '复制 (Copy)',
    key: 'copy_terminal',
    icon: renderIcon(CopyOutline)
  },
  {
    label: '粘贴 (Paste)',
    key: 'paste_terminal',
    icon: renderIcon(ClipboardOutline)
  },
  {
    type: 'divider',
    key: 'd1'
  },
  {
    label: '清屏 (Clear)',
    key: 'clear_terminal',
    icon: renderIcon(TrashOutline)
  },
  {
    label: '重置连接',
    key: 'reset_terminal',
    icon: renderIcon(RefreshOutline)
  }
]

const isStatsPaused = ref(false)
const statsMenuOptions = computed((): MenuOption[] => [
  {
    label: isStatsPaused.value ? '继续采集' : '暂停采集',
    key: 'toggle_stats',
    icon: renderIcon(isStatsPaused.value ? PlayOutline : PauseOutline)
  },
  {
    label: '重置图表',
    key: 'reset_stats',
    icon: renderIcon(RefreshOutline)
  }
])

const inspectMenuOptions: MenuOption[] = [
  {
    label: '复制完整 JSON',
    key: 'copy_inspect',
    icon: renderIcon(CopyOutline)
  },
  {
    label: '重新获取元数据',
    key: 'reload_inspect',
    icon: renderIcon(RefreshOutline)
  }
]

const handleLogsContext = (e: MouseEvent) => {
  selectedLogText.value = window.getSelection()?.toString() || ''
  handleContextMenu(e, logMenuOptions.value)
}

const handleTerminalContext = (e: MouseEvent) => {
  if (term) {
    selectedTerminalText.value = term.getSelection()
  } else {
    selectedTerminalText.value = ''
  }
  handleContextMenu(e, terminalMenuOptions)
}

const handleStatsContext = (e: MouseEvent) => {
  handleContextMenu(e, statsMenuOptions.value)
}

const handleInspectContext = (e: MouseEvent) => {
  handleContextMenu(e, inspectMenuOptions)
}

const handleMenuSelect = async (key: string) => {
  showDropdown.value = false
  if (key === 'clear_logs') {
    emit('clean-logs')
  } else if (key === 'copy_all_logs') {
    const text = props.logsList.join('\n')
    try {
      await navigator.clipboard.writeText(text)
      message.success('已复制全部日志到剪贴板')
    } catch (err) {
      message.error(`复制失败: ${err}`)
    }
  } else if (key === 'copy_selected_logs') {
    const text = selectedLogText.value
    if (text) {
      try {
        await navigator.clipboard.writeText(text)
        message.success('已复制选中日志')
      } catch (err) {
        message.error(`复制失败: ${err}`)
      }
    } else {
      message.warning('没有选中文本')
    }
  } else if (key === 'toggle_wrap') {
    wordWrap.value = !wordWrap.value
  } else if (key === 'toggle_follow') {
    toggleTailFollow()
  } else if (key === 'copy_terminal') {
    const text = selectedTerminalText.value
    if (text) {
      try {
        await navigator.clipboard.writeText(text)
        message.success('已复制终端内容')
      } catch (err) {
        message.error(`复制失败: ${err}`)
      }
    } else {
      message.warning('终端中没有选中文本')
    }
    setTimeout(() => term?.focus(), 10)
  } else if (key === 'paste_terminal') {
    try {
      const text = await navigator.clipboard.readText()
      if (text && terminalExecId) {
        const encoder = new TextEncoder()
        const bytes = Array.from(encoder.encode(text))
        await terminalApi.write(terminalExecId, bytes)
      }
    } catch (err) {
      message.error(`粘贴失败: ${err}`)
    }
    setTimeout(() => term?.focus(), 10)
  } else if (key === 'clear_terminal') {
    term?.clear()
    setTimeout(() => term?.focus(), 10)
  } else if (key === 'reset_terminal') {
    initTerminal()
  } else if (key === 'toggle_stats') {
    isStatsPaused.value = !isStatsPaused.value
    emit('toggle-stats', isStatsPaused.value)
  } else if (key === 'reset_stats') {
    emit('reset-stats')
  } else if (key === 'copy_inspect') {
    try {
      await navigator.clipboard.writeText(JSON.stringify(props.container, null, 2))
      message.success('已复制元数据 JSON 到剪贴板')
    } catch (err) {
      message.error(`复制失败: ${err}`)
    }
  } else if (key === 'reload_inspect') {
    emit('restart') // 借用 restart 或者触发外层的重新 fetchDetails
  }
}

const activeTab = ref('logs')

const getTerminalTheme = () => {
  const theme = settingsStore.theme
  if (theme === 'zed-gray') {
    return { 
      background: '#121212', 
      foreground: '#cccccc', 
      cursor: '#cccccc',
      selectionBackground: 'rgba(255, 255, 255, 0.3)' 
    }
  } else if (theme === 'light-apple') {
    return { 
      background: '#f5f5f7', 
      foreground: '#424245', 
      cursor: '#424245',
      selectionBackground: 'rgba(0, 112, 227, 0.3)' 
    }
  }
  return { 
    background: '#05070c', 
    foreground: '#cbd5e1', 
    cursor: '#cbd5e1',
    selectionBackground: 'rgba(255, 255, 255, 0.25)' 
  }
}

// 监听全局主题变化，实时同步给 xterm
watch(() => settingsStore.theme, () => {
  if (term) {
    term.options.theme = getTerminalTheme()
  }
})
const showUserMenu = ref(false)
const selectedUser = ref<'default' | 'root'>('default')

const tabs = [
  { label: '运行日志', value: 'logs', icon: DocumentTextOutline },
  { label: '文件浏览', value: 'files', icon: FolderOutline },
  { label: '交互终端', value: 'terminal', icon: TerminalOutline },
  { label: '性能仪表盘', value: 'stats', icon: BarChartOutline },
  { label: '元数据详情', value: 'inspect', icon: ClipboardOutline }
]

// 容器是否运行中
const isUp = computed(() => {
  if (!props.container) return false
  const status = (props.container.status || props.container.State?.Status || '').toLowerCase()
  return status === 'running' || status === 'up'
})

// 解析环境变量
const envs = ref<{ key: string; value: string; masked: boolean }[]>([])
const ports = computed(() => props.container?.ports || [])
const mounts = computed(() => props.container?.mounts || [])

const togglePower = () => {
  if (isUp.value) {
    emit('stop', props.container.id)
  } else {
    emit('start', props.container.id)
  }
}

// 终端身份切换
const selectUser = (user: 'default' | 'root') => {
  selectedUser.value = user
  showUserMenu.value = false
  if (activeTab.value === 'terminal') {
    initTerminal()
  }
}

// 触发原位终端切换
const triggerTerminal = () => {
  activeTab.value = 'terminal'
}

// --- Pty 原位终端实现 ---
const terminalRef = ref<HTMLElement | null>(null)
let term: Terminal | null = null
let fitAddon: FitAddon | null = null
let terminalExecId = ''
let termUnlisten: UnlistenFn | null = null

const initTerminal = async () => {
  destroyTerminal()
  if (!props.container || !props.container.id) return

  await nextTick()
  if (!terminalRef.value) return

  term = new Terminal({
    cursorBlink: true,
    theme: getTerminalTheme(),
    fontFamily: 'JetBrains Mono, Consolas, monospace',
    fontSize: 11,
    disableStdin: false,
    macOptionClickForcesSelection: true
  })

  fitAddon = new FitAddon()
  term.loadAddon(fitAddon)
  term.open(terminalRef.value)
  fitAddon.fit()
  term.focus()

  // 监听输入输入并写回 tauri backend
  term.onData(async (data) => {
    if (!terminalExecId) return
    const encoder = new TextEncoder()
    const bytes = Array.from(encoder.encode(data))
    try {
      await terminalApi.write(terminalExecId, bytes)
    } catch (e) {
      console.error('写入终端失败', e)
    }
  })

  // 监听 resize
  term.onResize(async (size) => {
    if (!terminalExecId) return
    try {
      await terminalApi.resize(terminalExecId, { cols: size.cols, rows: size.rows })
    } catch (e) {
      console.error('调整终端大小失败', e)
    }
  })

  try {
    terminalExecId = await terminalApi.create(
      props.container.id,
      selectedUser.value === 'default' ? 'default' : 'root'
    )

    termUnlisten = await listen<number[]>(EVT.containerTerminalStdout(terminalExecId), (event) => {
      const arr = new Uint8Array(event.payload)
      const str = new TextDecoder().decode(arr)
      term?.write(str)
    })

    setTimeout(() => {
      fitAddon?.fit()
    }, 150)
  } catch (e: any) {
    term.write(`\r\n创建终端失败: ${e}\r\n`)
  }
}

const destroyTerminal = () => {
  if (terminalExecId) {
    terminalApi.close(terminalExecId).catch(err => {
      console.error('关闭终端连接失败:', err)
    })
  }
  if (term) {
    term.dispose()
    term = null
  }
  if (fitAddon) {
    fitAddon = null
  }
  if (termUnlisten) {
    termUnlisten()
    termUnlisten = null
  }
  terminalExecId = ''
}

watch(() => props.container, (newVal) => {
  if (!newVal) {
    envs.value = []
    destroyTerminal()
    return
  }
  const rawEnv = newVal.env || newVal.Config?.Env || []
  envs.value = rawEnv.map((e: string) => {
    const idx = e.indexOf('=')
    const key = idx !== -1 ? e.substring(0, idx) : e
    const value = idx !== -1 ? e.substring(idx + 1) : ''
    const isSensitive = /pass|key|secret|token|auth/i.test(key)
    return { key, value, masked: isSensitive }
  })

  // 如果处于终端模式，当容器改变时，重新初始化终端
  if (activeTab.value === 'terminal') {
    nextTick(() => {
      initTerminal()
    })
  }
}, { immediate: true })

// 监听窗口大小改变时重绘终端
const handleWindowResize = () => {
  if (activeTab.value === 'terminal' && fitAddon) {
    fitAddon.fit()
  }
}

// 监听键盘按键：Ctrl + ` (反引号) 触发 logs / terminal 无感切换
const handleKeyDown = (e: KeyboardEvent) => {
  if (e.ctrlKey && e.key === '`') {
    e.preventDefault()
    if (activeTab.value === 'terminal') {
      activeTab.value = 'logs'
    } else {
      activeTab.value = 'terminal'
    }
  }
}

watch(activeTab, (newTab) => {
  if (newTab === 'terminal') {
    initTerminal()
  } else {
    destroyTerminal()
    if (newTab === 'logs') {
      nextTick(() => {
        updateScrollbar()
      })
    }
  }
})

// --- 文本查看器与自定义滚动条逻辑 ---
const logTextRef = ref<HTMLElement | null>(null)
const wordWrap = ref(true)
const tailFollow = ref(true)

const scrollHover = ref(false)
const isDragging = ref(false)
const thumbHeight = ref(40)
const thumbTop = ref(0)
let startY = 0
let startScrollTop = 0

const updateScrollbar = () => {
  const el = logTextRef.value
  if (!el) return
  const sh = el.scrollHeight
  const ch = el.clientHeight
  if (sh <= ch) {
    thumbHeight.value = ch
    thumbTop.value = 0
    return
  }
  const heightRatio = ch / sh
  thumbHeight.value = Math.max(20, ch * heightRatio)
  
  const scrollRatio = el.scrollTop / (sh - ch)
  thumbTop.value = scrollRatio * (ch - thumbHeight.value)
}

watch(() => props.logsList.length, () => {
  nextTick(() => {
    const el = logTextRef.value
    if (!el) return
    if (tailFollow.value) {
      el.scrollTop = el.scrollHeight - el.clientHeight
    }
    updateScrollbar()
  })
})

const closeUserMenu = () => {
  showUserMenu.value = false
}

let resizeObserver: ResizeObserver | null = null
onMounted(() => {
  document.addEventListener('click', closeUserMenu)
  window.addEventListener('resize', handleWindowResize)
  window.addEventListener('keydown', handleKeyDown)
  
  nextTick(() => {
    const el = logTextRef.value
    if (el) {
      resizeObserver = new ResizeObserver(() => {
        updateScrollbar()
      })
      resizeObserver.observe(el)
      updateScrollbar()
    }
  })
})

onUnmounted(() => {
  document.removeEventListener('click', closeUserMenu)
  if (resizeObserver) resizeObserver.disconnect()
  window.removeEventListener('resize', handleWindowResize)
  window.removeEventListener('keydown', handleKeyDown)
  destroyTerminal()
})

defineExpose({
  activeTab,
  selectedUser
})

const onLogScroll = () => {
  updateScrollbar()
}

let scrollTimer: any = null
const startScroll = (dir: 'up' | 'down') => {
  const el = logTextRef.value
  if (!el) return
  const step = dir === 'up' ? -80 : 80
  el.scrollBy({ top: step, behavior: 'smooth' })

  scrollTimer = setTimeout(() => {
    scrollTimer = setInterval(() => {
      el.scrollBy({ top: dir === 'up' ? -40 : 40 })
    }, 16)
  }, 350)
}

const stopScroll = () => {
  if (scrollTimer) {
    clearTimeout(scrollTimer)
    clearInterval(scrollTimer)
    scrollTimer = null
  }
}

const toggleTailFollow = () => {
  tailFollow.value = !tailFollow.value
  if (tailFollow.value) {
    const el = logTextRef.value
    if (el) el.scrollTop = el.scrollHeight - el.clientHeight
  }
}

const onScrollThumbMouseDown = (e: MouseEvent) => {
  isDragging.value = true
  startY = e.clientY
  const el = logTextRef.value
  if (el) startScrollTop = el.scrollTop
  document.addEventListener('mousemove', onScrollThumbMouseMove)
  document.addEventListener('mouseup', onScrollThumbMouseUp)
  e.preventDefault()
}

const onScrollThumbMouseMove = (e: MouseEvent) => {
  if (!isDragging.value) return
  const el = logTextRef.value
  if (!el) return
  const deltaY = e.clientY - startY
  const trackHeight = el.clientHeight - thumbHeight.value
  if (trackHeight <= 0) return
  
  const scrollRatio = deltaY / trackHeight
  const maxScroll = el.scrollHeight - el.clientHeight
  el.scrollTop = startScrollTop + scrollRatio * maxScroll
  
  if (deltaY < 0) {
    tailFollow.value = false
  }
}

const onScrollThumbMouseUp = () => {
  isDragging.value = false
  document.removeEventListener('mousemove', onScrollThumbMouseMove)
  document.removeEventListener('mouseup', onScrollThumbMouseUp)
}

const onScrollTrackMouseDown = (e: MouseEvent) => {
  const el = logTextRef.value
  if (!el) return
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  const clickY = e.clientY - rect.top
  const trackHeight = el.clientHeight
  const targetRatio = (clickY - thumbHeight.value / 2) / (trackHeight - thumbHeight.value)
  const maxScroll = el.scrollHeight - el.clientHeight
  el.scrollTop = Math.max(0, Math.min(maxScroll, targetRatio * maxScroll))
  tailFollow.value = false
}
</script>

<style scoped>
.container-detail {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
  background-color: var(--bg-main);
  overflow: hidden;
}

.loading-overlay {
  position: absolute;
  inset: 0;
  background-color: rgba(11, 15, 25, 0.7);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 10;
}

/* 顶层双行控制栏 */
.control-header-wrapper {
  height: 72px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  user-select: none;
}

/* 行 1: 选项卡 */
.tab-line-1 {
  height: 32px;
  display: flex;
  align-items: center;
  padding-left: 16px;
  gap: 20px;
}

.obs-tab {
  position: relative;
  height: 100%;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 16px;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-muted);
  cursor: pointer;
  transition: color 0.15s ease;
}

.tab-icon {
  opacity: 0.7;
}

.obs-tab.active .tab-icon {
  opacity: 1;
}

.obs-tab:hover {
  color: var(--text-title);
}

.obs-tab.active {
  color: var(--text-title);
  font-weight: 700;
}

.tab-indicator {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 2px;
  background-color: var(--brand-primary);
  transform: scaleX(0);
  transition: transform 0.15s ease;
}

.obs-tab.active .tab-indicator {
  transform: scaleX(1);
}

/* 行 2: 详情与操作 */
.meta-line-2 {
  height: 40px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
}

.meta-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #64748b;
}
.status-dot.running, .status-dot.up {
  background-color: #10b981;
}
.status-dot.exited, .status-dot.stopped {
  background-color: #64748b;
}
.status-dot.paused {
  background-color: var(--brand-warn);
}

.container-name {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-title);
}

.container-id {
  font-size: 11px;
  font-family: monospace;
  color: var(--text-muted);
}

.vertical-divider {
  width: 1px;
  height: 14px;
  background-color: var(--border-color);
}

.meta-badge {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.image-badge {
  background-color: rgba(255, 255, 255, 0.05);
  color: var(--text-body);
}

.project-badge {
  background-color: rgba(255, 255, 255, 0.03);
  color: var(--text-muted);
}

.meta-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 按钮样式 */
.action-btn {
  height: 26px;
  width: 72px;
  background: transparent;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.start-border {
  border: 1px solid var(--brand-primary);
  color: var(--brand-primary);
}
.start-border:hover {
  background-color: rgba(16, 185, 129, 0.1);
}

.stop-border {
  border: 1px solid var(--brand-danger);
  color: var(--brand-danger);
}
.stop-border:hover {
  background-color: rgba(239, 68, 68, 0.1);
}

/* 组合分裂按钮 */
.split-btn-group {
  position: relative;
  display: flex;
  height: 26px;
  z-index: 50;
}

.split-main-btn {
  width: 80px;
  height: 100%;
  background-color: rgba(255, 255, 255, 0.08);
  border: 1px solid var(--border-color);
  border-right: none;
  border-radius: 4px 0 0 4px;
  color: var(--text-title);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
.split-main-btn:hover {
  background-color: rgba(255, 255, 255, 0.12);
}
.split-main-btn.active {
  border-color: var(--brand-primary);
  color: var(--brand-primary);
  background-color: rgba(16, 185, 129, 0.05);
}

.split-arrow-btn {
  width: 24px;
  height: 100%;
  background-color: rgba(255, 255, 255, 0.08);
  border: 1px solid var(--border-color);
  border-radius: 0 4px 4px 0;
  color: var(--text-muted);
  font-size: 9px;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
  display: flex;
  align-items: center;
  justify-content: center;
}
.split-arrow-btn:hover {
  background-color: rgba(255, 255, 255, 0.12);
}
.split-arrow-btn.highlighted {
  color: #38bdf8;
  border-color: rgba(56, 189, 248, 0.3);
}

/* 终端身份下拉菜单 */
.user-dropdown-menu {
  position: absolute;
  right: 0;
  top: 28px;
  width: 140px;
  background-color: var(--bg-main);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 4px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.user-option {
  height: 26px;
  display: flex;
  align-items: center;
  padding: 0 8px;
  font-size: 10px;
  border-radius: 3px;
  cursor: pointer;
  color: var(--text-body);
  transition: background-color 0.15s ease;
}

.user-option:hover {
  background-color: var(--bg-active);
  color: var(--text-title);
}

.user-option.active {
  background-color: rgba(255, 255, 255, 0.06);
  font-weight: 600;
}

/* 下方主内容区 */
.obs-content {
  flex: 1;
  min-height: 0;
  position: relative;
}

/* 1. 运行日志面板 */
.logs-pane {
  display: flex;
  height: 100%;
}

.logs-text-area {
  flex: 1;
  height: 100%;
  overflow-y: scroll;
  background-color: var(--bg-terminal);
  color: var(--text-terminal);
  padding: 12px 12px 24px 12px;
  font-size: 11px;
  line-height: 1.5;
  white-space: pre;
  word-wrap: normal;
  overflow-x: auto;
}

.logs-text-area::-webkit-scrollbar {
  display: none;
}

.logs-text-area.word-wrap {
  white-space: pre-wrap;
  word-break: break-all;
}

.log-line {
  margin-bottom: 2px;
}

.empty-logs-text {
  color: var(--text-muted);
  font-style: italic;
  margin-top: 12px;
  text-align: center;
}

/* 2. 原位交互式终端面板 */
.terminal-pane {
  height: 100%;
  background-color: var(--bg-terminal);
  padding: 8px;
}

.pty-terminal-container {
  width: 100%;
  height: 100%;
  background-color: var(--bg-terminal);
}

/* 中栏: 独立垂直滚动条 */
.custom-scrollbar-track {
  width: 8px;
  height: 100%;
  background-color: rgba(255, 255, 255, 0.01);
  position: relative;
  cursor: pointer;
  border-left: 1px solid rgba(255, 255, 255, 0.02);
}

.custom-scrollbar-thumb {
  position: absolute;
  width: 4px;
  left: 2px;
  border-radius: 2px;
  background-color: var(--text-muted);
  transition: opacity 0.15s ease;
}

/* 右栏: 极窄飞梭动作纽扣控制带 */
.shuttle-controls {
  width: 32px;
  height: 100%;
  background-color: var(--bg-sidebar);
  border-left: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 8px;
  gap: 8px;
  user-select: none;
}

.shuttle-btn {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  background-color: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 10px;
  color: var(--text-body);
  transition: all 0.15s ease;
  outline: none;
}

.shuttle-btn:hover {
  background-color: var(--bg-active);
  color: var(--text-title);
}

.shuttle-btn.active {
  background-color: rgba(56, 189, 248, 0.15);
  border-color: rgba(56, 189, 248, 0.3);
  color: #38bdf8;
}

.shuttle-btn.danger-btn:hover {
  background-color: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.3);
  color: var(--brand-danger);
}

/* 3. 性能面板 */
.stats-pane {
  height: 100%;
  overflow: hidden;
}

/* 4. Inspect 面板 */
.inspect-pane {
  height: 100%;
  padding: 16px;
}

.files-pane {
  height: 100%;
  overflow: hidden;
}

.inspect-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.inspect-section-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-title);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.inspect-card {
  background-color: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 12px;
}

.meta-row {
  display: flex;
  align-items: center;
  font-size: 11px;
  gap: 8px;
  height: 24px;
}

.meta-key {
  color: var(--text-muted);
}

.meta-val {
  color: var(--text-body);
  font-family: monospace;
}

.highlight-val {
  color: var(--brand-primary);
  font-weight: 600;
}

.meta-arrow {
  color: var(--text-muted);
}

.empty-meta-text {
  font-size: 11px;
  color: var(--text-muted);
  font-style: italic;
}

.mount-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  border-bottom: 1px solid var(--border-color);
  padding: 6px 0;
}
.mount-row:last-child {
  border-bottom: none;
}

.mount-path {
  font-size: 11px;
  font-family: monospace;
  color: var(--text-body);
}

.path-tag {
  display: inline-block;
  font-size: 9px;
  font-weight: 700;
  padding: 1px 4px;
  border-radius: 2px;
  background-color: rgba(255, 255, 255, 0.06);
  color: var(--text-muted);
  margin-right: 6px;
}

.path-tag.dest {
  background-color: rgba(16, 185, 129, 0.1);
  color: var(--brand-primary);
}

.mode-tag {
  color: var(--text-muted);
}

.env-row {
  display: flex;
  font-size: 11px;
  font-family: monospace;
  height: 22px;
  align-items: center;
}

.env-key {
  color: #38bdf8;
  font-weight: 600;
}

.env-equal {
  color: var(--text-muted);
  margin: 0 6px;
}

.env-val {
  color: var(--text-body);
  cursor: pointer;
}

.env-val.masked {
  color: var(--text-muted);
  letter-spacing: 2px;
}

/* 空状态 */
.empty-state {
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
  user-select: none;
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
