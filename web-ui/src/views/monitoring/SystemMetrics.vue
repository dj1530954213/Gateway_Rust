<template>
  <div class="system-metrics">
    <!-- 页面头部 -->
    <div class="metrics-header">
      <div class="header-title">
        <el-icon :size="24"><DataAnalysis /></el-icon>
        <h1>系统性能指标</h1>
        <el-tag :type="systemHealthType" size="large">
          {{ systemHealthText }}
        </el-tag>
      </div>

      <div class="header-actions">
        <el-select v-model="timeRange" size="small" style="width: 150px">
          <el-option label="最近5分钟" value="5m" />
          <el-option label="最近30分钟" value="30m" />
          <el-option label="最近1小时" value="1h" />
          <el-option label="最近6小时" value="6h" />
          <el-option label="最近24小时" value="24h" />
        </el-select>

        <el-button
          :type="autoRefresh ? 'primary' : 'default'"
          size="small"
          @click="toggleAutoRefresh"
        >
          <el-icon
            ><VideoPlay v-if="!autoRefresh" /><VideoPause v-else
          /></el-icon>
          {{ autoRefresh ? '停止刷新' : '自动刷新' }}
        </el-button>

        <el-button :loading="loading" size="small" @click="refreshMetrics">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- 关键指标卡片 -->
    <div class="metrics-overview">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-card shadow="never" class="metric-card metric-card--cpu">
            <div class="metric-content">
              <div class="metric-icon">
                <el-icon><Cpu /></el-icon>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ cpuUsage }}%</div>
                <div class="metric-label">CPU使用率</div>
                <div class="metric-trend" :class="cpuTrend.class">
                  <el-icon><component :is="cpuTrend.icon" /></el-icon>
                  {{ cpuTrend.text }}
                </div>
              </div>
            </div>
            <el-progress
              :percentage="cpuUsage"
              :status="
                cpuUsage > 80
                  ? 'exception'
                  : cpuUsage > 60
                    ? 'warning'
                    : 'success'
              "
              :stroke-width="6"
              :show-text="false"
            />
          </el-card>
        </el-col>

        <el-col :span="6">
          <el-card shadow="never" class="metric-card metric-card--memory">
            <div class="metric-content">
              <div class="metric-icon">
                <el-icon><MostlyCloudy /></el-icon>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ memoryUsage }}%</div>
                <div class="metric-label">内存使用率</div>
                <div class="metric-trend" :class="memoryTrend.class">
                  <el-icon><component :is="memoryTrend.icon" /></el-icon>
                  {{ memoryTrend.text }}
                </div>
              </div>
            </div>
            <el-progress
              :percentage="memoryUsage"
              :status="
                memoryUsage > 85
                  ? 'exception'
                  : memoryUsage > 70
                    ? 'warning'
                    : 'success'
              "
              :stroke-width="6"
              :show-text="false"
            />
          </el-card>
        </el-col>

        <el-col :span="6">
          <el-card shadow="never" class="metric-card metric-card--disk">
            <div class="metric-content">
              <div class="metric-icon">
                <el-icon><Files /></el-icon>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ diskUsage }}%</div>
                <div class="metric-label">磁盘使用率</div>
                <div class="metric-trend" :class="diskTrend.class">
                  <el-icon><component :is="diskTrend.icon" /></el-icon>
                  {{ diskTrend.text }}
                </div>
              </div>
            </div>
            <el-progress
              :percentage="diskUsage"
              :status="
                diskUsage > 90
                  ? 'exception'
                  : diskUsage > 75
                    ? 'warning'
                    : 'success'
              "
              :stroke-width="6"
              :show-text="false"
            />
          </el-card>
        </el-col>

        <el-col :span="6">
          <el-card shadow="never" class="metric-card metric-card--network">
            <div class="metric-content">
              <div class="metric-icon">
                <el-icon><Connection /></el-icon>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ networkSpeed }}</div>
                <div class="metric-label">网络吞吐量</div>
                <div class="metric-trend" :class="networkTrend.class">
                  <el-icon><component :is="networkTrend.icon" /></el-icon>
                  {{ networkTrend.text }}
                </div>
              </div>
            </div>
            <div class="network-details">
              <span class="upload">↑ {{ uploadSpeed }}</span>
              <span class="download">↓ {{ downloadSpeed }}</span>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 详细指标图表 -->
    <div class="metrics-charts">
      <el-row :gutter="20">
        <!-- CPU和内存趋势 -->
        <el-col :span="12">
          <el-card header="CPU & 内存趋势" class="chart-card">
            <div class="chart-container">
              <VChart class="chart" :option="cpuMemoryChartOption" autoresize />
            </div>
          </el-card>
        </el-col>

        <!-- 网络流量 -->
        <el-col :span="12">
          <el-card header="网络流量" class="chart-card">
            <div class="chart-container">
              <VChart class="chart" :option="networkChartOption" autoresize />
            </div>
          </el-card>
        </el-col>
      </el-row>

      <el-row :gutter="20" style="margin-top: 20px">
        <!-- 磁盘I/O -->
        <el-col :span="12">
          <el-card header="磁盘I/O" class="chart-card">
            <div class="chart-container">
              <VChart class="chart" :option="diskIOChartOption" autoresize />
            </div>
          </el-card>
        </el-col>

        <!-- 系统负载 -->
        <el-col :span="12">
          <el-card header="系统负载" class="chart-card">
            <div class="chart-container">
              <VChart class="chart" :option="loadChartOption" autoresize />
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 详细性能表格 -->
    <el-card header="详细性能指标" class="metrics-table">
      <el-table :data="detailedMetrics" stripe>
        <el-table-column prop="category" label="类别" width="120" />
        <el-table-column prop="metric" label="指标" width="200" />
        <el-table-column prop="current" label="当前值" width="120">
          <template #default="{ row }">
            <span :class="getMetricValueClass(row)">{{ row.current }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="average" label="平均值" width="120" />
        <el-table-column prop="peak" label="峰值" width="120" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)" size="small">
              {{ row.status }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column
          prop="lastUpdate"
          label="最后更新"
          show-overflow-tooltip
        />
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { LineChart, BarChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
} from 'echarts/components'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { ElMessage } from 'element-plus'
import { ref, computed, onMounted, onUnmounted } from 'vue'
import VChart from 'vue-echarts'

import { metricsApi, wsClient } from '@/api'
import type {
  SystemMetrics,
  MetricsHistory,
  DetailedMetric,
  SystemHealthStatus,
} from '@/api/metrics'
import { formatTime } from '@/utils/date'

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  BarChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
])

// 响应式状态
const loading = ref(false)
const autoRefresh = ref(true)
const timeRange = ref('1h')

// 性能指标数据
const currentMetrics = ref<SystemMetrics>({
  cpuUsage: 0,
  memoryUsage: 0,
  diskUsage: 0,
  networkSpeed: '0 MB/s',
  uploadSpeed: '0 MB/s',
  downloadSpeed: '0 MB/s',
  timestamp: new Date().toISOString(),
})

const systemHealth = ref<SystemHealthStatus>({
  overall: 'healthy',
  cpuStatus: 'normal',
  memoryStatus: 'normal',
  diskStatus: 'normal',
  networkStatus: 'normal',
})

// 历史数据
const metricsHistory = ref<MetricsHistory>({
  timeLabels: [],
  cpuHistory: [],
  memoryHistory: [],
  networkUpHistory: [],
  networkDownHistory: [],
  diskReadHistory: [],
  diskWriteHistory: [],
  loadAverage1m: [],
  loadAverage5m: [],
  loadAverage15m: [],
})

// 详细指标数据
const detailedMetrics = ref<DetailedMetric[]>([])

// 计算属性
const systemHealthType = computed(() => {
  switch (systemHealth.value.overall) {
    case 'critical':
      return 'danger'
    case 'warning':
      return 'warning'
    default:
      return 'success'
  }
})

const systemHealthText = computed(() => {
  switch (systemHealth.value.overall) {
    case 'critical':
      return '系统负载过高'
    case 'warning':
      return '系统负载较高'
    default:
      return '系统运行正常'
  }
})

const cpuUsage = computed(() => currentMetrics.value.cpuUsage)
const memoryUsage = computed(() => currentMetrics.value.memoryUsage)
const diskUsage = computed(() => currentMetrics.value.diskUsage)
const networkSpeed = computed(() => currentMetrics.value.networkSpeed)
const uploadSpeed = computed(() => currentMetrics.value.uploadSpeed)
const downloadSpeed = computed(() => currentMetrics.value.downloadSpeed)

// 趋势数据
const cpuTrend = ref({ class: 'trend-down', icon: 'Bottom', text: '0%' })
const memoryTrend = ref({ class: 'trend-down', icon: 'Bottom', text: '0%' })
const diskTrend = ref({ class: 'trend-down', icon: 'Bottom', text: '0%' })
const networkTrend = ref({
  class: 'trend-down',
  icon: 'Bottom',
  text: '0 MB/s',
})

// 计算趋势数据
const calculateTrend = (current: number, history: number[]) => {
  if (history.length < 2)
    return { class: 'trend-down', icon: 'Bottom', text: '0%' }

  const previous = history[history.length - 2]
  const change = current - previous
  const changePercent =
    previous > 0 ? ((change / previous) * 100).toFixed(1) : '0'

  if (change > 0) {
    return {
      class: 'trend-up',
      icon: 'Top',
      text: `+${changePercent}%`,
    }
  } else {
    return {
      class: 'trend-down',
      icon: 'Bottom',
      text: `${changePercent}%`,
    }
  }
}

const updateTrends = () => {
  cpuTrend.value = calculateTrend(
    currentMetrics.value.cpuUsage,
    metricsHistory.value.cpuHistory
  )
  memoryTrend.value = calculateTrend(
    currentMetrics.value.memoryUsage,
    metricsHistory.value.memoryHistory
  )
  diskTrend.value = calculateTrend(currentMetrics.value.diskUsage, [])

  const networkChange =
    metricsHistory.value.networkUpHistory.length > 0
      ? metricsHistory.value.networkUpHistory[
          metricsHistory.value.networkUpHistory.length - 1
        ] -
        (metricsHistory.value.networkUpHistory[
          metricsHistory.value.networkUpHistory.length - 2
        ] || 0)
      : 0

  networkTrend.value = {
    class: networkChange > 0 ? 'trend-up' : 'trend-down',
    icon: networkChange > 0 ? 'Top' : 'Bottom',
    text: `${networkChange > 0 ? '+' : ''}${networkChange.toFixed(0)} MB/s`,
  }
}

// 图表配置
const cpuMemoryChartOption = computed(() => ({
  tooltip: {
    trigger: 'axis',
  },
  legend: {
    data: ['CPU使用率', '内存使用率'],
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true,
  },
  xAxis: {
    type: 'category',
    data: metricsHistory.value.timeLabels,
  },
  yAxis: {
    type: 'value',
    max: 100,
    axisLabel: {
      formatter: '{value}%',
    },
  },
  series: [
    {
      name: 'CPU使用率',
      type: 'line',
      smooth: true,
      data: metricsHistory.value.cpuHistory,
      lineStyle: { color: '#409eff' },
    },
    {
      name: '内存使用率',
      type: 'line',
      smooth: true,
      data: metricsHistory.value.memoryHistory,
      lineStyle: { color: '#67c23a' },
    },
  ],
}))

const networkChartOption = computed(() => ({
  tooltip: {
    trigger: 'axis',
    formatter: (params: any) => {
      let result = `${params[0].axisValue}<br/>`
      params.forEach((param: any) => {
        result += `${param.seriesName}: ${param.value} MB/s<br/>`
      })
      return result
    },
  },
  legend: {
    data: ['上传', '下载'],
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true,
  },
  xAxis: {
    type: 'category',
    data: metricsHistory.value.timeLabels,
  },
  yAxis: {
    type: 'value',
    axisLabel: {
      formatter: '{value} MB/s',
    },
  },
  series: [
    {
      name: '上传',
      type: 'line',
      smooth: true,
      data: metricsHistory.value.networkUpHistory,
      lineStyle: { color: '#e6a23c' },
    },
    {
      name: '下载',
      type: 'line',
      smooth: true,
      data: metricsHistory.value.networkDownHistory,
      lineStyle: { color: '#f56c6c' },
    },
  ],
}))

const diskIOChartOption = computed(() => ({
  tooltip: {
    trigger: 'axis',
  },
  legend: {
    data: ['读取速度', '写入速度'],
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true,
  },
  xAxis: {
    type: 'category',
    data: metricsHistory.value.timeLabels,
  },
  yAxis: {
    type: 'value',
    axisLabel: {
      formatter: '{value} MB/s',
    },
  },
  series: [
    {
      name: '读取速度',
      type: 'bar',
      data: metricsHistory.value.diskReadHistory,
      itemStyle: { color: '#909399' },
    },
    {
      name: '写入速度',
      type: 'bar',
      data: metricsHistory.value.diskWriteHistory,
      itemStyle: { color: '#606266' },
    },
  ],
}))

const loadChartOption = computed(() => ({
  tooltip: {
    trigger: 'axis',
  },
  legend: {
    data: ['1分钟负载', '5分钟负载', '15分钟负载'],
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true,
  },
  xAxis: {
    type: 'category',
    data: metricsHistory.value.timeLabels,
  },
  yAxis: {
    type: 'value',
  },
  series: [
    {
      name: '1分钟负载',
      type: 'line',
      smooth: true,
      data: metricsHistory.value.loadAverage1m,
      lineStyle: { color: '#409eff' },
    },
    {
      name: '5分钟负载',
      type: 'line',
      smooth: true,
      data: metricsHistory.value.loadAverage5m,
      lineStyle: { color: '#67c23a' },
    },
    {
      name: '15分钟负载',
      type: 'line',
      smooth: true,
      data: metricsHistory.value.loadAverage15m,
      lineStyle: { color: '#e6a23c' },
    },
  ],
}))

// 方法
const getMetricValueClass = (row: any) => {
  if (row.status === '异常') return 'metric-danger'
  if (row.status === '警告') return 'metric-warning'
  return 'metric-normal'
}

const getStatusType = (status: string) => {
  const types: Record<string, string> = {
    normal: 'success',
    warning: 'warning',
    critical: 'danger',
    正常: 'success',
    警告: 'warning',
    异常: 'danger',
  }
  return types[status] || 'info'
}

// 从API加载真实的系统指标数据
const loadMetricsData = async () => {
  try {
    const response = await metricsApi.getMetricsHistory({
      timeRange: timeRange.value as any,
    })
    metricsHistory.value = response.data
    updateTrends()
  } catch (error) {
    console.error('加载指标历史数据失败:', error)
  }
}

const loadCurrentMetrics = async () => {
  try {
    const [metricsResponse, healthResponse, detailsResponse] =
      await Promise.all([
        metricsApi.getCurrentMetrics(),
        metricsApi.getSystemHealth(),
        metricsApi.getDetailedMetrics(),
      ])

    currentMetrics.value = metricsResponse.data
    systemHealth.value = healthResponse.data
    detailedMetrics.value = detailsResponse.data
    updateTrends()
  } catch (error) {
    console.error('加载当前指标数据失败:', error)
  }
}

const refreshMetrics = async () => {
  loading.value = true
  try {
    await Promise.all([loadCurrentMetrics(), loadMetricsData()])
    ElMessage.success('指标数据已刷新')
  } catch (error) {
    console.error('刷新指标数据失败:', error)
    ElMessage.error('刷新失败')
  } finally {
    loading.value = false
  }
}

const toggleAutoRefresh = () => {
  autoRefresh.value = !autoRefresh.value
  ElMessage.success(autoRefresh.value ? '已开启自动刷新' : '已停止自动刷新')
}

// 定时器
let refreshTimer: number | null = null

const startAutoRefresh = () => {
  if (refreshTimer) clearInterval(refreshTimer)

  refreshTimer = setInterval(() => {
    if (autoRefresh.value) {
      refreshMetrics()
    }
  }, 30000) // 每30秒刷新一次
}

// 生命周期
onMounted(async () => {
  await Promise.all([loadCurrentMetrics(), loadMetricsData()])
  startAutoRefresh()

  // 设置WebSocket监听器
  wsClient.on('system_metrics', (data: SystemMetrics) => {
    currentMetrics.value = data
    updateTrends()
  })

  wsClient.on('system_health', (data: SystemHealthStatus) => {
    systemHealth.value = data
  })
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
  wsClient.off('system_metrics')
  wsClient.off('system_health')
})
</script>

<style scoped lang="scss">
.system-metrics {
  .metrics-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 0 20px 0;

    .header-title {
      display: flex;
      align-items: center;
      gap: 12px;

      h1 {
        margin: 0;
        font-size: 20px;
        font-weight: 600;
      }
    }

    .header-actions {
      display: flex;
      align-items: center;
      gap: 12px;
    }
  }

  .metrics-overview {
    margin-bottom: 20px;

    .metric-card {
      border: none;
      transition: all 0.3s;

      &:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
      }

      .metric-content {
        display: flex;
        align-items: flex-start;
        gap: 16px;
        margin-bottom: 16px;

        .metric-icon {
          width: 48px;
          height: 48px;
          border-radius: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 24px;
        }

        .metric-info {
          flex: 1;

          .metric-value {
            font-size: 28px;
            font-weight: 600;
            color: var(--el-text-color-primary);
          }

          .metric-label {
            font-size: 14px;
            color: var(--el-text-color-secondary);
            margin: 4px 0;
          }

          .metric-trend {
            font-size: 12px;
            display: flex;
            align-items: center;
            gap: 4px;

            &.trend-up {
              color: var(--el-color-success);
            }

            &.trend-down {
              color: var(--el-color-danger);
            }
          }
        }
      }

      &.metric-card--cpu .metric-icon {
        background: var(--el-color-primary-light-9);
        color: var(--el-color-primary);
      }

      &.metric-card--memory .metric-icon {
        background: var(--el-color-success-light-9);
        color: var(--el-color-success);
      }

      &.metric-card--disk .metric-icon {
        background: var(--el-color-warning-light-9);
        color: var(--el-color-warning);
      }

      &.metric-card--network .metric-icon {
        background: var(--el-color-info-light-9);
        color: var(--el-color-info);
      }

      .network-details {
        display: flex;
        justify-content: space-between;
        font-size: 12px;
        color: var(--el-text-color-secondary);

        .upload {
          color: var(--el-color-warning);
        }

        .download {
          color: var(--el-color-success);
        }
      }
    }
  }

  .metrics-charts {
    margin-bottom: 20px;

    .chart-card {
      .chart-container {
        height: 300px;

        .chart {
          height: 100%;
          width: 100%;
        }
      }
    }
  }

  .metrics-table {
    .metric-normal {
      color: var(--el-color-success);
    }

    .metric-warning {
      color: var(--el-color-warning);
    }

    .metric-danger {
      color: var(--el-color-danger);
    }
  }
}
</style>
