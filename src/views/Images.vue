<script setup lang="ts">
import {computed, h, nextTick, onMounted, ref} from 'vue'
import {useImageStore} from '../store/image'
import {
  NButton,
  NDescriptions,
  NDescriptionsItem,
  NDropdown,
  NIcon,
  NInput,
  NInputGroup,
  NModal,
  NScrollbar,
  NSpace,
  NTag,
  useMessage
} from 'naive-ui'
import {CloudDownloadOutline, InformationCircleOutline, SearchOutline, TrashOutline} from '@vicons/ionicons5'

import MacOSList from '../components/common/MacOSList.vue'
import ResourceDetail from '../components/common/ResourceDetail.vue'

const imageStore = useImageStore()
const message = useMessage()

const selectedId = ref<string | null>(null)
const selectedItem = computed(() => imageStore.images.find(img => img.id === selectedId.value))
const pullImageName = ref('')
const showPullDrawer = ref(false)

const onSelect = (id: string) => {
  selectedId.value = id
}

const formatBytes = (bytes: number, decimals = 2) => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

const formatDate = (timestamp: number) => {
  return new Date(timestamp * 1000).toLocaleString()
}

const handleDelete = async (id: string) => {
  try {
    await imageStore.removeImage(id)
    message.success('镜像已删除')
    if (selectedId.value === id) selectedId.value = null
  } catch (err) {
    message.error('删除镜像失败: ' + err)
  }
}

const handlePull = async () => {
  if (!pullImageName.value) {
    message.warning('请输入镜像名称')
    return
  }
  showPullDrawer.value = true
  try {
    await imageStore.pullImage(pullImageName.value)
    pullImageName.value = ''
  } catch (err) {
    message.error('拉取失败: ' + err)
  }
}

const menuOptions = [
  {label: '详情', key: 'detail', icon: () => h(NIcon, null, {default: () => h(InformationCircleOutline)})},
  {label: '删除', key: 'delete', icon: () => h(NIcon, null, {default: () => h(TrashOutline)})}
]

const showMenu = ref(false)
const x = ref(0)
const y = ref(0)
const menuTarget = ref<any>(null)

const handleContextMenu = (e: MouseEvent, type: string, item: any) => {
  e.preventDefault()
  showMenu.value = false
  nextTick(() => {
    x.value = e.clientX
    y.value = e.clientY
    menuTarget.value = item
    showMenu.value = true
  })
}

const handleMenuSelect = (key: string) => {
  showMenu.value = false
  if (!menuTarget.value) return
  if (key === 'delete') handleDelete(menuTarget.value.id)
  else if (key === 'detail') onSelect(menuTarget.value.id)
}

onMounted(() => {
  imageStore.fetchImages()
})
</script>

<template>
  <div class="images-view">
    <div class="list-column floating-card">
      <div class="pull-section">
        <n-input-group>
          <n-input v-model:value="pullImageName" placeholder="拉取镜像..." round size="small" @keyup.enter="handlePull">
            <template #prefix>
              <n-icon :component="SearchOutline"/>
            </template>
          </n-input>
          <n-button :loading="imageStore.pulling" round size="small" type="primary" @click="handlePull">
            <template #icon>
              <n-icon :component="CloudDownloadOutline"/>
            </template>
          </n-button>
        </n-input-group>
      </div>
      <MacOSList
          :items="imageStore.images"
          :render-name="(item) => item.tags?.[0] || '<none>'"
          :render-sub="(item) => item.id.split(':')[1]?.substring(0, 12) || item.id.substring(0, 12)"
          :search-fields="['tags', 'id']"
          :selected-id="selectedId"
          id-key="id"
          placeholder="搜索镜像..."
          @contextmenu="handleContextMenu"
          @select="onSelect"
      />
    </div>

    <div class="detail-column floating-card">
      <ResourceDetail
          :item="selectedItem"
          :loading="imageStore.loading"
          :subtitle="selectedItem?.id"
          :title="selectedItem?.tags?.[0] || '镜像详情'"
          empty-text="请选择一个镜像以查看详情"
      >
        <template #actions>
          <n-button circle quaternary size="small" type="error" @click="handleDelete(selectedItem.id)">
            <template #icon>
              <n-icon>
                <TrashOutline/>
              </n-icon>
            </template>
          </n-button>
        </template>

        <n-scrollbar class="detail-content-scroll">
          <n-descriptions :column="1" bordered size="small" style="padding: 24px">
            <n-descriptions-item label="镜像 ID">
              <code>{{ selectedItem?.id }}</code>
            </n-descriptions-item>
            <n-descriptions-item label="标签">
              <n-space>
                <n-tag v-for="tag in selectedItem?.tags" :key="tag" bordered="false" size="small" type="info">
                  {{ tag }}
                </n-tag>
                <span v-if="!selectedItem?.tags || selectedItem?.tags.length === 0">无标签</span>
              </n-space>
            </n-descriptions-item>
            <n-descriptions-item label="占用空间">
              {{ formatBytes(selectedItem?.size || 0) }}
            </n-descriptions-item>
            <n-descriptions-item label="创建时间">
              {{ formatDate(selectedItem?.created || 0) }}
            </n-descriptions-item>
          </n-descriptions>
        </n-scrollbar>
      </ResourceDetail>
    </div>

    <!-- Pull Progress Modal -->
    <n-modal v-model:show="showPullDrawer" preset="card" style="width: 600px" title="拉取进度">
      <n-scrollbar style="max-height: 400px; background: #1e1e1e; padding: 12px; border-radius: 4px;">
        <div v-for="(log, idx) in imageStore.pullLogs" :key="idx" class="log-line">
          <span v-if="log.id" style="color: #888; margin-right: 8px">[{{ log.id }}]</span>
          <span>{{ log.status }}</span>
          <span v-if="log.progress" style="color: #aaa; margin-left: 8px">{{ log.progress }}</span>
        </div>
      </n-scrollbar>
    </n-modal>

    <n-dropdown
        :on-clickoutside="() => showMenu = false"
        :options="menuOptions"
        :show="showMenu"
        :x="x"
        :y="y"
        placement="bottom-start"
        trigger="manual"
        @select="handleMenuSelect"
    />
  </div>
</template>

<style scoped>
.images-view {
  display: flex;
  gap: 16px;
  height: calc(100vh - 64px - 32px);
}

.list-column {
  width: 320px;
  flex-shrink: 0;
}

.detail-column {
  flex: 1;
  min-width: 0;
}

.floating-card {
  background-color: var(--macos-card-bg-light);
  border-radius: var(--macos-radius);
  border: 1px solid var(--macos-border-color);
  box-shadow: var(--macos-shadow);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.pull-section {
  padding: 16px;
  border-bottom: 0.5px solid var(--macos-border-color);
}

.detail-content-scroll {
  flex: 1;
}
.log-line {
  color: #d4d4d4;
  font-family: monospace;
  font-size: 12px;
  margin-bottom: 2px;
}
</style>
