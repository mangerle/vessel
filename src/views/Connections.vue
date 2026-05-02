<template>
  <div class="connections-view">
    <n-card title="Docker 链接管理">
      <n-result status="info" title="连接列表" description="这里将显示和管理 Docker 连接（本地/远程/WSL）">
      </n-result>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { NCard, NResult } from 'naive-ui'
</script>

<style scoped>
.connections-view {
  padding: 24px;
}
</style>
