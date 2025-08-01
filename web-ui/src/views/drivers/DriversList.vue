<template>
  <div class="drivers-list-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-title">
        <el-icon :size="24">
          <Connection />
        </el-icon>
        <h1>驱动管理</h1>
        <el-tag v-if="driversStats" type="info" size="large">
          {{ driversStats.total }} 个驱动
        </el-tag>
      </div>
      
      <div class="header-actions">
        <el-button
          type="primary"
          :icon="Plus"
          @click="handleAddDriver"
        >
          新增驱动
        </el-button>
        
        <el-dropdown trigger="click" @command="handleHeaderAction">
          <el-button :icon="MoreFilled" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="import" :icon="Upload">
                导入配置
              </el-dropdown-item>
              <el-dropdown-item command="export" :icon="Download">
                导出配置
              </el-dropdown-item>
              <el-dropdown-item command="template" :icon="DocumentCopy">
                模板管理
              </el-dropdown-item>
              <el-dropdown-item command="refresh" :icon="Refresh">
                刷新数据
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
    
    <!-- 驱动状态概览 -->
    <div class="drivers-overview">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--total">
            <div class="stat-content">
              <div class="stat-number">{{ driversStats.total }}</div>
              <div class="stat-label">总驱动数</div>
            </div>
            <el-icon class="stat-icon">
              <Monitor />
            </el-icon>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--running">
            <div class="stat-content">
              <div class="stat-number">{{ driversStats.running }}</div>
              <div class="stat-label">运行中</div>
            </div>
            <el-icon class="stat-icon">
              <VideoPlay />
            </el-icon>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--stopped">
            <div class="stat-content">
              <div class="stat-number">{{ driversStats.stopped }}</div>
              <div class="stat-label">已停止</div>
            </div>
            <el-icon class="stat-icon">
              <VideoPause />
            </el-icon>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--error">
            <div class="stat-content">
              <div class="stat-number">{{ driversStats.error }}</div>
              <div class="stat-label">异常</div>
            </div>
            <el-icon class="stat-icon">
              <Warning />
            </el-icon>
          </el-card>
        </el-col>
      </el-row>
    </div>
    
    <!-- 筛选和搜索 -->
    <div class="drivers-filters">
      <el-card shadow="never">
        <div class="filter-content">
          <div class="filter-left">
            <SearchBox
              v-model="searchKeyword"
              placeholder="搜索驱动名称、协议类型或地址"
              :suggestions="searchSuggestions"
              style="width: 300px"
              @search="handleSearch"
            />
            
            <FilterPanel
              v-model="filterConditions"
              :filters="filterOptions"
              :show-quick-filters="true"
              :quick-filters="quickFilters"
              :show-apply-button="false"
              style="margin-left: 16px"
              @filter="handleFilter"
            />
          </div>
          
          <div class="filter-right">
            <ActionButtons
              :actions="batchActions"
              :disabled="selectedDrivers.length === 0"
              @actionClick="handleBatchAction"
            />
          </div>
        </div>
      </el-card>
    </div>
    
    <!-- 驱动列表表格 -->
    <div class="drivers-table">
      <LoadingCard
        v-if="loading && drivers.length === 0"
        status="loading"
        loading-text="正在加载驱动数据..."
        :show-progress="true"
        :progress="loadingProgress"
        min-height="400px"
      />
      
      <BaseTable
        v-else
        v-model:selection="selectedDrivers"
        :data="filteredDrivers"
        :columns="tableColumns"
        :loading="loading"
        :pagination="paginationConfig"
        show-selection
        @action="handleTableAction"
        @refresh="handleRefresh"
      />
    </div>
    
    <!-- 驱动详情抽屉 -->
    <el-drawer
      v-model="detailDrawerVisible"
      :title="`驱动详情 - ${selectedDriver?.name}`"
      size="600px"
      :before-close="handleDetailDrawerClose"
    >
      <div v-if="selectedDriver" class="driver-detail">
        <el-tabs v-model="detailActiveTab">
          <!-- 基本信息 -->
          <el-tab-pane label="基本信息" name="basic">
            <el-descriptions border column="2">
              <el-descriptions-item label="驱动名称">
                {{ selectedDriver.name }}
              </el-descriptions-item>
              <el-descriptions-item label="协议类型">
                <el-tag :type="getProtocolTagType(selectedDriver.protocol)">
                  {{ selectedDriver.protocol }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="运行状态">
                <StatusTag :status="selectedDriver.status" />
              </el-descriptions-item>
              <el-descriptions-item label="连接地址">
                {{ selectedDriver.address }}
              </el-descriptions-item>
              <el-descriptions-item label="创建时间">
                {{ formatTime(selectedDriver.createTime) }}
              </el-descriptions-item>
              <el-descriptions-item label="最后更新">
                {{ formatTime(selectedDriver.updateTime) }}
              </el-descriptions-item>
            </el-descriptions>
          </el-tab-pane>
          
          <!-- 连接状态 -->
          <el-tab-pane label="连接状态" name="connection">
            <ConnectionStatus
              :connection="getConnectionInfo(selectedDriver)"
              @reconnect="handleDriverReconnect"
              @disconnect="handleDriverDisconnect"
            />
          </el-tab-pane>
          
          <!-- 性能指标 -->
          <el-tab-pane label="性能指标" name="metrics">
            <div class="driver-metrics">
              <el-row :gutter="16">
                <el-col :span="12">
                  <el-statistic title="数据点数量" :value="selectedDriver.dataPointCount || 0" />
                </el-col>
                <el-col :span="12">
                  <el-statistic title="消息数/秒" :value="selectedDriver.messageRate || 0" :precision="1" />
                </el-col>
              </el-row>
              
              <el-row :gutter="16" style="margin-top: 16px">
                <el-col :span="12">
                  <el-statistic title="平均延迟" :value="selectedDriver.avgLatency || 0" suffix="ms" />
                </el-col>
                <el-col :span="12">
                  <el-statistic title="错误率" :value="selectedDriver.errorRate || 0" suffix="%" :precision="2" />
                </el-col>
              </el-row>
              
              <!-- 性能图表 -->
              <div style="margin-top: 20px">
                <ChartContainer
                  title="实时性能监控"
                  chart-type="line"
                  :data="getDriverChartData(selectedDriver)"
                  :height="'300px'"
                  support-realtime
                />
              </div>
            </div>
          </el-tab-pane>
          
          <!-- 日志查看 -->
          <el-tab-pane label="日志查看" name="logs">
            <LogViewer
              :title="`${selectedDriver.name} 日志`"
              :logs="getDriverLogs(selectedDriver)"
              support-realtime
              height="400px"
            />
          </el-tab-pane>
        </el-tabs>
      </div>
      
      <template #footer>
        <div class="drawer-footer">
          <el-button @click="detailDrawerVisible = false">关闭</el-button>
          <el-button 
            v-if="selectedDriver?.status === 'stopped'"
            type="success"
            @click="handleDriverStart(selectedDriver)"
          >
            启动驱动
          </el-button>
          <el-button 
            v-if="selectedDriver?.status === 'running'"
            type="warning"
            @click="handleDriverStop(selectedDriver)"
          >
            停止驱动
          </el-button>
          <el-button 
            type="primary"
            @click="handleDriverConfig(selectedDriver)"
          >
            配置驱动
          </el-button>
        </div>
      </template>
    </el-drawer>
    
    <!-- 驱动配置对话框 -->
    <el-dialog
      v-model="configDialogVisible"
      :title="configDialogTitle"
      width="800px"
      :before-close="handleConfigDialogClose"
    >
      <ProtocolConfig
        v-if="editingDriver"
        v-model="driverConfig"
        :protocol-type="editingDriver.protocol"
        :title="`${editingDriver.name} 配置`"
        @save="handleConfigSave"
        @test="handleConfigTest"
        @cancel="handleConfigCancel"
      />
    </el-dialog>
    
    <!-- 模板管理对话框 -->
    <el-dialog
      v-model="templateDialogVisible"
      title="驱动模板管理"
      width="700px"
      :before-close="handleTemplateDialogClose"
    >
      <div class="template-management">
        <div class="template-actions">
          <el-button type="primary" @click="handleCreateTemplate">
            新建模板
          </el-button>
          <el-button @click="handleImportTemplate">
            导入模板
          </el-button>
        </div>
        
        <BaseTable
          :data="driverTemplates"
          :columns="templateColumns"
          @action="handleTemplateAction"
        />
      </div>
    </el-dialog>
    
    <!-- 文件上传对话框 -->
    <el-dialog
      v-model="uploadDialogVisible"
      title="导入驱动配置"
      width="600px"
      :before-close="handleUploadDialogClose"
    >
      <FileUploader
        title="选择配置文件"
        :upload-types="configUploadTypes"
        @success="handleConfigUploadSuccess"
      />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, onUnmounted, h } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Connection,
  Plus,
  MoreFilled,
  Upload,
  Download,
  DocumentCopy,
  Refresh,
  Monitor,
  VideoPlay,
  VideoPause,
  Warning
} from '@element-plus/icons-vue'

// 导入基础组件
import { BaseTable, SearchBox, FilterPanel, ActionButtons, LoadingCard, StatusTag } from '../../components/base'

// 导入业务组件
import { ConnectionStatus, ProtocolConfig, ChartContainer, LogViewer, FileUploader } from '../../components/business'

// 类型定义
export interface Driver {
  id: string
  name: string
  protocol: 'modbus_tcp' | 'modbus_rtu' | 'opcua' | 'mqtt5' | 'ethernet_ip'
  address: string
  port?: number
  status: 'running' | 'stopped' | 'error' | 'connecting'
  createTime: Date
  updateTime: Date
  dataPointCount?: number
  messageRate?: number
  avgLatency?: number
  errorRate?: number
  config?: any
}

export interface DriversStats {
  total: number
  running: number
  stopped: number
  error: number
}

export interface DriverTemplate {
  id: string
  name: string
  protocol: string
  description: string
  config: any
  createTime: Date
}

// 状态管理
const drivers = ref<Driver[]>([
  {
    id: '1',
    name: 'PLC主控制器',
    protocol: 'modbus_tcp',
    address: '192.168.1.100',
    port: 502,
    status: 'running',
    createTime: new Date('2024-01-15'),
    updateTime: new Date(),
    dataPointCount: 156,
    messageRate: 2.5,
    avgLatency: 45,
    errorRate: 0.2
  },
  {
    id: '2', 
    name: '温度传感器组',
    protocol: 'modbus_rtu',
    address: '/dev/ttyUSB0',
    status: 'running',
    createTime: new Date('2024-01-16'),
    updateTime: new Date(Date.now() - 300000),
    dataPointCount: 24,
    messageRate: 1.2,
    avgLatency: 120,
    errorRate: 0.0
  },
  {
    id: '3',
    name: 'SCADA系统',
    protocol: 'opcua',
    address: 'opc.tcp://192.168.1.50:4840',
    status: 'error',
    createTime: new Date('2024-01-10'),
    updateTime: new Date(Date.now() - 1800000),
    dataPointCount: 89,
    messageRate: 0,
    avgLatency: 0,
    errorRate: 100
  },
  {
    id: '4',
    name: 'MQTT网关',
    protocol: 'mqtt5',
    address: 'mqtt://broker.hivemq.com:1883',
    status: 'stopped',
    createTime: new Date('2024-01-12'),
    updateTime: new Date(Date.now() - 7200000),
    dataPointCount: 67,
    messageRate: 0,
    avgLatency: 0,
    errorRate: 0
  }
])

const loading = ref(false)
const loadingProgress = ref(0)
const selectedDrivers = ref<Driver[]>([])
const searchKeyword = ref('')
const filterConditions = ref<any>({})

// 对话框状态
const detailDrawerVisible = ref(false)
const configDialogVisible = ref(false)
const templateDialogVisible = ref(false)
const uploadDialogVisible = ref(false)

// 选中的驱动和配置
const selectedDriver = ref<Driver | null>(null)
const editingDriver = ref<Driver | null>(null)
const driverConfig = ref<any>({})
const detailActiveTab = ref('basic')

// 驱动模板
const driverTemplates = ref<DriverTemplate[]>([
  {
    id: '1',
    name: 'Modbus TCP 标准模板',
    protocol: 'modbus_tcp',
    description: '适用于标准Modbus TCP设备',
    config: {
      basic: { deviceId: 1, scanRate: 1000 },
      connection: { timeout: 5000, retries: 3 }
    },
    createTime: new Date('2024-01-01')
  }
])

// WebSocket连接（模拟实时数据）
const wsConnection: WebSocket | null = null

// 计算属性
const driversStats = computed<DriversStats>(() => {
  const stats = {
    total: drivers.value.length,
    running: 0,
    stopped: 0,
    error: 0
  }
  
  drivers.value.forEach(driver => {
    switch (driver.status) {
      case 'running':
        stats.running++
        break
      case 'stopped':
        stats.stopped++
        break
      case 'error':
        stats.error++
        break
    }
  })
  
  return stats
})

const filteredDrivers = computed(() => {
  let result = drivers.value
  
  // 搜索过滤
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase()
    result = result.filter(driver => 
      driver.name.toLowerCase().includes(keyword) ||
      driver.protocol.toLowerCase().includes(keyword) ||
      driver.address.toLowerCase().includes(keyword)
    )
  }
  
  // 状态过滤
  if (filterConditions.value.status) {
    result = result.filter(driver => driver.status === filterConditions.value.status)
  }
  
  // 协议过滤
  if (filterConditions.value.protocol) {
    result = result.filter(driver => driver.protocol === filterConditions.value.protocol)
  }
  
  return result
})

const searchSuggestions = computed(() => {
  const suggestions = []
  drivers.value.forEach(driver => {
    suggestions.push(driver.name)
    suggestions.push(driver.protocol)
    suggestions.push(driver.address)
  })
  return [...new Set(suggestions)]
})

// 筛选器配置
const filterOptions = [
  {
    key: 'status',
    label: '运行状态',
    type: 'select',
    options: [
      { label: '运行中', value: 'running' },
      { label: '已停止', value: 'stopped' },
      { label: '异常', value: 'error' },
      { label: '连接中', value: 'connecting' }
    ]
  },
  {
    key: 'protocol',
    label: '协议类型',
    type: 'select',
    options: [
      { label: 'Modbus TCP', value: 'modbus_tcp' },
      { label: 'Modbus RTU', value: 'modbus_rtu' },
      { label: 'OPC UA', value: 'opcua' },
      { label: 'MQTT5', value: 'mqtt5' },
      { label: 'Ethernet/IP', value: 'ethernet_ip' }
    ]
  },
  {
    key: 'createTime',
    label: '创建时间',
    type: 'daterange'
  }
]

const quickFilters = [
  { label: '运行中', key: 'status', value: 'running' },
  { label: '异常', key: 'status', value: 'error' },
  { label: 'Modbus', key: 'protocol', value: 'modbus_tcp' },
  { label: 'OPC UA', key: 'protocol', value: 'opcua' }
]

// 批量操作
const batchActions = [
  {
    key: 'start',
    label: '批量启动',
    type: 'success',
    icon: 'VideoPlay',
    confirm: {
      title: '确定要启动选中的驱动吗？',
      confirmText: '启动',
      cancelText: '取消'
    }
  },
  {
    key: 'stop',
    label: '批量停止',
    type: 'warning',
    icon: 'VideoPause',
    confirm: {
      title: '确定要停止选中的驱动吗？',
      confirmText: '停止',
      cancelText: '取消'
    }
  },
  {
    key: 'restart',
    label: '批量重启',
    type: 'primary',
    icon: 'Refresh',
    confirm: {
      title: '确定要重启选中的驱动吗？',
      confirmText: '重启',
      cancelText: '取消'
    }
  },
  {
    key: 'delete',
    label: '批量删除',
    type: 'danger',
    icon: 'Delete',
    confirm: {
      title: '确定要删除选中的驱动吗？此操作不可恢复！',
      confirmText: '删除',
      cancelText: '取消',
      type: 'warning'
    }
  }
]

// 表格列配置
const tableColumns = [
  {
    key: 'name',
    label: '驱动名称',
    width: 200,
    sortable: true
  },
  {
    key: 'protocol',
    label: '协议类型',
    width: 120,
    type: 'tag',
    formatter: (row: Driver) => {
      const protocolMap: Record<string, string> = {
        modbus_tcp: 'Modbus TCP',
        modbus_rtu: 'Modbus RTU',
        opcua: 'OPC UA',
        mqtt5: 'MQTT5',
        ethernet_ip: 'Ethernet/IP'
      }
      return protocolMap[row.protocol] || row.protocol
    }
  },
  {
    key: 'address',
    label: '连接地址',
    width: 180,
    showOverflowTooltip: true
  },
  {
    key: 'status',
    label: '运行状态',
    width: 100,
    type: 'custom',
    render: (row: Driver) => {
      return h(StatusTag, { status: row.status })
    }
  },
  {
    key: 'dataPointCount',
    label: '数据点',
    width: 80,
    align: 'center',
    formatter: (row: Driver) => row.dataPointCount || 0
  },
  {
    key: 'messageRate',
    label: '消息率',
    width: 100,
    align: 'center',
    formatter: (row: Driver) => `${(row.messageRate || 0).toFixed(1)}/s`
  },
  {
    key: 'avgLatency',
    label: '平均延迟',
    width: 100,
    align: 'center',
    formatter: (row: Driver) => `${row.avgLatency || 0}ms`
  },
  {
    key: 'updateTime',
    label: '最后更新',
    width: 160,
    type: 'datetime'
  },
  {
    key: 'actions',
    label: '操作',
    width: 200,
    type: 'action',
    actions: [
      {
        key: 'view',
        label: '详情',
        type: 'primary',
        icon: 'View'
      },
      {
        key: 'config',
        label: '配置',
        type: 'default',
        icon: 'Setting'
      },
      {
        key: 'start',
        label: '启动',
        type: 'success',
        icon: 'VideoPlay',
        show: (row: Driver) => row.status !== 'running'
      },
      {
        key: 'stop',
        label: '停止',
        type: 'warning',
        icon: 'VideoPause',
        show: (row: Driver) => row.status === 'running'
      },
      {
        key: 'delete',
        label: '删除',
        type: 'danger',
        icon: 'Delete',
        confirm: {
          title: '确定要删除这个驱动吗？',
          confirmText: '删除',
          cancelText: '取消'
        }
      }
    ]
  }
]

// 分页配置
const paginationConfig = {
  pageSize: 10,
  showSizeChanger: true,
  showQuickJumper: true,
  showTotal: true
}

// 模板表格列
const templateColumns = [
  { key: 'name', label: '模板名称', width: 200 },
  { key: 'protocol', label: '协议类型', width: 120 },
  { key: 'description', label: '描述', showOverflowTooltip: true },
  { key: 'createTime', label: '创建时间', width: 160, type: 'datetime' },
  {
    key: 'actions',
    label: '操作',
    width: 150,
    type: 'action',
    actions: [
      { key: 'use', label: '使用', type: 'primary', icon: 'Check' },
      { key: 'edit', label: '编辑', type: 'default', icon: 'Edit' },
      { key: 'delete', label: '删除', type: 'danger', icon: 'Delete' }
    ]
  }
]

// 配置上传类型
const configUploadTypes = [
  {
    value: 'json',
    label: 'JSON配置',
    accept: '.json',
    maxSize: 10 * 1024 * 1024,
    description: '支持JSON格式配置文件，最大10MB'
  },
  {
    value: 'xml',
    label: 'XML配置',
    accept: '.xml',
    maxSize: 10 * 1024 * 1024,
    description: '支持XML格式配置文件，最大10MB'
  }
]

// 事件处理方法
const handleSearch = (keyword: string) => {
  searchKeyword.value = keyword
}

const handleFilter = (conditions: any) => {
  filterConditions.value = conditions
}

const handleTableAction = (action: string, row: Driver) => {
  switch (action) {
    case 'view':
      handleViewDriver(row)
      break
    case 'config':
      handleDriverConfig(row)
      break
    case 'start':
      handleDriverStart(row)
      break
    case 'stop':
      handleDriverStop(row)
      break
    case 'delete':
      handleDeleteDriver(row)
      break
  }
}

const handleBatchAction = (action: string) => {
  if (selectedDrivers.value.length === 0) {
    ElMessage.warning('请先选择要操作的驱动')
    return
  }
  
  switch (action) {
    case 'start':
      handleBatchStart()
      break
    case 'stop':
      handleBatchStop()
      break
    case 'restart':
      handleBatchRestart()
      break
    case 'delete':
      handleBatchDelete()
      break
  }
}

const handleViewDriver = (driver: Driver) => {
  selectedDriver.value = driver
  detailDrawerVisible.value = true
  detailActiveTab.value = 'basic'
}

const handleDriverConfig = (driver: Driver) => {
  editingDriver.value = driver
  driverConfig.value = driver.config || {}
  configDialogVisible.value = true
}

const handleDriverStart = async (driver: Driver) => {
  try {
    ElMessage.loading('正在启动驱动...', 0)
    
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // 更新驱动状态
    const index = drivers.value.findIndex(d => d.id === driver.id)
    if (index > -1) {
      drivers.value[index].status = 'running'
      drivers.value[index].updateTime = new Date()
    }
    
    ElMessage.closeAll()
    ElMessage.success(`驱动 ${driver.name} 启动成功`)
  } catch (error) {
    ElMessage.closeAll()
    ElMessage.error(`驱动启动失败: ${error}`)
  }
}

const handleDriverStop = async (driver: Driver) => {
  try {
    ElMessage.loading('正在停止驱动...', 0)
    
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    const index = drivers.value.findIndex(d => d.id === driver.id)
    if (index > -1) {
      drivers.value[index].status = 'stopped'
      drivers.value[index].updateTime = new Date()
      drivers.value[index].messageRate = 0
    }
    
    ElMessage.closeAll()
    ElMessage.success(`驱动 ${driver.name} 停止成功`)
  } catch (error) {
    ElMessage.closeAll()
    ElMessage.error(`驱动停止失败: ${error}`)
  }
}

const handleDriverReconnect = async (connection: any) => {
  try {
    ElMessage.loading('正在重新连接...', 0)
    await new Promise(resolve => setTimeout(resolve, 2000))
    ElMessage.closeAll()
    ElMessage.success('重新连接成功')
  } catch (error) {
    ElMessage.closeAll()
    ElMessage.error('重新连接失败')
  }
}

const handleDriverDisconnect = (connection: any) => {
  ElMessage.success('连接已断开')
}

const handleDeleteDriver = async (driver: Driver) => {
  const index = drivers.value.findIndex(d => d.id === driver.id)
  if (index > -1) {
    drivers.value.splice(index, 1)
    ElMessage.success(`驱动 ${driver.name} 删除成功`)
  }
}

const handleBatchStart = async () => {
  const stoppedDrivers = selectedDrivers.value.filter(d => d.status !== 'running')
  if (stoppedDrivers.length === 0) {
    ElMessage.warning('选中的驱动都已在运行中')
    return
  }
  
  ElMessage.loading(`正在启动 ${stoppedDrivers.length} 个驱动...`, 0)
  
  for (const driver of stoppedDrivers) {
    await handleDriverStart(driver)
  }
  
  ElMessage.closeAll()
  ElMessage.success(`成功启动 ${stoppedDrivers.length} 个驱动`)
  selectedDrivers.value = []
}

const handleBatchStop = async () => {
  const runningDrivers = selectedDrivers.value.filter(d => d.status === 'running')
  if (runningDrivers.length === 0) {
    ElMessage.warning('选中的驱动都已停止')
    return
  }
  
  ElMessage.loading(`正在停止 ${runningDrivers.length} 个驱动...`, 0)
  
  for (const driver of runningDrivers) {
    await handleDriverStop(driver)
  }
  
  ElMessage.closeAll()
  ElMessage.success(`成功停止 ${runningDrivers.length} 个驱动`)
  selectedDrivers.value = []
}

const handleBatchRestart = async () => {
  ElMessage.loading(`正在重启 ${selectedDrivers.value.length} 个驱动...`, 0)
  
  for (const driver of selectedDrivers.value) {
    if (driver.status === 'running') {
      await handleDriverStop(driver)
    }
    await handleDriverStart(driver)
  }
  
  ElMessage.closeAll()
  ElMessage.success(`成功重启 ${selectedDrivers.value.length} 个驱动`)
  selectedDrivers.value = []
}

const handleBatchDelete = () => {
  selectedDrivers.value.forEach(driver => {
    const index = drivers.value.findIndex(d => d.id === driver.id)
    if (index > -1) {
      drivers.value.splice(index, 1)
    }
  })
  
  ElMessage.success(`成功删除 ${selectedDrivers.value.length} 个驱动`)
  selectedDrivers.value = []
}

const handleAddDriver = () => {
  // 跳转到添加驱动页面或打开对话框
  ElMessage.info('跳转到添加驱动页面')
}

const handleHeaderAction = (command: string) => {
  switch (command) {
    case 'import':
      uploadDialogVisible.value = true
      break
    case 'export':
      handleExportConfig()
      break
    case 'template':
      templateDialogVisible.value = true
      break
    case 'refresh':
      handleRefresh()
      break
  }
}

const handleExportConfig = () => {
  const config = {
    drivers: drivers.value.map(d => ({
      name: d.name,
      protocol: d.protocol,
      address: d.address,
      port: d.port,
      config: d.config
    })),
    exportTime: new Date().toISOString()
  }
  
  const blob = new Blob([JSON.stringify(config, null, 2)], {
    type: 'application/json'
  })
  
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `drivers_config_${new Date().toISOString().split('T')[0]}.json`
  a.click()
  
  URL.revokeObjectURL(url)
  ElMessage.success('配置导出成功')
}

const handleRefresh = () => {
  loading.value = true
  loadingProgress.value = 0
  
  const interval = setInterval(() => {
    loadingProgress.value += 10
    if (loadingProgress.value >= 100) {
      clearInterval(interval)
      loading.value = false
      loadingProgress.value = 0
      ElMessage.success('数据刷新成功')
    }
  }, 100)
}

// 对话框处理
const handleDetailDrawerClose = () => {
  detailDrawerVisible.value = false
  selectedDriver.value = null
}

const handleConfigDialogClose = () => {
  configDialogVisible.value = false
  editingDriver.value = null
  driverConfig.value = {}
}

const handleConfigSave = (config: any) => {
  if (editingDriver.value) {
    const index = drivers.value.findIndex(d => d.id === editingDriver.value!.id)
    if (index > -1) {
      drivers.value[index].config = config
      drivers.value[index].updateTime = new Date()
    }
    ElMessage.success('配置保存成功')
    handleConfigDialogClose()
  }
}

const handleConfigTest = async (config: any) => {
  ElMessage.loading('正在测试连接...', 0)
  
  try {
    await new Promise(resolve => setTimeout(resolve, 2000))
    ElMessage.closeAll()
    ElMessage.success('连接测试成功')
  } catch (error) {
    ElMessage.closeAll()
    ElMessage.error('连接测试失败')
  }
}

const handleConfigCancel = () => {
  handleConfigDialogClose()
}

// 模板管理
const handleTemplateDialogClose = () => {
  templateDialogVisible.value = false
}

const handleTemplateAction = (action: string, row: DriverTemplate) => {
  switch (action) {
    case 'use':
      ElMessage.success(`使用模板: ${row.name}`)
      handleTemplateDialogClose()
      break
    case 'edit':
      ElMessage.info(`编辑模板: ${row.name}`)
      break
    case 'delete':
      const index = driverTemplates.value.findIndex(t => t.id === row.id)
      if (index > -1) {
        driverTemplates.value.splice(index, 1)
        ElMessage.success('模板删除成功')
      }
      break
  }
}

const handleCreateTemplate = () => {
  ElMessage.info('创建新模板')
}

const handleImportTemplate = () => {
  ElMessage.info('导入模板')
}

// 文件上传
const handleUploadDialogClose = () => {
  uploadDialogVisible.value = false
}

const handleConfigUploadSuccess = (file: any, response: any) => {
  ElMessage.success(`配置文件 ${file.name} 上传成功`)
  handleUploadDialogClose()
  // 这里可以解析上传的配置文件并更新驱动列表
}

// 辅助方法
const getProtocolTagType = (protocol: string) => {
  const typeMap: Record<string, string> = {
    modbus_tcp: 'primary',
    modbus_rtu: 'success',
    opcua: 'warning',
    mqtt5: 'info',
    ethernet_ip: 'danger'
  }
  return typeMap[protocol] || 'default'
}

const formatTime = (time: Date) => {
  return time.toLocaleString('zh-CN')
}

const getConnectionInfo = (driver: Driver) => {
  return {
    id: driver.id,
    name: driver.name,
    status: driver.status === 'running' ? 'connected' : 'disconnected',
    protocol: driver.protocol,
    host: driver.address.includes('://') ? driver.address.split('://')[1].split(':')[0] : driver.address.split(':')[0],
    port: driver.port || (driver.address.includes(':') ? parseInt(driver.address.split(':').pop() || '0') : 0),
    lastUpdate: driver.updateTime,
    metrics: {
      latency: driver.avgLatency || 0,
      packetLoss: driver.errorRate || 0,
      throughput: (driver.messageRate || 0) * 64 // 假设每条消息64字节
    }
  }
}

const getDriverChartData = (driver: Driver) => {
  // 生成模拟的性能数据
  const now = Date.now()
  const data = []
  
  for (let i = 29; i >= 0; i--) {
    const time = now - i * 60000 // 30分钟的数据
    data.push({
      timestamp: new Date(time),
      messageRate: (driver.messageRate || 0) + Math.random() * 0.5 - 0.25,
      latency: (driver.avgLatency || 0) + Math.random() * 10 - 5,
      errorRate: (driver.errorRate || 0) + Math.random() * 0.1
    })
  }
  
  return {
    series: [
      {
        name: '消息率',
        data: data.map(d => ({ x: d.timestamp, y: d.messageRate }))
      },
      {
        name: '延迟',
        data: data.map(d => ({ x: d.timestamp, y: d.latency })),
        yAxis: 1
      },
      {
        name: '错误率',
        data: data.map(d => ({ x: d.timestamp, y: d.errorRate })),
        yAxis: 2
      }
    ]
  }
}

const getDriverLogs = (driver: Driver) => {
  // 返回模拟的日志数据
  return [
    {
      id: '1',
      timestamp: new Date(),
      level: 'info',
      source: driver.name,
      message: `驱动 ${driver.name} 运行正常，消息率: ${driver.messageRate}/s`
    },
    {
      id: '2',
      timestamp: new Date(Date.now() - 60000),
      level: 'debug',
      source: driver.name,
      message: `建立连接到 ${driver.address}`
    },
    {
      id: '3',
      timestamp: new Date(Date.now() - 120000),
      level: 'info',
      source: driver.name,
      message: `驱动启动完成，加载 ${driver.dataPointCount} 个数据点`
    }
  ]
}

const configDialogTitle = computed(() => {
  return editingDriver.value ? `配置驱动 - ${editingDriver.value.name}` : '驱动配置'
})

// 实时数据更新
const startRealtimeUpdate = () => {
  // 模拟WebSocket连接
  const updateInterval = setInterval(() => {
    drivers.value.forEach(driver => {
      if (driver.status === 'running') {
        // 更新实时指标
        driver.messageRate = Math.max(0, (driver.messageRate || 0) + (Math.random() - 0.5) * 0.2)
        driver.avgLatency = Math.max(1, (driver.avgLatency || 50) + (Math.random() - 0.5) * 10)
        driver.errorRate = Math.max(0, Math.min(100, (driver.errorRate || 0) + (Math.random() - 0.5) * 0.1))
        driver.updateTime = new Date()
      }
    })
  }, 5000)
  
  return updateInterval
}

// 生命周期钩子
onMounted(() => {
  // 启动实时更新
  const updateInterval = startRealtimeUpdate()
  
  // 组件卸载时清理
  onUnmounted(() => {
    if (updateInterval) {
      clearInterval(updateInterval)
    }
  })
})
</script>

<style scoped lang="scss">
.drivers-list-page {
  padding: 16px;
  background: var(--el-bg-color-page);
  min-height: 100vh;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  
  .header-title {
    display: flex;
    align-items: center;
    gap: 12px;
    
    h1 {
      margin: 0;
      font-size: 24px;
      font-weight: 600;
      color: var(--el-text-color-primary);
    }
  }
  
  .header-actions {
    display: flex;
    gap: 12px;
  }
}

.drivers-overview {
  margin-bottom: 20px;
  
  .stat-card {
    position: relative;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.3s;
    
    &:hover {
      transform: translateY(-2px);
      box-shadow: var(--el-box-shadow);
    }
    
    .stat-content {
      position: relative;
      z-index: 2;
      
      .stat-number {
        font-size: 32px;
        font-weight: bold;
        margin-bottom: 4px;
      }
      
      .stat-label {
        font-size: 14px;
        color: var(--el-text-color-secondary);
      }
    }
    
    .stat-icon {
      position: absolute;
      right: 16px;
      top: 50%;
      transform: translateY(-50%);
      font-size: 48px;
      opacity: 0.1;
    }
    
    &.stat-card--total {
      .stat-number {
        color: var(--el-color-primary);
      }
      
      .stat-icon {
        color: var(--el-color-primary);
      }
    }
    
    &.stat-card--running {
      .stat-number {
        color: var(--el-color-success);
      }
      
      .stat-icon {
        color: var(--el-color-success);
      }
    }
    
    &.stat-card--stopped {
      .stat-number {
        color: var(--el-color-info);
      }
      
      .stat-icon {
        color: var(--el-color-info);
      }
    }
    
    &.stat-card--error {
      .stat-number {
        color: var(--el-color-danger);
      }
      
      .stat-icon {
        color: var(--el-color-danger);
      }
    }
  }
}

.drivers-filters {
  margin-bottom: 20px;
  
  .filter-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    
    .filter-left {
      display: flex;
      align-items: center;
    }
    
    .filter-right {
      display: flex;
      gap: 8px;
    }
  }
}

.drivers-table {
  background: var(--el-bg-color);
  border-radius: 8px;
  overflow: hidden;
}

.driver-detail {
  .driver-metrics {
    .el-row {
      margin-bottom: 16px;
      
      &:last-child {
        margin-bottom: 0;
      }
    }
  }
}

.drawer-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.template-management {
  .template-actions {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .drivers-list-page {
    padding: 8px;
  }
  
  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }
  
  .drivers-overview {
    .el-col {
      margin-bottom: 12px;
    }
  }
  
  .filter-content {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
    
    .filter-left,
    .filter-right {
      width: 100%;
    }
  }
}
</style>