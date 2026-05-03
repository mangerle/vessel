<script setup lang="ts">
import {computed, h, nextTick, onMounted, ref} from 'vue'
import {useVolumeStore} from '../store/volume'
import {
  NButton,
  NDescriptions,
  NDescriptionsItem,
  NDropdown,
  NIcon,
  NPopconfirm,
  NScrollbar,
  NSpace,
  useMessage
} from 'naive-ui'
import {InformationCircleOutline, LeafOutline, RefreshOutline, TrashOutline} from '@vicons/ionicons5'

import MacOSList from '../components/common/MacOSList.vue'
import ResourceDetail from '../components/common/ResourceDetail.vue'

const volumeStore = useVolumeStore()
const message = useMessage()

const selectedId = ref<string | null>(null)
const selectedItem = computed(() => volumeStore.volumes.find(v => v.name === selectedId.value))

const onSelect = (id: string) => {
  selectedId.value = id
}

const handleDelete = async (name: string) => {
  try {
    await volumeStore.removeVolume(name)
    message.success('数据卷已删除')
    if (selectedId.value === name) selectedId.value = null
  } catch (err) {
    message.error('删除失败: ' + err)
  }
}

const handlePrune = async () => {
  try {
    await volumeStore.pruneVolumes()
    message.success('清理完成')
  } catch (err) {
    message.error('清理失败: ' + err)
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
  if (key === 'delete') handleDelete(menuTarget.value.name)
  else if (key === 'detail') onSelect(menuTarget.value.name)
}

onMounted(() => {
  volumeStore.fetchVolumes()
})
</script>

<template>
  <div class="volumes-view">
    <div class="list-column floating-card">
      <div class="action-bar">
        <n-space>
          <n-button circle quaternary size="small" @click="volumeStore.fetchVolumes">
            <template #icon>
              <n-icon>
                <RefreshOutline/>
              </n-icon>
            </template>
          </n-button>
          <n-popconfirm @positive-click="handlePrune">
            <template #trigger>
              <n-button circle quaternary size="small" type="warning">
                <template #icon>
                  <n-icon>
                    <LeafOutline/>
                  </n-icon>
                </template>
              </n-button>
            </template>
            确定要清理所有未使用的卷吗？
          </n-popconfirm>
        </n-space>
      </div>
      <MacOSList
          :items="volumeStore.volumes"
          :render-name="(item) => item.name"
          :render-sub="(item) => item.driver"
          :search-fields="['name', 'driver']"
          :selected-id="selectedId"
          id-key="name"
          placeholder="搜索数据卷..."
          @contextmenu="handleContextMenu"
          @select="onSelect"
      />
    </div>

    <div class="detail-column floating-card">
      <ResourceDetail
          :item="selectedItem"
          :loading="volumeStore.loading"
          :subtitle="selectedItem?.driver"
          :title="selectedItem?.name || '数据卷详情'"
          empty-text="请选择一个数据卷以查看详情"
      >
        <template #actions>
          <n-button circle quaternary size="small" type="error" @click="handleDelete(selectedItem.name)">
            <template #icon>
              <n-icon>
                <TrashOutline/>
              </n-icon>
            </template>
          </n-button>
        </template>

        <n-scrollbar class="detail-content-scroll">
          <n-descriptions :column="1" bordered size="small" style="padding: 24px">
            <n-descriptions-item label="名称">
              {{ selectedItem?.name }}
            </n-descriptions-item>
            <n-descriptions-item label="驱动">
              {{ selectedItem?.driver }}
            </n-descriptions-item>
            <n-descriptions-item label="挂载点">
              <code>{{ selectedItem?.mountpoint }}</code>
            </n-descriptions-item>
            <n-descriptions-item label="创建时间">
              {{ selectedItem?.created ? new Date(selectedItem.created).toLocaleString() : '未知' }}
            </n-descriptions-item>
          </n-descriptions>
        </n-scrollbar>
      </ResourceDetail>
    </div>

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
.volumes-view {
  display: flex;
  gap: 16px;
  height: calc(100vh - 40px);
}

.list-column {
  width: 260px;
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

.action-bar {
  padding: 16px;
  border-bottom: 0.5px solid var(--macos-border-color);
}

.detail-content-scroll {
  flex: 1;
}
</style>
