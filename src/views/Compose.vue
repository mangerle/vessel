<template>
  <div class="compose-view">
    <n-card title="Compose 管理">
      <n-result status="info" title="项目列表" description="这里将显示和管理 Docker Compose 项目">
      </n-result>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { NCard, NResult } from 'naive-ui'
</script>

<style scoped>
.compose-view {
  padding: 24px;
}
</style>
