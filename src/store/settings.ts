import { defineStore } from 'pinia'
import { ref } from 'vue'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'

export const useSettingsStore = defineStore('settings', () => {
  const autoStart = ref(false)
  
  const loadSettings = async () => {
    try {
      autoStart.value = await isEnabled()
    } catch (e) {
      console.error('Failed to check autostart status:', e)
      // Fallback to local storage if plugin fails
      const saved = localStorage.getItem('vessel-settings')
      if (saved) {
        const parsed = JSON.parse(saved)
        autoStart.value = parsed.autoStart || false
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
  
  const saveSettings = () => {
    localStorage.setItem('vessel-settings', JSON.stringify({
      autoStart: autoStart.value
    }))
  }
  
  return {
    autoStart,
    loadSettings,
    setAutoStart
  }
})
