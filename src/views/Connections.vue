<template>
  <div class="connections-view">
    <n-space vertical size="large">
      <div class="header">
        <n-h2 style="margin: 0">Docker 连接管理</n-h2>
        <n-button type="primary" @click="showAddModal = true">
          <template #icon>
            <n-icon><AddIcon /></n-icon>
          </template>
          添加连接
        </n-button>
      </div>

      <n-data-table
        :columns="columns"
        :data="connectionStore.connections"
        :loading="connectionStore.loading"
      />
    </n-space>

    <n-modal
      v-model:show="showAddModal"
      preset="card"
      title="添加 Docker 连接"
      style="width: 500px"
    >
      <n-form :model="formValue" :rules="rules" ref="formRef">
        <n-form-item label="名称" path="name">
          <n-input v-model:value="formValue.name" placeholder="例如: 本地 Docker" />
        </n-form-item>
        <n-form-item label="驱动" path="driver">
          <n-select
            v-model:value="formValue.driver"
            :options="driverOptions"
          />
        </n-form-item>
        <n-form-item label="主机/发行版" path="host">
          <n-input
            v-model:value="formValue.host"
            :placeholder="hostPlaceholder"
          />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showAddModal = false">取消</n-button>
          <n-button type="primary" @click="handleAdd" :loading="submitting">确定</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, h } from 'vue'
import {
  NSpace, NH2, NButton, NIcon, NDataTable, NModal, NForm, NFormItem, NInput, NSelect, NTag, useMessage
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { Add as AddIcon, TrashOutline as TrashIcon, Link as LinkIcon } from '@vicons/ionicons5'
import { useConnectionStore, type Connection } from '../store/connection'

const connectionStore = useConnectionStore()
const message = useMessage()
const showAddModal = ref(false)
const submitting = ref(false)
const formRef = ref<any>(null)

const formValue = ref({
  name: '',
  driver: 'NamedPipe',
  host: '//./pipe/docker_engine'
})

const driverOptions = [
  { label: 'Named Pipe (Windows)', value: 'NamedPipe' },
  { label: 'WSL Bridge', value: 'WslBridge' },
  { label: 'TCP', value: 'Tcp' }
]

const hostPlaceholder = computed(() => {
  switch (formValue.value.driver) {
    case 'NamedPipe': return '//./pipe/docker_engine'
    case 'WslBridge': return 'Ubuntu (发行版名称)'
    case 'Tcp': return 'tcp://localhost:2375'
    default: return ''
  }
})

const rules = {
  name: { required: true, message: '请输入名称', trigger: 'blur' },
  driver: { required: true, message: '请选择驱动', trigger: 'change' },
  host: { required: true, message: '请输入主机地址或发行版', trigger: 'blur' }
}

const columns: DataTableColumns<Connection> = [
  { title: '名称', key: 'name' },
  {
    title: '驱动',
    key: 'driver',
    render(row) {
      return h(NTag, { type: 'info', bordered: false }, { default: () => row.driver })
    }
  },
  { title: '地址', key: 'host' },
  {
    title: '状态',
    key: 'status',
    render(row) {
      const isActive = row.id === connectionStore.activeConnectionId
      return isActive
        ? h(NTag, { type: 'success' }, { default: () => '已连接' })
        : h('span', { style: 'color: #999' }, '未激活')
    }
  },
  {
    title: '操作',
    key: 'actions',
    render(row) {
      return h(NSpace, null, {
        default: () => [
          h(
            NButton,
            {
              size: 'small',
              type: 'primary',
              ghost: true,
              disabled: row.id === connectionStore.activeConnectionId,
              onClick: () => handleSwitch(row.id)
            },
            {
              default: () => '连接',
              icon: () => h(NIcon, null, { default: () => h(LinkIcon) })
            }
          ),
          h(
            NButton,
            {
              size: 'small',
              type: 'error',
              ghost: true,
              onClick: () => handleDelete(row.id)
            },
            {
              default: () => '删除',
              icon: () => h(NIcon, null, { default: () => h(TrashIcon) })
            }
          )
        ]
      })
    }
  }
]

onMounted(() => {
  connectionStore.fetchConnections()
})

const handleAdd = async () => {
  formRef.value?.validate(async (errors: any) => {
    if (!errors) {
      submitting.value = true
      try {
        await connectionStore.addConnection(
          formValue.value.name,
          formValue.value.driver,
          formValue.value.host
        )
        showAddModal.value = false
        message.success('添加成功')
        // 重置表单
        formValue.value = {
          name: '',
          driver: 'NamedPipe',
          host: '//./pipe/docker_engine'
        }
      } catch (e: any) {
        message.error('添加失败: ' + (e as string))
      } finally {
        submitting.value = false
      }
    }
  })
}

const handleDelete = async (id: string) => {
  try {
    await connectionStore.deleteConnection(id)
    message.success('删除成功')
  } catch (e: any) {
    message.error('删除失败: ' + (e as string))
  }
}

const handleSwitch = async (id: string) => {
  try {
    await connectionStore.switchConnection(id)
    message.success('已切换连接')
  } catch (e: any) {
    message.error('切换失败: ' + (e as string))
  }
}
</script>

<style scoped>
.connections-view {
  padding: 0;
}
.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}
</style>
