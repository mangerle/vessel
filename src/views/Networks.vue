<template>
  <div class="networks-view">
    <n-card title="网络管理">
      <n-result status="info" title="网络列表" description="这里将显示和管理 Docker 网络">
      </n-result>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { NCard, NResult } from 'naive-ui'
</script>

<style scoped>
.networks-view {
  padding: 24px;
}
</style>
