import { h, ref } from 'vue'
import { NIcon } from 'naive-ui'

/**
 * 辅助渲染图标函数
 * 导出以便在业务组件配置菜单项时使用
 */
export const renderIcon = (icon: any) => {
  return () => h(NIcon, null, { default: () => h(icon) })
}

export function useContextMenu() {
  const showDropdown = ref(false)
  const x = ref(0)
  const y = ref(0)
  const currentOptions = ref<any[]>([])
  const currentTarget = ref<any>(null)

  /**
   * 处理右键菜单弹出逻辑
   * @param e 鼠标事件
   * @param options 菜单项配置
   * @param data 关联的数据（如容器对象）
   */
  const handleContextMenu = (e: MouseEvent, options: any[], data?: any) => {
    e.preventDefault()
    showDropdown.value = false
    
    // 延迟以确保 Vue 响应式状态更新触发，避免闪烁
    setTimeout(() => {
      x.value = e.clientX
      y.value = e.clientY
      currentTarget.value = data
      currentOptions.value = options
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
    onClickOutside,
    renderIcon // 同样通过 hook 导出以便在 setup 中直接使用
  }
}
