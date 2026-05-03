<template>
  <n-layout has-sider position="absolute" style="height: 100vh">
    <n-layout-sider
        bordered
        collapse-mode="width"
        :collapsed-width="56"
        :width="200"
        :collapsed="collapsed"
        show-trigger
        @collapse="collapsed = true"
        @expand="collapsed = false"
    >
      <div class="logo">
        <span v-if="!collapsed">Vessel</span>
      </div>
      <n-menu
          :collapsed="collapsed"
          :collapsed-width="56"
          :collapsed-icon-size="22"
          :options="menuOptions"
          :value="activeKey"
          @update:value="handleMenuClick"
      />
    </n-layout-sider>
    <n-layout>
      <n-layout-content content-style="padding: 20px; background-color: var(--macos-bg-light);">
        <router-view/>
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<script setup lang="ts">
import {Component, h, ref, watch} from 'vue'
import type {MenuOption} from 'naive-ui'
import {NIcon, NLayout, NLayoutContent, NLayoutSider, NMenu} from 'naive-ui'
import {useRoute, useRouter} from 'vue-router'
import {CubeOutline, GlobeOutline, ImagesOutline, LayersOutline, SaveOutline} from '@vicons/ionicons5'

const router = useRouter()
const route = useRoute()
const collapsed = ref(false)

const activeKey = ref<string>(route.name as string || 'compose')

// 监听路由变化更新激活的菜单项
watch(() => route.name, (newName) => {
  if (newName) {
    activeKey.value = newName as string
  }
})

function renderIcon(icon: Component) {
  return () => h(NIcon, null, {default: () => h(icon)})
}

const menuOptions: MenuOption[] = [
  {
    label: 'Compose',
    key: 'compose',
    icon: renderIcon(CubeOutline)
  },
  {
    label: '容器',
    key: 'containers',
    icon: renderIcon(LayersOutline)
  },
  {
    label: '镜像',
    key: 'images',
    icon: renderIcon(ImagesOutline)
  },
  {
    label: '网络',
    key: 'networks',
    icon: renderIcon(GlobeOutline)
  },
  {
    label: '卷',
    key: 'volumes',
    icon: renderIcon(SaveOutline)
  }
]

function handleMenuClick(key: string) {
  activeKey.value = key
  router.push({name: key})
}
</script>

<style scoped>
.logo {
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  white-space: nowrap;
}
</style>
