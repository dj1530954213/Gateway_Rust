<template>
  <div class="connection-status" :class="containerClass">
    <!-- 连接状态卡片 -->
    <el-card :shadow="cardShadow" :body-style="{ padding: '16px' }">
      <!-- 卡片头部 -->
      <template #header>
        <div class="connection-header">
          <div class="connection-title">
            <el-icon :size="18">
              <component :is="statusIcon" />
            </el-icon>
            <span>{{ title || getConnectionName() }}</span>
          </div>

          <div class="connection-actions">
            <!-- 刷新按钮 -->
            <el-button
              v-if="showRefresh"
              type="text"
              size="small"
              :icon="Refresh"
              :loading="refreshing"
              @click="handleRefresh"
            />

            <!-- 更多操作 -->
            <el-dropdown
              v-if="showActions"
              trigger="click"
              @command="handleAction"
            >
              <el-button type="text" size="small" :icon="MoreFilled" />
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="reconnect" :icon="Connection">
                    重新连接
                  </el-dropdown-item>
                  <el-dropdown-item command="disconnect" :icon="Disconnect">
                    断开连接
                  </el-dropdown-item>
                  <el-dropdown-item command="details" :icon="InfoFilled">
                    查看详情
                  </el-dropdown-item>
                  <el-dropdown-item command="history" :icon="Clock">
                    连接历史
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
      </template>

      <!-- 连接状态内容 -->
      <div class="connection-content">
        <!-- 主要状态显示 -->
        <div class="status-main">
          <StatusTag
            :status="connection.status"
            :mappings="statusMappings"
            size="large"
            :show-badge="showStatusBadge"
            :badge-value="badgeValue"
          />

          <div class="status-info">
            <div class="status-text">{{ getStatusText() }}</div>
            <div v-if="connection.lastUpdate" class="status-time">
              最后更新: {{ formatTime(connection.lastUpdate) }}
            </div>
          </div>
        </div>

        <!-- 连接详细信息 -->
        <div v-if="showDetails" class="connection-details">
          <el-row :gutter="16">
            <el-col :span="12">
              <div class="detail-item">
                <label>协议类型:</label>
                <span>{{ connection.protocol || '未知' }}</span>
              </div>
            </el-col>
            <el-col :span="12">
              <div class="detail-item">
                <label>目标地址:</label>
                <span>{{ getTargetAddress() }}</span>
              </div>
            </el-col>
          </el-row>

          <el-row :gutter="16">
            <el-col :span="12">
              <div class="detail-item">
                <label>连接时间:</label>
                <span>{{ formatDuration(connection.connectedDuration) }}</span>
              </div>
            </el-col>
            <el-col :span="12">
              <div class="detail-item">
                <label>重连次数:</label>
                <span>{{ connection.retryCount || 0 }}</span>
              </div>
            </el-col>
          </el-row>
        </div>

        <!-- 连接质量指标 -->
        <div
          v-if="showMetrics && connection.metrics"
          class="connection-metrics"
        >
          <el-divider content-position="left">连接质量</el-divider>

          <el-row :gutter="16">
            <el-col :span="8">
              <div class="metric-item">
                <div class="metric-label">延迟</div>
                <div
                  class="metric-value"
                  :class="getLatencyClass(connection.metrics.latency)"
                >
                  {{ connection.metrics.latency }}ms
                </div>
              </div>
            </el-col>

            <el-col :span="8">
              <div class="metric-item">
                <div class="metric-label">丢包率</div>
                <div
                  class="metric-value"
                  :class="getPacketLossClass(connection.metrics.packetLoss)"
                >
                  {{ connection.metrics.packetLoss }}%
                </div>
              </div>
            </el-col>

            <el-col :span="8">
              <div class="metric-item">
                <div class="metric-label">吞吐量</div>
                <div class="metric-value">
                  {{ formatThroughput(connection.metrics.throughput) }}
                </div>
              </div>
            </el-col>
          </el-row>
        </div>

        <!-- 错误信息 -->
        <div v-if="connection.error && showError" class="connection-error">
          <el-alert
            :title="connection.error.message || '连接错误'"
            type="error"
            :description="connection.error.details"
            :show-icon="true"
            :closable="false"
          />
        </div>

        <!-- 操作按钮 -->
        <div v-if="showActionButtons" class="connection-buttons">
          <el-button
            v-if="canReconnect"
            type="primary"
            size="small"
            :loading="connecting"
            @click="handleReconnect"
          >
            重新连接
          </el-button>

          <el-button
            v-if="canDisconnect"
            type="danger"
            size="small"
            :loading="disconnecting"
            @click="handleDisconnect"
          >
            断开连接
          </el-button>

          <el-button
            v-if="showTestButton"
            type="info"
            size="small"
            :loading="testing"
            @click="handleTest"
          >
            测试连接
          </el-button>
        </div>
      </div>
    </el-card>

    <!-- 连接历史对话框 -->
    <el-dialog
      v-model="historyVisible"
      title="连接历史"
      width="600px"
      :before-close="handleHistoryClose"
    >
      <div class="connection-history">
        <el-timeline>
          <el-timeline-item
            v-for="(record, index) in connectionHistory"
            :key="index"
            :timestamp="formatTime(record.timestamp)"
            :type="getHistoryItemType(record.type)"
          >
            <div class="history-content">
              <div class="history-title">
                {{ getHistoryTitle(record.type) }}
              </div>
              <div v-if="record.message" class="history-message">
                {{ record.message }}
              </div>
            </div>
          </el-timeline-item>
        </el-timeline>
      </div>

      <template #footer>
        <el-button @click="historyVisible = false">关闭</el-button>
        <el-button type="primary" @click="handleClearHistory">
          清空历史
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import {
  Connection,
  // Disconnect,  // 这个图标不存在，先移除
  Refresh,
  MoreFilled,
  InfoFilled,
  Clock,
  CircleCheck,
  CircleClose,
  Warning,
  Loading,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, onMounted, onUnmounted } from 'vue'

import { StatusTag } from '../base'

export interface ConnectionInfo {
  id: string
  name?: string
  status: 'connected' | 'disconnected' | 'connecting' | 'error' | 'unknown'
  protocol?: string
  host?: string
  port?: number
  lastUpdate?: Date
  connectedDuration?: number
  retryCount?: number
  error?: {
    message: string
    details?: string
    code?: string
  }
  metrics?: {
    latency: number
    packetLoss: number
    throughput: number
  }
}

export interface ConnectionHistoryRecord {
  timestamp: Date
  type: 'connected' | 'disconnected' | 'error' | 'retry'
  message?: string
}

interface Props {
  connection: ConnectionInfo
  title?: string

  // 显示控制
  showDetails?: boolean
  showMetrics?: boolean
  showError?: boolean
  showActionButtons?: boolean
  showRefresh?: boolean
  showActions?: boolean
  showStatusBadge?: boolean
  showTestButton?: boolean

  // 外观配置
  size?: 'small' | 'default' | 'large'
  cardShadow?: 'always' | 'hover' | 'never'

  // 状态配置
  autoRefresh?: boolean
  refreshInterval?: number

  // 自定义样式
  customClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  showDetails: true,
  showMetrics: true,
  showError: true,
  showActionButtons: true,
  showRefresh: true,
  showActions: true,
  showStatusBadge: false,
  showTestButton: false,
  size: 'default',
  cardShadow: 'never',
  autoRefresh: false,
  refreshInterval: 5000,
})

interface Emits {
  refresh: [connection: ConnectionInfo]
  reconnect: [connection: ConnectionInfo]
  disconnect: [connection: ConnectionInfo]
  test: [connection: ConnectionInfo]
  action: [action: string, connection: ConnectionInfo]
}

const emit = defineEmits<Emits>()

// 状态
const refreshing = ref(false)
const connecting = ref(false)
const disconnecting = ref(false)
const testing = ref(false)
const historyVisible = ref(false)

// 连接历史（模拟数据）
const connectionHistory = ref<ConnectionHistoryRecord[]>([
  {
    timestamp: new Date(Date.now() - 3600000),
    type: 'connected',
    message: '成功建立连接',
  },
  {
    timestamp: new Date(Date.now() - 1800000),
    type: 'error',
    message: '连接超时，正在重试',
  },
  {
    timestamp: new Date(Date.now() - 900000),
    type: 'connected',
    message: '重连成功',
  },
])

// 自动刷新定时器
let refreshTimer: NodeJS.Timeout | null = null

// 状态映射
const statusMappings = computed(() => ({
  connected: { type: 'success', text: '已连接', icon: CircleCheck },
  disconnected: { type: 'info', text: '未连接', icon: CircleClose },
  connecting: { type: 'warning', text: '连接中', icon: Loading },
  error: { type: 'danger', text: '连接错误', icon: Warning },
  unknown: { type: 'info', text: '未知状态', icon: CircleClose },
}))

// 计算属性
const containerClass = computed(() => {
  const classes = []

  classes.push(`connection-status--${props.size}`)

  if (props.customClass) {
    classes.push(props.customClass)
  }

  return classes.join(' ')
})

const statusIcon = computed(() => {
  const mapping = statusMappings.value[props.connection.status]
  return mapping?.icon || CircleClose
})

const badgeValue = computed(() => {
  if (props.connection.retryCount && props.connection.retryCount > 0) {
    return props.connection.retryCount
  }
  return null
})

const canReconnect = computed(() => {
  return ['disconnected', 'error'].includes(props.connection.status)
})

const canDisconnect = computed(() => {
  return ['connected', 'connecting'].includes(props.connection.status)
})

// 方法
const getConnectionName = () => {
  return props.connection.name || `连接 ${props.connection.id}`
}

const getStatusText = () => {
  const mapping = statusMappings.value[props.connection.status]
  return mapping?.text || '未知状态'
}

const getTargetAddress = () => {
  if (props.connection.host && props.connection.port) {
    return `${props.connection.host}:${props.connection.port}`
  }
  return props.connection.host || '未配置'
}

const formatTime = (date: Date) => {
  return date.toLocaleString('zh-CN')
}

const formatDuration = (duration?: number) => {
  if (!duration) return '未知'

  const hours = Math.floor(duration / 3600)
  const minutes = Math.floor((duration % 3600) / 60)
  const seconds = duration % 60

  if (hours > 0) {
    return `${hours}时${minutes}分${seconds}秒`
  } else if (minutes > 0) {
    return `${minutes}分${seconds}秒`
  } else {
    return `${seconds}秒`
  }
}

const formatThroughput = (throughput: number) => {
  if (throughput < 1024) return `${throughput} B/s`
  if (throughput < 1024 * 1024) return `${(throughput / 1024).toFixed(1)} KB/s`
  return `${(throughput / (1024 * 1024)).toFixed(1)} MB/s`
}

const getLatencyClass = (latency: number) => {
  if (latency < 50) return 'metric-good'
  if (latency < 200) return 'metric-warning'
  return 'metric-error'
}

const getPacketLossClass = (packetLoss: number) => {
  if (packetLoss < 1) return 'metric-good'
  if (packetLoss < 5) return 'metric-warning'
  return 'metric-error'
}

const getHistoryItemType = (type: string) => {
  const typeMap = {
    connected: 'success',
    disconnected: 'info',
    error: 'danger',
    retry: 'warning',
  }
  return typeMap[type] || 'info'
}

const getHistoryTitle = (type: string) => {
  const titleMap = {
    connected: '连接成功',
    disconnected: '连接断开',
    error: '连接错误',
    retry: '重试连接',
  }
  return titleMap[type] || '未知事件'
}

// 事件处理
const handleRefresh = async () => {
  refreshing.value = true
  try {
    emit('refresh', props.connection)
  } finally {
    setTimeout(() => {
      refreshing.value = false
    }, 1000)
  }
}

const handleReconnect = async () => {
  connecting.value = true
  try {
    emit('reconnect', props.connection)
    ElMessage.success('正在重新连接...')
  } finally {
    setTimeout(() => {
      connecting.value = false
    }, 2000)
  }
}

const handleDisconnect = async () => {
  disconnecting.value = true
  try {
    emit('disconnect', props.connection)
    ElMessage.info('正在断开连接...')
  } finally {
    setTimeout(() => {
      disconnecting.value = false
    }, 1000)
  }
}

const handleTest = async () => {
  testing.value = true
  try {
    emit('test', props.connection)
    ElMessage.info('正在测试连接...')
  } finally {
    setTimeout(() => {
      testing.value = false
    }, 2000)
  }
}

const handleAction = (command: string) => {
  switch (command) {
    case 'reconnect':
      handleReconnect()
      break
    case 'disconnect':
      handleDisconnect()
      break
    case 'details':
      emit('action', 'details', props.connection)
      break
    case 'history':
      historyVisible.value = true
      break
    default:
      emit('action', command, props.connection)
  }
}

const handleHistoryClose = () => {
  historyVisible.value = false
}

const handleClearHistory = () => {
  connectionHistory.value = []
  ElMessage.success('连接历史已清空')
}

// 自动刷新
const startAutoRefresh = () => {
  if (props.autoRefresh && props.refreshInterval > 0) {
    refreshTimer = setInterval(() => {
      if (!refreshing.value) {
        handleRefresh()
      }
    }, props.refreshInterval)
  }
}

const stopAutoRefresh = () => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
}

// 生命周期
onMounted(() => {
  startAutoRefresh()
})

onUnmounted(() => {
  stopAutoRefresh()
})
</script>

<style scoped lang="scss">
.connection-status {
  width: 100%;

  &.connection-status--small {
    .status-text {
      font-size: 13px;
    }

    .detail-item label {
      font-size: 12px;
    }
  }

  &.connection-status--large {
    .status-text {
      font-size: 16px;
    }
  }
}

.connection-header {
  display: flex;
  justify-content: space-between;
  align-items: center;

  .connection-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    color: var(--el-text-color-primary);
  }

  .connection-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }
}

.connection-content {
  .status-main {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 16px;

    .status-info {
      flex: 1;

      .status-text {
        font-size: 14px;
        font-weight: 500;
        color: var(--el-text-color-primary);
      }

      .status-time {
        font-size: 12px;
        color: var(--el-text-color-secondary);
        margin-top: 4px;
      }
    }
  }

  .connection-details {
    margin-bottom: 16px;

    .detail-item {
      display: flex;
      align-items: center;
      margin-bottom: 8px;

      label {
        font-size: 13px;
        color: var(--el-text-color-secondary);
        width: 80px;
        flex-shrink: 0;
      }

      span {
        font-size: 13px;
        color: var(--el-text-color-primary);
        word-break: break-all;
      }
    }
  }

  .connection-metrics {
    margin-bottom: 16px;

    .metric-item {
      text-align: center;

      .metric-label {
        font-size: 12px;
        color: var(--el-text-color-secondary);
        margin-bottom: 4px;
      }

      .metric-value {
        font-size: 16px;
        font-weight: 600;

        &.metric-good {
          color: var(--el-color-success);
        }

        &.metric-warning {
          color: var(--el-color-warning);
        }

        &.metric-error {
          color: var(--el-color-danger);
        }
      }
    }
  }

  .connection-error {
    margin-bottom: 16px;
  }

  .connection-buttons {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
}

.connection-history {
  max-height: 400px;
  overflow-y: auto;

  .history-content {
    .history-title {
      font-size: 14px;
      font-weight: 500;
      color: var(--el-text-color-primary);
      margin-bottom: 4px;
    }

    .history-message {
      font-size: 13px;
      color: var(--el-text-color-secondary);
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .connection-status {
    .connection-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .status-main {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .connection-buttons {
      flex-direction: column;
      width: 100%;

      .el-button {
        width: 100%;
      }
    }
  }
}
</style>
