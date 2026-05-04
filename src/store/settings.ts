import { defineStore } from 'pinia'
import { ref } from 'vue'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'

export const useSettingsStore = defineStore('settings', () => {
  const autoStart = ref(false)
  const closeToTray = ref(true)
  
  const loadSettings = async () => {
    try {
      autoStart.value = await isEnabled()
      
      const saved = localStorage.getItem('vessel-settings')
      if (saved) {
        const parsed = JSON.parse(saved)
        closeToTray.value = parsed.closeToTray ?? true
      }
    } catch (e) {
      console.error('Failed to load settings:', e)
      const saved = localStorage.getItem('vessel-settings')
      if (saved) {
        const parsed = JSON.parse(saved)
        autoStart.value = parsed.autoStart || false
        closeToTray.value = parsed.closeToTray ?? true
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
      console.error('Failed to set autostart:', e)
    }
  }

  const setCloseToTray = (value: boolean) => {
    closeToTray.value = value
    saveSettings()
  }
  
  const saveSettings = () => {
    localStorage.setItem('vessel-settings', JSON.stringify({
      autoStart: autoStart.value,
      closeToTray: closeToTray.value
    }))
  }
  
  return {
    autoStart,
    closeToTray,
    loadSettings,
    setAutoStart,
    setCloseToTray
  }
})
