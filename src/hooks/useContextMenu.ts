import {h, ref} from 'vue'
import {NIcon} from 'naive-ui'
import {
  CopyOutline,
  DocumentTextOutline,
  PlayOutline,
  RefreshOutline,
  StopOutline,
  TerminalOutline,
  TrashOutline
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
    
    // Ensure reactivity update triggers before showing again
    setTimeout(() => {
      x.value = e.clientX
      y.value = e.clientY
      currentTarget.value = data

      if (type === 'container') {
        currentOptions.value = [
          { label: '启动', key: 'start', icon: renderIcon(PlayOutline), disabled: data.state === 'running' },
          { label: '停止', key: 'stop', icon: renderIcon(StopOutline), disabled: data.state !== 'running' },
          { label: '重启', key: 'restart', icon: renderIcon(RefreshOutline) },
          { type: 'divider', key: 'd1' },
          { 
            label: '打开终端', 
            key: 'terminal_group', 
            icon: renderIcon(TerminalOutline),
            children: [
              { label: '作为普通用户', key: 'terminal_user' },
              { label: '作为 Root 用户', key: 'terminal_root' }
            ]
          },
          { label: '查看日志', key: 'logs', icon: renderIcon(DocumentTextOutline) },
          { type: 'divider', key: 'd2' },
          { label: '复制 ID', key: 'copy_id', icon: renderIcon(CopyOutline) },
          { label: '删除', key: 'delete', icon: renderIcon(TrashOutline) }
        ]
      } else if (type === 'project') {
          currentOptions.value = [
              {label: '启动 (Up)', key: 'up', icon: renderIcon(PlayOutline)},
              {label: '停止 (Down)', key: 'down', icon: renderIcon(StopOutline)},
              {label: '重启', key: 'restart_project', icon: renderIcon(RefreshOutline)},
              {type: 'divider', key: 'd1'},
              {label: '编辑配置', key: 'edit', icon: renderIcon(DocumentTextOutline)},
              {type: 'divider', key: 'd2'},
              {label: '删除项目', key: 'delete_project', icon: renderIcon(TrashOutline)}
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
