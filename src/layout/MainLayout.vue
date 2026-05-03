<template>
  <n-layout has-sider position="absolute" style="height: 100vh">
    <n-layout-sider
      bordered
      collapse-mode="width"
      :collapsed-width="64"
      :width="240"
      :collapsed="collapsed"
      show-trigger
      @collapse="collapsed = true"
      @expand="collapsed = false"
    >
      <div class="logo">
        <span v-if="!collapsed">Docker Manager</span>
      </div>
      <n-menu
        :collapsed="collapsed"
        :collapsed-width="64"
        :collapsed-icon-size="22"
        :options="menuOptions"
        :value="activeKey"
        @update:value="handleMenuClick"
      />
    </n-layout-sider>
    <n-layout>
      <n-layout-header
          style="height: 64px; padding: 0 24px; display: flex; align-items: center; justify-content: space-between; background-color: var(--macos-bg-light);">
        <n-h3 style="margin: 0; font-weight: 700; color: #1d1d1f;">{{ pageTitle }}</n-h3>
        <n-space>
          <!-- 这里以后可以放全局操作或状态 -->
        </n-space>
      </n-layout-header>
      <n-layout-content content-style="padding: 0 24px 24px 24px; background-color: var(--macos-bg-light);">
        <router-view />
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<script setup lang="ts">
import {Component, computed, h, ref, watch} from 'vue'
import type {MenuOption} from 'naive-ui'
import {NH3, NIcon, NLayout, NLayoutContent, NLayoutHeader, NLayoutSider, NMenu, NSpace} from 'naive-ui'
import {useRoute, useRouter} from 'vue-router'
import {CubeOutline, GlobeOutline, ImagesOutline, LayersOutline, LinkOutline, SaveOutline} from '@vicons/ionicons5'

const router = useRouter()
const route = useRoute()
const collapsed = ref(false)

const activeKey = ref<string>(route.name as string || 'containers')

// 监听路由变化更新激活的菜单项
watch(() => route.name, (newName) => {
  if (newName) {
    activeKey.value = newName as string
  }
})

function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

const menuOptions: MenuOption[] = [
  {
    label: 'Docker 链接',
    key: 'connections',
    icon: renderIcon(LinkOutline)
  },
  {
    label: 'Compose 管理',
    key: 'compose',
    icon: renderIcon(CubeOutline)
  },
  {
    label: '容器管理',
    key: 'containers',
    icon: renderIcon(LayersOutline)
  },
  {
    label: '镜像管理',
    key: 'images',
    icon: renderIcon(ImagesOutline)
  },
  {
    label: '网络管理',
    key: 'networks',
    icon: renderIcon(GlobeOutline)
  },
  {
    label: '卷管理',
    key: 'volumes',
    icon: renderIcon(SaveOutline)
  }
]

const pageTitle = computed(() => {
  const option = menuOptions.find(opt => opt.key === activeKey.value)
  return option ? option.label : 'Docker Manager'
})

function handleMenuClick(key: string) {
  activeKey.value = key
  router.push({ name: key })
}
</script>

<style scoped>
.logo {
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: bold;
  border-bottom: 1px solid var(--n-border-color);
  overflow: hidden;
  white-space: nowrap;
}
</style>
