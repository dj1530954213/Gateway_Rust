<template>
  <div class="connectors-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-title">
        <el-icon :size="24">
          <Connection />
        </el-icon>
        <h1>连接器管理</h1>
        <el-tag :type="getOverallStatusType()" size="large">
          {{ getOverallStatusText() }}
        </el-tag>
      </div>

      <div class="header-actions">
        <el-button-group size="small">
          <el-button
            :type="viewMode === 'list' ? 'primary' : 'default'"
            :icon="List"
            @click="viewMode = 'list'"
          >
            列表
          </el-button>
          <el-button
            :type="viewMode === 'grid' ? 'primary' : 'default'"
            :icon="Grid"
            @click="viewMode = 'grid'"
          >
            卡片
          </el-button>
        </el-button-group>

        <el-button :icon="Refresh" @click="handleRefresh"> 刷新 </el-button>

        <el-button type="primary" :icon="Plus" @click="handleCreate">
          添加连接器
        </el-button>
      </div>
    </div>

    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <SearchBox
          v-model="searchText"
          placeholder="搜索连接器名称、类型..."
          style="width: 300px"
          @search="handleSearch"
        />

        <FilterPanel
          v-model="filterParams"
          :filters="filterConfig"
          @filter="handleFilter"
        />
      </div>

      <div class="toolbar-right">
        <el-dropdown
          :disabled="!selectedConnectors.length"
          @command="handleBatchAction"
        >
          <el-button :disabled="!selectedConnectors.length">
            批量操作
            <el-icon class="el-icon--right">
              <ArrowDown />
            </el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="start" :icon="VideoPlay">
                启动 ({{ selectedConnectors.length }})
              </el-dropdown-item>
              <el-dropdown-item command="stop" :icon="VideoPause">
                停止 ({{ selectedConnectors.length }})
              </el-dropdown-item>
              <el-dropdown-item command="restart" :icon="Refresh">
                重启 ({{ selectedConnectors.length }})
              </el-dropdown-item>
              <el-dropdown-item command="delete" :icon="Delete" divided>
                删除 ({{ selectedConnectors.length }})
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <el-dropdown @command="handleAction">
          <el-button :icon="More" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="export" :icon="Download">
                导出配置
              </el-dropdown-item>
              <el-dropdown-item command="import" :icon="Upload">
                导入配置
              </el-dropdown-item>
              <el-dropdown-item command="template" :icon="Document">
                配置模板
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- 列表视图 -->
    <div v-if="viewMode === 'list'" class="list-view">
      <BaseTable
        :data="filteredConnectors"
        :columns="tableColumns"
        :loading="loading"
        selection
        @selection-change="handleSelectionChange"
        @sort-change="handleSortChange"
      >
        <template #status="{ row }">
          <div class="status-cell">
            <ConnectionStatus
              :status="row.status"
              :name="row.name"
              :type="row.type"
              :show-metrics="true"
              size="small"
            />
          </div>
        </template>

        <template #config="{ row }">
          <div class="config-cell">
            <el-tag size="small" :type="getConfigStatusType(row.configValid)">
              {{ row.configValid ? '配置正常' : '配置异常' }}
            </el-tag>
            <span class="config-count"
              >{{ Object.keys(row.config).length }} 项配置</span
            >
          </div>
        </template>

        <template #metrics="{ row }">
          <div class="metrics-cell">
            <div class="metric-item">
              <span class="metric-label">消息:</span>
              <span class="metric-value">{{
                formatNumber(row.messagesSent)
              }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">错误:</span>
              <span class="metric-value error">{{ row.errorCount }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">延迟:</span>
              <span class="metric-value">{{ row.avgLatency }}ms</span>
            </div>
          </div>
        </template>

        <template #actions="{ row }">
          <ActionButtons
            :buttons="getActionButtons(row)"
            :data="row"
            @action="handleRowAction"
          />
        </template>
      </BaseTable>
    </div>

    <!-- 卡片视图 -->
    <div v-else class="grid-view">
      <el-row :gutter="16">
        <el-col
          v-for="connector in filteredConnectors"
          :key="connector.id"
          :span="8"
        >
          <el-card
            shadow="hover"
            class="connector-card"
            :class="getCardClass(connector)"
          >
            <template #header>
              <div class="card-header">
                <div class="connector-info">
                  <el-icon
                    class="connector-icon"
                    :class="getTypeIconClass(connector.type)"
                  >
                    <component :is="getTypeIcon(connector.type)" />
                  </el-icon>
                  <div class="connector-basic">
                    <h4 class="connector-name">{{ connector.name }}</h4>
                    <span class="connector-type">{{
                      getTypeLabel(connector.type)
                    }}</span>
                  </div>
                </div>

                <div class="card-actions">
                  <ConnectionStatus
                    :status="connector.status"
                    :name="connector.name"
                    :type="connector.type"
                    size="small"
                  />

                  <el-dropdown
                    trigger="click"
                    @command="cmd => handleRowAction(cmd, connector)"
                  >
                    <el-button :icon="More" size="small" text />
                    <template #dropdown>
                      <el-dropdown-menu>
                        <el-dropdown-item command="view" :icon="View">
                          查看详情
                        </el-dropdown-item>
                        <el-dropdown-item command="edit" :icon="Edit">
                          编辑配置
                        </el-dropdown-item>
                        <el-dropdown-item command="test" :icon="Link">
                          连接测试
                        </el-dropdown-item>
                        <el-dropdown-item
                          :command="
                            connector.status === 'connected' ? 'stop' : 'start'
                          "
                          :icon="
                            connector.status === 'connected'
                              ? VideoPause
                              : VideoPlay
                          "
                          divided
                        >
                          {{
                            connector.status === 'connected' ? '停止' : '启动'
                          }}
                        </el-dropdown-item>
                        <el-dropdown-item command="delete" :icon="Delete">
                          删除
                        </el-dropdown-item>
                      </el-dropdown-menu>
                    </template>
                  </el-dropdown>
                </div>
              </div>
            </template>

            <div class="card-content">
              <!-- 连接信息 -->
              <div class="connection-info">
                <div class="info-item">
                  <span class="info-label">目标地址:</span>
                  <span class="info-value">{{
                    getConnectionTarget(connector)
                  }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">协议:</span>
                  <span class="info-value">{{
                    connector.protocol || 'N/A'
                  }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">创建时间:</span>
                  <span class="info-value">{{
                    formatTime(connector.createdAt)
                  }}</span>
                </div>
              </div>

              <!-- 性能指标 -->
              <div class="performance-metrics">
                <div class="metrics-grid">
                  <div class="metric-box">
                    <div class="metric-number">
                      {{ formatNumber(connector.messagesSent) }}
                    </div>
                    <div class="metric-title">消息发送</div>
                  </div>
                  <div class="metric-box">
                    <div class="metric-number error">
                      {{ connector.errorCount }}
                    </div>
                    <div class="metric-title">错误计数</div>
                  </div>
                  <div class="metric-box">
                    <div class="metric-number">
                      {{ connector.avgLatency }}ms
                    </div>
                    <div class="metric-title">平均延迟</div>
                  </div>
                  <div class="metric-box">
                    <div class="metric-number">
                      {{ connector.uptime || 0 }}h
                    </div>
                    <div class="metric-title">运行时间</div>
                  </div>
                </div>
              </div>

              <!-- 状态信息 -->
              <div class="status-info">
                <div class="status-indicators">
                  <div class="indicator">
                    <el-tag
                      size="small"
                      :type="getConfigStatusType(connector.configValid)"
                    >
                      配置{{ connector.configValid ? '正常' : '异常' }}
                    </el-tag>
                  </div>
                  <div class="indicator">
                    <el-tag size="small" type="info">
                      {{ Object.keys(connector.config).length }}项配置
                    </el-tag>
                  </div>
                  <div v-if="connector.lastError" class="indicator">
                    <el-tooltip :content="connector.lastError" placement="top">
                      <el-tag size="small" type="danger"> 最近错误 </el-tag>
                    </el-tooltip>
                  </div>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 空状态 -->
    <div v-if="!filteredConnectors.length && !loading" class="empty-state">
      <el-empty description="暂无连接器配置">
        <el-button type="primary" :icon="Plus" @click="handleCreate">
          添加第一个连接器
        </el-button>
      </el-empty>
    </div>

    <!-- 连接器表单对话框 -->
    <el-dialog
      v-model="formVisible"
      :title="editingConnector?.id ? '编辑连接器' : '添加连接器'"
      width="800px"
      :before-close="handleFormClose"
    >
      <BaseForm
        v-if="editingConnector"
        v-model="editingConnector"
        :fields="formFields"
        :rules="formRules"
        label-width="120px"
      />

      <template #footer>
        <el-button @click="formVisible = false">取消</el-button>
        <el-button :loading="testing" @click="handleTestConnection">
          测试连接
        </el-button>
        <el-button type="primary" :loading="saving" @click="handleSave">
          {{ editingConnector?.id ? '更新' : '创建' }}
        </el-button>
      </template>
    </el-dialog>

    <!-- 连接测试结果对话框 -->
    <el-dialog v-model="testResultVisible" title="连接测试结果" width="600px">
      <div class="test-result">
        <div class="result-header">
          <el-icon :size="24" :class="testResult.success ? 'success' : 'error'">
            <component
              :is="testResult.success ? 'CircleCheck' : 'CircleClose'"
            />
          </el-icon>
          <h3>{{ testResult.success ? '连接成功' : '连接失败' }}</h3>
        </div>

        <div class="result-details">
          <el-descriptions :column="1" border>
            <el-descriptions-item label="测试时间">{{
              formatTime(testResult.timestamp)
            }}</el-descriptions-item>
            <el-descriptions-item label="响应时间"
              >{{ testResult.responseTime }}ms</el-descriptions-item
            >
            <el-descriptions-item label="状态">{{
              testResult.message
            }}</el-descriptions-item>
          </el-descriptions>

          <div v-if="testResult.details" class="result-message">
            <h4>详细信息:</h4>
            <pre>{{ testResult.details }}</pre>
          </div>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import {
  Connection,
  List,
  Grid,
  Refresh,
  Plus,
  More,
  VideoPlay,
  VideoPause,
  Delete,
  Download,
  Upload,
  Document,
  View,
  Edit,
  Link,
  CircleCheck,
  CircleClose,
  ArrowDown,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, reactive, onMounted, onUnmounted } from 'vue'

// 导入组件
import {
  BaseTable,
  SearchBox,
  FilterPanel,
  ActionButtons,
  BaseForm,
} from '../../components/base'
import { ConnectionStatus } from '../../components/business'

// 导入API
import { connectorApi } from '@/api/connector'

// 类型定义
interface Connector {
  id: string
  name: string
  type: 'mqtt' | 'http' | 'influxdb' | 'websocket' | 'tcp' | 'opcua'
  status: 'connected' | 'disconnected' | 'connecting' | 'error'
  config: Record<string, any>
  configValid: boolean
  protocol?: string
  host: string
  port: number
  messagesSent: number
  errorCount: number
  avgLatency: number
  uptime?: number
  lastError?: string
  createdAt: Date
  updatedAt: Date
}

interface TestResult {
  success: boolean
  message: string
  responseTime: number
  timestamp: Date
  details?: string
}

// 状态管理
const viewMode = ref<'list' | 'grid'>('list')
const loading = ref(false)
const saving = ref(false)
const testing = ref(false)
const searchText = ref('')
const selectedConnectors = ref<Connector[]>([])

// 表单状态
const formVisible = ref(false)
const testResultVisible = ref(false)
const editingConnector = ref<Connector | null>(null)
const testResult = ref<TestResult>({
  success: false,
  message: '',
  responseTime: 0,
  timestamp: new Date(),
})

// 筛选参数
const filterParams = ref({
  type: '',
  status: '',
  configValid: '',
})

// 连接器数据 - 从API获取
const connectors = ref<Connector[]>([])

let updateInterval: NodeJS.Timeout | null = null

// 计算属性
const filteredConnectors = computed(() => {
  let result = connectors.value

  // 搜索筛选
  if (searchText.value) {
    const search = searchText.value.toLowerCase()
    result = result.filter(
      conn =>
        conn.name.toLowerCase().includes(search) ||
        conn.type.toLowerCase().includes(search) ||
        conn.host.toLowerCase().includes(search)
    )
  }

  // 属性筛选
  if (filterParams.value.type) {
    result = result.filter(conn => conn.type === filterParams.value.type)
  }

  if (filterParams.value.status) {
    result = result.filter(conn => conn.status === filterParams.value.status)
  }

  if (filterParams.value.configValid) {
    const isValid = filterParams.value.configValid === 'true'
    result = result.filter(conn => conn.configValid === isValid)
  }

  return result
})

const tableColumns = computed(() => [
  {
    prop: 'name',
    label: '连接器名称',
    sortable: true,
    minWidth: 160,
  },
  {
    prop: 'type',
    label: '类型',
    sortable: true,
    width: 100,
    formatter: (row: Connector) => getTypeLabel(row.type),
  },
  {
    prop: 'status',
    label: '连接状态',
    width: 180,
    slot: 'status',
  },
  {
    prop: 'host',
    label: '目标地址',
    minWidth: 150,
    formatter: (row: Connector) => `${row.host}:${row.port}`,
  },
  {
    prop: 'config',
    label: '配置状态',
    width: 140,
    slot: 'config',
  },
  {
    prop: 'metrics',
    label: '性能指标',
    width: 200,
    slot: 'metrics',
  },
  {
    prop: 'updatedAt',
    label: '最后更新',
    width: 140,
    sortable: true,
    formatter: (row: Connector) => formatTime(row.updatedAt),
  },
  {
    prop: 'actions',
    label: '操作',
    width: 180,
    fixed: 'right',
    slot: 'actions',
  },
])

const filterConfig = computed(() => [
  {
    name: 'type',
    label: '连接器类型',
    type: 'select',
    options: [
      { label: 'MQTT', value: 'mqtt' },
      { label: 'HTTP/HTTPS', value: 'http' },
      { label: 'InfluxDB', value: 'influxdb' },
      { label: 'WebSocket', value: 'websocket' },
      { label: 'TCP', value: 'tcp' },
      { label: 'OPC UA', value: 'opcua' },
    ],
  },
  {
    name: 'status',
    label: '连接状态',
    type: 'select',
    options: [
      { label: '已连接', value: 'connected' },
      { label: '已断开', value: 'disconnected' },
      { label: '连接中', value: 'connecting' },
      { label: '错误', value: 'error' },
    ],
  },
  {
    name: 'configValid',
    label: '配置状态',
    type: 'select',
    options: [
      { label: '配置正常', value: 'true' },
      { label: '配置异常', value: 'false' },
    ],
  },
])

const formFields = computed(() => {
  if (!editingConnector.value) return []

  const baseFields = [
    {
      name: 'name',
      label: '连接器名称',
      type: 'input',
      required: true,
      placeholder: '请输入连接器名称',
    },
    {
      name: 'type',
      label: '连接器类型',
      type: 'select',
      required: true,
      options: [
        { label: 'MQTT', value: 'mqtt' },
        { label: 'HTTP/HTTPS', value: 'http' },
        { label: 'InfluxDB', value: 'influxdb' },
        { label: 'WebSocket', value: 'websocket' },
        { label: 'TCP', value: 'tcp' },
        { label: 'OPC UA', value: 'opcua' },
      ],
    },
  ]

  // 根据类型添加特定配置字段
  const typeFields = getTypeSpecificFields(editingConnector.value.type)

  return [...baseFields, ...typeFields]
})

const formRules = {
  name: [
    { required: true, message: '请输入连接器名称', trigger: 'blur' },
    { min: 2, max: 50, message: '名称长度应在2-50字符之间', trigger: 'blur' },
  ],
  type: [{ required: true, message: '请选择连接器类型', trigger: 'change' }],
}

// API调用方法
const loadConnectors = async () => {
  try {
    const response = await connectorApi.getConnectors()
    connectors.value = response.data || []
  } catch (error) {
    console.error('获取连接器列表失败:', error)
    // 如果API未实现，使用空数组
    connectors.value = []
  }
}

// 方法
const formatNumber = (num: number) => {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toString()
}

const formatTime = (time: Date) => {
  return time.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const getOverallStatusType = () => {
  const hasError = connectors.value.some(conn => conn.status === 'error')
  const hasDisconnected = connectors.value.some(
    conn => conn.status === 'disconnected'
  )

  if (hasError) return 'danger'
  if (hasDisconnected) return 'warning'
  return 'success'
}

const getOverallStatusText = () => {
  const connectedCount = connectors.value.filter(
    conn => conn.status === 'connected'
  ).length
  return `${connectedCount}/${connectors.value.length} 已连接`
}

const getTypeLabel = (type: string) => {
  const labels: Record<string, string> = {
    mqtt: 'MQTT',
    http: 'HTTP/HTTPS',
    influxdb: 'InfluxDB',
    websocket: 'WebSocket',
    tcp: 'TCP',
    opcua: 'OPC UA',
  }
  return labels[type] || type.toUpperCase()
}

const getTypeIcon = (type: string) => {
  const icons: Record<string, any> = {
    mqtt: Connection,
    http: Link,
    influxdb: Document,
    websocket: Connection,
    tcp: Link,
    opcua: Connection,
  }
  return icons[type] || Connection
}

const getTypeIconClass = (type: string) => {
  return `connector-icon--${type}`
}

const getConfigStatusType = (valid: boolean) => {
  return valid ? 'success' : 'danger'
}

const getCardClass = (connector: Connector) => {
  return {
    'connector-card--connected': connector.status === 'connected',
    'connector-card--error': connector.status === 'error',
    'connector-card--disconnected': connector.status === 'disconnected',
  }
}

const getConnectionTarget = (connector: Connector) => {
  switch (connector.type) {
    case 'mqtt':
      return `${connector.host}:${connector.port}`
    case 'http':
      return connector.config.url || `${connector.host}:${connector.port}`
    case 'influxdb':
      return connector.config.url || `${connector.host}:${connector.port}`
    case 'websocket':
      return connector.config.url || `${connector.host}:${connector.port}`
    default:
      return `${connector.host}:${connector.port}`
  }
}

const getActionButtons = (connector: Connector) => {
  return [
    {
      label: '详情',
      command: 'view',
      icon: View,
      type: 'primary',
    },
    {
      label: '编辑',
      command: 'edit',
      icon: Edit,
      type: 'default',
    },
    {
      label: '测试',
      command: 'test',
      icon: Link,
      type: 'default',
    },
    {
      label: connector.status === 'connected' ? '停止' : '启动',
      command: connector.status === 'connected' ? 'stop' : 'start',
      icon: connector.status === 'connected' ? VideoPause : VideoPlay,
      type: connector.status === 'connected' ? 'warning' : 'success',
    },
    {
      label: '删除',
      command: 'delete',
      icon: Delete,
      type: 'danger',
    },
  ]
}

const getTypeSpecificFields = (type: string) => {
  const commonFields = [
    {
      name: 'host',
      label: '主机地址',
      type: 'input',
      required: true,
      placeholder: '请输入主机地址或IP',
    },
    {
      name: 'port',
      label: '端口号',
      type: 'number',
      required: true,
      placeholder: '请输入端口号',
    },
  ]

  switch (type) {
    case 'mqtt':
      return [
        ...commonFields,
        {
          name: 'config.username',
          label: '用户名',
          type: 'input',
          placeholder: '请输入用户名（可选）',
        },
        {
          name: 'config.password',
          label: '密码',
          type: 'password',
          placeholder: '请输入密码（可选）',
        },
        {
          name: 'config.topic',
          label: '主题',
          type: 'input',
          required: true,
          placeholder: '请输入MQTT主题',
        },
        {
          name: 'config.qos',
          label: 'QoS等级',
          type: 'select',
          options: [
            { label: '0 - 至多一次', value: 0 },
            { label: '1 - 至少一次', value: 1 },
            { label: '2 - 恰好一次', value: 2 },
          ],
        },
      ]

    case 'http':
      return [
        {
          name: 'config.url',
          label: '请求URL',
          type: 'input',
          required: true,
          placeholder: '请输入完整的URL地址',
        },
        {
          name: 'config.method',
          label: '请求方法',
          type: 'select',
          options: [
            { label: 'POST', value: 'POST' },
            { label: 'PUT', value: 'PUT' },
            { label: 'PATCH', value: 'PATCH' },
          ],
        },
        {
          name: 'config.timeout',
          label: '超时时间(ms)',
          type: 'number',
          placeholder: '请求超时时间，默认5000ms',
        },
      ]

    case 'influxdb':
      return [
        {
          name: 'config.url',
          label: 'InfluxDB URL',
          type: 'input',
          required: true,
          placeholder: 'http://localhost:8086',
        },
        {
          name: 'config.database',
          label: '数据库名称',
          type: 'input',
          required: true,
          placeholder: '请输入数据库名称',
        },
        {
          name: 'config.username',
          label: '用户名',
          type: 'input',
          placeholder: '请输入用户名',
        },
        {
          name: 'config.password',
          label: '密码',
          type: 'password',
          placeholder: '请输入密码',
        },
        {
          name: 'config.batchSize',
          label: '批量大小',
          type: 'number',
          placeholder: '批量写入的数据点数量',
        },
      ]

    default:
      return commonFields
  }
}

// 事件处理
const handleRefresh = async () => {
  loading.value = true

  try {
    // 调用真实的连接器列表API
    await loadConnectors()
    ElMessage.success('连接器状态已刷新')
  } catch (error) {
    console.error('刷新连接器失败:', error)
    ElMessage.error('刷新连接器失败')
  } finally {
    loading.value = false
  }
}

const handleSearch = () => {
  ElMessage.info(`搜索: ${searchText.value}`)
}

const handleFilter = () => {
  ElMessage.info('筛选条件已应用')
}

const handleSelectionChange = (selection: Connector[]) => {
  selectedConnectors.value = selection
}

const handleSortChange = ({ prop, order }: any) => {
  ElMessage.info(`排序: ${prop} ${order}`)
}

const handleCreate = () => {
  editingConnector.value = {
    id: '',
    name: '',
    type: 'mqtt',
    status: 'disconnected',
    config: {},
    configValid: true,
    host: '',
    port: 1883,
    messagesSent: 0,
    errorCount: 0,
    avgLatency: 0,
    createdAt: new Date(),
    updatedAt: new Date(),
  }
  formVisible.value = true
}

const handleRowAction = (action: string, connector: Connector) => {
  switch (action) {
    case 'view':
      ElMessage.info(`查看连接器: ${connector.name}`)
      // TODO: 跳转到详情页面
      break

    case 'edit':
      editingConnector.value = { ...connector }
      formVisible.value = true
      break

    case 'test':
      handleTestConnection(connector)
      break

    case 'start':
      handleStartConnector(connector)
      break

    case 'stop':
      handleStopConnector(connector)
      break

    case 'delete':
      handleDeleteConnector(connector)
      break
  }
}

const handleBatchAction = async (command: string) => {
  const count = selectedConnectors.value.length
  const connectorIds = selectedConnectors.value.map(conn => conn.id)

  try {
    switch (command) {
      case 'start':
        await connectorApi.batchStartConnectors(connectorIds)
        ElMessage.success(`已启动 ${count} 个连接器`)
        break

      case 'stop':
        await connectorApi.batchStopConnectors(connectorIds)
        ElMessage.success(`已停止 ${count} 个连接器`)
        break

      case 'restart':
        await connectorApi.batchRestartConnectors(connectorIds)
        ElMessage.success(`已重启 ${count} 个连接器`)
        break

      case 'delete':
        await ElMessageBox.confirm(
          `确定要删除选中的 ${count} 个连接器吗？此操作不可撤销。`,
          '确认删除',
          {
            confirmButtonText: '删除',
            cancelButtonText: '取消',
            type: 'warning',
          }
        )

        await connectorApi.batchDeleteConnectors(connectorIds)
        selectedConnectors.value = []
        ElMessage.success(`已删除 ${count} 个连接器`)
        break
    }

    await handleRefresh()
  } catch (error) {
    if (error !== 'cancel') {
      console.error('批量操作失败:', error)
      ElMessage.error('批量操作失败')
    }
  }
}

const handleAction = (command: string) => {
  switch (command) {
    case 'export':
      handleExportConfig()
      break
    case 'import':
      ElMessage.info('导入配置功能开发中')
      break
    case 'template':
      ElMessage.info('配置模板功能开发中')
      break
  }
}

const handleExportConfig = () => {
  const exportData = {
    connectors: connectors.value.map(conn => ({
      name: conn.name,
      type: conn.type,
      config: conn.config,
      host: conn.host,
      port: conn.port,
    })),
    exportTime: new Date().toISOString(),
  }

  const blob = new Blob([JSON.stringify(exportData, null, 2)], {
    type: 'application/json',
  })

  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `connectors_config_${new Date().toISOString().split('T')[0]}.json`
  a.click()

  URL.revokeObjectURL(url)
  ElMessage.success('配置导出成功')
}

const handleStartConnector = async (connector: Connector) => {
  connector.status = 'connecting'
  ElMessage.info(`正在启动连接器: ${connector.name}`)

  try {
    // 调用真实的连接器启动API
    await connectorApi.startConnector(connector.id)
    connector.status = 'connected'
    ElMessage.success(`连接器 ${connector.name} 已启动`)
    await handleRefresh()
  } catch (error) {
    console.error('启动连接器失败:', error)
    connector.status = 'error'
    connector.lastError = '启动失败'
    ElMessage.error(`连接器 ${connector.name} 启动失败`)
  }
}

const handleStopConnector = async (connector: Connector) => {
  try {
    // 调用真实的连接器停止API
    await connectorApi.stopConnector(connector.id)
    connector.status = 'disconnected'
    ElMessage.success(`连接器 ${connector.name} 已停止`)
    await handleRefresh()
  } catch (error) {
    console.error('停止连接器失败:', error)
    ElMessage.error(`连接器 ${connector.name} 停止失败`)
  }
}

const handleDeleteConnector = async (connector: Connector) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除连接器 "${connector.name}" 吗？此操作不可撤销。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    // 调用真实的删除API
    await connectorApi.deleteConnector(connector.id)
    ElMessage.success(`连接器 ${connector.name} 已删除`)
    await handleRefresh()
  } catch (error) {
    if (error !== 'cancel') {
      console.error('删除连接器失败:', error)
      ElMessage.error('删除连接器失败')
    }
  }
}

const handleTestConnection = async (connector?: Connector) => {
  const testConnector = connector || editingConnector.value
  if (!testConnector) return

  testing.value = true

  try {
    // 调用真实的连接测试API
    const result = await connectorApi.testConnection(
      testConnector.id || testConnector
    )

    testResult.value = {
      success: result.success,
      message: result.message,
      responseTime: result.responseTime,
      timestamp: new Date(),
      details: result.details,
    }

    testResultVisible.value = true

    if (result.success) {
      ElMessage.success('连接测试成功')
    } else {
      ElMessage.error('连接测试失败')
    }
  } catch (error) {
    console.error('连接测试失败:', error)
    testResult.value = {
      success: false,
      message: '连接测试异常',
      responseTime: 0,
      timestamp: new Date(),
      details: '测试过程中发生错误',
    }
    testResultVisible.value = true
    ElMessage.error('连接测试异常')
  } finally {
    testing.value = false
  }
}

const handleSave = async () => {
  if (!editingConnector.value) return

  saving.value = true

  try {
    if (editingConnector.value.id) {
      // 更新现有连接器
      await connectorApi.updateConnector(
        editingConnector.value.id,
        editingConnector.value
      )
      ElMessage.success('连接器配置已更新')
    } else {
      // 创建新连接器
      await connectorApi.createConnector(editingConnector.value)
      ElMessage.success('连接器创建成功')
    }

    formVisible.value = false
    editingConnector.value = null
    await handleRefresh()
  } catch (error) {
    console.error('保存连接器失败:', error)
    ElMessage.error('保存连接器失败')
  } finally {
    saving.value = false
  }
}

const handleFormClose = () => {
  formVisible.value = false
  editingConnector.value = null
}

// 实时更新 - 从API获取最新状态
const startRealtimeUpdate = () => {
  updateInterval = setInterval(async () => {
    try {
      await loadConnectors()
    } catch (error) {
      console.warn('自动刷新连接器状态失败:', error)
    }
  }, 30000) // 每30秒更新一次
}

const stopRealtimeUpdate = () => {
  if (updateInterval) {
    clearInterval(updateInterval)
    updateInterval = null
  }
}

// 生命周期
onMounted(async () => {
  await loadConnectors()
  startRealtimeUpdate()
})

onUnmounted(() => {
  stopRealtimeUpdate()
})
</script>

<style scoped lang="scss">
.connectors-page {
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
    align-items: center;
    gap: 12px;
  }
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding: 16px;
  background: var(--el-bg-color);
  border-radius: 8px;
  border: 1px solid var(--el-border-color-light);

  .toolbar-left {
    display: flex;
    gap: 16px;
    align-items: center;
  }

  .toolbar-right {
    display: flex;
    gap: 12px;
  }
}

// 列表视图
.list-view {
  background: var(--el-bg-color);
  border-radius: 8px;
  overflow: hidden;

  .status-cell {
    display: flex;
    align-items: center;
  }

  .config-cell {
    display: flex;
    flex-direction: column;
    gap: 4px;

    .config-count {
      font-size: 12px;
      color: var(--el-text-color-secondary);
    }
  }

  .metrics-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;

    .metric-item {
      display: flex;
      justify-content: space-between;
      font-size: 12px;

      .metric-label {
        color: var(--el-text-color-secondary);
      }

      .metric-value {
        color: var(--el-text-color-primary);
        font-weight: 500;

        &.error {
          color: var(--el-color-danger);
        }
      }
    }
  }
}

// 卡片视图
.grid-view {
  .connector-card {
    height: 420px;
    transition: all 0.3s;

    &:hover {
      transform: translateY(-2px);
      box-shadow: var(--el-box-shadow);
    }

    &.connector-card--connected {
      border-left: 4px solid var(--el-color-success);
    }

    &.connector-card--error {
      border-left: 4px solid var(--el-color-danger);
    }

    &.connector-card--disconnected {
      border-left: 4px solid var(--el-color-warning);
    }

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;

      .connector-info {
        display: flex;
        gap: 12px;
        align-items: center;
        flex: 1;

        .connector-icon {
          font-size: 24px;

          &.connector-icon--mqtt {
            color: var(--el-color-primary);
          }

          &.connector-icon--http {
            color: var(--el-color-success);
          }

          &.connector-icon--influxdb {
            color: var(--el-color-warning);
          }

          &.connector-icon--websocket {
            color: var(--el-color-info);
          }

          &.connector-icon--tcp {
            color: var(--el-color-danger);
          }

          &.connector-icon--opcua {
            color: #9c27b0;
          }
        }

        .connector-basic {
          .connector-name {
            margin: 0 0 4px 0;
            font-size: 16px;
            font-weight: 600;
            color: var(--el-text-color-primary);
          }

          .connector-type {
            font-size: 12px;
            color: var(--el-text-color-secondary);
            background: var(--el-fill-color-light);
            padding: 2px 8px;
            border-radius: 12px;
          }
        }
      }

      .card-actions {
        display: flex;
        align-items: center;
        gap: 8px;
      }
    }

    .card-content {
      height: calc(100% - 60px);
      display: flex;
      flex-direction: column;
      gap: 16px;

      .connection-info {
        .info-item {
          display: flex;
          justify-content: space-between;
          margin-bottom: 8px;
          font-size: 13px;

          .info-label {
            color: var(--el-text-color-secondary);
            font-weight: 500;
          }

          .info-value {
            color: var(--el-text-color-primary);
            font-family: monospace;
          }
        }
      }

      .performance-metrics {
        flex: 1;

        .metrics-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 12px;

          .metric-box {
            background: var(--el-fill-color-light);
            padding: 12px;
            border-radius: 6px;
            text-align: center;

            .metric-number {
              font-size: 18px;
              font-weight: bold;
              color: var(--el-color-primary);
              margin-bottom: 4px;

              &.error {
                color: var(--el-color-danger);
              }
            }

            .metric-title {
              font-size: 11px;
              color: var(--el-text-color-secondary);
            }
          }
        }
      }

      .status-info {
        .status-indicators {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;

          .indicator {
            .el-tag {
              font-size: 11px;
            }
          }
        }
      }
    }
  }
}

// 空状态
.empty-state {
  text-align: center;
  padding: 60px 20px;
}

// 对话框样式
.test-result {
  .result-header {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--el-border-color-lighter);

    .el-icon {
      &.success {
        color: var(--el-color-success);
      }

      &.error {
        color: var(--el-color-danger);
      }
    }

    h3 {
      margin: 0;
      color: var(--el-text-color-primary);
    }
  }

  .result-details {
    .result-message {
      margin-top: 16px;

      h4 {
        margin: 0 0 8px 0;
        font-size: 14px;
        color: var(--el-text-color-primary);
      }

      pre {
        background: var(--el-fill-color-light);
        padding: 12px;
        border-radius: 4px;
        font-size: 12px;
        line-height: 1.4;
        margin: 0;
        white-space: pre-wrap;
        word-break: break-word;
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .grid-view {
    .el-col {
      margin-bottom: 16px;
    }
  }

  .connector-card {
    height: auto !important;
    min-height: 350px;

    .card-content {
      height: auto !important;

      .performance-metrics {
        .metrics-grid {
          grid-template-columns: 1fr 1fr;
        }
      }
    }
  }
}

@media (max-width: 768px) {
  .connectors-page {
    padding: 8px;
  }

  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }

  .toolbar {
    flex-direction: column;
    align-items: stretch;
    gap: 16px;

    .toolbar-left,
    .toolbar-right {
      justify-content: flex-start;
    }
  }

  .grid-view {
    .el-row {
      flex-direction: column;
    }

    .el-col {
      width: 100% !important;
      max-width: none !important;
    }

    .connector-card {
      .performance-metrics {
        .metrics-grid {
          grid-template-columns: repeat(4, 1fr);
        }
      }
    }
  }
}

@media (max-width: 480px) {
  .connector-card {
    .card-header {
      .connector-info {
        .connector-basic {
          .connector-name {
            font-size: 14px;
          }
        }
      }
    }

    .performance-metrics {
      .metrics-grid {
        grid-template-columns: 1fr 1fr !important;

        .metric-box {
          padding: 8px;

          .metric-number {
            font-size: 16px;
          }
        }
      }
    }
  }
}
</style>
