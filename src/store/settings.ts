import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'

export const useSettingsStore = defineStore('settings', () => {
  const autoStart = ref(false)
  const closeToTray = ref(true)
  const theme = ref<'deep-black' | 'zed-gray' | 'light-apple'>('deep-black')
  const refreshInterval = ref(3) // 默认 3 秒
  const connectionMode = ref<'wsl' | 'ssh'>('wsl')
  const wslDistro = ref('')
  const sshHost = ref('')
  const sshPort = ref(22)
  const sshUser = ref('')
  const sshPassword = ref('')
  
  // 监听主题变化，更新 HTML 属性
  watch(theme, (newTheme) => {
    document.documentElement.setAttribute('data-theme', newTheme)
  }, { immediate: true })

  const loadSettings = async () => {
    try {
      autoStart.value = await isEnabled()
      
      const saved = localStorage.getItem('vessel-settings')
      if (saved) {
        const parsed = JSON.parse(saved)
        closeToTray.value = parsed.closeToTray ?? true
        theme.value = parsed.theme ?? 'deep-black'
        refreshInterval.value = parsed.refreshInterval ?? 3
        connectionMode.value = parsed.connectionMode ?? 'wsl'
        wslDistro.value = parsed.wslDistro ?? ''
        sshHost.value = parsed.sshHost ?? ''
        sshPort.value = parsed.sshPort ?? 22
        sshUser.value = parsed.sshUser ?? ''
        sshPassword.value = parsed.sshPassword ?? ''
      }
    } catch (e) {
      console.error('加载设置失败:', e)
      const saved = localStorage.getItem('vessel-settings')
      if (saved) {
        const parsed = JSON.parse(saved)
        autoStart.value = parsed.autoStart || false
        closeToTray.value = parsed.closeToTray ?? true
        theme.value = parsed.theme ?? 'deep-black'
        refreshInterval.value = parsed.refreshInterval ?? 3
        connectionMode.value = parsed.connectionMode ?? 'wsl'
        wslDistro.value = parsed.wslDistro ?? ''
        sshHost.value = parsed.sshHost ?? ''
        sshPort.value = parsed.sshPort ?? 22
        sshUser.value = parsed.sshUser ?? ''
        sshPassword.value = parsed.sshPassword ?? ''
      }
    }
  }
  
  const setAutoStart = async (value: boolean) => {
    try {
      if (value) {
        await enable()
      } else {
        await disable()
      }
      autoStart.value = value
      saveSettings()
    } catch (e) {
      console.error('设置自启动失败:', e)
    }
  }

  const setCloseToTray = (value: boolean) => {
    closeToTray.value = value
    saveSettings()
  }

  const setTheme = (value: 'deep-black' | 'zed-gray' | 'light-apple') => {
    theme.value = value
    saveSettings()
  }
  
  const saveSettings = () => {
    localStorage.setItem('vessel-settings', JSON.stringify({
      autoStart: autoStart.value,
      closeToTray: closeToTray.value,
      theme: theme.value,
      refreshInterval: refreshInterval.value,
      connectionMode: connectionMode.value,
      wslDistro: wslDistro.value,
      sshHost: sshHost.value,
      sshPort: sshPort.value,
      sshUser: sshUser.value,
      sshPassword: sshPassword.value
    }))
  }
  
  return {
    autoStart,
    closeToTray,
    theme,
    refreshInterval,
    connectionMode,
    wslDistro,
    sshHost,
    sshPort,
    sshUser,
    sshPassword,
    loadSettings,
    setAutoStart,
    setCloseToTray,
    setTheme,
    saveSettings
  }
})

