<script setup lang="ts">
import {computed, h, nextTick, onMounted, ref, watch} from 'vue'
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
  NTable,
  NTag,
  useMessage
} from 'naive-ui'
import {InformationCircleOutline, LeafOutline, RefreshOutline, TrashOutline, FolderOpenOutline} from '@vicons/ionicons5'

import MacOSList from '../components/common/MacOSList.vue'
import ResourceDetail from '../components/common/ResourceDetail.vue'

const volumeStore = useVolumeStore()
const message = useMessage()

const selectedId = ref<string | null>(null)
const selectedItem = computed(() => volumeStore.volumes.find(v => v.name === selectedId.value))

const tabs = [
  { label: '概览', value: 'overview' },
  { label: '使用者', value: 'users' }
]

const onSelect = async (id: string) => {
  selectedId.value = id
  await volumeStore.fetchVolumeUsers(id)
}

watch(selectedId, async (newId) => {
  if (newId) {
    await volumeStore.fetchVolumeUsers(newId)
  }
})

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

const handleOpenPath = async (path: string) => {
  try {
    await volumeStore.openPath(path)
    message.success('已打开路径')
  } catch (err) {
    message.error('打开失败: ' + err)
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

const handleContextMenu = (e: MouseEvent, _type: string, item: any) => {
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
          :subtitle="selectedItem?.driver || ''"
          :title="selectedItem?.name || '数据卷详情'"
          :tabs="tabs"
          empty-text="请选择一个数据卷以查看详情"
      >
        <template #actions>
          <n-button v-if="selectedItem" circle quaternary size="small" type="error"
                    @click="handleDelete(selectedItem.name)">
            <template #icon>
              <n-icon>
                <TrashOutline/>
              </n-icon>
            </template>
          </n-button>
        </template>

        <template #overview>
          <n-scrollbar class="tab-pane-content">
            <div style="padding: 24px">
              <n-descriptions :column="1" bordered size="small">
                <n-descriptions-item label="名称">
                  {{ selectedItem?.name }}
                </n-descriptions-item>
                <n-descriptions-item label="驱动">
                  {{ selectedItem?.driver }}
                </n-descriptions-item>
                <n-descriptions-item label="挂载点">
                  <n-space vertical>
                    <code>{{ selectedItem?.mountpoint }}</code>
                    <n-button 
                      v-if="selectedItem?.mountpoint" 
                      size="tiny" 
                      secondary 
                      type="primary"
                      @click="handleOpenPath(selectedItem.mountpoint)"
                    >
                      <template #icon>
                        <n-icon><FolderOpenOutline /></n-icon>
                      </template>
                      在文件管理器中打开
                    </n-button>
                  </n-space>
                </n-descriptions-item>
                <n-descriptions-item label="创建时间">
                  {{ selectedItem?.created ? new Date(selectedItem.created).toLocaleString() : '未知' }}
                </n-descriptions-item>
              </n-descriptions>
            </div>
          </n-scrollbar>
        </template>

        <template #users>
          <n-scrollbar class="tab-pane-content">
            <div style="padding: 24px">
              <div v-if="volumeStore.volumeUsers.length === 0" class="empty-users">
                暂无容器使用此数据卷
              </div>
              <n-table v-else :bordered="false" :single-column="false" size="small">
                <thead>
                  <tr>
                    <th>容器</th>
                    <th>源路径</th>
                    <th>目标路径</th>
                    <th>模式</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="user in volumeStore.volumeUsers" :key="user.container_id">
                    <td>
                      <n-tag :bordered="false" size="small" type="info">
                        {{ user.container_name }}
                      </n-tag>
                    </td>
                    <td><code class="path-code">{{ user.source }}</code></td>
                    <td><code class="path-code">{{ user.destination }}</code></td>
                    <td>
                      <n-tag :bordered="false" size="small" :type="user.rw ? 'success' : 'warning'">
                        {{ user.mode }} ({{ user.rw ? '读写' : '只读' }})
                      </n-tag>
                    </td>
                  </tr>
                </tbody>
              </n-table>
            </div>
          </n-scrollbar>
        </template>
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

.tab-pane-content {
  height: calc(100vh - 180px);
}

.empty-users {
  text-align: center;
  padding: 40px;
  color: #86868b;
  font-size: 14px;
}

.path-code {
  font-size: 12px;
  word-break: break-all;
}
</style>
