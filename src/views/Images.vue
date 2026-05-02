<template>
  <div class="images-view">
    <n-card title="镜像管理">
      <n-result status="info" title="镜像列表" description="这里将显示和管理 Docker 镜像">
      </n-result>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { NCard, NResult } from 'naive-ui'
</script>

<style scoped>
.images-view {
  padding: 24px;
}
</style>
