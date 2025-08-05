<template>
  <div class="connectors-detail-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <el-button :icon="ArrowLeft" @click="handleGoBack">
          返回列表
        </el-button>
        
        <div class="header-title">
          <el-icon :size="32" :class="getStatusIconClass()">
            <component :is="getStatusIcon()" />
          </el-icon>
          <div class="title-info">
            <h1>{{ connectionData.name }}</h1>
            <p class="connection-type">{{ connectionData.protocol }} - {{ connectionData.connectionType }}</p>
          </div>
        </div>
      </div>
      
      <div class="header-actions">
        <el-dropdown @command="handleCommand">
          <el-button :icon="Operation">
            操作
            <el-icon class="el-icon--right"><arrow-down /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="start" :disabled="connectionData.status === 'connected'">
                <el-icon><VideoPlay /></el-icon>
                启动连接
              </el-dropdown-item>
              <el-dropdown-item command="stop" :disabled="connectionData.status !== 'connected'">
                <el-icon><VideoPause /></el-icon>
                停止连接
              </el-dropdown-item>
              <el-dropdown-item command="restart" :disabled="connectionData.status === 'disconnected'">
                <el-icon><Refresh /></el-icon>
                重启连接
              </el-dropdown-item>
              <el-dropdown-item command="test" divided>
                <el-icon><Link /></el-icon>
                测试连接
              </el-dropdown-item>
              <el-dropdown-item command="export">
                <el-icon><Download /></el-icon>
                导出配置
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        
        <el-button type="primary" :icon="Edit" @click="handleEditConnection">
          编辑配置
        </el-button>
      </div>
    </div>

    <!-- 状态概览 -->
    <div class="status-overview">
      <el-row :gutter="20">
        <el-col :span="6">
          <div class="status-card">
            <div class="card-header">
              <el-icon><Connection /></el-icon>
              <span>连接状态</span>
            </div>
            <div class="card-content">
              <ConnectionStatus
                :status="connectionData.status"
                :name="connectionData.name"
                :type="connectionData.connectionType"
                :size="'large'"
              />
            </div>
          </div>
        </el-col>
        
        <el-col :span="6">
          <div class="status-card">
            <div class="card-header">
              <el-icon><Timer /></el-icon>
              <span>运行时间</span>
            </div>
            <div class="card-content">
              <div class="metric-value">{{ formatUptime(connectionData.uptime) }}</div>
              <div class="metric-label">已运行</div>
            </div>
          </div>
        </el-col>
        
        <el-col :span="6">
          <div class="status-card">
            <div class="card-header">
              <el-icon><DataAnalysis /></el-icon>
              <span>数据传输</span>
            </div>
            <div class="card-content">
              <div class="metric-value">{{ formatBytes(connectionData.bytesTransferred) }}</div>
              <div class="metric-label">总传输量</div>
            </div>
          </div>
        </el-col>
        
        <el-col :span="6">
          <div class="status-card">
            <div class="card-header">
              <el-icon><Warning /></el-icon>
              <span>错误统计</span>
            </div>
            <div class="card-content">
              <div class="metric-value" :class="{ error: connectionData.errorCount > 0 }">
                {{ connectionData.errorCount }}
              </div>
              <div class="metric-label">错误次数</div>
            </div>
          </div>
        </el-col>
      </el-row>
    </div>

    <!-- 主要内容 -->
    <div class="main-content">
      <el-row :gutter="20">
        <!-- 左侧列 -->
        <el-col :span="16">
          <!-- 性能监控 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><TrendCharts /></el-icon>
                性能监控
              </h3>
              <div class="header-actions">
                <el-button-group size="small">
                  <el-button 
                    v-for="period in timePeriods" 
                    :key="period.value"
                    :type="selectedPeriod === period.value ? 'primary' : ''"
                    @click="selectedPeriod = period.value"
                  >
                    {{ period.label }}
                  </el-button>
                </el-button-group>
              </div>
            </div>
            
            <div class="charts-container">
              <el-row :gutter="16">
                <el-col :span="12">
                  <div class="chart-card">
                    <h4>网络延迟 (ms)</h4>
                    <ChartContainer
                      title="网络延迟"
                      chart-type="line"
                      :data="latencyData"
                      :height="'200px'"
                    />
                  </div>
                </el-col>
                <el-col :span="12">
                  <div class="chart-card">
                    <h4>吞吐率 (KB/s)</h4>
                    <ChartContainer
                      title="吐吐率"
                      chart-type="line"
                      :data="throughputData"
                      :height="'200px'"
                    />
                  </div>
                </el-col>
              </el-row>
              
              <el-row :gutter="16" style="margin-top: 16px">
                <el-col :span="12">
                  <div class="chart-card">
                    <h4>连接数量</h4>
                    <ChartContainer
                      title="连接数量"
                      chart-type="bar"
                      :data="connectionCountData"
                      :height="'200px'"
                    />
                  </div>
                </el-col>
                <el-col :span="12">
                  <div class="chart-card">
                    <h4>错误率 (%)</h4>
                    <ChartContainer
                      title="错误率"
                      chart-type="line"
                      :data="errorRateData"
                      :height="'200px'"
                    />
                  </div>
                </el-col>
              </el-row>
            </div>
          </div>
          
          <!-- 连接日志 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><Document /></el-icon>
                连接日志
              </h3>
              <div class="header-actions">
                <el-input
                  v-model="logFilter"
                  placeholder="搜索日志..."
                  style="width: 200px"
                  clearable
                >
                  <template #prefix>
                    <el-icon><Search /></el-icon>
                  </template>
                </el-input>
                <el-select v-model="logLevel" placeholder="日志级别" style="width: 120px">
                  <el-option label="全部" value="" />
                  <el-option label="错误" value="error" />
                  <el-option label="警告" value="warn" />
                  <el-option label="信息" value="info" />
                  <el-option label="调试" value="debug" />
                </el-select>
                <el-button :icon="Refresh" @click="refreshLogs">
                  刷新
                </el-button>
              </div>
            </div>
            
            <div class="logs-container">
              <el-table
                :data="filteredLogs"
                stripe
                max-height="400"
                empty-text="暂无日志数据"
              >
                <el-table-column prop="timestamp" label="时间" width="180">
                  <template #default="{ row }">
                    <span class="log-time">{{ formatTime(row.timestamp) }}</span>
                  </template>
                </el-table-column>
                <el-table-column prop="level" label="级别" width="80">
                  <template #default="{ row }">
                    <el-tag :type="getLogLevelType(row.level)" size="small">
                      {{ row.level.toUpperCase() }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="message" label="消息" show-overflow-tooltip>
                  <template #default="{ row }">
                    <span class="log-message">{{ row.message }}</span>
                  </template>
                </el-table-column>
                <el-table-column prop="source" label="来源" width="120">
                  <template #default="{ row }">
                    <el-tag size="small" type="info">{{ row.source }}</el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </div>
        </el-col>
        
        <!-- 右侧列 -->
        <el-col :span="8">
          <!-- 基本信息 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><InfoFilled /></el-icon>
                基本信息
              </h3>
              <el-button size="small" text @click="handleEditConnection">
                <el-icon><Edit /></el-icon>
                编辑
              </el-button>
            </div>
            
            <div class="info-container">
              <el-descriptions :column="1" border>
                <el-descriptions-item label="连接名称">
                  <strong>{{ connectionData.name }}</strong>
                </el-descriptions-item>
                <el-descriptions-item label="连接类型">
                  <el-tag type="primary">{{ connectionData.connectionType }}</el-tag>
                </el-descriptions-item>
                <el-descriptions-item label="应用协议">
                  <el-tag type="success">{{ connectionData.protocol }}</el-tag>
                </el-descriptions-item>
                <el-descriptions-item label="本地地址">
                  {{ connectionData.config?.host || 'N/A' }}:{{ connectionData.config?.port || 'N/A' }}
                </el-descriptions-item>
                <el-descriptions-item label="启用状态">
                  <el-switch 
                    v-model="connectionData.enabled" 
                    @change="handleToggleEnabled"
                    :disabled="loading"
                  />
                </el-descriptions-item>
                <el-descriptions-item label="自动重连">
                  <el-switch 
                    v-model="connectionData.autoReconnect" 
                    @change="handleToggleAutoReconnect"
                    :disabled="loading"
                  />
                </el-descriptions-item>
                <el-descriptions-item label="创建时间">
                  {{ formatTime(connectionData.createdAt) }}
                </el-descriptions-item>
                <el-descriptions-item label="更新时间">
                  {{ formatTime(connectionData.updatedAt) }}
                </el-descriptions-item>
              </el-descriptions>
            </div>
          </div>
          
          <!-- 连接配置 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><Setting /></el-icon>
                连接配置
              </h3>
              <el-button size="small" text @click="showConfigDialog = true">
                <el-icon><View /></el-icon>
                查看详情
              </el-button>
            </div>
            
            <div class="config-summary">
              <div class="config-item" v-for="(value, key) in connectionData.config" :key="key">
                <span class="config-label">{{ formatConfigLabel(key) }}:</span>
                <span class="config-value">{{ value }}</span>
              </div>
            </div>
            
            <div class="section-header" style="margin-top: 20px">
              <h4>安全配置</h4>
            </div>
            
            <el-descriptions :column="1" size="small">
              <el-descriptions-item label="TLS/SSL">
                <el-tag :type="connectionData.security?.enableTls ? 'success' : 'info'" size="small">
                  {{ connectionData.security?.enableTls ? '已启用' : '未启用' }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="证书验证">
                <el-tag :type="connectionData.security?.verifyCertificate ? 'success' : 'warning'" size="small">
                  {{ connectionData.security?.verifyCertificate ? '已启用' : '禁用' }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="身份验证" v-if="connectionData.security?.username">
                <el-tag type="success" size="small">已配置</el-tag>
              </el-descriptions-item>
            </el-descriptions>
          </div>
          
          <!-- 连接池信息 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><Cpu /></el-icon>
                连接池
              </h3>
            </div>
            
            <div class="pool-stats">
              <el-progress 
                :percentage="poolUsagePercentage" 
                :color="getPoolUsageColor()"
                :show-text="false"
                :stroke-width="8"
              />
              <div class="pool-info">
                <span class="pool-current">{{ connectionData.activeConnections }}</span>
                <span class="pool-separator">/</span>
                <span class="pool-max">{{ connectionData.pool?.maxConnections }}</span>
                <span class="pool-label">连接数</span>
              </div>
            </div>
            
            <el-descriptions :column="1" size="small">
              <el-descriptions-item label="最小连接数">
                {{ connectionData.pool?.minConnections || 0 }}
              </el-descriptions-item>
              <el-descriptions-item label="连接超时">
                {{ connectionData.pool?.connectionTimeout }}ms
              </el-descriptions-item>
              <el-descriptions-item label="空闲超时">
                {{ connectionData.pool?.idleTimeout }}ms
              </el-descriptions-item>
            </el-descriptions>
          </div>
          
          <!-- 最近事件 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><Clock /></el-icon>
                最近事件
              </h3>
            </div>
            
            <div class="events-timeline">
              <el-timeline>
                <el-timeline-item
                  v-for="event in recentEvents"
                  :key="event.id"
                  :timestamp="formatTime(event.timestamp)"
                  :type="getEventType(event.type)"
                >
                  <div class="event-content">
                    <div class="event-title">{{ event.title }}</div>
                    <div class="event-description">{{ event.description }}</div>
                  </div>
                </el-timeline-item>
              </el-timeline>
            </div>
          </div>
        </el-col>
      </el-row>
    </div>

    <!-- 配置详情对话框 -->
    <el-dialog
      v-model="showConfigDialog"
      title="连接配置详情"
      width="800px"
      :destroy-on-close="true"
    >
      <el-tabs v-model="configTab">
        <el-tab-pane label="JSON 格式" name="json">
          <div class="config-json">
            <pre><code>{{ formatConfigJson() }}</code></pre>
          </div>
        </el-tab-pane>
        <el-tab-pane label="YAML 格式" name="yaml">
          <div class="config-yaml">
            <pre><code>{{ formatConfigYaml() }}</code></pre>
          </div>
        </el-tab-pane>
      </el-tabs>
    </el-dialog>

    <!-- 加载遮罩 -->
    <div v-loading="loading" element-loading-text="正在加载..." class="loading-overlay" v-if="loading"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowLeft,
  Operation,
  ArrowDown,
  VideoPlay,
  VideoPause,
  Refresh,
  Link,
  Download,
  Edit,
  Connection,
  Timer,
  DataAnalysis,
  Warning,
  TrendCharts,
  Document,
  Search,
  InfoFilled,
  Setting,
  View,
  Cpu,
  Clock
} from '@element-plus/icons-vue'

// 导入业务组件
import {
  ConnectionStatus,
  ChartContainer
} from '../../components/business'
// import { PerformanceChart } from '../../components/business' // 暂时注释掉不存在的组件

// 类型定义
interface ConnectionData {
  id: string
  name: string
  description: string
  connectionType: string
  protocol: string
  status: 'connected' | 'disconnected' | 'connecting' | 'error'
  enabled: boolean
  autoReconnect: boolean
  uptime: number
  bytesTransferred: number
  errorCount: number
  activeConnections: number
  config: Record<string, any>
  security?: {
    enableTls: boolean
    verifyCertificate: boolean
    username?: string
  }
  pool?: {
    maxConnections: number
    minConnections: number
    connectionTimeout: number
    idleTimeout: number
  }
  createdAt: Date
  updatedAt: Date
}

interface LogEntry {
  id: string
  timestamp: Date
  level: 'error' | 'warn' | 'info' | 'debug'
  message: string
  source: string
}

interface Event {
  id: string
  timestamp: Date
  type: 'info' | 'success' | 'warning' | 'error'
  title: string
  description: string
}

interface ChartData {
  name: string
  value: number
  time: string
}

// 路由
const route = useRoute()
const router = useRouter()

// 状态管理
const loading = ref(false)
const showConfigDialog = ref(false)
const configTab = ref('json')
const selectedPeriod = ref('1h')
const logFilter = ref('')
const logLevel = ref('')

// 时间期间选项
const timePeriods = [
  { label: '1小时', value: '1h' },
  { label: '6小时', value: '6h' },
  { label: '24小时', value: '24h' },
  { label: '7天', value: '7d' }
]

// 连接数据
const connectionData = ref<ConnectionData>({
  id: '',
  name: '',
  description: '',
  connectionType: '',
  protocol: '',
  status: 'disconnected',
  enabled: false,
  autoReconnect: false,
  uptime: 0,
  bytesTransferred: 0,
  errorCount: 0,
  activeConnections: 0,
  config: {},
  createdAt: new Date(),
  updatedAt: new Date()
})

// 日志数据
const logs = ref<LogEntry[]>([])

// 事件数据
const recentEvents = ref<Event[]>([])

// 性能数据
const latencyData = ref<ChartData[]>([])
const throughputData = ref<ChartData[]>([])
const connectionCountData = ref<ChartData[]>([])
const errorRateData = ref<ChartData[]>([])

// 计算属性
const filteredLogs = computed(() => {
  return logs.value.filter(log => {
    const matchesFilter = !logFilter.value || 
      log.message.toLowerCase().includes(logFilter.value.toLowerCase())
    const matchesLevel = !logLevel.value || log.level === logLevel.value
    return matchesFilter && matchesLevel
  })
})

const poolUsagePercentage = computed(() => {
  if (!connectionData.value.pool?.maxConnections) return 0
  return Math.round(
    (connectionData.value.activeConnections / connectionData.value.pool.maxConnections) * 100
  )
})

// 方法
const getStatusIcon = () => {
  const iconMap = {
    connected: 'Connection',
    connecting: 'Loading',
    disconnected: 'Connection',
    error: 'Warning'
  }
  return iconMap[connectionData.value.status] || 'Connection'
}

const getStatusIconClass = () => {
  const classMap = {
    connected: 'status-icon connected',
    connecting: 'status-icon connecting',
    disconnected: 'status-icon disconnected',
    error: 'status-icon error'
  }
  return classMap[connectionData.value.status] || 'status-icon'
}

const getPoolUsageColor = () => {
  const percentage = poolUsagePercentage.value
  if (percentage < 60) return '#67c23a'
  if (percentage < 80) return '#e6a23c'
  return '#f56c6c'
}

const getLogLevelType = (level: string) => {
  const typeMap: Record<string, string> = {
    error: 'danger',
    warn: 'warning',
    info: 'info',
    debug: 'success'
  }
  return typeMap[level] || 'info'
}

const getEventType = (type: string) => {
  const typeMap: Record<string, string> = {
    error: 'danger',
    warning: 'warning',
    success: 'success',
    info: 'primary'
  }
  return typeMap[type] || 'primary'
}

const formatTime = (time: Date) => {
  return time.toLocaleString('zh-CN')
}

const formatUptime = (seconds: number) => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  
  if (days > 0) {
    return `${days}天 ${hours}小时`
  } else if (hours > 0) {
    return `${hours}小时 ${minutes}分钟`
  } else {
    return `${minutes}分钟`
  }
}

const formatBytes = (bytes: number) => {
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  if (bytes === 0) return '0 B'
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return `${Math.round(bytes / Math.pow(1024, i) * 100) / 100  } ${  sizes[i]}`
}

const formatConfigLabel = (key: string) => {
  const labelMap: Record<string, string> = {
    host: '主机地址',
    port: '端口号',
    timeout: '超时时间',
    serialPort: '串口号',
    baudRate: '波特率',
    unitId: '单元ID',
    endpoint: '端点URL',
    broker: '代理地址'
  }
  return labelMap[key] || key
}

const formatConfigJson = () => {
  try {
    return JSON.stringify(connectionData.value, null, 2)
  } catch (error) {
    return '配置格式错误'
  }
}

const formatConfigYaml = () => {
  try {
    const obj = connectionData.value
    let yaml = ''
    
    const toYaml = (obj: any, indent = 0) => {
      const spaces = '  '.repeat(indent)
      for (const [key, value] of Object.entries(obj)) {
        if (typeof value === 'object' && value !== null && !(value instanceof Date)) {
          yaml += `${spaces}${key}:\n`
          toYaml(value, indent + 1)
        } else {
          yaml += `${spaces}${key}: ${value}\n`
        }
      }
    }
    
    toYaml(obj)
    return yaml
  } catch (error) {
    return '配置格式错误'
  }
}

// 事件处理
const handleGoBack = () => {
  router.push('/connectors')
}

const handleEditConnection = () => {
  router.push(`/connectors/edit/${connectionData.value.id}`)
}

const handleCommand = async (command: string) => {
  loading.value = true
  
  try {
    switch (command) {
      case 'start':
        await handleStartConnection()
        break
      case 'stop':
        await handleStopConnection()
        break
      case 'restart':
        await handleRestartConnection()
        break
      case 'test':
        await handleTestConnection()
        break
      case 'export':
        handleExportConfig()
        break
    }
  } finally {
    loading.value = false
  }
}

const handleStartConnection = async () => {
  await new Promise(resolve => setTimeout(resolve, 1000))
  connectionData.value.status = 'connected'
  ElMessage.success('连接已启动')
  addEvent('成功', '连接启动', '连接已成功启动，状态正常')
}

const handleStopConnection = async () => {
  await new Promise(resolve => setTimeout(resolve, 800))
  connectionData.value.status = 'disconnected'
  ElMessage.success('连接已停止')
  addEvent('信息', '连接停止', '用户手动停止连接')
}

const handleRestartConnection = async () => {
  connectionData.value.status = 'connecting'
  await new Promise(resolve => setTimeout(resolve, 2000))
  connectionData.value.status = 'connected'
  ElMessage.success('连接已重启')
  addEvent('成功', '连接重启', '连接重启成功，恢复正常通信')
}

const handleTestConnection = async () => {
  ElMessage.info('正在测试连接...')
  await new Promise(resolve => setTimeout(resolve, 2000))
  const success = Math.random() > 0.2
  if (success) {
    ElMessage.success('连接测试成功')
    addEvent('成功', '连接测试', '连接测试通过，网络通信正常')
  } else {
    ElMessage.error('连接测试失败')
    addEvent('错误', '连接测试', '连接测试失败，请检查网络和配置')
  }
}

const handleExportConfig = () => {
  const exportData = {
    connection: connectionData.value,
    exportTime: new Date().toISOString(),
    version: '1.0'
  }
  
  const blob = new Blob([JSON.stringify(exportData, null, 2)], {
    type: 'application/json'
  })
  
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `connection_${connectionData.value.name}_${new Date().toISOString().split('T')[0]}.json`
  a.click()
  
  URL.revokeObjectURL(url)
  ElMessage.success('配置已导出')
}

const handleToggleEnabled = async (enabled: boolean) => {
  loading.value = true
  
  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    ElMessage.success(`连接已${enabled ? '启用' : '禁用'}`)
    addEvent('info', '状态更改', `连接已${enabled ? '启用' : '禁用'}`)
  } catch (error) {
    connectionData.value.enabled = !enabled
    ElMessage.error('操作失败')
  } finally {
    loading.value = false
  }
}

const handleToggleAutoReconnect = async (enabled: boolean) => {
  loading.value = true
  
  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    ElMessage.success(`自动重连已${enabled ? '启用' : '禁用'}`)
    addEvent('info', '配置更改', `自动重连已${enabled ? '启用' : '禁用'}`)
  } catch (error) {
    connectionData.value.autoReconnect = !enabled
    ElMessage.error('操作失败')
  } finally {
    loading.value = false
  }
}

const refreshLogs = () => {
  // 模拟刷新日志
  generateMockLogs()
  ElMessage.success('日志已刷新')
}

const addEvent = (type: string, title: string, description: string) => {
  const event: Event = {
    id: Date.now().toString(),
    timestamp: new Date(),
    type: type as any,
    title,
    description
  }
  recentEvents.value.unshift(event)
  if (recentEvents.value.length > 10) {
    recentEvents.value = recentEvents.value.slice(0, 10)
  }
}

// 数据初始化
const initializeData = () => {
  const connectionId = route.params.id as string
  
  // 模拟连接数据
  connectionData.value = {
    id: connectionId,
    name: '主控制器连接',
    description: '生产线主控制器TCP连接',
    connectionType: 'TCP',
    protocol: 'Modbus TCP',
    status: 'connected',
    enabled: true,
    autoReconnect: true,
    uptime: 259200, // 3 days
    bytesTransferred: 1024 * 1024 * 128, // 128MB
    errorCount: 5,
    activeConnections: 3,
    config: {
      host: '',
      port: 502,
      timeout: 5000,
      unitId: 1
    },
    security: {
      enableTls: false,
      verifyCertificate: true,
      username: 'admin'
    },
    pool: {
      maxConnections: 10,
      minConnections: 2,
      connectionTimeout: 5000,
      idleTimeout: 30000
    },
    createdAt: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000), // 7 days ago
    updatedAt: new Date(Date.now() - 2 * 60 * 60 * 1000) // 2 hours ago
  }
  
  generateMockLogs()
  generateMockEvents()
  generateMockChartData()
}

const generateMockLogs = () => {
  const levels = ['info', 'warn', 'error', 'debug'] as const
  const sources = ['connection', 'protocol', 'security', 'pool']
  const messages = [
    '连接建立成功',
    '数据传输正常',
    '心跳检测通过',
    '连接超时重试',
    '安全认证失败',
    '连接池状态更新',
    '配置加载完成',
    '协议解析错误'
  ]
  
  logs.value = Array.from({ length: 50 }, (_, i) => ({
    id: (i + 1).toString(),
    timestamp: new Date(Date.now() - i * 60000), // Every minute
    level: levels[Math.floor(Math.random() * levels.length)],
    message: messages[Math.floor(Math.random() * messages.length)],
    source: sources[Math.floor(Math.random() * sources.length)]
  }))
}

const generateMockEvents = () => {
  const events: Event[] = [
    {
      id: '1',
      timestamp: new Date(Date.now() - 10 * 60000),
      type: 'success',
      title: '连接成功',
      description: '与设备 :502 建立连接成功'
    },
    {
      id: '2',
      timestamp: new Date(Date.now() - 30 * 60000),
      type: 'warning',
      title: '连接重试',
      description: '检测到网络中断，正在重试连接'
    },
    {
      id: '3',
      timestamp: new Date(Date.now() - 2 * 60 * 60000),
      type: 'info',
      title: '配置更新',
      description: '连接配置已更新，重新加载连接参数'
    },
    {
      id: '4',
      timestamp: new Date(Date.now() - 6 * 60 * 60000),
      type: 'error',
      title: '认证失败',
      description: '安全认证失败，请检查用户名和密码'
    },
    {
      id: '5',
      timestamp: new Date(Date.now() - 24 * 60 * 60000),
      type: 'success',
      title: '连接初始化',
      description: '连接初始化完成，开始数据通信'
    }
  ]
  
  recentEvents.value = events
}

const generateMockChartData = () => {
  const now = Date.now()
  const points = 24
  const interval = 60 * 60 * 1000 // 1 hour
  
  // 延迟数据
  latencyData.value = Array.from({ length: points }, (_, i) => {
    const time = now - (points - 1 - i) * interval
    return {
      name: new Date(time).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
      value: Math.floor(Math.random() * 50) + 20, // 20-70ms
      time: new Date(time).toISOString()
    }
  })
  
  // 吞吐率数据
  throughputData.value = Array.from({ length: points }, (_, i) => {
    const time = now - (points - 1 - i) * interval
    return {
      name: new Date(time).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
      value: Math.floor(Math.random() * 200) + 50, // 50-250 KB/s
      time: new Date(time).toISOString()
    }
  })
  
  // 连接数数据
  connectionCountData.value = Array.from({ length: points }, (_, i) => {
    const time = now - (points - 1 - i) * interval
    return {
      name: new Date(time).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
      value: Math.floor(Math.random() * 5) + 2, // 2-7 connections
      time: new Date(time).toISOString()
    }
  })
  
  // 错误率数据
  errorRateData.value = Array.from({ length: points }, (_, i) => {
    const time = now - (points - 1 - i) * interval
    return {
      name: new Date(time).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
      value: Math.random() * 5, // 0-5%
      time: new Date(time).toISOString()
    }
  })
}

// 定时刷新
let refreshTimer: NodeJS.Timeout

const startAutoRefresh = () => {
  refreshTimer = setInterval(() => {
    // 更新实时数据
    if (connectionData.value.status === 'connected') {
      connectionData.value.uptime += 30
      connectionData.value.bytesTransferred += Math.floor(Math.random() * 1024 * 10) // Random bytes
    }
    
    // 更新图表数据
    if (Math.random() > 0.7) {
      generateMockChartData()
    }
  }, 30000) // Every 30 seconds
}

// 生命周期
onMounted(() => {
  initializeData()
  startAutoRefresh()
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<style scoped lang="scss">
.connectors-detail-page {
  padding: 20px;
  max-width: 1600px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding: 20px 0;
  border-bottom: 1px solid var(--el-border-color-light);
  
  .header-left {
    display: flex;
    align-items: center;
    gap: 20px;
    
    .header-title {
      display: flex;
      align-items: center;
      gap: 16px;
      
      .status-icon {
        &.connected {
          color: var(--el-color-success);
        }
        &.connecting {
          color: var(--el-color-warning);
          animation: pulse 1.5s ease-in-out infinite alternate;
        }
        &.disconnected {
          color: var(--el-color-info);
        }
        &.error {
          color: var(--el-color-danger);
        }
      }
      
      .title-info {
        h1 {
          margin: 0 0 4px 0;
          font-size: 28px;
          color: var(--el-text-color-primary);
        }
        
        .connection-type {
          margin: 0;
          color: var(--el-text-color-secondary);
          font-size: 14px;
        }
      }
    }
  }
  
  .header-actions {
    display: flex;
    gap: 12px;
  }
}

.status-overview {
  margin-bottom: 24px;
  
  .status-card {
    background: white;
    border-radius: 12px;
    padding: 20px;
    border: 1px solid var(--el-border-color-light);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.02);
    
    .card-header {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 16px;
      color: var(--el-text-color-regular);
      font-size: 14px;
      font-weight: 500;
      
      .el-icon {
        font-size: 16px;
        color: var(--el-color-primary);
      }
    }
    
    .card-content {
      .metric-value {
        font-size: 24px;
        font-weight: 600;
        color: var(--el-text-color-primary);
        margin-bottom: 4px;
        
        &.error {
          color: var(--el-color-danger);
        }
      }
      
      .metric-label {
        font-size: 12px;
        color: var(--el-text-color-secondary);
      }
    }
  }
}

.main-content {
  .content-section {
    background: white;
    border-radius: 12px;
    padding: 24px;
    margin-bottom: 20px;
    border: 1px solid var(--el-border-color-light);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.02);
    
    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;
      
      h3, h4 {
        margin: 0;
        font-size: 18px;
        color: var(--el-text-color-primary);
        display: flex;
        align-items: center;
        gap: 8px;
        
        .el-icon {
          color: var(--el-color-primary);
        }
      }
      
      h4 {
        font-size: 16px;
      }
      
      .header-actions {
        display: flex;
        align-items: center;
        gap: 12px;
      }
    }
  }
}

.charts-container {
  .chart-card {
    background: var(--el-fill-color-extra-light);
    border-radius: 8px;
    padding: 16px;
    
    h4 {
      margin: 0 0 12px 0;
      font-size: 14px;
      color: var(--el-text-color-regular);
      font-weight: 500;
    }
  }
}

.logs-container {
  .log-time {
    font-family: 'Courier New', monospace;
    font-size: 12px;
  }
  
  .log-message {
    font-size: 13px;
  }
}

.info-container {
  :deep(.el-descriptions__label) {
    font-weight: 500;
  }
}

.config-summary {
  .config-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--el-border-color-lighter);
    
    &:last-child {
      border-bottom: none;
    }
    
    .config-label {
      font-weight: 500;
      color: var(--el-text-color-regular);
    }
    
    .config-value {
      color: var(--el-text-color-primary);
      font-family: 'Courier New', monospace;
      font-size: 13px;
    }
  }
}

.pool-stats {
  margin-bottom: 16px;
  
  .pool-info {
    display: flex;
    align-items: baseline;
    gap: 4px;
    margin-top: 8px;
    
    .pool-current {
      font-size: 20px;
      font-weight: 600;
      color: var(--el-color-primary);
    }
    
    .pool-separator {
      font-size: 16px;
      color: var(--el-text-color-secondary);
    }
    
    .pool-max {
      font-size: 16px;
      color: var(--el-text-color-secondary);
    }
    
    .pool-label {
      margin-left: 8px;
      font-size: 14px;
      color: var(--el-text-color-regular);
    }
  }
}

.events-timeline {
  .event-content {
    .event-title {
      font-weight: 500;
      color: var(--el-text-color-primary);
      margin-bottom: 4px;
    }
    
    .event-description {
      font-size: 13px;
      color: var(--el-text-color-secondary);
      line-height: 1.4;
    }
  }
}

.config-json, .config-yaml {
  pre {
    background: var(--el-fill-color-light);
    padding: 16px;
    border-radius: 6px;
    font-size: 12px;
    max-height: 400px;
    overflow-y: auto;
    font-family: 'Courier New', monospace;
    
    code {
      color: var(--el-text-color-primary);
    }
  }
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(255, 255, 255, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

@keyframes pulse {
  from { opacity: 1; }
  to { opacity: 0.4; }
}

// 响应式设计
@media (max-width: 1200px) {
  .main-content {
    .el-row {
      flex-direction: column;
      
      .el-col {
        width: 100% !important;
        max-width: none !important;
      }
    }
  }
  
  .status-overview {
    .el-row {
      flex-direction: column;
      
      .el-col {
        width: 100% !important;
        margin-bottom: 16px;
      }
    }
  }
}

@media (max-width: 768px) {
  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
    
    .header-actions {
      flex-wrap: wrap;
    }
  }
  
  .charts-container {
    .el-row {
      flex-direction: column;
      
      .el-col {
        width: 100% !important;
        margin-bottom: 16px;
      }
    }
  }
}
</style>