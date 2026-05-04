<script setup lang="ts">
import {computed, h, nextTick, onMounted, ref} from 'vue'
import {useNetworkStore} from '../store/network'
import {
  NButton,
  NDescriptions,
  NDescriptionsItem,
  NDropdown,
  NIcon,
  NPopconfirm,
  NScrollbar,
  NSpace,
  NDataTable,
  useMessage
} from 'naive-ui'
import {InformationCircleOutline, LeafOutline, RefreshOutline, TrashOutline} from '@vicons/ionicons5'

import MacOSList from '../components/common/MacOSList.vue'
import ResourceDetail from '../components/common/ResourceDetail.vue'

const networkStore = useNetworkStore()
const message = useMessage()

const selectedId = ref<string | null>(null)
const selectedItem = computed(() => networkStore.currentNetwork)

const tabs = [
  {label: '概览', value: 'overview'},
  {label: '已连接容器', value: 'containers'}
]

const containerColumns = [
  {title: '名称', key: 'name', minWidth: 150},
  {title: 'IPv4 地址', key: 'ipv4_address', minWidth: 120},
  {title: 'IPv6 地址', key: 'ipv6_address', minWidth: 120},
  {title: 'MAC 地址', key: 'mac_address', minWidth: 140}
]

const onSelect = async (id: string) => {
  selectedId.value = id
  try {
    await networkStore.fetchNetworkDetails(id)
  } catch (err) {
    message.error('获取网络详情失败: ' + err)
  }
}

const handleDelete = async (id: string) => {
  try {
    await networkStore.removeNetwork(id)
    message.success('网络已删除')
    if (selectedId.value === id) {
      selectedId.value = null
      networkStore.currentNetwork = null
    }
  } catch (err) {
    message.error('删除网络失败: ' + err)
  }
}

const handlePrune = async () => {
  try {
    await networkStore.pruneNetworks()
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
  if (key === 'delete') handleDelete(menuTarget.value.id)
  else if (key === 'detail') onSelect(menuTarget.value.id)
}

onMounted(() => {
  networkStore.fetchNetworks()
})
</script>

<template>
  <div class="networks-view">
    <div class="list-column floating-card">
      <div class="action-bar">
        <n-space>
          <n-button circle quaternary size="small" @click="networkStore.fetchNetworks">
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
            确定要清理未使用的网络吗？
          </n-popconfirm>
        </n-space>
      </div>
      <MacOSList
          :items="networkStore.networks"
          :render-name="(item) => item.name"
          :render-sub="(item) => `${item.driver} | ${item.scope}`"
          :search-fields="['name', 'id', 'driver']"
          :selected-id="selectedId"
          id-key="id"
          placeholder="搜索网络..."
          @contextmenu="handleContextMenu"
          @select="onSelect"
      />
    </div>

    <div class="detail-column floating-card">
      <ResourceDetail
          :item="selectedItem"
          :loading="networkStore.loading"
          :subtitle="selectedItem?.id || ''"
          :tabs="tabs"
          :title="selectedItem?.name || '网络详情'"
          empty-text="请选择一个网络以查看详情"
      >
        <template #actions>
          <n-button v-if="selectedItem" circle quaternary size="small" type="error"
                    @click="handleDelete(selectedItem.id)">
            <template #icon>
              <n-icon>
                <TrashOutline/>
              </n-icon>
            </template>
          </n-button>
        </template>

        <template #overview>
          <n-scrollbar class="tab-pane-content">
            <div class="detail-section">
              <n-descriptions :column="1" bordered label-placement="left" size="small">
                <n-descriptions-item label="名称">
                  {{ selectedItem?.name }}
                </n-descriptions-item>
                <n-descriptions-item label="网络 ID">
                  <code>{{ selectedItem?.id }}</code>
                </n-descriptions-item>
                <n-descriptions-item label="驱动">
                  {{ selectedItem?.driver }}
                </n-descriptions-item>
                <n-descriptions-item label="范围">
                  {{ selectedItem?.scope }}
                </n-descriptions-item>
                <n-descriptions-item label="子网">
                  {{ selectedItem?.subnet || 'N/A' }}
                </n-descriptions-item>
                <n-descriptions-item label="网关">
                  {{ selectedItem?.gateway || 'N/A' }}
                </n-descriptions-item>
                <n-descriptions-item label="内部网络">
                  {{ selectedItem?.internal ? '是' : '否' }}
                </n-descriptions-item>
                <n-descriptions-item label="可连接">
                  {{ selectedItem?.attachable ? '是' : '否' }}
                </n-descriptions-item>
                <n-descriptions-item label="创建时间">
                  {{ selectedItem?.created ? new Date(selectedItem.created).toLocaleString() : '未知' }}
                </n-descriptions-item>
              </n-descriptions>
            </div>
          </n-scrollbar>
        </template>

        <template #containers>
          <div class="tab-pane-content" style="padding: 0">
            <n-data-table
                :columns="containerColumns"
                :data="selectedItem?.containers || []"
                :max-height="'calc(100vh - 200px)'"
                size="small"
            />
          </div>
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
.networks-view {
  display: flex;
  gap: 16px;
  height: calc(100vh - 40px);
}

.list-column {
  width: 280px;
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

.detail-section {
  padding: 24px;
}
</style>
