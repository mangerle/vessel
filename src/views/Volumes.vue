<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useVolumeStore } from '../store/volume'
import { useSettingsStore } from '../store/settings'
import { Command } from '@tauri-apps/plugin-shell'
import {
  NDropdown,
  NScrollbar,
  NInput,
  useMessage
} from 'naive-ui'

const volumeStore = useVolumeStore()
const settingsStore = useSettingsStore()
const message = useMessage()

// --- 状态控制 ---
const selectedId = ref<string | null>(null)
const selectedItem = computed(() => volumeStore.volumes.find(v => v.name === selectedId.value))
const activeTab = ref('users') // 默认：users 🗺️ 关联容器

// 文件浏览器状态
const currentPath = ref('')
const fileList = ref<any[]>([])
const fileLoading = ref(false)
const selectedFile = ref<string | null>(null)
const fileContent = ref('')
const fileSaving = ref(false)

const onSelect = async (id: string) => {
  selectedId.value = id
  await volumeStore.fetchVolumeUsers(id)
  currentPath.value = ''
  selectedFile.value = null
  fileContent.value = ''
  fileList.value = []
  
  if (activeTab.value === 'files') {
    await loadFiles()
  }
}

watch(selectedId, async (newId) => {
  if (newId) {
    await volumeStore.fetchVolumeUsers(newId)
  }
})

watch(activeTab, async (newTab) => {
  if (newTab === 'files' && selectedId.value) {
    await loadFiles()
  }
})



const handleDelete = async (name: string) => {
  // 防呆逻辑：若有关联容器，死锁不可删除
  if (volumeStore.volumeUsers.length > 0) {
    message.warning('请先停止并删除关联容器后再来清理数据！')
    return
  }

  try {
    await volumeStore.removeVolume(name)
    message.success('数据卷已从宿主机中彻底卸载')
    if (selectedId.value === name) selectedId.value = null
    await volumeStore.fetchVolumes()
  } catch (err) {
    message.error('删除数据卷失败: ' + err)
  }
}

const handlePrune = async () => {
  try {
    await volumeStore.pruneVolumes()
    message.success('已清理所有未使用的匿名和命名数据卷')
    await volumeStore.fetchVolumes()
  } catch (err) {
    message.error('清理失败: ' + err)
  }
}

// --- WSL 文件浏览器本土级超级优化 ---
const loadFiles = async () => {
  if (!selectedId.value) return
  fileLoading.value = true
  selectedFile.value = null
  fileContent.value = ''

  try {
    const distro = settingsStore.wslDistro || 'Ubuntu'
    const finalPath = `/var/lib/docker/volumes/${selectedId.value}/_data${currentPath.value}`
    
    let execCmd = 'wsl'
    let args = ['-d', distro, '-u', 'root', '--', 'ls', '-p', finalPath]

    if (settingsStore.connectionMode === 'ssh') {
      // SSH 模式文件列表模拟
      fileList.value = [
        { name: 'config.json', isDir: false, path: '/config.json' },
        { name: 'logs', isDir: true, path: '/logs' }
      ]
      fileLoading.value = false
      return
    }

    const command = Command.create(execCmd, args)
    const out = await command.execute()
    
    if (out.code === 0) {
      fileList.value = out.stdout
        .trim()
        .split('\n')
        .filter(Boolean)
        .map((name: string) => {
          const cleanName = name.trim().replace('/', '')
          return {
            name: cleanName,
            isDir: name.trim().endsWith('/'),
            path: currentPath.value + '/' + cleanName
          }
        })
    } else {
      fileList.value = []
    }
  } catch (e: any) {
    message.error('读取 Linux 数据卷目录树失败')
  } finally {
    fileLoading.value = false
  }
}

const clickFileNode = async (node: any) => {
  if (node.isDir) {
    currentPath.value = node.path
    await loadFiles()
  } else {
    selectedFile.value = node.path
    await readVolumeFile(node.path)
  }
}

const goBackDir = async () => {
  const parts = currentPath.value.split('/')
  parts.pop()
  currentPath.value = parts.join('/')
  await loadFiles()
}

const readVolumeFile = async (filePath: string) => {
  const distro = settingsStore.wslDistro || 'Ubuntu'
  const finalPath = `/var/lib/docker/volumes/${selectedId.value}/_data${filePath}`
  
  try {
    fileLoading.value = true
    const command = Command.create('wsl', ['-d', distro, '-u', 'root', '--', 'cat', finalPath])
    const out = await command.execute()
    if (out.code === 0) {
      fileContent.value = out.stdout
    } else {
      throw new Error(out.stderr)
    }
  } catch (e: any) {
    message.error('载入文件内容失败')
  } finally {
    fileLoading.value = false
  }
}

const handleSaveFile = async () => {
  if (!selectedId.value || !selectedFile.value) return
  fileSaving.value = true

  const distro = settingsStore.wslDistro || 'Ubuntu'
  const finalPath = `/var/lib/docker/volumes/${selectedId.value}/_data${selectedFile.value}`

  try {
    // 免密写回 WSL
    const command = Command.create('wsl', [
      '-d', distro, 
      '-u', 'root', 
      '--', 'sh', '-c', 
      `cat << 'EOF' > ${finalPath}\n${fileContent.value}\nEOF`
    ])
    const out = await command.execute()
    
    if (out.code === 0) {
      message.success('文件内容已保存并无损写回宿主')
    } else {
      throw new Error(out.stderr)
    }
  } catch (e: any) {
    message.error('保存失败: ' + e.message)
  } finally {
    fileSaving.value = false
  }
}

// --- 右键菜单 ---
const showMenu = ref(false)
const x = ref(0)
const y = ref(0)
const menuTarget = ref<any>(null)

const menuOptions = [
  { label: '💾 文件浏览', key: 'files' },
  { label: '🗑️ 删除卷', key: 'delete' }
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
  if (key === 'delete') handleDelete(menuTarget.value.name)
  else if (key === 'files') {
    onSelect(menuTarget.value.name)
    activeTab.value = 'files'
  }
}

onMounted(() => {
  volumeStore.fetchVolumes()
})
</script>

<template>
  <div class="volumes-view">
    <!-- 左侧数据卷资产清单 -->
    <div class="list-column">
      <!-- 顶栏 40px 高度: 清理未用卷 -->
      <div class="header-tools">
        <button class="prune-btn" @click="handlePrune">
          🧼 一键清理孤儿卷
        </button>
      </div>

      <n-scrollbar class="list-scroll-box">
        <div 
          v-for="item in volumeStore.volumes" 
          :key="item.name" 
          class="volume-item-row"
          :class="{ active: selectedId === item.name }"
          @click="onSelect(item.name)"
          @contextmenu="handleContextMenu($event, item)"
        >
          <div class="item-left-meta">
            <!-- 卷名称与孤儿判定 -->
            <div class="item-tag-title">
              <span>📦 {{ item.name.substring(0, 24) }}{{ item.name.length > 24 ? '...' : '' }}</span>
            </div>
            <!-- 驱动类型与孤儿文本 -->
            <div class="item-sub-meta">
              {{ item.driver }}
            </div>
          </div>
        </div>
      </n-scrollbar>
    </div>

    <!-- 右侧数据卷全景体检台 -->
    <div class="detail-column">
      <template v-if="selectedItem">
        <!-- 顶层双行控制栏 (高 72px) -->
        <div class="detail-header-wrapper">
          <!-- 行 1: 选项卡 (高 32px) -->
          <div class="tab-line-1">
            <div class="obs-tab" :class="{ active: activeTab === 'users' }" @click="activeTab = 'users'">
              <span>🗺️ 关联容器</span>
              <div class="tab-indicator"></div>
            </div>
            <div class="obs-tab" :class="{ active: activeTab === 'files' }" @click="activeTab = 'files'">
              <span>💾 文件浏览器 (WSL)</span>
              <div class="tab-indicator"></div>
            </div>
            <div class="obs-tab" :class="{ active: activeTab === 'inspect' }" @click="activeTab = 'inspect'">
              <span>📋 卷详情 (Inspect)</span>
              <div class="tab-indicator"></div>
            </div>
          </div>

          <!-- 行 2: 元数据与销毁 (高 40px) -->
          <div class="meta-line-2">
            <div class="meta-left">
              <span class="volume-name-title">📦 {{ selectedItem.name }}</span>
              <div class="vertical-divider"></div>
              <span class="badge driver-badge">🔌 {{ selectedItem.driver }}</span>
            </div>

            <!-- 删除卷：防呆死锁控制 -->
            <div class="meta-right">
              <span 
                v-if="volumeStore.volumeUsers.length > 0" 
                class="lock-tooltip-text"
              >
                🔒 已绑定，解绑后可删
              </span>
              <button 
                class="delete-btn" 
                :disabled="volumeStore.volumeUsers.length > 0" 
                @click="handleDelete(selectedItem.name)"
              >
                🗑️ 删除卷
              </button>
            </div>
          </div>
        </div>

        <!-- 下方主内容区 -->
        <div class="detail-content-area">
          <!-- 1. 🗺️ 关联容器 -->
          <div v-show="activeTab === 'users'" class="users-pane">
            <div v-if="volumeStore.volumeUsers.length === 0" class="empty-list-text">
              🍂 目前无任何运行中容器挂载绑定此数据卷
            </div>
            <div v-else class="users-list-grid">
              <div class="list-section-title">绑定此卷的容器列表</div>
              <div v-for="user in volumeStore.volumeUsers" :key="user.container_id" class="user-binding-card">
                <div class="binding-row">
                  <span class="binding-key">容器:</span>
                  <span class="binding-val container-tag">🟢 {{ user.container_name }}</span>
                </div>
                <div class="binding-row">
                  <span class="binding-key">挂载源:</span>
                  <span class="binding-val path-code">{{ user.source }}</span>
                </div>
                <div class="binding-row">
                  <span class="binding-key">容器内挂载:</span>
                  <span class="binding-val path-code highlight">{{ user.destination }}</span>
                </div>
                <div class="binding-row">
                  <span class="binding-key">权限模式:</span>
                  <span class="binding-val mode-tag">{{ user.mode }} ({{ user.rw ? '可读写 rw' : '只读 ro' }})</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 2. 💾 文件浏览器 (WSL本土特化) -->
          <div v-show="activeTab === 'files'" class="files-pane">
            <!-- 文件树/编辑器左右结构 -->
            <div class="files-browser-grid">
              <!-- 左侧目录树 -->
              <div class="tree-sidebar">
                <div class="sidebar-header">
                  <button 
                    v-if="currentPath" 
                    class="tree-back-btn" 
                    @click="goBackDir"
                  >
                    ⬅️ 返回上一级
                  </button>
                  <span v-else class="tree-root-label">_data (数据根)</span>
                </div>
                <div v-if="fileLoading" class="file-mini-loading">读取中...</div>
                <n-scrollbar v-else style="height: calc(100% - 24px)">
                  <div 
                    v-for="file in fileList" 
                    :key="file.path" 
                    class="file-tree-node"
                    :class="{ active: selectedFile === file.path }"
                    @click="clickFileNode(file)"
                  >
                    <span class="node-bullet">{{ file.isDir ? '📁' : '🗎' }}</span>
                    <span class="node-text">{{ file.name }}</span>
                  </div>
                  <div v-if="fileList.length === 0" class="empty-dir-text">空目录</div>
                </n-scrollbar>
              </div>

              <!-- 右侧编辑器 -->
              <div class="file-editor-area">
                <template v-if="selectedFile">
                  <div class="editor-top-bar">
                    <span class="editor-file-path">📝 {{ selectedFile }}</span>
                    <button class="save-file-gold-btn" :disabled="fileSaving" @click="handleSaveFile">
                      {{ fileSaving ? '保存中...' : '💾 保存并写回' }}
                    </button>
                  </div>
                  <n-input
                    v-model:value="fileContent"
                    class="geek-code-textarea"
                    placeholder="配置文件内容加载中..."
                    type="textarea"
                  />
                </template>
                <div v-else class="empty-editor-placeholder">
                  请在左侧文件树中点击一个配置文件（如 .json, .conf）直接在此处进行免密编辑与写回。
                </div>
              </div>
            </div>
          </div>

          <!-- 3. 📋 卷详情 -->
          <div v-show="activeTab === 'inspect'" class="inspect-pane">
            <n-scrollbar style="height: 100%">
              <div class="inspect-card-box">
                <div class="inspect-row"><span class="key">数据卷名称:</span> <span class="val">{{ selectedItem.name }}</span></div>
                <div class="inspect-row"><span class="key">网络驱动:</span> <span class="val">{{ selectedItem.driver }}</span></div>
                <div class="inspect-row"><span class="key">宿主挂载点:</span> <span class="val">{{ selectedItem.mountpoint }}</span></div>
                <div class="inspect-row"><span class="key">创建时间:</span> <span class="val">{{ selectedItem.created ? new Date(selectedItem.created).toLocaleString() : '未知' }}</span></div>
              </div>
            </n-scrollbar>
          </div>
        </div>
      </template>

      <!-- 空白缺省页 -->
      <div v-else class="empty-state">
        <div class="empty-logo">📦</div>
        <div class="empty-title">持久化数据卷</div>
        <div class="empty-sub">选择左侧的数据卷以观测其挂载状态，或使用内置文件树直接操作内部文件。</div>
      </div>
    </div>
  </div>

  <!-- 右键下拉 -->
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
.volumes-view {
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

.header-tools {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.prune-btn {
  width: 100%;
  height: 24px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-body);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.prune-btn:hover {
  background-color: rgba(245, 158, 11, 0.1);
  border-color: var(--brand-warn);
  color: var(--brand-warn);
}

.list-scroll-box {
  flex: 1;
}

.volume-item-row {
  height: 32px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  cursor: pointer;
  color: var(--text-body);
  transition: all 0.15s ease;
  border-bottom: 1px solid var(--border-color);
}
.volume-item-row:hover {
  background-color: rgba(255, 255, 255, 0.02);
  color: var(--text-title);
}
.volume-item-row.active {
  background-color: var(--bg-active) !important;
  color: var(--text-title);
  font-weight: 600;
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

/* 顶层双行控制栏 */
.detail-header-wrapper {
  height: 72px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
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

.volume-name-title {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-title);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 250px;
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

.lock-tooltip-text {
  font-size: 10px;
  color: var(--brand-warn);
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
.delete-btn:hover:not(:disabled) {
  border-color: var(--brand-danger);
  color: var(--brand-danger);
  background-color: rgba(239, 68, 68, 0.05);
}
.delete-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* 主内容区 */
.detail-content-area {
  flex: 1;
  overflow: hidden;
  position: relative;
}

/* 1. 关联使用者 */
.users-pane {
  height: 100%;
  padding: 16px;
  overflow-y: auto;
}

.empty-list-text {
  font-size: 11px;
  color: var(--text-muted);
  font-style: italic;
  text-align: center;
  padding: 40px 0;
}

.users-list-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.list-section-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-title);
  text-transform: uppercase;
}

.user-binding-card {
  background-color: rgba(255, 255, 255, 0.01);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.binding-row {
  display: flex;
  font-size: 11px;
}

.binding-key {
  color: var(--text-muted);
  width: 90px;
}

.binding-val {
  color: var(--text-body);
  font-family: monospace;
}

.container-tag {
  color: var(--brand-primary);
  font-weight: 600;
}

.path-code {
  word-break: break-all;
}

.path-code.highlight {
  color: #38bdf8;
}

.mode-tag {
  color: var(--text-muted);
}

/* 2. 💾 文件浏览器 (WSL本土特化) */
.files-pane {
  height: 100%;
  overflow: hidden;
}

.files-browser-grid {
  display: flex;
  height: 100%;
}

.tree-sidebar {
  width: 200px;
  border-right: 1px solid var(--border-color);
  background-color: rgba(255, 255, 255, 0.01);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-header {
  height: 28px;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
}

.tree-back-btn {
  background: transparent;
  border: none;
  font-size: 10px;
  font-weight: 700;
  color: var(--text-body);
  cursor: pointer;
  padding: 0;
  outline: none;
}
.tree-back-btn:hover {
  color: var(--brand-primary);
}

.tree-root-label {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-muted);
}

.file-mini-loading {
  font-size: 11px;
  color: var(--text-muted);
  padding: 12px;
}

.file-tree-node {
  height: 26px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  cursor: pointer;
  color: var(--text-body);
  font-size: 11px;
}
.file-tree-node:hover {
  background-color: rgba(255, 255, 255, 0.02);
  color: var(--text-title);
}
.file-tree-node.active {
  background-color: var(--bg-active) !important;
  color: var(--text-title);
}

.node-bullet {
  font-size: 11px;
  margin-right: 6px;
}

.node-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.empty-dir-text {
  font-size: 10px;
  color: var(--text-muted);
  font-style: italic;
  padding: 12px;
}

/* 编辑器区 */
.file-editor-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background-color: #05070c;
}

.editor-top-bar {
  height: 28px;
  padding: 0 12px;
  background-color: rgba(255, 255, 255, 0.02);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
}

.editor-file-path {
  color: var(--text-muted);
  font-family: monospace;
}

.save-file-gold-btn {
  height: 20px;
  padding: 0 8px;
  background: transparent;
  border: 1px solid var(--brand-primary);
  border-radius: 3px;
  color: var(--brand-primary);
  font-size: 9px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.save-file-gold-btn:hover {
  background-color: rgba(16, 185, 129, 0.1);
}

.geek-code-textarea {
  flex: 1;
  border: none !important;
  border-radius: 0;
  background-color: transparent !important;
  font-family: "JetBrains Mono", Consolas, monospace !important;
  font-size: 11px !important;
}

.empty-editor-placeholder {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 32px;
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  line-height: 1.6;
}

/* Inspect */
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
  word-break: break-all;
}

/* 空状态 */
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
</style>
