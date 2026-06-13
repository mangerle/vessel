<template>
  <div class="file-browser-container">
    <!-- 容器未运行时的提示覆盖层 -->
    <div v-if="!containerStatus" class="not-running-overlay">
      <div class="overlay-content">
        <n-icon :component="CloudOfflineOutline" size="48" class="offline-icon" />
        <div class="overlay-title">容器未运行</div>
        <div class="overlay-sub">请启动该容器后浏览和管理容器内文件。</div>
      </div>
    </div>

    <!-- 正常浏览时的布局 -->
    <template v-else>
      <!-- 头部操作与导航栏 -->
      <div class="browser-header">
        <!-- 路径导航 -->
        <div class="path-navigation">
          <n-button 
            quaternary 
            size="small" 
            :disabled="currentPath === '/'" 
            @click="goUp"
            title="返回上级"
          >
            <template #icon>
              <n-icon :component="ArrowUpOutline" />
            </template>
          </n-button>
          
          <n-input 
            v-model:value="pathInput" 
            size="small" 
            placeholder="当前路径" 
            class="path-input"
            @keyup.enter="handlePathEnter"
          />
        </div>

        <!-- 动作按钮 -->
        <n-space size="small" class="action-buttons">
          <n-button size="small" secondary type="primary" @click="showCreateModal = true">
            <template #icon>
              <n-icon :component="AddOutline" />
            </template>
            新建
          </n-button>
          
          <n-dropdown trigger="click" :options="uploadOptions" @select="handleUploadSelect">
            <n-button size="small" secondary>
              <template #icon>
                <n-icon :component="CloudUploadOutline" />
              </template>
              上传
            </n-button>
          </n-dropdown>

          <n-button size="small" secondary @click="fetchFiles" :loading="loading">
            <template #icon>
              <n-icon :component="RefreshOutline" />
            </template>
            刷新
          </n-button>
        </n-space>
      </div>

      <!-- 文件列表区域 -->
      <div class="browser-body" @contextmenu.prevent.stop="handleBgContext">
        <div v-if="loading && files.length === 0" class="list-loading">
          <n-spin size="medium" />
        </div>
        
        <div v-else-if="files.length === 0" class="empty-list">
          <n-icon :component="FolderOpenOutline" size="36" class="empty-icon" />
          <div class="empty-text">此目录为空</div>
        </div>

        <n-table v-else striped :bordered="false" size="small" class="file-table">
          <thead>
            <tr>
              <th>名称</th>
              <th style="width: 100px;">大小</th>
              <th style="width: 150px;">权限</th>
              <th style="width: 180px;">修改时间</th>
            </tr>
          </thead>
          <tbody>
            <tr 
              v-for="file in files" 
              :key="file.name" 
              @dblclick="handleDblClick(file)"
              @contextmenu.prevent.stop="handleFileContext($event, file)"
              class="file-row"
            >
              <td class="file-name-cell">
                <n-icon 
                  :component="file.is_dir ? FolderOutline : DocumentOutline" 
                  :class="file.is_dir ? 'folder-icon' : 'file-icon'"
                  size="16" 
                />
                <span class="file-name">{{ file.name }}</span>
              </td>
              <td>{{ file.is_dir ? '-' : formatBytes(file.size) }}</td>
              <td class="perm-cell">{{ file.permissions || '-' }}</td>
              <td>{{ formatTime(file.mtime) }}</td>
            </tr>
          </tbody>
        </n-table>
      </div>
    </template>

    <!-- 新建对话框 -->
    <n-modal
      v-model:show="showCreateModal"
      preset="dialog"
      title="新建项"
      positive-text="确认"
      negative-text="取消"
      @positive-click="handleCreate"
      @after-leave="createName = ''; createIsDir = false"
    >
      <n-space vertical size="medium" style="margin-top: 12px;">
        <n-radio-group v-model:value="createIsDir" name="createType">
          <n-space>
            <n-radio :value="false">文件</n-radio>
            <n-radio :value="true">文件夹</n-radio>
          </n-space>
        </n-radio-group>
        <n-input v-model:value="createName" placeholder="名称" autofocus @keyup.enter="handleCreate" />
      </n-space>
    </n-modal>

    <!-- 重命名对话框 -->
    <n-modal
      v-model:show="showRenameModal"
      preset="dialog"
      title="重命名"
      positive-text="确认"
      negative-text="取消"
      @positive-click="handleRename"
      @after-leave="renameName = ''"
    >
      <n-space vertical size="medium" style="margin-top: 12px;">
        <div style="font-size: 12px; color: var(--text-muted);">原名称: {{ oldFileName }}</div>
        <n-input v-model:value="renameName" placeholder="新名称" autofocus @keyup.enter="handleRename" />
      </n-space>
    </n-modal>

    <!-- 文本编辑器 Modal -->
    <n-modal
      v-model:show="showEditorModal"
      preset="card"
      style="width: 800px; height: 600px;"
      :title="`编辑文件: ${editingFilePath}`"
      :segmented="{ content: 'soft', footer: 'soft' }"
    >
      <div class="text-editor-wrapper">
        <textarea 
          v-model="editingContent" 
          class="text-editor-area" 
          spellcheck="false"
        ></textarea>
      </div>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showEditorModal = false">取消</n-button>
          <n-button type="primary" :loading="editorSaving" @click="saveEditor">保存并应用</n-button>
        </n-space>
      </template>
    </n-modal>

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
import { ref, watch, onMounted } from 'vue'
import {
  useMessage, useDialog, NButton, NSpace, NInput, NTable,
  NModal, NRadioGroup, NRadio, NDropdown, NSpin, NIcon
} from 'naive-ui'
import {
  CloudOfflineOutline, ArrowUpOutline, AddOutline,
  CloudUploadOutline, RefreshOutline, FolderOutline,
  FolderOpenOutline, DocumentOutline, CloudDownloadOutline,
  CreateOutline, TrashOutline, CopyOutline, DocumentTextOutline
} from '@vicons/ionicons5'
import { containerFsApi } from '../../api/containerFsApi'
import { useContextMenu, MenuOption } from '../../hooks/useContextMenu'
import { formatBytes } from '../../utils/format'
import type { ContainerFileInfo } from '../../api/types'

const props = defineProps<{
  containerId: string
  containerStatus: boolean
  active: boolean
}>()

const message = useMessage()
const dialog = useDialog()
const { 
  showDropdown, x, y, currentOptions, currentTarget, 
  handleContextMenu, onClickOutside, renderIcon 
} = useContextMenu()

// 核心路径与列表状态
const currentPath = ref('/')
const pathInput = ref('/')
const files = ref<ContainerFileInfo[]>([])
const loading = ref(false)
const lastLoadedContainerId = ref('') // 记录最后成功加载文件树的容器 ID

// 路径拼接助手函数
const joinPath = (base: string, name: string) => {
  const divider = base === '/' ? '' : '/'
  return `${base}${divider}${name}`
}

// 新建对话框状态
const showCreateModal = ref(false)
const createName = ref('')
const createIsDir = ref(false)

// 重命名对话框状态
const showRenameModal = ref(false)
const oldFileName = ref('')
const renameName = ref('')

// 编辑器对话框状态
const showEditorModal = ref(false)
const editingFilePath = ref('')
const editingContent = ref('')
const editorSaving = ref(false)

// 上传下拉菜单选项
const uploadOptions = [
  { label: '上传文件', key: 'file' },
  { label: '上传文件夹', key: 'dir' }
]

// 文件项右键菜单选项
const fileItemOptions = (file: any): MenuOption[] => [
  ...(!file.is_dir && isTextFile(file.name) ? [{
    label: '编辑',
    key: 'edit',
    icon: renderIcon(DocumentTextOutline)
  }] : []),
  {
    label: '下载',
    key: 'download',
    icon: renderIcon(CloudDownloadOutline)
  },
  {
    label: '重命名',
    key: 'rename',
    icon: renderIcon(CreateOutline)
  },
  {
    label: '复制路径',
    key: 'copy_path',
    icon: renderIcon(CopyOutline)
  },
  {
    type: 'divider',
    key: 'd1'
  },
  {
    label: '删除',
    key: 'delete',
    icon: renderIcon(TrashOutline),
    props: {
      style: { color: 'var(--brand-danger)' }
    }
  }
]

// 文件夹背景右键菜单选项
const folderBackgroundOptions: MenuOption[] = [
  {
    label: '新建',
    key: 'create',
    icon: renderIcon(AddOutline)
  },
  {
    label: '刷新',
    key: 'refresh',
    icon: renderIcon(RefreshOutline)
  }
]

// 处理文件项右键点击
const handleFileContext = (e: MouseEvent, file: any) => {
  handleContextMenu(e, fileItemOptions(file), file)
}

// 处理背景右键点击
const handleBgContext = (e: MouseEvent) => {
  handleContextMenu(e, folderBackgroundOptions)
}

// 统一处理菜单选择动作
const handleMenuSelect = (key: string) => {
  showDropdown.value = false
  const data = currentTarget.value as ContainerFileInfo | undefined
  if (!data) return
  
  switch (key) {
    case 'edit':
      openEditor(data)
      break
    case 'download':
      handleDownload(data)
      break
    case 'rename':
      openRename(data)
      break
    case 'delete':
      handleDelete(data)
      break
    case 'create':
      showCreateModal.value = true
      break
    case 'refresh':
      fetchFiles()
      break
    case 'copy_path':
      const fullPath = joinPath(currentPath.value, data.name)
      navigator.clipboard.writeText(fullPath).then(() => {
        message.success('路径已复制到剪贴板')
      }).catch(err => {
        message.error(`复制失败: ${err}`)
      })
      break
  }
}

// 格式化修改时间
const formatTime = (timestamp: number) => {
  if (!timestamp) return '-'
  const date = new Date(timestamp)
  return date.toLocaleString()
}

// 识别文本文件，支持编辑
const isTextFile = (filename: string) => {
  const ext = filename.split('.').pop()?.toLowerCase() || ''
  const textExtensions = [
    'txt', 'log', 'conf', 'cfg', 'json', 'yaml', 'yml', 
    'sh', 'xml', 'html', 'css', 'js', 'ts', 'py', 'go', 
    'toml', 'ini', 'md', 'env', 'properties', 'sql', 'bashrc', 'profile'
  ]
  return textExtensions.includes(ext) || !filename.includes('.')
}

// 拉取文件列表
const fetchFiles = async () => {
  if (!props.containerId || !props.containerStatus) return
  loading.value = true
  try {
    const list = await containerFsApi.listFiles(props.containerId, currentPath.value)
    files.value = list
    pathInput.value = currentPath.value
    lastLoadedContainerId.value = props.containerId
  } catch (e: any) {
    message.error(`加载文件列表失败: ${e}`)
  } finally {
    loading.value = false
  }
}

// 监听路径输入回车跳转
const handlePathEnter = () => {
  let target = pathInput.value.trim()
  if (!target.startsWith('/')) {
    target = '/' + target
  }
  // 移除末尾多余斜杠（如果是根目录除外）
  if (target.length > 1 && target.endsWith('/')) {
    target = target.slice(0, -1)
  }
  currentPath.value = target
  fetchFiles()
}

// 返回上级目录
const goUp = () => {
  if (currentPath.value === '/') return
  const parts = currentPath.value.split('/').filter(Boolean)
  parts.pop()
  currentPath.value = '/' + parts.join('/')
  fetchFiles()
}

// 双击文件夹进入，双击可编辑文件打开编辑
const handleDblClick = (file: any) => {
  if (file.is_dir) {
    currentPath.value = joinPath(currentPath.value, file.name)
    fetchFiles()
  } else if (isTextFile(file.name)) {
    openEditor(file)
  }
}

// 新建文件或文件夹
const handleCreate = async () => {
  if (!createName.value.trim()) {
    message.warning('请输入名称')
    return
  }
  const targetPath = joinPath(currentPath.value, createName.value.trim())
  
  try {
    await containerFsApi.create(props.containerId, targetPath, createIsDir.value)
    message.success('新建成功')
    showCreateModal.value = false
    fetchFiles()
  } catch (e: any) {
    message.error(`新建失败: ${e}`)
  }
}

// 触发上传文件/文件夹
const handleUploadSelect = async (key: string) => {
  const { open } = await import('@tauri-apps/plugin-dialog')
  try {
    const selected = await open({
      directory: key === 'dir',
      multiple: false
    })
    if (selected) {
      loading.value = true
      await containerFsApi.upload(props.containerId, selected, currentPath.value)
      message.success('上传成功')
      fetchFiles()
    }
  } catch (e: any) {
    message.error(`上传失败: ${e}`)
  } finally {
    loading.value = false
  }
}

// 触发下载容器内文件
const handleDownload = async (file: any) => {
  const { save } = await import('@tauri-apps/plugin-dialog')
  try {
    const selected = await save({
      defaultPath: file.name
    })
    if (selected) {
      loading.value = true
      const containerPath = joinPath(currentPath.value, file.name)
      await containerFsApi.download(props.containerId, containerPath, selected)
      message.success('下载成功')
    }
  } catch (e: any) {
    message.error(`下载失败: ${e}`)
  } finally {
    loading.value = false
  }
}

// 打开重命名对话框
const openRename = (file: any) => {
  oldFileName.value = file.name
  renameName.value = file.name
  showRenameModal.value = true
}

// 触发重命名文件/目录
const handleRename = async () => {
  if (!renameName.value.trim() || renameName.value.trim() === oldFileName.value) {
    showRenameModal.value = false
    return
  }
  const srcPath = joinPath(currentPath.value, oldFileName.value)
  const destPath = joinPath(currentPath.value, renameName.value.trim())
  
  try {
    await containerFsApi.rename(props.containerId, srcPath, destPath)
    message.success('重命名成功')
    showRenameModal.value = false
    fetchFiles()
  } catch (e: any) {
    message.error(`重命名失败: ${e}`)
  }
}

// 触发删除容器内文件/文件夹
const handleDelete = async (file: ContainerFileInfo) => {
  const targetPath = joinPath(currentPath.value, file.name)
  
  dialog.warning({
    title: '确认删除',
    content: `确定要永久删除 ${file.name} 吗？`,
    positiveText: '确认',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await containerFsApi.delete(props.containerId, targetPath)
        message.success('删除成功')
        fetchFiles()
      } catch (e: any) {
        message.error(`删除失败: ${e}`)
      }
    }
  })
}

// 打开编辑器并拉取文本文件内容
const openEditor = async (file: ContainerFileInfo) => {
  const targetPath = joinPath(currentPath.value, file.name)
  editingFilePath.value = targetPath
  loading.value = true
  
  try {
    const text = await containerFsApi.readText(props.containerId, targetPath)
    editingContent.value = text
    showEditorModal.value = true
  } catch (e: any) {
    message.error(`读取文件内容失败: ${e}`)
  } finally {
    loading.value = false
  }
}

// 保存修改后的文本内容
const saveEditor = async () => {
  editorSaving.value = true
  try {
    await containerFsApi.writeText(props.containerId, editingFilePath.value, editingContent.value)
    message.success('文件保存成功')
    showEditorModal.value = false
    fetchFiles()
  } catch (e: any) {
    message.error(`保存失败: ${e}`)
  } finally {
    editorSaving.value = false
  }
}

// 监听容器选择变更与运行状态变更
watch(() => props.containerId, () => {
  currentPath.value = '/'
  if (props.active) {
    fetchFiles()
  }
})

watch(() => props.containerStatus, (newVal) => {
  if (newVal) {
    if (props.active) {
      fetchFiles()
    }
  } else {
    files.value = []
    lastLoadedContainerId.value = ''
  }
})

// 监听激活状态（在其他 Tab 时切换容器，回到文件浏览器时再触发加载）
watch(() => props.active, (newActive) => {
  if (newActive && lastLoadedContainerId.value !== props.containerId) {
    fetchFiles()
  }
})

onMounted(() => {
  if (props.active) {
    fetchFiles()
  }
})
</script>

<style scoped>
.file-browser-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
  overflow: hidden;
  background-color: var(--bg-main);
  color: var(--text-body);
}

/* 未运行状态遮罩样式 */
.not-running-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgba(11, 15, 25, 0.65);
  z-index: 5;
  user-select: none;
  backdrop-filter: blur(2px);
}
.overlay-content {
  text-align: center;
}
.offline-icon {
  color: var(--text-muted);
  opacity: 0.6;
  margin-bottom: 12px;
}
.overlay-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-title);
  margin-bottom: 6px;
}
.overlay-sub {
  font-size: 11px;
  color: var(--text-muted);
}

/* 文件管理器头部操作区 */
.browser-header {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
  gap: 16px;
}
.path-navigation {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-grow: 1;
  max-width: 600px;
}
.path-input {
  flex-grow: 1;
}

/* 主列表区 */
.browser-body {
  flex-grow: 1;
  overflow-y: auto;
  position: relative;
}

.list-loading, .empty-list {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  height: 200px;
  width: 100%;
}
.empty-icon {
  color: var(--text-muted);
  opacity: 0.5;
  margin-bottom: 8px;
}
.empty-text {
  font-size: 12px;
  color: var(--text-muted);
}

/* 列表格样式 */
.file-table {
  background-color: transparent !important;
}
.file-table th {
  background-color: rgba(255, 255, 255, 0.02) !important;
  color: var(--text-muted) !important;
  font-size: 11px;
  font-weight: 600;
  border-bottom: 1px solid var(--border-color) !important;
  padding: 8px 16px !important;
  user-select: none;
}
.file-table td {
  padding: 6px 16px !important;
  font-size: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03) !important;
}

.file-row {
  cursor: pointer;
  transition: background-color 0.15s ease;
}
.file-row:hover {
  background-color: rgba(255, 255, 255, 0.04) !important;
}

.file-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.folder-icon {
  color: #eab308; /* 金黄色目录 */
}
.file-icon {
  color: #94a3b8; /* 灰色文件 */
}

.file-name {
  font-weight: 500;
  color: var(--text-title);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.perm-cell {
  font-family: monospace;
  color: var(--text-muted);
  font-size: 11px;
}

/* 文本编辑器框样式 */
.text-editor-wrapper {
  height: 450px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
}
.text-editor-area {
  width: 100%;
  height: 100%;
  padding: 12px;
  border: none;
  resize: none;
  background-color: #0c1017;
  color: #e6edf3;
  font-family: 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;
  outline: none;
}
</style>
