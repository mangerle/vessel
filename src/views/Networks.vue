<script setup lang="ts">
import { computed, nextTick, onMounted, ref, h } from 'vue'
import { useNetworkStore } from '../store/network'
import {
  NDropdown,
  NScrollbar,
  NIcon,
  useMessage
} from 'naive-ui'
import {
  GitNetworkOutline,
  DocumentTextOutline,
  TrashOutline,
  SparklesOutline,
  SwapHorizontalOutline,
  FlashOutline,
  CloseCircleOutline,
  FileTrayFullOutline
} from '@vicons/ionicons5'

const networkStore = useNetworkStore()
const message = useMessage()

// --- 状态控制 ---
const selectedId = ref<string | null>(null)
const selectedItem = computed(() => networkStore.currentNetwork)
const activeTab = ref('topology') // 默认：topology 🗺️ 拓扑结构

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
    message.success('虚拟网络已从宿主机中移除')
    if (selectedId.value === id) {
      selectedId.value = null
      networkStore.currentNetwork = null
    }
    await networkStore.fetchNetworks()
  } catch (err) {
    message.error('删除网络失败: ' + err)
  }
}

const handleDisconnect = async (containerId: string) => {
  if (!selectedId.value) return
  try {
    await networkStore.disconnectContainer(selectedId.value, containerId)
    message.success('已断开容器与该网络的连接')
    // 重新获取详情
    await networkStore.fetchNetworkDetails(selectedId.value)
  } catch (err) {
    message.error('断开连接失败: ' + err)
  }
}

const handlePrune = async () => {
  try {
    await networkStore.pruneNetworks()
    message.success('清理未使用的虚拟网络成功')
    await networkStore.fetchNetworks()
  } catch (err) {
    message.error('清理失败: ' + err)
  }
}

// 驱动图标前缀
const getDriverIcon = (driver: string) => {
  if (driver === 'bridge') return SwapHorizontalOutline
  if (driver === 'host') return FlashOutline
  return CloseCircleOutline
}

// --- 右键菜单 ---
const showMenu = ref(false)
const x = ref(0)
const y = ref(0)
const menuTarget = ref<{ id: string; name: string } | null>(null)

const menuOptions = [
  { label: '拓扑详情', key: 'detail', icon: () => h(NIcon, null, { default: () => h(GitNetworkOutline) }) },
  { label: '删除网络', key: 'delete', icon: () => h(NIcon, null, { default: () => h(TrashOutline) }) }
]

const handleContextMenu = (e: MouseEvent, item: any) => {
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

const sortedNetworks = computed(() => {
  return [...networkStore.networks].sort((a, b) => a.name.localeCompare(b.name))
})

onMounted(() => {
  networkStore.fetchNetworks()
})
</script>

<template>
  <div class="networks-view">
    <!-- 左侧网络驱动分流列表 -->
    <div class="list-column">
      <!-- 顶栏 40px 高度: 一键清理全部未用网络 -->
      <div class="header-tools">
        <button class="prune-btn" @click="handlePrune">
          <n-icon :component="SparklesOutline" />
          一键清理全部未用网络
        </button>
      </div>

      <n-scrollbar class="list-scroll-box">
        <div 
          v-for="item in sortedNetworks" 
          :key="item.id" 
          class="network-item-row"
          :class="{ active: selectedId === item.id }"
          @click="onSelect(item.id)"
          @contextmenu="handleContextMenu($event, item)"
        >
          <div class="item-left-meta">
            <!-- 网络名与驱动类型图标 -->
            <div class="item-tag-title">
              <n-icon :component="getDriverIcon(item.driver)" style="margin-right: 6px" />
              <span>{{ item.name }}</span>
            </div>
            <!-- 网关/驱动 -->
            <div class="item-sub-meta">
              {{ item.driver }} · {{ item.scope }}
            </div>
          </div>
        </div>
      </n-scrollbar>
    </div>

    <!-- 右侧网络详情全景观测台 -->
    <div class="detail-column">
      <template v-if="selectedItem">
        <!-- 顶层双行控制栏 (高 72px) -->
        <div class="detail-header-wrapper">
          <!-- 行 1: 选项卡 (高 32px) -->
          <div class="tab-line-1">
            <div class="obs-tab" :class="{ active: activeTab === 'topology' }" @click="activeTab = 'topology'">
              <n-icon :component="GitNetworkOutline" />
              <span>拓扑结构</span>
              <div class="tab-indicator"></div>
            </div>
            <div class="obs-tab" :class="{ active: activeTab === 'inspect' }" @click="activeTab = 'inspect'">
              <n-icon :component="DocumentTextOutline" />
              <span>网络详情 (Inspect)</span>
              <div class="tab-indicator"></div>
            </div>
          </div>

          <!-- 行 2: 基础元数据与动作 (高 40px) -->
          <div class="meta-line-2">
            <div class="meta-left">
              <span class="network-name-title">🌐 {{ selectedItem.name }}</span>
              <span class="network-meta-sub">{{ selectedItem.id.substring(0, 12) }}</span>
              <div class="vertical-divider"></div>
              <span class="badge driver-badge">🔌 {{ selectedItem.driver }}</span>
              <span class="badge gw-badge">网关: {{ selectedItem.gateway || 'N/A' }}</span>
            </div>

            <!-- 安全删除网络 -->
            <div class="meta-right">
              <button class="delete-btn" @click="handleDelete(selectedItem.id)">
                <n-icon :component="TrashOutline" />
                移除网络
              </button>
            </div>
          </div>
        </div>

        <!-- 下方主内容区 -->
        <div class="detail-content-area">
          <!-- 1. 🗺️ 拓扑结构 -->
          <div v-show="activeTab === 'topology'" class="topology-pane">
            <div class="topology-tree-box">
              <div class="gateway-node">
                [🌐 虚拟网关 IP: {{ selectedItem.gateway || '172.17.0.1' }}]
              </div>
              <div v-if="selectedItem.containers && selectedItem.containers.length > 0" class="connected-containers-branch">
                <div 
                  v-for="(c, idx) in selectedItem.containers" 
                  :key="c.id" 
                  class="container-branch-row"
                >
                  <span class="branch-bullet">
                    {{ idx === selectedItem.containers.length - 1 ? '└──' : '├──' }}
                  </span>
                  <!-- 小圆点表示已连接 -->
                  <span class="conn-dot"></span>
                  <span class="conn-name">{{ c.name }}</span>
                  <span class="connector-line">------------------------</span>
                  <span class="conn-ip">IP: {{ c.ipv4_address || 'N/A' }}</span>
                  <span class="conn-mac" v-if="c.mac_address">(Mac: {{ c.mac_address }})</span>
                  
                  <!-- 断开网络按钮 -->
                  <button class="disconnect-link-btn" @click="handleDisconnect(c.id)">
                    <n-icon :component="FlashOutline" />
                    断开
                  </button>
                </div>
              </div>
              <div v-else class="empty-topology-text">
                └── <n-icon :component="FileTrayFullOutline" /> 目前无任何容器挂载在该虚拟局域网下
              </div>
            </div>
          </div>

          <!-- 2. 📋 网络详情 -->
          <div v-show="activeTab === 'inspect'" class="inspect-pane">
            <n-scrollbar style="height: 100%">
              <div class="inspect-card-box">
                <div class="inspect-row"><span class="key">网络名称:</span> <span class="val">{{ selectedItem.name }}</span></div>
                <div class="inspect-row"><span class="key">网络 ID:</span> <span class="val">{{ selectedItem.id }}</span></div>
                <div class="inspect-row"><span class="key">网络驱动:</span> <span class="val">{{ selectedItem.driver }}</span></div>
                <div class="inspect-row"><span class="key">网络范围:</span> <span class="val">{{ selectedItem.scope }}</span></div>
                <div class="inspect-row"><span class="key">子网范围:</span> <span class="val">{{ selectedItem.subnet || 'N/A' }}</span></div>
                <div class="inspect-row"><span class="key">网关地址:</span> <span class="val">{{ selectedItem.gateway || 'N/A' }}</span></div>
                <div class="inspect-row"><span class="key">内部局域网:</span> <span class="val">{{ selectedItem.internal ? '是' : '否' }}</span></div>
                <div class="inspect-row"><span class="key">物理可连接:</span> <span class="val">{{ selectedItem.attachable ? '是' : '否' }}</span></div>
              </div>
            </n-scrollbar>
          </div>
        </div>
      </template>

      <!-- 空白缺省页 -->
      <div v-else class="empty-state">
        <div class="empty-logo">
          <img src="/logo.png" alt="Vessel Logo" style="width: 80px; height: 80px; object-fit: contain;" />
        </div>
        <div class="empty-title">网络局域网观测台</div>
        <div class="empty-sub">选择左侧的虚拟网络以观察其网关、已挂载服务容器以及物理局域网拓扑结构。</div>
      </div>
    </div>
  </div>

  <!-- 右键下拉 -->
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
</template>

<style scoped>
.networks-view {
  display: flex;
  width: 100%;
  height: 100%;
  gap: 12px;
}

.list-column {
  width: 240px;
  background-color: var(--bg-sidebar);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.header-tools {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.prune-btn {
  width: 100%;
  height: 24px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-body);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.prune-btn:hover {
  background-color: rgba(245, 158, 11, 0.1);
  border-color: var(--brand-warn);
  color: var(--brand-warn);
}

.list-scroll-box {
  flex: 1;
}

.network-item-row {
  height: 32px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  cursor: pointer;
  color: var(--text-body);
  transition: all 0.15s ease;
  border-bottom: 1px solid var(--border-color);
}
.network-item-row:hover {
  background-color: rgba(255, 255, 255, 0.02);
  color: var(--text-title);
}
.network-item-row.active {
  background-color: var(--bg-active) !important;
  color: var(--text-title);
  font-weight: 600;
}

.item-left-meta {
  display: flex;
  flex-direction: column;
  justify-content: center;
  overflow: hidden;
  width: 100%;
}

.item-tag-title {
  font-size: 11px;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-sub-meta {
  font-size: 9px;
  color: var(--text-muted);
  margin-top: 1px;
}

.detail-column {
  flex: 1;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-main);
  overflow: hidden;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

/* 顶层双行控制栏 */
.detail-header-wrapper {
  height: 72px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.tab-line-1 {
  height: 32px;
  display: flex;
  align-items: center;
  padding-left: 16px;
  gap: 20px;
}

.obs-tab {
  position: relative;
  height: 100%;
  display: flex;
  align-items: center;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-muted);
  cursor: pointer;
  transition: color 0.15s ease;
}
.obs-tab:hover {
  color: var(--text-title);
}
.obs-tab.active {
  color: var(--text-title);
  font-weight: 700;
}
.tab-indicator {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 2px;
  background-color: var(--brand-primary);
  transform: scaleX(0);
  transition: transform 0.15s ease;
}
.obs-tab.active .tab-indicator {
  transform: scaleX(1);
}

.meta-line-2 {
  height: 40px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
}

.meta-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.network-name-title {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-title);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 250px;
}

.network-meta-sub {
  font-size: 11px;
  font-family: monospace;
  color: var(--text-muted);
}

.vertical-divider {
  width: 1px;
  height: 14px;
  background-color: var(--border-color);
}

.badge {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  background-color: rgba(255, 255, 255, 0.04);
  color: var(--text-body);
}

.meta-right {
  display: flex;
  align-items: center;
}

.delete-btn {
  height: 26px;
  padding: 0 12px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.delete-btn:hover {
  border-color: var(--brand-danger);
  color: var(--brand-danger);
  background-color: rgba(239, 68, 68, 0.05);
}

/* 拓扑图与关系树 */
.detail-content-area {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.topology-pane {
  height: 100%;
  background-color: var(--bg-terminal);
  padding: 24px;
  overflow-y: auto;
}

.topology-tree-box {
  font-family: monospace;
  font-size: 11px;
  line-height: 1.8;
  color: var(--text-terminal);
}

.gateway-node {
  color: #38bdf8;
  font-weight: 700;
  margin-bottom: 6px;
}

.connected-containers-branch {
  display: flex;
  flex-direction: column;
}

.container-branch-row {
  display: flex;
  align-items: center;
  white-space: nowrap;
}

.branch-bullet {
  color: var(--text-muted);
  margin-right: 8px;
  opacity: 0.7;
}

.conn-dot {
  margin-right: 6px;
  font-size: 9px;
}

.conn-name {
  color: var(--text-title);
  font-weight: 600;
}

.connector-line {
  color: rgba(255, 255, 255, 0.02);
  margin: 0 8px;
  letter-spacing: -1px;
}

.conn-ip {
  color: var(--brand-primary);
  font-weight: 600;
  margin-right: 8px;
}

.conn-mac {
  color: var(--text-muted);
}

.disconnect-link-btn {
  margin-left: 16px;
  height: 18px;
  padding: 0 6px;
  background-color: rgba(239, 68, 68, 0.05);
  border: 1px solid rgba(239, 68, 68, 0.15);
  border-radius: 3px;
  color: var(--brand-danger);
  font-size: 9px;
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.disconnect-link-btn:hover {
  background-color: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.3);
}

.empty-topology-text {
  color: var(--text-muted);
  font-style: italic;
  padding-left: 20px;
}

/* Inspect */
.inspect-pane {
  padding: 16px;
  height: 100%;
}

.inspect-card-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 11px;
}

.inspect-row {
  display: flex;
  gap: 8px;
}

.inspect-row .key {
  color: var(--text-muted);
  width: 90px;
}
.inspect-row .val {
  color: var(--text-body);
  font-family: monospace;
}

/* 空状态 */
.empty-state {
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
}

.empty-logo {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-title);
  margin-bottom: 6px;
}

.empty-sub {
  font-size: 11px;
  color: var(--text-muted);
  max-width: 300px;
  line-height: 1.4;
}
</style>
