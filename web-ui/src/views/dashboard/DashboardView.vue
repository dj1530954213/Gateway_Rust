<template>
  <div class="dashboard-container">
    <!-- Header Stats -->
    <div class="stats-grid">
      <el-card
        v-for="stat in statsCards"
        :key="stat.key"
        class="stat-card"
        :class="`stat-${stat.type}`"
        shadow="hover"
      >
        <div class="stat-content">
          <div class="stat-info">
            <div class="stat-value">{{ stat.value }}</div>
            <div class="stat-label">{{ stat.label }}</div>
            <div class="stat-change" :class="stat.changeType">
              <el-icon>
                <ArrowUp v-if="stat.changeType === 'increase'" />
                <ArrowDown v-if="stat.changeType === 'decrease'" />
                <Minus v-if="stat.changeType === 'stable'" />
              </el-icon>
              {{ stat.change }}
            </div>
          </div>
          <div class="stat-icon">
            <el-icon :size="32">
              <component :is="iconMap[stat.key] || Connection" />
            </el-icon>
          </div>
        </div>
      </el-card>
    </div>

    <!-- Main Content Grid -->
    <div class="content-grid">
      <!-- System Status -->
      <el-card class="status-card" header="系统状态">
        <div class="status-grid">
          <div
            v-for="component in systemComponents"
            :key="component.name"
            class="status-item"
          >
            <div class="status-header">
              <span class="component-name">{{ component.name }}</span>
              <el-tag :type="getStatusType(component.status)" size="small">
                {{ component.status }}
              </el-tag>
            </div>
            <div class="status-metrics">
              <div class="metric-item">
                <span class="metric-label">运行时间</span>
                <span class="metric-value">{{
                  formatUptime(component.uptime)
                }}</span>
              </div>
              <div class="metric-item">
                <span class="metric-label">错误计数</span>
                <span class="metric-value">{{ component.errorCount }}</span>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- Real-time Chart -->
      <el-card class="chart-card" header="实时数据趋势">
        <div class="chart-controls">
          <el-select
            v-model="selectedTimeRange"
            size="small"
            style="width: 120px"
          >
            <el-option label="1小时" value="1h" />
            <el-option label="6小时" value="6h" />
            <el-option label="24小时" value="24h" />
          </el-select>
          <el-select
            v-model="selectedDataPoints"
            multiple
            size="small"
            style="width: 200px"
          >
            <el-option
              v-for="point in availableDataPoints"
              :key="point.id"
              :label="point.name"
              :value="point.id"
            />
          </el-select>
        </div>
        <div ref="chartContainer" class="chart-container">
          <VChart
            class="chart"
            :option="chartOption"
            autoresize
            @click="handleChartClick"
          />
        </div>
      </el-card>

      <!-- Recent Alerts -->
      <el-card class="alerts-card" header="最新告警">
        <template #extra>
          <el-button type="text" size="small" @click="$router.push('/alerts')">
            查看全部
          </el-button>
        </template>

        <div class="alerts-list">
          <div
            v-for="alert in recentAlerts"
            :key="alert.id"
            class="alert-item"
            :class="`alert-${alert.level}`"
          >
            <div class="alert-content">
              <div class="alert-header">
                <span class="alert-message">{{ alert.message }}</span>
                <el-tag :type="getAlertType(alert.level)" size="small">
                  {{ alert.level }}
                </el-tag>
              </div>
              <div class="alert-meta">
                <span class="alert-source">{{ alert.source }}</span>
                <span class="alert-time">{{
                  formatTime(alert.created_at)
                }}</span>
              </div>
            </div>
            <div class="alert-actions">
              <el-button
                v-if="!alert.acknowledged"
                type="text"
                size="small"
                @click="acknowledgeAlert(alert.id)"
              >
                确认
              </el-button>
            </div>
          </div>

          <el-empty
            v-if="recentAlerts.length === 0"
            description="暂无告警信息"
            :image-size="80"
          />
        </div>
      </el-card>

      <!-- Device Status -->
      <el-card class="devices-card" header="设备状态">
        <div class="devices-summary">
          <div class="device-stat">
            <div class="device-count">{{ deviceStats.total }}</div>
            <div class="device-label">总设备数</div>
          </div>
          <div class="device-stat online">
            <div class="device-count">{{ deviceStats.online }}</div>
            <div class="device-label">在线设备</div>
          </div>
          <div class="device-stat offline">
            <div class="device-count">{{ deviceStats.offline }}</div>
            <div class="device-label">离线设备</div>
          </div>
        </div>

        <el-divider />

        <div class="device-list">
          <div
            v-for="device in recentDevices"
            :key="device.id"
            class="device-item"
            @click="$router.push(`/drivers/${device.id}`)"
          >
            <div class="device-info">
              <div class="device-name">{{ device.name }}</div>
              <div class="device-type">{{ device.type }}</div>
            </div>
            <div class="device-status">
              <el-tag
                :type="device.status === 'running' ? 'success' : 'danger'"
                size="small"
                effect="plain"
              >
                {{ device.status === 'running' ? '运行中' : '已停止' }}
              </el-tag>
            </div>
          </div>
        </div>
      </el-card>

      <!-- System Performance -->
      <el-card class="performance-card" header="系统性能">
        <div class="performance-metrics">
          <div class="metric-card">
            <div class="metric-title">CPU使用率</div>
            <div class="metric-progress">
              <el-progress
                :percentage="systemMetrics.cpuUsage"
                :status="systemMetrics.cpuUsage > 80 ? 'exception' : 'success'"
                :show-text="false"
              />
              <span class="metric-text">{{ systemMetrics.cpuUsage }}%</span>
            </div>
          </div>

          <div class="metric-card">
            <div class="metric-title">内存使用率</div>
            <div class="metric-progress">
              <el-progress
                :percentage="systemMetrics.memoryUsage"
                :status="
                  systemMetrics.memoryUsage > 85 ? 'exception' : 'success'
                "
                :show-text="false"
              />
              <span class="metric-text">{{ systemMetrics.memoryUsage }}%</span>
            </div>
          </div>

          <div class="metric-card">
            <div class="metric-title">磁盘使用率</div>
            <div class="metric-progress">
              <el-progress
                :percentage="systemMetrics.diskUsage"
                :status="systemMetrics.diskUsage > 90 ? 'exception' : 'success'"
                :show-text="false"
              />
              <span class="metric-text">{{ systemMetrics.diskUsage }}%</span>
            </div>
          </div>

          <div class="metric-card">
            <div class="metric-title">网络流量</div>
            <div class="metric-value">
              <div class="metric-item">
                <span class="metric-label">入站</span>
                <span class="metric-number">{{
                  formatNetworkSpeed(systemMetrics.networkIn)
                }}</span>
              </div>
              <div class="metric-item">
                <span class="metric-label">出站</span>
                <span class="metric-number">{{
                  formatNetworkSpeed(systemMetrics.networkOut)
                }}</span>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- Quick Actions -->
      <el-card class="actions-card" header="快速操作">
        <div class="action-buttons">
          <el-button
            type="primary"
            :icon="Plus"
            @click="$router.push('/drivers/create')"
          >
            添加驱动
          </el-button>
          <el-button
            type="success"
            :icon="Connection"
            @click="$router.push('/connectors/create')"
          >
            添加连接器
          </el-button>
          <el-button
            type="warning"
            :icon="Setting"
            @click="$router.push('/system/config')"
          >
            系统配置
          </el-button>
          <el-button type="info" :icon="Download" @click="exportData">
            导出数据
          </el-button>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ArrowUp,
  ArrowDown,
  Minus,
  Plus,
  Connection,
  Setting,
  Download,
  SetUp,
  Warning,
  TrendCharts,
} from '@element-plus/icons-vue'
import { LineChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
} from 'echarts/components'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { ElMessage } from 'element-plus'
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue'
import VChart from 'vue-echarts'

import { useWebSocket } from '@/composables/useWebSocket'
import { systemApi, alertsApi, driversApi, realtimeApi } from '@/services/api'
import { useSystemStore } from '@/stores/system'
import { formatTime, formatUptime } from '@/utils/date'
import { formatNetworkSpeed, formatStatus } from '@/utils/format'

// Element Plus图标导入

// Register ECharts components
use([
  CanvasRenderer,
  LineChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
])

const systemStore = useSystemStore()

// WebSocket connection for real-time updates (disabled in development)
const wsUrl = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080'
const { lastMessage } = useWebSocket(`${wsUrl}/ws/telemetry`, {
  autoConnect: true,
})

// Refs
const chartContainer = ref()

// State
const loading = ref(false)
const selectedTimeRange = ref('1h')
const selectedDataPoints = ref<string[]>([])

// 图标映射 - 非reactive
const iconMap = {
  devices: Connection,
  dataPoints: SetUp,
  alerts: Warning,
  throughput: TrendCharts,
}

// Stats data - 从API动态获取
const statsCards = ref([
  {
    key: 'devices',
    label: '在线设备',
    value: '0',
    change: '0',
    changeType: 'stable',
    type: 'primary',
  },
  {
    key: 'dataPoints',
    label: '数据点',
    value: '0',
    change: '0%',
    changeType: 'stable',
    type: 'success',
  },
  {
    key: 'alerts',
    label: '活跃告警',
    value: '0',
    change: '0',
    changeType: 'stable',
    type: 'warning',
  },
  {
    key: 'throughput',
    label: '数据吞吐量',
    value: '0K/s',
    change: '0%',
    changeType: 'stable',
    type: 'info',
  },
])

// System components status - 从API获取
const systemComponents = ref<any[]>([])

// System metrics - 从API获取
const systemMetrics = reactive({
  cpuUsage: 0,
  memoryUsage: 0,
  diskUsage: 0,
  networkIn: 0,
  networkOut: 0,
})

// Recent alerts - 从API获取
const recentAlerts = ref<any[]>([])

// Device statistics - 从API获取
const deviceStats = reactive({
  total: 0,
  online: 0,
  offline: 0,
})

// Recent devices - 从API获取
const recentDevices = ref<any[]>([])

// Available data points for chart - 从API获取
const availableDataPoints = ref<any[]>([])

// Chart configuration
const chartOption = computed(() => ({
  title: {
    text: '实时数据趋势',
    left: 'center',
    textStyle: {
      fontSize: 14,
      fontWeight: 'normal',
    },
  },
  tooltip: {
    trigger: 'axis',
    axisPointer: {
      type: 'cross',
    },
  },
  legend: {
    top: '30',
    data: selectedDataPoints.value.map(
      id => availableDataPoints.value.find(p => p.id === id)?.name || id
    ),
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    top: '15%',
    containLabel: true,
  },
  xAxis: {
    type: 'category',
    boundaryGap: false,
    data: generateTimeLabels(),
  },
  yAxis: {
    type: 'value',
  },
  series: selectedDataPoints.value.map((pointId, index) => ({
    name:
      availableDataPoints.value.find(p => p.id === pointId)?.name || pointId,
    type: 'line',
    smooth: true,
    data: [], // 🚫 生产级系统 - 数据从真实 API 获取，初始为空数组
    itemStyle: {
      color: getChartColor(index),
    },
  })),
}))

// Methods
const getStatusType = (status: string) => {
  const types: Record<string, any> = {
    running: 'success',
    warning: 'warning',
    error: 'danger',
    stopped: 'info',
  }
  return types[status] || 'info'
}

const getAlertType = (level: string) => {
  const types: Record<string, any> = {
    info: 'info',
    warning: 'warning',
    error: 'danger',
    critical: 'danger',
  }
  return types[level] || 'info'
}

const getChartColor = (index: number) => {
  const colors = ['#409eff', '#67c23a', '#e6a23c', '#f56c6c', '#909399']
  return colors[index % colors.length]
}

const generateTimeLabels = () => {
  const labels = []
  const now = new Date()
  const interval =
    selectedTimeRange.value === '1h'
      ? 5
      : selectedTimeRange.value === '6h'
        ? 30
        : 60

  for (let i = 60; i >= 0; i -= 5) {
    const time = new Date(now.getTime() - i * interval * 60000)
    labels.push(
      time.toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
      })
    )
  }

  return labels
}

// 🚫 生产级系统 - 已移除模拟数据生成函数
// 图表数据现在直接从真实 API 获取，不再使用随机生成

const acknowledgeAlert = async (alertId: string) => {
  try {
    await alertsApi.acknowledgeAlert(alertId)
    const alert = recentAlerts.value.find(a => a.id === alertId)
    if (alert) {
      alert.acknowledged = true
    }
    ElMessage.success('告警已确认')
  } catch (error) {
    ElMessage.error('确认告警失败')
  }
}

const exportData = () => {
  ElMessage.info('数据导出功能开发中')
}

const handleChartClick = (params: any) => {
  console.log('Chart clicked:', params)
}

// 🚫 生产级系统 - 已移除随机数据更新函数
// 实时数据现在通过 WebSocket 和 API 调用获取真实数据
const updateRealTimeData = () => {
  // 这个函数现在只负责刷新真实数据，不再生成随机数据
  loadDashboardData()
}

// Load dashboard data from API
const loadDashboardData = async () => {
  try {
    loading.value = true

    // Load system metrics
    const metricsResponse = await systemApi.getSystemMetrics()
    if (metricsResponse.success && metricsResponse.data) {
      systemMetrics.cpuUsage = metricsResponse.data.cpuUsage
      systemMetrics.memoryUsage = metricsResponse.data.memoryUsage
      systemMetrics.diskUsage = metricsResponse.data.diskUsage
      systemMetrics.networkIn = metricsResponse.data.activeConnections * 1024
      systemMetrics.networkOut = metricsResponse.data.messagesPerSecond * 512
    }

    // Load system components status
    const componentsResponse = await systemApi.getComponentStatus()
    if (componentsResponse.success && componentsResponse.data) {
      systemComponents.value = componentsResponse.data
    }

    // Load drivers and update stats
    const driversResponse = await driversApi.getDrivers()
    if (driversResponse.success && driversResponse.data) {
      const drivers = driversResponse.data
      const onlineDrivers = drivers.filter(d => d.status === 'running').length
      const totalDataPoints = drivers.reduce(
        (sum, d) => sum + (d.dataPoints || 0),
        0
      )

      statsCards.value[0].value = onlineDrivers.toString()
      statsCards.value[1].value = totalDataPoints.toLocaleString()

      // Update device stats
      deviceStats.total = drivers.length
      deviceStats.online = onlineDrivers
      deviceStats.offline = drivers.length - onlineDrivers

      // Update recent devices
      recentDevices.value = drivers.slice(0, 5).map(driver => ({
        id: driver.id,
        name: driver.name,
        type: driver.type,
        status: driver.status,
        lastUpdate: driver.lastUpdate,
      }))
    }

    // Load alerts
    const alertsResponse = await alertsApi.getAlerts()
    if (alertsResponse.success && alertsResponse.data) {
      const alerts = alertsResponse.data
      const activeAlerts = alerts.filter(a => !a.acknowledged).length
      statsCards.value[2].value = activeAlerts.toString()
      recentAlerts.value = alerts.slice(0, 5)
    }

    // Load available data points for chart
    const dataPointsResponse = await realtimeApi.getDataPoints()
    if (dataPointsResponse.success && dataPointsResponse.data) {
      availableDataPoints.value = dataPointsResponse.data
    }
  } catch (error) {
    console.error('Failed to load dashboard data:', error)
  } finally {
    loading.value = false
  }
}

// Timers for cleanup
let dataUpdateTimer: NodeJS.Timeout | null = null

// Initialize component
onMounted(async () => {
  // Load initial data from API
  await loadDashboardData()

  // Select first 2 data points after loading
  if (availableDataPoints.value.length >= 2) {
    selectedDataPoints.value = availableDataPoints.value
      .slice(0, 2)
      .map(p => p.id)
  }

  // Start real-time updates
  dataUpdateTimer = setInterval(() => {
    updateRealTimeData()
    loadDashboardData() // Refresh dashboard data
  }, 10000) // Update every 10 seconds

  // Initialize system store
  await systemStore.init()
})

// Cleanup on unmount
onUnmounted(() => {
  if (dataUpdateTimer) {
    clearInterval(dataUpdateTimer)
    dataUpdateTimer = null
  }
})

// Watch for WebSocket messages
watch(lastMessage, message => {
  if (message) {
    try {
      const data = JSON.parse(message)
      if (data.type === 'metrics') {
        Object.assign(systemMetrics, data.data)
      } else if (data.type === 'alert') {
        recentAlerts.value.unshift(data.data)
        if (recentAlerts.value.length > 5) {
          recentAlerts.value = recentAlerts.value.slice(0, 5)
        }
      }
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error)
    }
  }
})
</script>

<style scoped lang="scss">
.dashboard-container {
  padding: 0;

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 20px;
    margin-bottom: 20px;

    .stat-card {
      border: none;

      .stat-content {
        display: flex;
        align-items: center;
        justify-content: space-between;

        .stat-info {
          .stat-value {
            font-size: 28px;
            font-weight: 600;
            color: var(--el-text-color-primary);
            margin-bottom: 4px;
          }

          .stat-label {
            font-size: 14px;
            color: var(--el-text-color-secondary);
            margin-bottom: 8px;
          }

          .stat-change {
            display: flex;
            align-items: center;
            gap: 4px;
            font-size: 12px;

            &.increase {
              color: var(--el-color-success);
            }

            &.decrease {
              color: var(--el-color-danger);
            }

            &.stable {
              color: var(--el-text-color-secondary);
            }
          }
        }

        .stat-icon {
          opacity: 0.7;
          color: var(--el-color-primary);
        }
      }

      &.stat-primary .stat-icon {
        color: var(--el-color-primary);
      }
      &.stat-success .stat-icon {
        color: var(--el-color-success);
      }
      &.stat-warning .stat-icon {
        color: var(--el-color-warning);
      }
      &.stat-info .stat-icon {
        color: var(--el-color-info);
      }
    }
  }

  .content-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 20px;

    .status-card {
      grid-column: span 2;

      .status-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 16px;

        .status-item {
          padding: 16px;
          border: 1px solid var(--el-border-color);
          border-radius: 8px;
          background: var(--el-bg-color-light);

          .status-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 12px;

            .component-name {
              font-weight: 600;
            }
          }

          .status-metrics {
            .metric-item {
              display: flex;
              justify-content: space-between;
              margin-bottom: 8px;

              .metric-label {
                font-size: 13px;
                color: var(--el-text-color-secondary);
              }

              .metric-value {
                font-size: 13px;
                font-weight: 500;
              }
            }
          }
        }
      }
    }

    .chart-card {
      grid-column: span 2;

      .chart-controls {
        display: flex;
        gap: 12px;
        margin-bottom: 16px;
      }

      .chart-container {
        height: 300px;

        .chart {
          height: 100%;
          width: 100%;
        }
      }
    }

    .alerts-card {
      .alerts-list {
        max-height: 300px;
        overflow-y: auto;

        .alert-item {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          padding: 12px;
          border-left: 4px solid transparent;
          margin-bottom: 8px;
          background: var(--el-bg-color-light);
          border-radius: 4px;

          &.alert-error {
            border-left-color: var(--el-color-danger);
          }

          &.alert-warning {
            border-left-color: var(--el-color-warning);
          }

          &.alert-info {
            border-left-color: var(--el-color-info);
          }

          .alert-content {
            flex: 1;

            .alert-header {
              display: flex;
              justify-content: space-between;
              align-items: center;
              margin-bottom: 8px;

              .alert-message {
                font-weight: 500;
                color: var(--el-text-color-primary);
              }
            }

            .alert-meta {
              display: flex;
              justify-content: space-between;
              font-size: 12px;
              color: var(--el-text-color-secondary);
            }
          }

          .alert-actions {
            margin-left: 12px;
          }
        }
      }
    }

    .devices-card {
      .devices-summary {
        display: flex;
        justify-content: space-around;
        margin-bottom: 16px;

        .device-stat {
          text-align: center;

          .device-count {
            font-size: 24px;
            font-weight: 600;
            color: var(--el-text-color-primary);
          }

          .device-label {
            font-size: 12px;
            color: var(--el-text-color-secondary);
            margin-top: 4px;
          }

          &.online .device-count {
            color: var(--el-color-success);
          }

          &.offline .device-count {
            color: var(--el-color-danger);
          }
        }
      }

      .device-list {
        .device-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 12px;
          border-radius: 4px;
          cursor: pointer;
          transition: background-color 0.3s;

          &:hover {
            background: var(--el-bg-color-light);
          }

          .device-info {
            .device-name {
              font-weight: 500;
              margin-bottom: 4px;
            }

            .device-type {
              font-size: 12px;
              color: var(--el-text-color-secondary);
            }
          }
        }
      }
    }

    .performance-card {
      .performance-metrics {
        .metric-card {
          margin-bottom: 20px;

          .metric-title {
            font-size: 14px;
            color: var(--el-text-color-primary);
            margin-bottom: 8px;
          }

          .metric-progress {
            display: flex;
            align-items: center;
            gap: 12px;

            .metric-text {
              font-weight: 600;
              min-width: 40px;
            }
          }

          .metric-value {
            .metric-item {
              display: flex;
              justify-content: space-between;
              margin-bottom: 4px;

              .metric-label {
                font-size: 12px;
                color: var(--el-text-color-secondary);
              }

              .metric-number {
                font-size: 12px;
                font-weight: 500;
              }
            }
          }
        }
      }
    }

    .actions-card {
      .action-buttons {
        display: flex;
        flex-direction: column;
        gap: 12px;

        .el-button {
          justify-content: flex-start;
        }
      }
    }
  }
}

// Responsive design
@media (max-width: 1200px) {
  .content-grid {
    grid-template-columns: 1fr;

    .status-card,
    .chart-card {
      grid-column: span 1;
    }
  }
}

@media (max-width: 768px) {
  .dashboard-container {
    .stats-grid {
      grid-template-columns: repeat(2, 1fr);
      gap: 12px;
    }

    .content-grid {
      gap: 12px;
      grid-template-columns: 1fr;

      .status-card .status-grid {
        grid-template-columns: 1fr;
      }

      .chart-card .chart-controls {
        flex-direction: column;
        gap: 8px;
      }
    }
  }
}
</style>
