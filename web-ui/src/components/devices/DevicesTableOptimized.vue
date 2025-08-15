<template>
  <div class="devices-table-optimized">
    <!-- 工具栏 -->
    <div class="table-toolbar">
      <div class="toolbar-left">
        <el-input
          v-model="searchQuery"
          placeholder="搜索设备..."
          :prefix-icon="Search"
          class="search-input"
          clearable
          @input="handleSearchDebounced"
        />
        
        <el-select
          v-model="protocolFilter"
          placeholder="协议类型"
          clearable
          class="protocol-filter"
          @change="handleProtocolFilter"
        >
          <el-option label="Modbus TCP" value="ModbusTcp" />
          <el-option label="OPC UA" value="OpcUa" />
          <el-option label="MQTT" value="Mqtt" />
        </el-select>
        
        <el-select
          v-model="statusFilter"
          placeholder="设备状态"
          clearable
          class="status-filter"
          @change="handleStatusFilter"
        >
          <el-option label="启用" :value="true" />
          <el-option label="禁用" :value="false" />
        </el-select>
      </div>
      
      <div class="toolbar-right">
        <el-badge :value="selectedDevices.length" :hidden="selectedDevices.length === 0">
          <el-dropdown @command="handleBatchCommand">
            <el-button :disabled="selectedDevices.length === 0">
              批量操作 <el-icon class="el-icon--right"><arrow-down /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="enable">批量启用</el-dropdown-item>
                <el-dropdown-item command="disable">批量禁用</el-dropdown-item>
                <el-dropdown-item command="delete" divided>批量删除</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </el-badge>
        
        <el-button @click="handleRefresh" :loading="isLoading" :icon="Refresh">
          刷新
        </el-button>
        
        <el-button type="primary" @click="handleCreate" :icon="Plus">
          新建设备
        </el-button>
      </div>
    </div>

    <!-- 虚拟表格 -->
    <div class="table-container" ref="tableContainer">
      <el-auto-resizer>
        <template #default="{ height, width }">
          <el-table-v2
            ref="tableRef"
            :columns="columns"
            :data="visibleDevices"
            :width="width"
            :height="height"
            :header-height="50"
            :row-height="60"
            fixed
            :loading="isLoading"
            @row-click="handleRowClick"
            @selection-change="handleSelectionChange"
          >
            <!-- 自定义空状态 -->
            <template #empty>
              <div class="empty-state">
                <el-empty 
                  :description="hasFilters ? '没有找到匹配的设备' : '暂无设备数据'"
                  :image-size="120"
                >
                  <el-button type="primary" @click="hasFilters ? clearFilters() : handleCreate()">
                    {{ hasFilters ? '清空筛选' : '添加设备' }}
                  </el-button>
                </el-empty>
              </div>
            </template>
          </el-table-v2>
        </template>
      </el-auto-resizer>
    </div>

    <!-- 分页器 -->
    <div class="table-pagination" v-if="pagination.total > 0">
      <div class="pagination-info">
        显示第 {{ (pagination.page - 1) * pagination.size + 1 }} - 
        {{ Math.min(pagination.page * pagination.size, pagination.total) }} 项，
        共 {{ pagination.total }} 项
      </div>
      
      <el-pagination
        v-model:current-page="pagination.page"
        v-model:page-size="pagination.size"
        :page-sizes="[10, 20, 50, 100]"
        :total="pagination.total"
        layout="sizes, prev, pager, next, jumper"
        @size-change="handlePageSizeChange"
        @current-change="handlePageChange"
      />
    </div>

    <!-- 设备详情侧边栏 -->
    <device-details-drawer
      v-model="showDetailsDrawer"
      :device="selectedDevice"
      @updated="handleDeviceUpdated"
    />

    <!-- 设备编辑对话框 -->
    <device-form-dialog
      v-model="showFormDialog"
      :device="editingDevice"
      @saved="handleDeviceSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, reactive, onMounted, nextTick, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search, Refresh, Plus, ArrowDown, Edit, Delete, View } from '@element-plus/icons-vue'
import type { TableV2Instance } from 'element-plus'
import { debounce } from 'lodash-es'

// 组件和组合式函数导入
import { useDevicesStore } from '@/stores/devices-new'
import { useWebSocketEnhanced } from '@/composables/useWebSocketEnhanced'
import DeviceDetailsDrawer from './DeviceDetailsDrawer.vue'
import DeviceFormDialog from './DeviceFormDialog.vue'

// 类型导入
import type { DeviceVO, ProtocolKind } from '@/types/api'

// ========== Props & Emits ==========

interface Props {
  height?: number
  showActions?: boolean
  selectable?: boolean
  autoRefresh?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  height: 600,
  showActions: true,
  selectable: true,
  autoRefresh: true,
})

const emit = defineEmits<{
  deviceSelected: [device: DeviceVO]
  devicesChanged: []
}>()

// ========== 状态管理 ==========

const devicesStore = useDevicesStore()
const wsClient = useWebSocketEnhanced({
  enableLogging: import.meta.env.DEV,
})

// ========== 响应式数据 ==========

const tableRef = ref<TableV2Instance>()
const tableContainer = ref<HTMLElement>()

// 搜索和筛选
const searchQuery = ref('')
const protocolFilter = ref<ProtocolKind | ''>('')
const statusFilter = ref<boolean | ''>('')

// UI状态
const showDetailsDrawer = ref(false)
const showFormDialog = ref(false)
const selectedDevice = ref<DeviceVO | null>(null)
const editingDevice = ref<DeviceVO | null>(null)
const selectedDevices = ref<DeviceVO[]>([])

// 性能优化相关
const visibleRange = reactive({
  start: 0,
  end: 50, // 初始渲染50行
})

// ========== 计算属性 ==========

const { 
  items: devices, 
  pagination, 
  isLoading, 
  hasError,
  deviceStats 
} = devicesStore

const visibleDevices = computed(() => {
  return devices.value.slice(visibleRange.start, visibleRange.end)
})

const hasFilters = computed(() => {
  return !!(searchQuery.value || protocolFilter.value || statusFilter.value)
})

const connectionStatus = computed(() => {
  return wsClient.isConnected.value ? '已连接' : '未连接'
})

// ========== 表格列定义 ==========

const columns = computed(() => {
  const baseColumns = [
    {
      key: 'selection',
      title: '',
      width: 50,
      cellRenderer: ({ rowData }: { rowData: DeviceVO }) => (
        <el-checkbox
          model-value={selectedDevices.value.some(d => d.id === rowData.id)}
          onChange={(checked: boolean) => toggleDeviceSelection(rowData, checked)}
        />
      ),
    },
    {
      key: 'status',
      title: '状态',
      width: 80,
      cellRenderer: ({ rowData }: { rowData: DeviceVO }) => (
        <div class="status-cell">
          <el-tag
            size="small"
            type={rowData.enabled ? 'success' : 'info'}
            effect={rowData.enabled ? 'dark' : 'plain'}
          >
            {rowData.enabled ? '启用' : '禁用'}
          </el-tag>
        </div>
      ),
    },
    {
      key: 'name',
      title: '设备名称',
      width: 200,
      cellRenderer: ({ rowData }: { rowData: DeviceVO }) => (
        <div class="name-cell">
          <div class="device-name">{rowData.name}</div>
          <div class="device-id">{rowData.id}</div>
        </div>
      ),
    },
    {
      key: 'protocol',
      title: '协议',
      width: 120,
      cellRenderer: ({ rowData }: { rowData: DeviceVO }) => (
        <el-tag size="small" type="primary">
          {getProtocolDisplayName(rowData.protocol)}
        </el-tag>
      ),
    },
    {
      key: 'endpoint',
      title: '连接端点',
      width: 180,
      cellRenderer: ({ rowData }: { rowData: DeviceVO }) => (
        <div class="endpoint-cell">
          <code>{rowData.endpoint || '未配置'}</code>
        </div>
      ),
    },
    {
      key: 'location',
      title: '位置',
      width: 150,
      cellRenderer: ({ rowData }: { rowData: DeviceVO }) => (
        <span>{rowData.location || '-'}</span>
      ),
    },
    {
      key: 'connection',
      title: '连接状态',
      width: 120,
      cellRenderer: ({ rowData }: { rowData: DeviceVO }) => (
        <div class="connection-status">
          <el-tooltip content="实时连接状态" placement="top">
            <div class={`connection-indicator ${getConnectionStatus(rowData)}`}>
              <span class="indicator-dot"></span>
              <span class="indicator-text">
                {getConnectionStatusText(rowData)}
              </span>
            </div>
          </el-tooltip>
        </div>
      ),
    },
    {
      key: 'updated_at',
      title: '更新时间',
      width: 160,
      cellRenderer: ({ rowData }: { rowData: DeviceVO }) => (
        <div class="time-cell">
          <el-tooltip content={rowData.updated_at} placement="top">
            <span>{formatRelativeTime(rowData.updated_at)}</span>
          </el-tooltip>
        </div>
      ),
    },
  ]

  if (props.showActions) {
    baseColumns.push({
      key: 'actions',
      title: '操作',
      width: 180,
      fixed: 'right',
      cellRenderer: ({ rowData }: { rowData: DeviceVO }) => (
        <div class="action-buttons">
          <el-button-group>
            <el-tooltip content="查看详情" placement="top">
              <el-button
                size="small"
                icon={View}
                onClick={() => handleViewDevice(rowData)}
              />
            </el-tooltip>
            
            <el-tooltip content="编辑设备" placement="top">
              <el-button
                size="small"
                icon={Edit}
                onClick={() => handleEditDevice(rowData)}
              />
            </el-tooltip>
            
            <el-tooltip content="删除设备" placement="top">
              <el-button
                size="small"
                type="danger"
                icon={Delete}
                onClick={() => handleDeleteDevice(rowData)}
              />
            </el-tooltip>
          </el-button-group>
          
          <el-switch
            model-value={rowData.enabled}
            size="small"
            inline-prompt
            active-text="启"
            inactive-text="禁"
            onChange={(enabled: boolean) => handleToggleDevice(rowData, enabled)}
          />
        </div>
      ),
    } as any)
  }

  return baseColumns
})

// ========== 方法定义 ==========

// 防抖搜索
const handleSearchDebounced = debounce(async (query: string) => {
  await devicesStore.search(query)
}, 300)

// 筛选处理
const handleProtocolFilter = async (protocol: ProtocolKind | '') => {
  await devicesStore.filterByProtocol(protocol || undefined)
}

const handleStatusFilter = async (enabled: boolean | '') => {
  await devicesStore.filterByStatus(enabled !== '' ? enabled : undefined)
}

// 清空筛选
const clearFilters = async () => {
  searchQuery.value = ''
  protocolFilter.value = ''
  statusFilter.value = ''
  await devicesStore.resetFilters()
}

// 刷新数据
const handleRefresh = async () => {
  await devicesStore.refresh()
}

// 分页处理
const handlePageChange = async (page: number) => {
  await devicesStore.changePage(page)
}

const handlePageSizeChange = async (size: number) => {
  await devicesStore.changePageSize(size)
}

// 设备操作
const handleCreate = () => {
  editingDevice.value = null
  showFormDialog.value = true
}

const handleViewDevice = (device: DeviceVO) => {
  selectedDevice.value = device
  showDetailsDrawer.value = true
  emit('deviceSelected', device)
}

const handleEditDevice = (device: DeviceVO) => {
  editingDevice.value = device
  showFormDialog.value = true
}

const handleDeleteDevice = async (device: DeviceVO) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除设备 "${device.name}" 吗？此操作不可撤销。`,
      '确认删除',
      {
        type: 'warning',
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        confirmButtonClass: 'el-button--danger',
      }
    )

    await devicesStore.deleteDevice(device.id)
    emit('devicesChanged')
  } catch (error) {
    // 用户取消删除
  }
}

const handleToggleDevice = async (device: DeviceVO, enabled: boolean) => {
  await devicesStore.toggleDeviceStatus(device.id, enabled)
}

// 选择处理
const toggleDeviceSelection = (device: DeviceVO, checked: boolean) => {
  if (checked) {
    if (!selectedDevices.value.some(d => d.id === device.id)) {
      selectedDevices.value.push(device)
    }
  } else {
    const index = selectedDevices.value.findIndex(d => d.id === device.id)
    if (index !== -1) {
      selectedDevices.value.splice(index, 1)
    }
  }
}

const handleSelectionChange = (selection: DeviceVO[]) => {
  selectedDevices.value = selection
}

const handleRowClick = (rowData: DeviceVO) => {
  handleViewDevice(rowData)
}

// 批量操作
const handleBatchCommand = async (command: string) => {
  const deviceIds = selectedDevices.value.map(d => d.id)
  
  switch (command) {
    case 'enable':
      await devicesStore.batchUpdateStatus(deviceIds, true)
      break
      
    case 'disable':
      await devicesStore.batchUpdateStatus(deviceIds, false)
      break
      
    case 'delete':
      try {
        await ElMessageBox.confirm(
          `确定要删除选中的 ${selectedDevices.value.length} 个设备吗？`,
          '批量删除确认',
          {
            type: 'warning',
            confirmButtonText: '删除',
            cancelButtonText: '取消',
            confirmButtonClass: 'el-button--danger',
          }
        )
        
        await devicesStore.batchDeleteDevices(deviceIds)
        selectedDevices.value = []
        emit('devicesChanged')
      } catch (error) {
        // 用户取消
      }
      break
  }
}

// 对话框事件处理
const handleDeviceUpdated = () => {
  devicesStore.refresh()
  emit('devicesChanged')
}

const handleDeviceSaved = () => {
  devicesStore.refresh()
  emit('devicesChanged')
  showFormDialog.value = false
}

// 辅助函数
const getProtocolDisplayName = (protocol: ProtocolKind): string => {
  const names = {
    ModbusTcp: 'Modbus TCP',
    OpcUa: 'OPC UA',
    Mqtt: 'MQTT',
  }
  return names[protocol] || protocol
}

const getConnectionStatus = (device: DeviceVO): string => {
  // 这里可以从WebSocket实时数据中获取连接状态
  return device.enabled ? 'connected' : 'disconnected'
}

const getConnectionStatusText = (device: DeviceVO): string => {
  return device.enabled ? '在线' : '离线'
}

const formatRelativeTime = (timestamp: string): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  
  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`
  return `${Math.floor(diff / 86400000)} 天前`
}

// ========== 生命周期 ==========

onMounted(async () => {
  // 初始化数据
  await devicesStore.fetchDevices()
  
  // 建立WebSocket连接
  await wsClient.connect()
  
  // 订阅设备数据（如果开启自动刷新）
  if (props.autoRefresh && devices.value.length > 0) {
    wsClient.subscribeToDevices(devices.value)
  }
})

// 监听设备变化，更新WebSocket订阅
watch(
  () => devices.value,
  (newDevices) => {
    if (props.autoRefresh && wsClient.isConnected.value && newDevices.length > 0) {
      wsClient.subscribeToDevices(newDevices)
    }
  },
  { deep: true }
)

// ========== 样式定义 ==========
</script>

<style scoped lang="scss">
.devices-table-optimized {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--el-bg-color-page);
  
  .table-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: var(--el-bg-color);
    border-radius: 8px 8px 0 0;
    border-bottom: 1px solid var(--el-border-color-light);
    
    .toolbar-left {
      display: flex;
      gap: 12px;
      align-items: center;
      
      .search-input {
        width: 240px;
      }
      
      .protocol-filter,
      .status-filter {
        width: 120px;
      }
    }
    
    .toolbar-right {
      display: flex;
      gap: 8px;
      align-items: center;
    }
  }
  
  .table-container {
    flex: 1;
    background: var(--el-bg-color);
    
    // 表格行样式
    :deep(.el-table-v2__row) {
      transition: background-color 0.2s;
      
      &:hover {
        background-color: var(--el-fill-color-lighter);
      }
    }
    
    // 状态单元格
    .status-cell {
      display: flex;
      align-items: center;
    }
    
    // 名称单元格
    .name-cell {
      .device-name {
        font-weight: 500;
        color: var(--el-text-color-primary);
        margin-bottom: 2px;
      }
      
      .device-id {
        font-size: 12px;
        color: var(--el-text-color-regular);
        font-family: 'Consolas', 'Monaco', monospace;
      }
    }
    
    // 端点单元格
    .endpoint-cell {
      code {
        font-size: 12px;
        background: var(--el-fill-color-lighter);
        padding: 2px 6px;
        border-radius: 4px;
      }
    }
    
    // 连接状态
    .connection-status {
      .connection-indicator {
        display: flex;
        align-items: center;
        gap: 6px;
        
        .indicator-dot {
          width: 8px;
          height: 8px;
          border-radius: 50%;
          display: inline-block;
        }
        
        .indicator-text {
          font-size: 12px;
        }
        
        &.connected {
          .indicator-dot {
            background-color: var(--el-color-success);
            box-shadow: 0 0 4px var(--el-color-success);
          }
          
          .indicator-text {
            color: var(--el-color-success);
          }
        }
        
        &.disconnected {
          .indicator-dot {
            background-color: var(--el-color-info);
          }
          
          .indicator-text {
            color: var(--el-color-info);
          }
        }
      }
    }
    
    // 时间单元格
    .time-cell {
      font-size: 13px;
      color: var(--el-text-color-regular);
    }
    
    // 操作按钮
    .action-buttons {
      display: flex;
      align-items: center;
      gap: 8px;
    }
    
    // 空状态
    .empty-state {
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
    }
  }
  
  .table-pagination {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: var(--el-bg-color);
    border-radius: 0 0 8px 8px;
    border-top: 1px solid var(--el-border-color-light);
    
    .pagination-info {
      font-size: 14px;
      color: var(--el-text-color-regular);
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .devices-table-optimized {
    .table-toolbar {
      flex-direction: column;
      gap: 12px;
      
      .toolbar-left,
      .toolbar-right {
        width: 100%;
        justify-content: space-between;
      }
    }
    
    .table-pagination {
      flex-direction: column;
      gap: 12px;
      
      .pagination-info {
        text-align: center;
      }
    }
  }
}
</style>