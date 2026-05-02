<template>
  <div class="volumes-view">
    <n-card title="卷管理">
      <n-result status="info" title="卷列表" description="这里将显示和管理 Docker 数据卷">
      </n-result>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { NCard, NResult } from 'naive-ui'
</script>

<style scoped>
.volumes-view {
  padding: 24px;
}
</style>
