import { h, ref } from 'vue'
import { NIcon } from 'naive-ui'
import {
  CopyOutline,
  DocumentTextOutline,
  PlayOutline,
  RefreshOutline,
  StopOutline,
  TerminalOutline,
  TrashOutline,
  PauseOutline,
  EyeOutline,
  FlashOutline,
  ListOutline,
  CreateOutline,
  SaveOutline,
  FolderOutline
} from '@vicons/ionicons5'

export function useContextMenu() {
  const showDropdown = ref(false)
  const x = ref(0)
  const y = ref(0)
  const currentOptions = ref<any[]>([])
  const currentTarget = ref<any>(null)

  const renderIcon = (icon: any) => {
    return () => h(NIcon, null, { default: () => h(icon) })
  }

  const handleContextMenu = (e: MouseEvent, type: 'container' | 'project' | 'global', data?: any) => {
    e.preventDefault()
    showDropdown.value = false
    
    // 延迟以确保 Vue 响应式状态更新触发，避免闪烁
    setTimeout(() => {
      x.value = e.clientX
      y.value = e.clientY
      currentTarget.value = data

      if (type === 'container') {
        const isRunning = data.state === 'running' || (data.State && data.State.Running)
        const isPaused = data.state === 'paused' || (data.State && data.State.Paused)

        currentOptions.value = [
          // 组 1: 生命周期控制
          { label: '启动', key: 'start', icon: renderIcon(PlayOutline), disabled: isRunning },
          { label: '停止', key: 'stop', icon: renderIcon(StopOutline), disabled: !isRunning },
          { label: '重启', key: 'restart', icon: renderIcon(RefreshOutline) },
          { 
            label: isPaused ? '恢复' : '暂停', 
            key: isPaused ? 'unpause' : 'pause', 
            icon: renderIcon(PauseOutline), 
            disabled: !isRunning 
          },
          { type: 'divider', key: 'div1' },
          
          // 组 2: 资产与属性管理
          { label: '重命名容器...', key: 'rename_container', icon: renderIcon(CreateOutline) },
          { label: '提交为新镜像...', key: 'commit_container', icon: renderIcon(SaveOutline) },
          { label: '查看对应镜像', key: 'view_image', icon: renderIcon(EyeOutline) },
          { label: '复制容器 ID', key: 'copy_id', icon: renderIcon(CopyOutline) },
          { label: '复制镜像 ID', key: 'copy_image_id', icon: renderIcon(CopyOutline) },
          { type: 'divider', key: 'div2' },

          // 组 3: 深度观测
          { label: '检查元数据', key: 'inspect_meta', icon: renderIcon(DocumentTextOutline) },
          { label: '显示内部进程 (Top)', key: 'show_top', icon: renderIcon(ListOutline) },
          { type: 'divider', key: 'div3' },

          // 组 4: 控制流注入
          { label: '文件管理', key: 'file_explorer', icon: renderIcon(FolderOutline) },
          { label: '附加标准流 (Attach)', key: 'attach_stream', icon: renderIcon(FlashOutline) },
          { label: '快速执行命令 (Exec...)', key: 'exec_cmd', icon: renderIcon(TerminalOutline) },
          { 
            label: '打开交互终端', 
            key: 'terminal_group', 
            icon: renderIcon(TerminalOutline),
            children: [
              { label: '普通用户', key: 'terminal_user' },
              { label: 'Root 用户', key: 'terminal_root' }
            ]
          },
          { type: 'divider', key: 'div4' },

          // 组 5: 危险项
          { label: '安全删除...', key: 'delete', icon: renderIcon(TrashOutline) }
        ]
      } else if (type === 'project') {
        currentOptions.value = [
          { label: '启动', key: 'up', icon: renderIcon(PlayOutline) },
          { label: '重启', key: 'restart_project', icon: renderIcon(RefreshOutline) },
          { label: '停止', key: 'stop_project', icon: renderIcon(StopOutline) },
          { label: '下线', key: 'down_project', icon: renderIcon(StopOutline) }
        ]
      } else if (type === 'global') {
        currentOptions.value = [
          { label: '刷新列表', key: 'refresh', icon: renderIcon(RefreshOutline) }
        ]
      }

      showDropdown.value = true
    }, 10)
  }

  const onClickOutside = () => {
    showDropdown.value = false
  }

  return {
    showDropdown,
    x,
    y,
    currentOptions,
    currentTarget,
    handleContextMenu,
    onClickOutside
  }
}
