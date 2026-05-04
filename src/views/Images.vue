<script setup lang="ts">
import {computed, h, nextTick, onMounted, ref} from 'vue'
import {useImageStore} from '../store/image'
import {
  NAutoComplete,
  NButton,
  NDescriptions,
  NDescriptionsItem,
  NDropdown,
  NIcon,
  NModal,
  NScrollbar,
  NSpace,
  NTag,
  NTimeline,
  NTimelineItem,
  NText,
  useMessage,
  AutoCompleteOption
} from 'naive-ui'
import {
  CloudDownloadOutline,
  InformationCircleOutline,
  TrashOutline,
  Star,
  ShieldCheckmark
} from '@vicons/ionicons5'

import MacOSList from '../components/common/MacOSList.vue'
import ResourceDetail from '../components/common/ResourceDetail.vue'

const imageStore = useImageStore()
const message = useMessage()

const selectedId = ref<string | null>(null)
const selectedDetails = ref<any>(null)
const loadingDetails = ref(false)
const pullImageName = ref('')
const showPullDrawer = ref(false)

const tabs = [
  {label: '概览', value: 'overview'},
  {label: '层级', value: 'layers'},
  {label: '配置', value: 'config'}
]

const onSelect = async (id: string) => {
  selectedId.value = id
  loadingDetails.value = true
  try {
    const [details] = await Promise.all([
      imageStore.inspectImage(id),
      imageStore.fetchImageHistory(id)
    ])
    selectedDetails.value = details
  } catch (err) {
    message.error('获取镜像详情失败: ' + err)
  } finally {
    loadingDetails.value = false
  }
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
  if (!timestamp) return 'N/A'
  return new Date(timestamp * 1000).toLocaleString()
}

const timeAgo = (timestamp: number) => {
  if (!timestamp) return ''
  const seconds = Math.floor(Date.now() / 1000 - timestamp)
  let interval = seconds / 31536000
  if (interval > 1) return Math.floor(interval) + ' 年前'
  interval = seconds / 2592000
  if (interval > 1) return Math.floor(interval) + ' 个月前'
  interval = seconds / 86400
  if (interval > 1) return Math.floor(interval) + ' 天前'
  interval = seconds / 3600
  if (interval > 1) return Math.floor(interval) + ' 小时前'
  interval = seconds / 60
  if (interval > 1) return Math.floor(interval) + ' 分钟前'
  return Math.floor(seconds) + ' 秒前'
}

const handleDelete = async (id: string) => {
  try {
    await imageStore.removeImage(id)
    message.success('镜像已删除')
    if (selectedId.value === id) {
      selectedId.value = null
      selectedDetails.value = null
    }
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

const handleSearch = async (query: string) => {
  if (query.length > 1) {
    await imageStore.searchImages(query)
  }
}

const autoCompleteOptions = computed(() => {
  return imageStore.searchResults.map(res => ({
    label: res.name,
    value: res.name,
    description: res.description,
    is_official: res.is_official,
    star_count: res.star_count
  }))
})

const renderLabel = (option: AutoCompleteOption) => {
  return h(
      'div',
      {style: {display: 'flex', flexDirection: 'column', padding: '4px 0'}},
      [
        h('div', {style: {display: 'flex', alignItems: 'center', justifyContent: 'space-between'}}, [
          h('div', {style: {fontWeight: 'bold'}}, option.label as string),
          h('div', {style: {display: 'flex', alignItems: 'center', gap: '8px'}}, [
            option.is_official ? h(NTag, {size: 'tiny', type: 'success', bordered: false}, {
              default: () => h(NSpace, {align: 'center', size: 2}, {
                default: () => [
                  h(NIcon, {component: ShieldCheckmark, size: 10}),
                  h('span', 'Official')
                ]
              })
            }) : null,
            h(NSpace, {align: 'center', size: 2}, {
              default: () => [
                h(NIcon, {component: Star, color: '#f0a020', size: 12}),
                h('span', {style: {fontSize: '11px'}}, option.star_count as string)
              ]
            })
          ])
        ]),
        h('div', {style: {fontSize: '11px', opacity: 0.6, marginTop: '2px', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis'}}, option.description as string)
      ]
  )
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
  imageStore.fetchImages()
})
</script>

<template>
  <div class="images-view">
    <div class="list-column floating-card">
      <div class="pull-header">
        <n-auto-complete
            v-model:value="pullImageName"
            :options="autoCompleteOptions"
            :render-label="renderLabel"
            placeholder="输入并拉取镜像..."
            size="small"
            @input="handleSearch"
            @select="(val) => pullImageName = val"
        >
          <template #suffix>
            <n-button :loading="imageStore.pulling" circle quaternary size="tiny" @click="handlePull">
              <template #icon>
                <n-icon :component="CloudDownloadOutline"/>
              </template>
            </n-button>
          </template>
        </n-auto-complete>
      </div>
      <MacOSList
          :id-key="'id'"
          :items="imageStore.images"
          :placeholder="'搜索镜像...'"
          :render-name="(item) => item.tags?.[0] || '<none>'"
          :render-sub="(item) => `${item.id.split(':')[1]?.substring(0, 12) || item.id.substring(0, 12)} · ${formatBytes(item.size)} · ${timeAgo(item.created)}`"
          :search-fields="['tags', 'id']"
          :selected-id="selectedId"
          @contextmenu="handleContextMenu"
          @select="onSelect"
      />
    </div>

    <div class="detail-column floating-card">
      <ResourceDetail
          :empty-text="'请选择一个镜像以查看详情'"
          :item="selectedDetails"
          :loading="loadingDetails"
          :subtitle="selectedDetails?.id || ''"
          :tabs="tabs"
          :title="selectedDetails?.tags?.[0] || '镜像详情'"
      >
        <template #actions>
          <n-button v-if="selectedDetails" circle quaternary size="small" type="error"
                    @click="handleDelete(selectedDetails.id)">
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
                <n-descriptions-item label="镜像 ID">
                  <code>{{ selectedDetails?.id }}</code>
                </n-descriptions-item>
                <n-descriptions-item label="架构">
                  {{ selectedDetails?.architecture || 'N/A' }}
                </n-descriptions-item>
                <n-descriptions-item label="操作系统">
                  {{ selectedDetails?.os || 'N/A' }}
                </n-descriptions-item>
                <n-descriptions-item label="占用空间">
                  {{ formatBytes(selectedDetails?.size || 0) }}
                </n-descriptions-item>
                <n-descriptions-item label="创建时间">
                  {{ formatDate(selectedDetails?.created || 0) }}
                </n-descriptions-item>
                <n-descriptions-item label="镜像标签">
                  <n-space size="small">
                    <n-tag v-for="tag in selectedDetails?.tags" :key="tag" :bordered="false" size="small" type="info">
                      {{ tag }}
                    </n-tag>
                    <span v-if="!selectedDetails?.tags || selectedDetails?.tags.length === 0">无标签</span>
                  </n-space>
                </n-descriptions-item>
              </n-descriptions>
            </div>
          </n-scrollbar>
        </template>

        <template #layers>
          <n-scrollbar class="tab-pane-content">
            <div class="detail-section timeline-section">
              <n-timeline size="medium">
                <n-timeline-item
                    v-for="(layer, idx) in imageStore.imageHistory"
                    :key="idx"
                    :content="layer.created_by"
                    :time="formatDate(layer.created)"
                    :type="layer.id === '<missing>' ? 'default' : 'info'"
                >
                  <template #header>
                    <div class="layer-header">
                      <span class="layer-id" v-if="layer.id !== '<missing>'">{{ layer.id.substring(0, 12) }}</span>
                      <span class="layer-size">{{ formatBytes(layer.size) }}</span>
                    </div>
                  </template>
                </n-timeline-item>
              </n-timeline>
            </div>
          </n-scrollbar>
        </template>

        <template #config>
          <n-scrollbar class="tab-pane-content">
            <div class="detail-section">
              <div class="config-group">
                <div class="config-title">环境变量 (ENV)</div>
                <div v-if="selectedDetails?.env?.length" class="config-list">
                  <div v-for="e in selectedDetails.env" :key="e" class="config-item">
                    <code>{{ e }}</code>
                  </div>
                </div>
                <n-text v-else depth="3">无环境变量</n-text>
              </div>

              <div class="config-group">
                <div class="config-title">暴露端口 (Exposed Ports)</div>
                <n-space v-if="selectedDetails?.exposed_ports?.length">
                  <n-tag v-for="p in selectedDetails.exposed_ports" :key="p" :bordered="false" size="small" type="success">
                    {{ p }}
                  </n-tag>
                </n-space>
                <n-text v-else depth="3">无暴露端口</n-text>
              </div>

              <div class="config-group">
                <div class="config-title">启动命令 (CMD)</div>
                <div v-if="selectedDetails?.cmd?.length" class="config-command">
                  <code>{{ selectedDetails.cmd.join(' ') }}</code>
                </div>
                <n-text v-else depth="3">无启动命令</n-text>
              </div>

              <div class="config-group">
                <div class="config-title">入口点 (Entrypoint)</div>
                <div v-if="selectedDetails?.entrypoint?.length" class="config-command">
                  <code>{{ selectedDetails.entrypoint.join(' ') }}</code>
                </div>
                <n-text v-else depth="3">无入口点</n-text>
              </div>
            </div>
          </n-scrollbar>
        </template>
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
  height: calc(100vh - 40px);
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

.pull-header {
  padding: 16px;
  border-bottom: 0.5px solid var(--macos-border-color);
}

.tab-pane-content {
  height: calc(100vh - 180px);
}

.detail-section {
  padding: 24px;
}

.timeline-section {
  padding-top: 16px;
}

.layer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.layer-id {
  font-family: monospace;
  font-weight: 600;
  font-size: 12px;
}

.layer-size {
  font-size: 11px;
  opacity: 0.7;
}

.config-group {
  margin-bottom: 24px;
}

.config-title {
  font-weight: 600;
  margin-bottom: 8px;
  font-size: 14px;
  color: #1d1d1f;
}

.config-list {
  background: rgba(0, 0, 0, 0.03);
  border-radius: 6px;
  padding: 8px;
}

.config-item {
  font-size: 12px;
  padding: 2px 0;
  word-break: break-all;
}

.config-command {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
  word-break: break-all;
}

.log-line {
  color: #d4d4d4;
  font-family: monospace;
  font-size: 12px;
  margin-bottom: 2px;
}
</style>
