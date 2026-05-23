<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useImageStore } from '../store/image'
import { Command } from '@tauri-apps/plugin-shell'
import { useSettingsStore } from '../store/settings'
import {
  NButton,
  NDropdown,
  NIcon,
  NModal,
  NScrollbar,
  NSpace,
  NTag,
  NInput,
  useMessage,
  NAutoComplete,
  NProgress
} from 'naive-ui'
import {
  Star,
  ShieldCheckmark,
  PlayOutline
} from '@vicons/ionicons5'
import { useTaskStore } from '../store/task'

const router = useRouter()
const imageStore = useImageStore()
const settingsStore = useSettingsStore()
const taskStore = useTaskStore()
const message = useMessage()

// 防止静态检测对 h 函数里引用过的组件报未使用警告
const _unused = [NIcon, NSpace, NTag, Star, ShieldCheckmark, PlayOutline]
if (_unused.length < 0) console.log(_unused)

// --- 状态控制 ---
const selectedId = ref<string | null>(null)
const selectedDetails = ref<any>(null)
const loadingDetails = ref(false)
const pullImageName = ref('')
const activeTab = ref('pull') // 默认：pull 🔍 镜像仓库

const localSearchQuery = ref('') // 本地镜像检索输入词

// 运行镜像弹窗
const showRunModal = ref(false)
const runContainerName = ref('')
const runPortMapping = ref('8080:80')
const runningImage = ref('')

// Hub 镜像详情查看
const selectedHubImage = ref<any>(null)

// 字符树飞梭控制
const wrapLayers = ref(true)

// 本地镜像资产清单的动态前端检索过滤
const filteredImages = computed(() => {
  let list = imageStore.images
  if (!localSearchQuery.value) return list
  const q = localSearchQuery.value.toLowerCase()
  return list.filter(item => {
    const firstTag = item.tags?.[0] || ''
    // 确保使用 .includes
    return firstTag.toLowerCase().includes(q) || item.id.toLowerCase().includes(q)
  })
})

// 监听当前正在拉取的后台任务
const activePullTasks = computed(() => {
  return taskStore.tasks.filter(t => t.name.startsWith('拉取镜像:') && (t.status === 'running' || t.status === 'error' || t.status === 'success'))
})

const viewHubImage = (image: any) => {
  selectedHubImage.value = image
}

const removePullTask = (id: string) => {
  taskStore.removeTask(id)
}

// 当清空所选本地镜像（如删除镜像）时，详情区强制切回全局镜像仓库视图
watch(selectedDetails, (newVal) => {
  if (!newVal) {
    activeTab.value = 'pull'
  }
})

// 监听搜索框输入，清空时重置结果
watch(pullImageName, (newVal) => {
  if (!newVal) {
    imageStore.clearSearchResults()
    selectedHubImage.value = null
  }
})

const onSelect = async (id: string) => {
  selectedId.value = id
  loadingDetails.value = true
  try {
    const details = await imageStore.inspectImage(id)
    await imageStore.fetchImageHistory(id)
    selectedDetails.value = details
    if (activeTab.value === 'pull' && details) {
      activeTab.value = 'layers'
    }
  } catch (err) {
    message.error('获取镜像详情失败: ' + err)
  } finally {
    loadingDetails.value = false
  }
}

const formatBytes = (bytes: number, decimals = 2) => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

const timeAgo = (val: number | string) => {
  if (!val) return ''
  const timestamp = typeof val === 'number' ? val : Math.floor(new Date(val).getTime() / 1000)
  const seconds = Math.floor(Date.now() / 1000 - timestamp)
  let interval = seconds / 31536000
  if (interval > 1) return Math.floor(interval) + ' 年前'
  interval = seconds / 2592000
  if (interval > 1) return Math.floor(interval) + ' 个月前'
  interval = seconds / 86400
  if (interval > 1) return Math.floor(interval) + ' 天前'
  interval = seconds / 3600
  if (interval > 1) return Math.floor(interval) + ' 小时前'
  interval = seconds / 60
  if (interval > 1) return Math.floor(interval) + ' 分钟前'
  return Math.floor(seconds) + ' 秒前'
}

const isDangling = (item: any) => {
  const firstTag = item.tags?.[0]
  // 确保使用 .includes
  return !firstTag || firstTag.includes('<none>')
}

const pruneDanglingImages = async () => {
  const dangling = imageStore.images.filter(isDangling)
  if (dangling.length === 0) {
    message.info('未检测到任何虚悬 (dangling) 垃圾镜像！')
    return
  }
  
  message.warning(`正在深度清理 ${dangling.length} 个虚悬镜像...`)
  try {
    await Promise.all(dangling.map(img => imageStore.removeImage(img.id)))
    message.success('清理完毕！成功释放磁盘空间。')
    selectedId.value = null
    selectedDetails.value = null
    await imageStore.fetchImages()
  } catch (err: any) {
    message.error(`部分清理失败: ${err}`)
  }
}

const handleDelete = async (id: string) => {
  try {
    await imageStore.removeImage(id)
    message.success('镜像已安全销毁')
    if (selectedId.value === id) {
      selectedId.value = null
      selectedDetails.value = null
    }
    await imageStore.fetchImages()
  } catch (err) {
    message.error('删除镜像失败: ' + err)
  }
}

const openRunModal = (imgTag: string) => {
  runningImage.value = imgTag
  runContainerName.value = 'test-' + imgTag.split(':')[0].replace(/[^a-zA-Z0-9]/g, '-')
  showRunModal.value = true
}

const handleRunImage = async () => {
  showRunModal.value = false
  message.info(`正在拉起容器 "${runContainerName.value}"...`)

  try {
    let cmdArgs = ['run', '-d', '--name', runContainerName.value]
    if (runPortMapping.value) {
      cmdArgs.push('-p', runPortMapping.value)
    }
    cmdArgs.push(runningImage.value)

    let execCmd = 'docker'
    let finalArgs = cmdArgs

    if (settingsStore.connectionMode === 'wsl') {
      execCmd = 'wsl'
      let wslArgs = ['-d', settingsStore.wslDistro || 'Ubuntu', 'docker']
      wslArgs.push(...cmdArgs)
      finalArgs = wslArgs
    }

    const command = Command.create(execCmd, finalArgs)
    const out = await command.execute()
    
    if (out.code === 0) {
      message.success('容器拉起成功！已成功侧载。')
      router.push({ name: 'containers' })
    } else {
      throw new Error(out.stderr || '未知错误')
    }
  } catch (e: any) {
    message.error('拉起失败: ' + e.message)
  }
}

// --- Hub 镜像拉取 ---
let searchTimer: any = null
const handleSearchInput = (val: string) => {
  pullImageName.value = val
  if (searchTimer) clearTimeout(searchTimer)
  if (!val) {
    imageStore.clearSearchResults()
    selectedHubImage.value = null
    return
  }
  searchTimer = setTimeout(() => {
    handleSearch(val)
  }, 300)
}

const handleSelectPull = (val: string) => {
  pullImageName.value = val
  const option = autoCompleteOptions.value.find(o => o.value === val)
  if (option) {
    selectedHubImage.value = option
  }
}

const handlePull = async (name?: string) => {
  const targetName = typeof name === 'string' ? name : pullImageName.value
  if (!targetName) {
    message.warning('请输入镜像名称')
    return
  }
  try {
    message.warning(`正在拉取镜像 ${targetName}...`)
    await imageStore.pullImage(targetName)
    message.info('镜像后台拉取中，请稍候在侧边栏刷新列表查看')
  } catch (err) {
    message.error('拉取失败: ' + err)
  }
}

const handleSearch = async (query: string) => {
  if (query.length > 1) {
    await imageStore.searchImages(query)
  }
}

const autoCompleteOptions = computed(() => {
  return imageStore.searchResults.map(res => ({
    label: res.name,
    value: res.name,
    description: res.description,
    is_official: res.is_official,
    star_count: res.star_count
  }))
})

// --- 右键菜单 ---
const showMenu = ref(false)
const x = ref(0)
const y = ref(0)
const menuTarget = ref<any>(null)

const menuOptions = [
  { label: '🔍 详情', key: 'detail' },
  { label: '🚀 快速运行 (Run)', key: 'run' },
  { label: '🗑️ 彻底删除', key: 'delete' }
]

const handleContextMenu = (e: MouseEvent, item: any) => {
  e.preventDefault()
  showMenu.value = false
  nextTick(() => {
    x.value = e.clientX
    y.value = e.clientY
    menuTarget.value = item
    showMenu.value = true
  })
}

const handleMenuSelect = (key: string) => {
  showMenu.value = false
  if (!menuTarget.value) return
  
  const id = menuTarget.value.id
  const tag = menuTarget.value.tags?.[0] || id

  switch (key) {
    case 'detail':
      onSelect(id)
      break
    case 'run':
      openRunModal(tag)
      break
    case 'delete':
      handleDelete(id)
      break
  }
}

onMounted(() => {
  imageStore.fetchImages()
})
</script>

<template>
  <div class="images-view">
    <!-- 左侧资产清单 -->
    <div class="list-column">
      <div class="header-tools-v3">
        <input 
          v-model="localSearchQuery" 
          class="local-search-input" 
          placeholder="过滤本地镜像..." 
        />
        <button class="prune-btn-v3" @click="pruneDanglingImages" title="清理无标签垃圾镜像">
          🧼 清理
        </button>
      </div>

      <n-scrollbar class="list-scroll-box">
        <div 
          v-for="item in filteredImages" 
          :key="item.id" 
          class="image-item-row"
          :class="{ active: selectedId === item.id, dangling: isDangling(item) }"
          @click="onSelect(item.id)"
          @contextmenu="handleContextMenu($event, item)"
        >
          <div class="item-left-meta">
            <div class="item-tag-title">
              <span v-if="isDangling(item)" class="dangling-label">🆔 (虚悬镜像)</span>
              <span v-else class="normal-tag-label">{{ item.tags?.[0] }}</span>
            </div>
            <div class="item-sub-meta">
              {{ (item.id.split(':')[1] || item.id).substring(0, 12) }} · {{ formatBytes(item.size) }} · {{ timeAgo(item.created) }}
            </div>
          </div>
        </div>
      </n-scrollbar>
    </div>

    <!-- 右侧详情与镜像仓库 -->
    <div class="detail-column">
      <!-- 顶层控制栏 -->
      <div class="detail-header-wrapper" :class="{ 'no-selected': !selectedDetails }">
        <div class="tab-line-1">
          <div 
            class="obs-tab" 
            :class="{ active: activeTab === 'pull' }" 
            @click="activeTab = 'pull'"
          >
            <span>🔍 镜像仓库</span>
            <div class="tab-indicator"></div>
          </div>
          <div 
            v-if="selectedDetails"
            class="obs-tab" 
            :class="{ active: activeTab === 'layers' }" 
            @click="activeTab = 'layers'"
          >
            <span>🔀 层级结构</span>
            <div class="tab-indicator"></div>
          </div>
          <div 
            v-if="selectedDetails"
            class="obs-tab" 
            :class="{ active: activeTab === 'inspect' }" 
            @click="activeTab = 'inspect'"
          >
            <span>📋 镜像详情 (Inspect)</span>
            <div class="tab-indicator"></div>
          </div>
        </div>

        <div v-if="selectedDetails" class="meta-line-2">
          <div class="meta-left">
            <span class="image-name-title">{{ selectedDetails.tags?.[0] || '(无标签)' }}</span>
            <span class="image-meta-sub">{{ (selectedDetails.id.split(':')[1] || selectedDetails.id).substring(0, 12) }}</span>
            <div class="vertical-divider"></div>
            <span class="badge size-badge">{{ formatBytes(selectedDetails.size || 0) }}</span>
            <span class="badge os-badge">🖥️ {{ selectedDetails.os || 'linux' }}/{{ selectedDetails.architecture || 'amd64' }}</span>
          </div>

          <div class="meta-right">
            <button class="run-image-gold-btn" @click="openRunModal(selectedDetails.tags?.[0] || selectedDetails.id)">
              🚀 运行 (Run)
            </button>
            <button class="delete-btn" @click="handleDelete(selectedDetails.id)">
              🗑️ 删除
            </button>
          </div>
        </div>
      </div>

      <!-- 主内容区 -->
      <div class="detail-content-area">
        <!-- 1. 🔍 镜像仓库 -->
        <div v-show="activeTab === 'pull'" class="pull-pane">
          <div class="search-pull-box-v2">
            <div class="search-title">探索 Docker Hub 官方与社区镜像</div>
            <div class="search-input-group">
              <n-auto-complete
                v-model:value="pullImageName"
                :options="autoCompleteOptions"
                placeholder="输入即联想, 回车或点击搜索, 如 mysql..."
                @input="handleSearchInput"
                @keyup.enter="handleSearch(pullImageName)"
                @select="handleSelectPull"
                class="search-autocomplete"
                :clearable="true"
              />
              <n-button type="primary" secondary @click="handleSearch(pullImageName)">
                🔍 搜索
              </n-button>
            </div>

            <!-- 选中镜像详情 -->
            <div v-if="selectedHubImage" class="hub-image-detail-card">
              <div class="hub-detail-header">
                <div class="hub-detail-name">
                  🐧 {{ selectedHubImage.name || selectedHubImage.label }}
                  <n-tag v-if="selectedHubImage.is_official" type="success" size="tiny" round style="margin-left: 8px">官方</n-tag>
                </div>
                <div class="hub-detail-stats">
                  <span class="stars">⭐ {{ selectedHubImage.star_count }}</span>
                </div>
              </div>
              <div class="hub-detail-desc">{{ selectedHubImage.description }}</div>
              <div class="hub-detail-actions">
                <n-button type="primary" :loading="imageStore.pulling" @click="handlePull(selectedHubImage.name || selectedHubImage.label)">
                  📥 立即拉取镜像
                </n-button>
                <n-button @click="selectedHubImage = null">取消选择</n-button>
              </div>
            </div>

            <!-- 搜索列表 -->
            <div v-else-if="imageStore.searchResults.length > 0" class="search-results-radar">
              <div class="radar-title">🔍 匹配的 Hub 资产 (点击查看详情):</div>
              <div class="radar-list">
                <div 
                  v-for="res in imageStore.searchResults.slice(0, 10)" 
                  :key="res.name" 
                  class="radar-item"
                  @click="viewHubImage(res)"
                >
                  <div class="radar-item-left">
                    <div class="radar-item-name">
                      🐧 {{ res.name }}
                      <n-tag v-if="res.is_official" type="success" size="tiny" round style="margin-left: 4px">官方</n-tag>
                    </div>
                    <div class="radar-item-desc">{{ res.description }}</div>
                  </div>
                  <div class="radar-item-right">
                    <span class="stars">⭐ {{ res.star_count }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- 任务进度 -->
            <div v-if="activePullTasks.length > 0" class="active-pull-panel">
              <div class="panel-title">⚡ 拉取任务状态:</div>
              <div v-for="task in activePullTasks" :key="task.id" class="pull-task-card">
                <div class="task-info">
                  <span class="task-name">{{ task.name }}</span>
                  <div class="task-right">
                    <span class="task-status" :class="task.status">
                      {{ task.status === 'running' ? '进行中' : (task.status === 'success' ? '已成功' : '失败') }}
                    </span>
                    <button class="task-remove-btn" @click="removePullTask(task.id)">✕</button>
                  </div>
                </div>
                <n-progress 
                  type="line" 
                  :percentage="task.progress" 
                  :indicator-placement="'inside'" 
                  processing
                  :status="task.status === 'error' ? 'error' : 'success'"
                  style="margin: 6px 0;"
                />
                <div v-if="task.logs.length > 0" class="task-log-preview">
                  {{ task.logs[task.logs.length - 1] }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 2. 🔀 层级结构 -->
        <div v-if="selectedDetails" v-show="activeTab === 'layers'" class="layers-pane">
          <div class="layers-text-stream" :class="{ 'word-wrap': wrapLayers }">
            <div class="history-tree-title">镜像构建历史字节流:</div>
            <div v-for="(layer, idx) in imageStore.imageHistory" :key="idx" class="history-layer-row">
              <span class="layer-bullet">├─</span>
              <span class="layer-id" :class="{ missing: layer.id === '<missing>' }">
                {{ layer.id === '<missing>' ? '📦 [Layer Missing]' : `📦 Layer: ${layer.id.substring(0, 12)}` }}
              </span>
              <span class="layer-size-badge">{{ formatBytes(layer.size) }}</span>
              <span class="layer-cmd-text">➔ {{ layer.created_by }}</span>
            </div>
          </div>
          <div class="shuttle-control-bar">
            <button class="shuttle-btn" :class="{ active: wrapLayers }" title="自动换行" @click="wrapLayers = !wrapLayers">↩️</button>
          </div>
        </div>

        <!-- 3. 📋 镜像详情 -->
        <div v-if="selectedDetails" v-show="activeTab === 'inspect'" class="inspect-pane">
          <n-scrollbar style="height: 100%">
            <div class="inspect-card-box">
              <div class="inspect-row"><span class="key">架构:</span> <span class="val">{{ selectedDetails.architecture }}</span></div>
              <div class="inspect-row"><span class="key">系统:</span> <span class="val">{{ selectedDetails.os }}</span></div>
              <div class="inspect-row"><span class="key">作者:</span> <span class="val">{{ selectedDetails.author || 'N/A' }}</span></div>
              <div class="inspect-row"><span class="key">Docker版本:</span> <span class="val">{{ selectedDetails.docker_version || 'N/A' }}</span></div>
              <div class="inspect-row"><span class="key">创建时间:</span> <span class="val">{{ new Date(selectedDetails.created).toLocaleString() }}</span></div>
              
              <div class="inspect-divider"></div>
              <div class="section-title">启动配置 (CMD & Entrypoint)</div>
              <div class="inspect-row">
                <span class="key">Entrypoint:</span>
                <span class="val cmd-box">{{ selectedDetails.entrypoint && selectedDetails.entrypoint.length > 0 ? JSON.stringify(selectedDetails.entrypoint) : '[]' }}</span>
              </div>
              <div class="inspect-row">
                <span class="key">CMD:</span>
                <span class="val cmd-box">{{ selectedDetails.cmd && selectedDetails.cmd.length > 0 ? JSON.stringify(selectedDetails.cmd) : '[]' }}</span>
              </div>
              
              <div class="inspect-divider"></div>
              <div class="section-title">默认暴露端口 (Exposed Ports)</div>
              <div class="config-list-box">
                <div v-if="selectedDetails.exposed_ports && selectedDetails.exposed_ports.length > 0" class="ports-grid">
                  <n-tag 
                    v-for="p in selectedDetails.exposed_ports" 
                    :key="p" 
                    type="success" 
                    size="small"
                    style="margin-right: 6px; margin-bottom: 6px;"
                  >
                    ⚡ {{ p }}
                  </n-tag>
                </div>
                <div v-else class="empty-text">无对外暴露端口</div>
              </div>

              <div class="inspect-divider"></div>
              <div class="section-title">配置环境变量</div>
              <div class="config-list-box">
                <div v-for="e in selectedDetails.env" :key="e" class="config-item-row">
                  <code>{{ e }}</code>
                </div>
                <div v-if="!selectedDetails.env || selectedDetails.env.length === 0" class="empty-text">无环境变量</div>
              </div>
            </div>
          </n-scrollbar>
        </div>
      </div>
    </div>
  </div>

  <!-- 运行弹窗 -->
  <n-modal
    v-model:show="showRunModal"
    preset="card"
    title="🚀 运行新容器"
    style="width: 450px"
  >
    <div class="run-modal-body">
      <div class="field-title">容器名称</div>
      <n-input v-model:value="runContainerName" placeholder="例如: my-nginx" />
      <div class="field-title">端口映射</div>
      <n-input v-model:value="runPortMapping" placeholder="例如: 8080:80" />
      <div style="margin-top: 12px; font-size: 11px; color: var(--text-muted)">
        镜像: {{ runningImage }}
      </div>
    </div>
    <template #footer>
      <div class="warning-modal-footer">
        <n-button @click="showRunModal = false">取消</n-button>
        <n-button type="primary" @click="handleRunImage">立即拉起</n-button>
      </div>
    </template>
  </n-modal>

  <!-- 右键菜单 -->
  <n-dropdown
    :on-clickoutside="() => showMenu = false"
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
.images-view {
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

.header-tools-v3 {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  gap: 8px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.local-search-input {
  flex: 1;
  min-width: 0;
  height: 24px;
  background-color: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-title);
  font-size: 10px;
  padding: 0 6px;
  outline: none;
}
.local-search-input::placeholder {
  color: var(--text-muted);
}

.prune-btn-v3 {
  height: 22px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-muted);
  font-size: 10px;
  padding: 0 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
  white-space: nowrap;
}
.prune-btn-v3:hover {
  background-color: rgba(239, 68, 68, 0.1);
  border-color: var(--brand-danger);
  color: var(--brand-danger);
}

.list-scroll-box {
  flex: 1;
}

.image-item-row {
  height: 36px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  cursor: pointer;
  color: var(--text-body);
  transition: all 0.15s ease;
  border-bottom: 1px solid var(--border-color);
}
.image-item-row:hover {
  background-color: var(--bg-hover);
  color: var(--text-title);
}
.image-item-row.active {
  background-color: var(--bg-active) !important;
  color: var(--text-title);
  font-weight: 600;
}
.image-item-row.dangling {
  opacity: 0.6;
}

.item-left-meta {
  display: flex;
  flex-direction: column;
  justify-content: center;
  overflow: hidden;
  width: 100%;
}

.item-tag-title {
  font-size: 11px;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dangling-label {
  color: var(--text-muted);
}

.item-sub-meta {
  font-size: 9px;
  color: var(--text-muted);
  margin-top: 1px;
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

.detail-header-wrapper {
  height: 72px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}
.detail-header-wrapper.no-selected {
  height: 32px;
}

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
  font-size: 11px;
  font-weight: 500;
  color: var(--text-muted);
  cursor: pointer;
  transition: color 0.15s ease;
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
  min-width: 0;
}

.image-name-title {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-title);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 250px;
}

.image-meta-sub {
  font-size: 11px;
  font-family: monospace;
  color: var(--text-muted);
}

.vertical-divider {
  width: 1px;
  height: 14px;
  background-color: var(--border-color);
}

.badge {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  background-color: rgba(255, 255, 255, 0.04);
  color: var(--text-body);
}

.meta-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.run-image-gold-btn {
  height: 26px;
  padding: 0 12px;
  background: transparent;
  border: 1px solid var(--brand-primary);
  border-radius: 4px;
  color: var(--brand-primary);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.run-image-gold-btn:hover {
  background-color: rgba(16, 185, 129, 0.1);
}

.delete-btn {
  height: 26px;
  padding: 0 12px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.delete-btn:hover {
  border-color: var(--brand-danger);
  color: var(--brand-danger);
  background-color: rgba(239, 68, 68, 0.05);
}

.detail-content-area {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.pull-pane {
  padding: 24px;
  height: 100%;
}

.search-pull-box-v2 {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 600px;
}

.search-title {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-title);
}

.search-input-group {
  display: flex;
  gap: 8px;
}
.search-autocomplete {
  flex: 1;
}

.search-results-radar {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.radar-title,
.panel-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-title);
}

.radar-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 6px;
}

.radar-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.15s ease;
}
.radar-item:hover {
  background-color: var(--bg-active);
}

.radar-item-left {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.radar-item-name {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-title);
  display: flex;
  align-items: center;
}

.radar-item-desc {
  font-size: 9px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.radar-item-right .stars {
  font-size: 10px;
  color: #f59e0b;
  font-weight: 600;
}

.active-pull-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 12px;
}

.pull-task-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 10px;
}

.task-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
}
.task-name {
  color: var(--text-title);
  font-weight: 600;
}
.task-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
.task-status.running {
  color: var(--brand-primary);
}
.task-status.success {
  color: var(--brand-primary);
}
.task-status.error {
  color: var(--brand-danger);
}

.task-remove-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 12px;
  padding: 2px 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  border-radius: 3px;
}
.task-remove-btn:hover {
  background-color: rgba(239, 68, 68, 0.1);
  color: var(--brand-danger);
}

.task-log-preview {
  font-family: monospace;
  font-size: 9px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  background-color: rgba(0, 0, 0, 0.2);
  padding: 2px 6px;
  border-radius: 2px;
}

/* Hub 详情卡片 */
.hub-image-detail-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.hub-detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.hub-detail-name {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-title);
  display: flex;
  align-items: center;
}
.hub-detail-stats .stars {
  font-size: 12px;
  color: #f59e0b;
  font-weight: 600;
}
.hub-detail-desc {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.5;
}
.hub-detail-actions {
  display: flex;
  gap: 12px;
}

.layers-pane {
  display: flex;
  height: 100%;
}

.layers-text-stream {
  flex: 1;
  height: 100%;
  overflow-y: auto;
  background-color: #05070c;
  color: var(--text-body);
  padding: 16px;
  font-size: 11px;
  line-height: 1.5;
}
.layers-text-stream.word-wrap {
  word-break: break-all;
}

.history-tree-title {
  color: var(--text-muted);
  font-weight: 700;
  margin-bottom: 8px;
}

.history-layer-row {
  margin-bottom: 6px;
  display: flex;
  align-items: flex-start;
  gap: 6px;
}

.layer-bullet {
  color: var(--text-muted);
  font-family: monospace;
}

.layer-id {
  color: #38bdf8;
  font-family: monospace;
  font-weight: 600;
  flex-shrink: 0;
}
.layer-id.missing {
  color: var(--text-muted);
}

.layer-size-badge {
  background-color: var(--bg-hover);
  color: var(--text-title);
  padding: 1px 4px;
  border-radius: 2px;
  font-size: 9px;
  font-family: monospace;
  flex-shrink: 0;
}

.layer-cmd-text {
  color: var(--text-body);
  font-family: monospace;
  word-break: break-all;
}

.shuttle-control-bar {
  width: 32px;
  height: 100%;
  background-color: var(--bg-sidebar);
  border-left: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 8px;
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
  transition: all 0.15s ease;
}
.shuttle-btn.active {
  background-color: rgba(56, 189, 248, 0.15);
  border-color: rgba(56, 189, 248, 0.3);
  color: #38bdf8;
}

.inspect-pane {
  padding: 16px;
  height: 100%;
}

.inspect-card-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 11px;
}

.inspect-row {
  display: flex;
  gap: 8px;
}

.inspect-row .key {
  color: var(--text-muted);
  width: 90px;
}
.inspect-row .val {
  color: var(--text-body);
  font-family: monospace;
}

.inspect-divider {
  height: 1px;
  background-color: var(--border-color);
  margin: 12px 0;
}

.section-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-title);
  margin-bottom: 6px;
}

.config-list-box {
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 8px;
}

.config-item-row {
  font-family: monospace;
  padding: 2px 0;
  word-break: break-all;
}

.cmd-box {
  background-color: var(--bg-hover);
  padding: 2px 6px;
  border-radius: 3px;
  word-break: break-all;
}

.empty-text {
  color: var(--text-muted);
  font-style: italic;
}

.run-modal-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
}

.warning-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>
