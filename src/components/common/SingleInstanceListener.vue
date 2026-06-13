<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useMessage } from 'naive-ui'
import { EVT } from '../../api/events'

const message = useMessage()
let unlisten: UnlistenFn | null = null

onMounted(async () => {
  // 监听 Tauri 发送的单实例检测事件
  unlisten = await listen(EVT.singleInstanceDetected, () => {
    message.warning('检测到 Vessel 已在后台运行，已为您激活当前窗口', {
      duration: 5000,
      closable: true,
      keepAliveOnHover: true
    })
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
})
</script>

<template>
  <!-- 此组件仅用于全局事件监听和通知触发，无实际渲染内容 -->
  <div style="display: none;"></div>
</template>
