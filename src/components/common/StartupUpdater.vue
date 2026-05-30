<script setup lang="ts">
import { onMounted } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { useDialog } from 'naive-ui'
import { useRouter } from 'vue-router'

const dialog = useDialog()
const router = useRouter()

onMounted(async () => {
  // 延迟检查，避免干扰应用启动时的其他重要初始化
  setTimeout(async () => {
    try {
      const update = await check()
      if (update) {
        dialog.info({
          title: '🚀 发现新版本',
          content: `Vessel 发现新版本 v${update.version}，是否立即前往设置页面进行升级？`,
          positiveText: '立即前往',
          negativeText: '以后再说',
          onPositiveClick: () => {
            router.push({ path: '/settings', query: { triggerUpdate: 'true' } })
          }
        })
      }
    } catch (e) {
      console.error('[StartupUpdater] 自动检查更新失败:', e)
    }
  }, 3000)
})
</script>

<template>
  <!-- 纯逻辑组件，无渲染内容 -->
  <div style="display: none;"></div>
</template>
