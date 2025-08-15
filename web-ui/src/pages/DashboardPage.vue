<template>
  <div class="dashboard-page">
    <!-- WebSocket连接状态指示器 -->
    <div
      class="connection-status"
      :class="{ connected: isConnected, disconnected: !isConnected }"
    >
      <el-icon v-if="isConnected"><CircleCheck /></el-icon>
      <el-icon v-else><CircleClose /></el-icon>
      <span>{{ isConnected ? '实时连接正常' : '实时连接断开' }}</span>
    </div>

    <!-- 统计卡片行 -->
    <div class="stats-row">
      <el-card class="stat-card devices">
        <div class="stat-content">
          <div class="stat-value">{{ devicesStore.state.total }}</div>
          <div class="stat-label">总设备数</div>
          <div class="stat-change">
            <span class="change-text" :class="deviceChange.type">
              {{ deviceChange.text }}
            </span>
          </div>
        </div>
        <el-icon class="stat-icon"><Monitor /></el-icon>
      </el-card>

      <el-card class="stat-card drivers">
        <div class="stat-content">
          <div class="stat-value">
            {{ driversStore.state.statusStats.loaded }}
          </div>
          <div class="stat-label">已加载驱动</div>
          <div class="stat-change">
            <span class="change-text success">
              {{ driversStore.state.statusStats.loaded }}/{{
                driversStore.state.total
              }}
            </span>
          </div>
        </div>
        <el-icon class="stat-icon"><Cpu /></el-icon>
      </el-card>

      <el-card class="stat-card tags">
        <div class="stat-content">
          <div class="stat-value">{{ tagsStore.state.total }}</div>
          <div class="stat-label">数据点位</div>
          <div class="stat-change">
            <span class="change-text" :class="tagChange.type">
              {{ tagChange.text }}
            </span>
          </div>
        </div>
        <el-icon class="stat-icon"><SetUp /></el-icon>
      </el-card>

      <el-card class="stat-card alerts">
        <div class="stat-content">
          <div class="stat-value">{{ alertsStore.state.unreadCount }}</div>
          <div class="stat-label">未读告警</div>
          <div class="stat-change">
            <span class="change-text" :class="alertChange.type">
              {{ alertChange.text }}
            </span>
          </div>
        </div>
        <el-icon class="stat-icon"><Warning /></el-icon>
      </el-card>
    </div>

    <!-- 主内容网格 -->
    <div class="content-grid">
      <!-- 系统状态监控 -->
      <el-card class="system-status-card" shadow="never">
        <template #header>
          <div class="card-header">
            <span>系统状态</span>
            <el-button
              type="text"
              size="small"
              :loading="systemLoading"
              @click="refreshSystemStatus"
            >
              刷新
            </el-button>
          </div>
        </template>

        <SystemStatusCards
          :loading="systemLoading"
          @refresh="refreshSystemStatus"
        />
      </el-card>

      <!-- 实时数据图表 -->
      <el-card class="chart-card" shadow="never">
        <template #header>
          <div class="card-header">
            <span>实时数据趋势</span>
            <div class="chart-controls">
              <el-select
                v-model="selectedTimeRange"
                size="small"
                style="width: 120px"
                @change="updateChartData"
              >
                <el-option label="1小时" value="1h" />
                <el-option label="6小时" value="6h" />
                <el-option label="24小时" value="24h" />
              </el-select>
            </div>
          </div>
        </template>

        <RealtimeChart
          :time-range="selectedTimeRange"
          :selected-tags="selectedChartTags"
          :loading="chartLoading"
          @tag-change="handleTagChange"
        />
      </el-card>

      <!-- 设备连接状态 -->
      <el-card class="devices-status-card" shadow="never">
        <template #header>
          <div class="card-header">
            <span>设备连接状态</span>
            <el-button
              type="text"
              size="small"
              @click="$router.push('/devices')"
            >
              查看全部
            </el-button>
          </div>
        </template>

        <DeviceStatusOverview
          :devices="recentDevices"
          :loading="devicesLoading"
          @device-click="handleDeviceClick"
        />
      </el-card>

      <!-- 最新告警 -->
      <el-card class="alerts-card" shadow="never">
        <template #header>
          <div class="card-header">
            <span>最新告警</span>
            <el-button
              type="text"
              size="small"
              @click="$router.push('/alerts')"
            >
              查看全部
            </el-button>
          </div>
        </template>

        <AlertNotificationList
          :alerts="recentAlerts"
          :loading="alertsLoading"
          @alert-action="handleAlertAction"
        />
      </el-card>

      <!-- 系统性能指标 -->
      <el-card class="performance-card" shadow="never">
        <template #header>
          <div class="card-header">
            <span>系统性能</span>
            <el-button
              type="text"
              size="small"
              :loading="performanceLoading"
              @click="refreshPerformance"
            >
              刷新
            </el-button>
          </div>
        </template>

        <SystemPerformanceMetrics
          :metrics="performanceMetrics"
          :loading="performanceLoading"
        />
      </el-card>

      <!-- 快速操作 -->
      <el-card class="quick-actions-card" shadow="never">
        <template #header>
          <span>快速操作</span>
        </template>

        <div class="quick-actions">
          <el-button
            type="primary"
            class="action-btn"
            @click="$router.push('/devices')"
          >
            <el-icon><Plus /></el-icon>
            新增设备
          </el-button>

          <el-button
            type="success"
            class="action-btn"
            @click="$router.push('/drivers')"
          >
            <el-icon><Upload /></el-icon>
            上传驱动
          </el-button>

          <el-button
            type="warning"
            class="action-btn"
            @click="$router.push('/tags')"
          >
            <el-icon><SetUp /></el-icon>
            配置点位
          </el-button>

          <el-button
            type="info"
            class="action-btn"
            @click="exportDashboardData"
          >
            <el-icon><Download /></el-icon>
            导出数据
          </el-button>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * DashboardPage —— 仪表板主页面
 *
 * 📝 Responsibilities:
 *  1. 系统整体状态概览
 *  2. 实时数据可视化
 *  3. 关键指标监控
 *  4. 快速导航入口
 *  5. WebSocket实时更新
 *
 * 📦 Dependencies:
 *  - SystemStatusCards 系统状态组件
 *  - RealtimeChart 实时数据图表
 *  - DeviceStatusOverview 设备状态概览
 *  - AlertNotificationList 告警通知列表
 *  - SystemPerformanceMetrics 性能指标组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  重构为新架构版本
 */

import {
  Monitor,
  Cpu,
  SetUp,
  Warning,
  Plus,
  Upload,
  Download,
  CircleCheck,
  CircleClose,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'

import AlertNotificationList from '@/components/dashboard/AlertNotificationList.vue'
import DeviceStatusOverview from '@/components/dashboard/DeviceStatusOverview.vue'
import RealtimeChart from '@/components/dashboard/RealtimeChart.vue'
import SystemPerformanceMetrics from '@/components/dashboard/SystemPerformanceMetrics.vue'
import SystemStatusCards from '@/components/dashboard/SystemStatusCards.vue'
import { useWebSocket } from '@/composables/useWebSocket'
import {
  useDevicesStore,
  useDriversStore,
  useTagsStore,
  useAlertsStore,
} from '@/stores'

// 组件导入

// ===== 路由器 =====
const router = useRouter()

// ===== Stores =====
const devicesStore = useDevicesStore()
const driversStore = useDriversStore()
const tagsStore = useTagsStore()
const alertsStore = useAlertsStore()

// ===== WebSocket连接 =====
const wsUrl = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080'
const { isConnected, lastMessage, send, subscribe } = useWebSocket(
  `${wsUrl}/ws/dashboard`,
  {
    autoConnect: false, // 开发环境下禁用自动连接
    reconnectInterval: 3000,
    maxReconnectAttempts: 10,
    heartbeatInterval: 30000,
    onOpen: () => {
      ElMessage.success('实时连接已建立')
      // 订阅仪表板相关消息
      send({
        type: 'subscribe',
        channels: [
          'device_status',
          'driver_status',
          'alert_notifications',
          'system_metrics',
        ],
      })
    },
    onClose: () => {
      ElMessage.warning('实时连接已断开')
    },
    onError: error => {
      console.error('WebSocket连接错误:', error)
    },
  }
)

// ===== 响应式数据 =====
const systemLoading = ref(false)
const chartLoading = ref(false)
const devicesLoading = ref(false)
const alertsLoading = ref(false)
const performanceLoading = ref(false)

const selectedTimeRange = ref('1h')
const selectedChartTags = ref<string[]>([])

// 最近设备数据
const recentDevices = ref([])
// 最近告警数据
const recentAlerts = ref([])
// 性能指标数据
const performanceMetrics = ref({
  cpu: {
    usage: 0,
    cores: 4,
    user: 0,
    system: 0,
    loadAverage: 0,
  },
  memory: {
    usage: 0,
    total: 8 * 1024 * 1024 * 1024,
    used: 0,
    available: 0,
    cached: 0,
  },
  disk: {
    usage: 0,
    total: 500 * 1024 * 1024 * 1024,
    used: 0,
    available: 0,
    iops: 0,
  },
  network: {
    interface: 'eth0',
    in: 0,
    out: 0,
    connections: 0,
    latency: 0,
    packetLoss: 0,
    history: {
      in: [],
      out: [],
    },
  },
})

// ===== 计算属性 =====
const deviceChange = computed(() => {
  const enabled = devicesStore.enabledDevices.length
  const total = devicesStore.state.total
  const percentage = total > 0 ? Math.round((enabled / total) * 100) : 0

  return {
    text: `${percentage}% 在线`,
    type:
      percentage >= 80 ? 'success' : percentage >= 60 ? 'warning' : 'danger',
  }
})

const tagChange = computed(() => {
  const enabled = tagsStore.enabledTags.length
  const total = tagsStore.state.total
  const percentage = total > 0 ? Math.round((enabled / total) * 100) : 0

  return {
    text: `${percentage}% 启用`,
    type:
      percentage >= 90 ? 'success' : percentage >= 70 ? 'warning' : 'danger',
  }
})

const alertChange = computed(() => {
  const unread = alertsStore.state.unreadCount
  const total = alertsStore.state.total

  return {
    text: unread > 0 ? `${unread} 待处理` : '无未读',
    type: unread === 0 ? 'success' : unread <= 5 ? 'warning' : 'danger',
  }
})

// ===== 方法 =====

/**
 * 刷新系统状态
 */
async function refreshSystemStatus() {
  systemLoading.value = true
  try {
    // 刷新所有相关Store数据
    await Promise.all([
      devicesStore.fetchDevices({ page: 1, size: 10 }),
      driversStore.fetchDriverStatus(),
      tagsStore.fetchTags({ page: 1, size: 10 }),
    ])
  } catch (error) {
    ElMessage.error('刷新系统状态失败')
  } finally {
    systemLoading.value = false
  }
}

/**
 * 更新图表数据
 */
async function updateChartData() {
  chartLoading.value = true
  try {
    // 这里可以根据时间范围获取相应的历史数据
    await new Promise(resolve => setTimeout(resolve, 1000))
  } catch (error) {
    ElMessage.error('更新图表数据失败')
  } finally {
    chartLoading.value = false
  }
}

/**
 * 处理图表标签变更
 */
function handleTagChange(tags: string[]) {
  selectedChartTags.value = tags
  updateChartData()
}

/**
 * 处理设备点击
 */
function handleDeviceClick(device: any) {
  router.push(`/devices/${device.id}`)
}

/**
 * 处理告警操作
 */
async function handleAlertAction(action: string, alertId: string) {
  try {
    switch (action) {
      case 'acknowledge':
        await alertsStore.acknowledgeAlert(alertId)
        ElMessage.success('告警已确认')
        break
      case 'dismiss':
        await alertsStore.dismissAlert(alertId)
        ElMessage.success('告警已忽略')
        break
    }
    // 刷新告警列表
    await loadRecentAlerts()
  } catch (error) {
    ElMessage.error('操作失败')
  }
}

/**
 * 刷新系统性能指标
 */
async function refreshPerformance() {
  performanceLoading.value = true
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 800))

    // 更新性能指标数据
    performanceMetrics.value.cpu.usage = Math.floor(Math.random() * 100)
    performanceMetrics.value.memory.usage = Math.floor(Math.random() * 100)
    performanceMetrics.value.disk.usage = Math.floor(Math.random() * 100)
    performanceMetrics.value.network.in = Math.floor(Math.random() * 1000000)
    performanceMetrics.value.network.out = Math.floor(Math.random() * 1000000)

    // 更新内存详细信息
    const memTotal = performanceMetrics.value.memory.total
    const memUsage = performanceMetrics.value.memory.usage
    performanceMetrics.value.memory.used = Math.floor(
      (memTotal * memUsage) / 100
    )
    performanceMetrics.value.memory.available =
      memTotal - performanceMetrics.value.memory.used
    performanceMetrics.value.memory.cached = Math.floor(
      Math.random() * performanceMetrics.value.memory.used * 0.3
    )

    // 更新磁盘详细信息
    const diskTotal = performanceMetrics.value.disk.total
    const diskUsage = performanceMetrics.value.disk.usage
    performanceMetrics.value.disk.used = Math.floor(
      (diskTotal * diskUsage) / 100
    )
    performanceMetrics.value.disk.available =
      diskTotal - performanceMetrics.value.disk.used
    performanceMetrics.value.disk.iops = Math.floor(Math.random() * 1000)
  } catch (error) {
    ElMessage.error('刷新性能指标失败')
  } finally {
    performanceLoading.value = false
  }
}

/**
 * 加载最近设备数据
 */
async function loadRecentDevices() {
  devicesLoading.value = true
  try {
    await devicesStore.fetchDevices({ page: 1, size: 5 })
    recentDevices.value = devicesStore.state.devices.slice(0, 5)
  } catch (error) {
    console.error('加载最近设备失败:', error)
  } finally {
    devicesLoading.value = false
  }
}

/**
 * 加载最近告警数据
 */
async function loadRecentAlerts() {
  alertsLoading.value = true
  try {
    await alertsStore.fetchAlerts({ page: 1, size: 5 })
    recentAlerts.value = alertsStore.state.alerts.slice(0, 5)
  } catch (error) {
    console.error('加载最近告警失败:', error)
  } finally {
    alertsLoading.value = false
  }
}

/**
 * 导出仪表板数据
 */
function exportDashboardData() {
  ElMessage.info('数据导出功能开发中...')
}

/**
 * 初始化仪表板数据
 */
async function initializeDashboard() {
  try {
    // 并行加载所有必要数据
    await Promise.all([
      devicesStore.fetchDevices({ page: 1, size: 20 }),
      driversStore.fetchDrivers({ page: 1, size: 20 }),
      tagsStore.fetchTags({ page: 1, size: 20 }),
      alertsStore.fetchAlerts({ page: 1, size: 20 }),
      loadRecentDevices(),
      loadRecentAlerts(),
      refreshPerformance(),
    ])

    // 初始化图表标签
    if (tagsStore.state.tags.length > 0) {
      selectedChartTags.value = tagsStore.state.tags
        .slice(0, 3)
        .map(tag => tag.id)
    }
  } catch (error) {
    console.error('初始化仪表板失败:', error)
    ElMessage.error('仪表板数据加载失败')
  }
}

/**
 * 处理WebSocket消息更新
 */
function handleWebSocketMessage(message: any) {
  try {
    let data
    if (typeof message === 'string') {
      data = JSON.parse(message)
    } else {
      data = message
    }

    switch (data.type) {
      case 'device_status_update':
        // 更新设备状态
        if (data.deviceId && data.status) {
          updateDeviceInList(data.deviceId, { status: data.status })
          ElMessage.info(
            `设备 ${data.deviceName || data.deviceId} 状态更新: ${data.status}`
          )
        }
        break

      case 'driver_status_update':
        // 更新驱动状态
        if (data.driverId && data.status) {
          driversStore.updateDriverStatus?.(data.driverId, data.status)
          ElMessage.info(
            `驱动 ${data.driverName || data.driverId} 状态更新: ${data.status}`
          )
        }
        break

      case 'new_alert':
        // 新告警通知
        if (data.alert) {
          alertsStore.addAlert?.(data.alert)
          recentAlerts.value.unshift(data.alert)
          if (recentAlerts.value.length > 5) {
            recentAlerts.value = recentAlerts.value.slice(0, 5)
          }
          ElMessage.warning(`新告警: ${data.alert.message}`)
        }
        break

      case 'system_metrics':
        // 更新系统性能指标
        if (data.metrics) {
          Object.assign(performanceMetrics.value, data.metrics)
        }
        break

      case 'tag_value_update':
        // 实时数据点位值更新
        if (data.tagId && data.value !== undefined) {
          updateTagValue(data.tagId, data.value, data.timestamp)
        }
        break

      case 'connection_status':
        // WebSocket连接状态更新
        if (data.connected !== undefined) {
          console.log(
            'WebSocket连接状态:',
            data.connected ? '已连接' : '已断开'
          )
        }
        break

      default:
        console.log('未处理的WebSocket消息类型:', data.type, data)
    }
  } catch (error) {
    console.error('处理WebSocket消息失败:', error, message)
  }
}

/**
 * 更新设备列表中的设备信息
 */
function updateDeviceInList(deviceId: string, updates: any) {
  const deviceIndex = recentDevices.value.findIndex(d => d.id === deviceId)
  if (deviceIndex !== -1) {
    Object.assign(recentDevices.value[deviceIndex], updates)
  }
}

/**
 * 更新标签值（用于实时图表）
 */
function updateTagValue(tagId: string, value: number, timestamp?: string) {
  // 这里可以将更新传递给图表组件
  console.log(`标签 ${tagId} 值更新:`, value, timestamp)
}

// ===== 生命周期 =====
onMounted(async () => {
  await initializeDashboard()

  // 设置定期刷新
  const refreshInterval = setInterval(() => {
    refreshSystemStatus()
    refreshPerformance()
  }, 30000) // 30秒刷新一次

  // 清理定时器
  onUnmounted(() => {
    clearInterval(refreshInterval)
  })
})

// ===== WebSocket消息监听 =====
watch(
  () => lastMessage.value,
  message => {
    if (message) {
      handleWebSocketMessage(message)
    }
  },
  { immediate: false }
)

// ===== 连接状态监听 =====
watch(isConnected, connected => {
  if (connected) {
    console.log('WebSocket已连接，开始接收实时数据')
  } else {
    console.log('WebSocket连接已断开')
  }
})
</script>

<style scoped lang="scss">
.dashboard-page {
  padding: 24px;
  background-color: #f5f5f5;
  min-height: 100vh;

  .connection-status {
    position: fixed;
    top: 20px;
    right: 20px;
    z-index: 1000;
    padding: 8px 16px;
    border-radius: 20px;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 500;
    transition: all 0.3s;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);

    &.connected {
      background: #f0f9ec;
      color: #67c23a;
      border: 1px solid #c2e7b0;

      .el-icon {
        color: #67c23a;
      }
    }

    &.disconnected {
      background: #fef0f0;
      color: #f56c6c;
      border: 1px solid #fbc4c4;
      animation: pulse 2s infinite;

      .el-icon {
        color: #f56c6c;
      }
    }

    .el-icon {
      font-size: 14px;
    }
  }
}

@keyframes pulse {
  0% {
    transform: scale(1);
    opacity: 1;
  }

  50% {
    transform: scale(1.05);
    opacity: 0.8;
  }

  100% {
    transform: scale(1);
    opacity: 1;
  }
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 16px;
  margin-bottom: 24px;

  .stat-card {
    border: none;

    .el-card__body {
      padding: 20px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .stat-content {
      .stat-value {
        font-size: 32px;
        font-weight: 700;
        color: #303133;
        line-height: 1;
        margin-bottom: 8px;
      }

      .stat-label {
        font-size: 14px;
        color: #909399;
        margin-bottom: 6px;
      }

      .stat-change {
        .change-text {
          font-size: 12px;
          font-weight: 500;

          &.success {
            color: #67c23a;
          }
          &.warning {
            color: #e6a23c;
          }
          &.danger {
            color: #f56c6c;
          }
        }
      }
    }

    .stat-icon {
      font-size: 36px;
      opacity: 0.8;
    }

    &.devices .stat-icon {
      color: #409eff;
    }
    &.drivers .stat-icon {
      color: #67c23a;
    }
    &.tags .stat-icon {
      color: #e6a23c;
    }
    &.alerts .stat-icon {
      color: #f56c6c;
    }
  }
}

.content-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 20px;

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-weight: 600;
    color: #303133;
  }

  .system-status-card {
    grid-column: span 2;
  }

  .chart-card {
    grid-column: span 2;

    .chart-controls {
      display: flex;
      gap: 12px;
    }
  }

  .devices-status-card,
  .alerts-card {
    min-height: 300px;
  }

  .performance-card {
    min-height: 250px;
  }

  .quick-actions-card {
    .quick-actions {
      display: flex;
      flex-direction: column;
      gap: 12px;

      .action-btn {
        justify-content: flex-start;
        padding: 12px 16px;

        .el-icon {
          margin-right: 8px;
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .content-grid {
    grid-template-columns: 1fr;

    .system-status-card,
    .chart-card {
      grid-column: span 1;
    }
  }
}

@media (max-width: 768px) {
  .dashboard-page {
    padding: 16px;
  }

  .stats-row {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
    margin-bottom: 16px;

    .stat-card {
      .el-card__body {
        padding: 16px;
      }

      .stat-content .stat-value {
        font-size: 24px;
      }

      .stat-icon {
        font-size: 28px;
      }
    }
  }

  .content-grid {
    gap: 12px;
  }
}
</style>
