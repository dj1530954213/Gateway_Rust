<template>
  <div class="realtime-monitoring-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-title">
        <el-icon :size="24">
          <Monitor />
        </el-icon>
        <h1>实时数据监控</h1>
        <el-tag :type="systemStatusType" size="large">
          {{ systemStatusText }}
        </el-tag>
      </div>
      
      <div class="header-actions">
        <el-date-picker
          v-model="timeRange"
          type="datetimerange"
          :shortcuts="shortcuts"
          range-separator="至"
          start-placeholder="开始日期"
          end-placeholder="结束日期"
          format="YYYY-MM-DD HH:mm:ss"
          value-format="YYYY-MM-DD HH:mm:ss"
          size="small"
        />
        
        <el-button
          :type="isAutoRefresh ? 'primary' : 'default'"
          @click="handleToggleAutoRefresh"
        >
          <el-icon><VideoPlay v-if="!isAutoRefresh" /><VideoPause v-else /></el-icon>
          {{ isAutoRefresh ? '停止刷新' : '自动刷新' }}
        </el-button>
        
        <el-dropdown trigger="click" @command="handleHeaderAction">
          <el-button>
            <el-icon><Setting /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="layout">
                <el-icon><Grid /></el-icon>
                布局设置
              </el-dropdown-item>
              <el-dropdown-item command="export">
                <el-icon><Download /></el-icon>
                导出数据
              </el-dropdown-item>
              <el-dropdown-item command="fullscreen">
                <el-icon><FullScreen /></el-icon>
                全屏显示
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
    
    <!-- 系统状态概览 -->
    <div class="system-overview">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--primary">
            <div class="stat-content">
              <div class="stat-icon">
                <el-icon><DataLine /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ systemStats.activeDataPoints }}</div>
                <div class="stat-label">活跃数据点</div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--success">
            <div class="stat-content">
              <div class="stat-icon">
                <el-icon><Connection /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ systemStats.connectedDrivers }}</div>
                <div class="stat-label">在线驱动</div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--warning">
            <div class="stat-content">
              <div class="stat-icon">
                <el-icon><Warning /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ systemStats.activeAlarms }}</div>
                <div class="stat-label">活跃报警</div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--info">
            <div class="stat-content">
              <div class="stat-icon">
                <el-icon><Odometer /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ systemStats.messageRate }}/s</div>
                <div class="stat-label">消息速率</div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 数据点实时监控 -->
    <div class="main-content">
      <el-row :gutter="20">
        <!-- 左侧数据点列表 -->
        <el-col :span="8">
          <el-card header="数据点列表" class="datapoints-panel">
            <!-- 过滤器 -->
            <div class="datapoint-filters">
              <el-input
                v-model="searchKeyword"
                placeholder="搜索数据点..."
                size="small"
                clearable
              >
                <template #prefix>
                  <el-icon><Search /></el-icon>
                </template>
              </el-input>
              
              <el-select v-model="qualityFilter" placeholder="质量过滤" size="small" style="margin-top: 8px;">
                <el-option label="全部" value="" />
                <el-option label="良好" value="good" />
                <el-option label="不确定" value="uncertain" />
                <el-option label="错误" value="bad" />
              </el-select>
            </div>

            <!-- 数据点列表 -->
            <div class="datapoints-list">
              <div
                v-for="datapoint in filteredDataPoints"
                :key="datapoint.id"
                class="datapoint-item"
                :class="{ 'selected': selectedDataPoints.includes(datapoint.id) }"
                @click="toggleDataPointSelection(datapoint.id)"
              >
                <div class="datapoint-header">
                  <span class="datapoint-name">{{ datapoint.name }}</span>
                  <el-tag
                    :type="getQualityTagType(datapoint.quality)"
                    size="small"
                    effect="plain"
                  >
                    {{ datapoint.quality }}
                  </el-tag>
                </div>
                
                <div class="datapoint-value">
                  <span class="current-value">
                    {{ formatValue(datapoint.currentValue, datapoint.dataType) }}
                    <span v-if="datapoint.unit" class="unit">{{ datapoint.unit }}</span>
                  </span>
                  
                  <div class="trend-indicator" v-if="datapoint.trend">
                    <el-icon v-if="datapoint.trend === 'up'" class="trend-up"><Top /></el-icon>
                    <el-icon v-if="datapoint.trend === 'down'" class="trend-down"><Bottom /></el-icon>
                    <span v-if="datapoint.trend === 'stable'" class="trend-stable">—</span>
                  </div>
                </div>
                
                <div class="datapoint-meta">
                  <span class="address">{{ datapoint.address }}</span>
                  <span class="last-read">{{ formatTime(datapoint.lastReadTime) }}</span>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>

        <!-- 右侧图表和详情 -->
        <el-col :span="16">
          <!-- 实时趋势图表 -->
          <el-card header="实时趋势" class="chart-panel">
            <template #extra>
              <el-button-group size="small">
                <el-button @click="setChartTimeRange('1h')">1小时</el-button>
                <el-button @click="setChartTimeRange('6h')">6小时</el-button>
                <el-button @click="setChartTimeRange('24h')">24小时</el-button>
              </el-button-group>
            </template>

            <div class="chart-container">
              <v-chart
                v-if="chartData.series.length > 0"
                class="chart"
                :option="chartOption"
                autoresize
              />
              <el-empty v-else description="请选择数据点查看趋势" />
            </div>
          </el-card>

          <!-- 驱动状态 -->
          <el-card header="驱动状态" class="drivers-status" style="margin-top: 20px;">
            <el-row :gutter="16">
              <el-col
                v-for="driver in driverStatuses"
                :key="driver.id"
                :span="8"
              >
                <div class="driver-card">
                  <div class="driver-header">
                    <span class="driver-name">{{ driver.name }}</span>
                    <el-tag
                      :type="getDriverStatusType(driver.status)"
                      size="small"
                    >
                      {{ getDriverStatusText(driver.status) }}
                    </el-tag>
                  </div>
                  
                  <div class="driver-metrics">
                    <div class="metric-item">
                      <span class="metric-label">消息速率</span>
                      <span class="metric-value">{{ driver.messageRate || 0 }}/s</span>
                    </div>
                    <div class="metric-item">
                      <span class="metric-label">平均延迟</span>
                      <span class="metric-value">{{ driver.avgLatency || 0 }}ms</span>
                    </div>
                  </div>
                </div>
              </el-col>
            </el-row>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 数据点详情抽屉 -->
    <el-drawer
      v-model="showDatapointDetail"
      title="数据点详情"
      direction="rtl"
      size="40%"
    >
      <div v-if="viewingDatapoint" class="datapoint-detail">
        <!-- 基本信息 -->
        <div class="detail-section">
          <h4>基本信息</h4>
          <el-descriptions :column="1" border>
            <el-descriptions-item label="名称">{{ viewingDatapoint.name }}</el-descriptions-item>
            <el-descriptions-item label="地址">{{ viewingDatapoint.address }}</el-descriptions-item>
            <el-descriptions-item label="数据类型">{{ viewingDatapoint.dataType }}</el-descriptions-item>
            <el-descriptions-item label="当前值">
              {{ formatValue(viewingDatapoint.currentValue, viewingDatapoint.dataType) }}
            </el-descriptions-item>
            <el-descriptions-item label="质量">
              <el-tag :type="getQualityTagType(viewingDatapoint.quality)">
                {{ viewingDatapoint.quality }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="最后读取">
              {{ formatTime(viewingDatapoint.lastReadTime) }}
            </el-descriptions-item>
          </el-descriptions>
        </div>
      </div>
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
} from 'echarts/components'
import VChart from 'vue-echarts'
import { formatTime } from '@/utils/date'
import { realtimeApi, driversApi } from '@/services/api'

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
])

// 类型定义
interface DataPoint {
  id: string
  name: string
  address: string
  dataType: 'boolean' | 'integer' | 'float' | 'string'
  driverId: string
  quality: 'good' | 'uncertain' | 'bad' | 'invalid'
  currentValue: any
  previousValue?: any
  lastReadTime: Date
  unit?: string
  trend?: 'up' | 'down' | 'stable'
}

interface SystemStats {
  activeDataPoints: number
  connectedDrivers: number
  activeAlarms: number
  messageRate: number
}

interface DriverStatus {
  id: string
  name: string
  status: 'running' | 'stopped' | 'error'
  messageRate?: number
  avgLatency?: number
}

// 响应式状态
const timeRange = ref([
  new Date(Date.now() - 60 * 60 * 1000).toISOString(),
  new Date().toISOString()
])

const isAutoRefresh = ref(true)
const searchKeyword = ref('')
const qualityFilter = ref('')
const selectedDataPoints = ref<string[]>([])
const showDatapointDetail = ref(false)
const viewingDatapoint = ref<DataPoint | null>(null)

// 模拟数据
const systemStats = reactive<SystemStats>({
  activeDataPoints: 156,
  connectedDrivers: 8,
  activeAlarms: 3,
  messageRate: 1247
})

const dataPoints = ref<DataPoint[]>([
  {
    id: '1',
    name: '温度传感器-001',
    address: '40001',
    dataType: 'float',
    driverId: 'modbus-1',
    quality: 'good',
    currentValue: 23.5,
    lastReadTime: new Date(),
    unit: '°C',
    trend: 'up'
  },
  {
    id: '2',
    name: '压力传感器-001',
    address: '40002',
    dataType: 'float',
    driverId: 'modbus-1',
    quality: 'good',
    currentValue: 1.2,
    lastReadTime: new Date(),
    unit: 'bar',
    trend: 'stable'
  },
  {
    id: '3',
    name: '流量计-001',
    address: '40003',
    dataType: 'float',
    driverId: 'modbus-2',
    quality: 'uncertain',
    currentValue: 45.8,
    lastReadTime: new Date(),
    unit: 'L/min',
    trend: 'down'
  }
])

const driverStatuses = ref<DriverStatus[]>([
  {
    id: 'modbus-1',
    name: 'Modbus TCP-001',
    status: 'running',
    messageRate: 150,
    avgLatency: 12
  },
  {
    id: 'modbus-2',
    name: 'Modbus RTU-001',
    status: 'running',
    messageRate: 89,
    avgLatency: 25
  },
  {
    id: 'opcua-1',
    name: 'OPC UA Server',
    status: 'error',
    messageRate: 0,
    avgLatency: 0
  }
])

const chartData = ref({
  series: [] as any[],
  xAxisData: [] as string[]
})

// 时间快捷选项
const shortcuts = [
  {
    text: '最近30分钟',
    value: () => {
      const end = new Date()
      const start = new Date()
      start.setTime(start.getTime() - 30 * 60 * 1000)
      return [start, end]
    }
  },
  {
    text: '最近1小时',
    value: () => {
      const end = new Date()
      const start = new Date()
      start.setTime(start.getTime() - 60 * 60 * 1000)
      return [start, end]
    }
  },
  {
    text: '最近6小时',
    value: () => {
      const end = new Date()
      const start = new Date()
      start.setTime(start.getTime() - 6 * 60 * 60 * 1000)
      return [start, end]
    }
  }
]

// 计算属性
const systemStatusType = computed(() => {
  if (systemStats.activeAlarms > 5) return 'danger'
  if (systemStats.activeAlarms > 0) return 'warning'
  return 'success'
})

const systemStatusText = computed(() => {
  if (systemStats.activeAlarms > 5) return '系统异常'
  if (systemStats.activeAlarms > 0) return '有告警'
  return '运行正常'
})

const filteredDataPoints = computed(() => {
  return dataPoints.value.filter(dp => {
    const matchesSearch = !searchKeyword.value || 
      dp.name.toLowerCase().includes(searchKeyword.value.toLowerCase())
    const matchesQuality = !qualityFilter.value || dp.quality === qualityFilter.value
    return matchesSearch && matchesQuality
  })
})

const chartOption = computed(() => ({
  title: {
    text: '实时数据趋势',
    left: 'center'
  },
  tooltip: {
    trigger: 'axis'
  },
  legend: {
    top: 30
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true
  },
  xAxis: {
    type: 'category',
    data: chartData.value.xAxisData
  },
  yAxis: {
    type: 'value'
  },
  series: chartData.value.series
}))

// 方法
const getQualityTagType = (quality: string) => {
  const types: Record<string, any> = {
    good: 'success',
    uncertain: 'warning',
    bad: 'danger',
    invalid: 'info'
  }
  return types[quality] || 'info'
}

const getDriverStatusType = (status: string) => {
  const types: Record<string, any> = {
    running: 'success',
    stopped: 'warning',
    error: 'danger'
  }
  return types[status] || 'info'
}

const getDriverStatusText = (status: string) => {
  const texts: Record<string, string> = {
    running: '运行中',
    stopped: '已停止',
    error: '错误'
  }
  return texts[status] || status
}

const formatValue = (value: any, dataType: string) => {
  if (value === null || value === undefined) return '--'
  
  switch (dataType) {
    case 'float':
      return Number(value).toFixed(2)
    case 'integer':
      return parseInt(value).toString()
    case 'boolean':
      return value ? '是' : '否'
    default:
      return String(value)
  }
}

const toggleDataPointSelection = (id: string) => {
  const index = selectedDataPoints.value.indexOf(id)
  if (index > -1) {
    selectedDataPoints.value.splice(index, 1)
  } else {
    selectedDataPoints.value.push(id)
  }
  updateChartData()
}

const updateChartData = () => {
  const selectedPoints = dataPoints.value.filter(dp => 
    selectedDataPoints.value.includes(dp.id)
  )
  
  const timeLabels = Array.from({ length: 20 }, (_, i) => {
    const time = new Date(Date.now() - (19 - i) * 60000)
    return time.toLocaleTimeString()
  })
  
  chartData.value = {
    xAxisData: timeLabels,
    series: selectedPoints.map((dp, index) => ({
      name: dp.name,
      type: 'line',
      smooth: true,
      data: Array.from({ length: 20 }, () => 
        Math.random() * 100 + (dp.currentValue as number || 50)
      )
    }))
  }
}

const handleToggleAutoRefresh = () => {
  isAutoRefresh.value = !isAutoRefresh.value
  ElMessage.success(isAutoRefresh.value ? '已开启自动刷新' : '已停止自动刷新')
}

const handleHeaderAction = (command: string) => {
  ElMessage.info(`${command} 功能开发中`)
}

const setChartTimeRange = (range: string) => {
  ElMessage.info(`设置时间范围: ${range}`)
}

// 定时刷新
let refreshTimer: number | null = null

const startAutoRefresh = () => {
  if (refreshTimer) clearInterval(refreshTimer)
  
  refreshTimer = setInterval(() => {
    if (isAutoRefresh.value) {
      // 更新数据点值
      dataPoints.value.forEach(dp => {
        dp.currentValue = Math.random() * 100
        dp.lastReadTime = new Date()
      })
      
      // 更新统计
      systemStats.messageRate = Math.floor(Math.random() * 500) + 1000
      
      // 更新图表
      updateChartData()
    }
  }, 5000)
}

// 生命周期
onMounted(async () => {
  selectedDataPoints.value = ['1', '2']
  updateChartData()
  startAutoRefresh()
  
  try {
    await loadRealtimeData()
  } catch (error) {
    console.error('Failed to load realtime data:', error)
  }
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})

const loadRealtimeData = async () => {
  try {
    const response = await realtimeApi.getCurrentValues()
    if (response.success && response.data) {
      // 处理实时数据
      console.log('Realtime data loaded:', response.data)
    }
  } catch (error) {
    console.error('Failed to load realtime data:', error)
  }
}
</script>

<style scoped lang="scss">
.realtime-monitoring-page {
  padding: 0;
  
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    background: var(--el-bg-color);
    border-bottom: 1px solid var(--el-border-color);
    
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
  
  .system-overview {
    padding: 20px;
    
    .stat-card {
      border: none;
      
      .stat-content {
        display: flex;
        align-items: center;
        gap: 16px;
        
        .stat-icon {
          width: 48px;
          height: 48px;
          border-radius: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 24px;
        }
        
        .stat-info {
          .stat-number {
            font-size: 24px;
            font-weight: 600;
            color: var(--el-text-color-primary);
          }
          
          .stat-label {
            font-size: 14px;
            color: var(--el-text-color-secondary);
            margin-top: 4px;
          }
        }
      }
      
      &.stat-card--primary .stat-icon {
        background: var(--el-color-primary-light-9);
        color: var(--el-color-primary);
      }
      
      &.stat-card--success .stat-icon {
        background: var(--el-color-success-light-9);
        color: var(--el-color-success);
      }
      
      &.stat-card--warning .stat-icon {
        background: var(--el-color-warning-light-9);
        color: var(--el-color-warning);
      }
      
      &.stat-card--info .stat-icon {
        background: var(--el-color-info-light-9);
        color: var(--el-color-info);
      }
    }
  }
  
  .main-content {
    padding: 0 20px 20px;
    
    .datapoints-panel {
      height: 600px;
      
      .datapoint-filters {
        margin-bottom: 16px;
      }
      
      .datapoints-list {
        height: 480px;
        overflow-y: auto;
        
        .datapoint-item {
          padding: 12px;
          border: 1px solid var(--el-border-color);
          border-radius: 8px;
          margin-bottom: 8px;
          cursor: pointer;
          transition: all 0.3s;
          
          &:hover {
            border-color: var(--el-color-primary);
          }
          
          &.selected {
            border-color: var(--el-color-primary);
            background: var(--el-color-primary-light-9);
          }
          
          .datapoint-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 8px;
            
            .datapoint-name {
              font-weight: 600;
            }
          }
          
          .datapoint-value {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 8px;
            
            .current-value {
              font-size: 18px;
              font-weight: 600;
              color: var(--el-color-primary);
              
              .unit {
                font-size: 12px;
                color: var(--el-text-color-secondary);
                margin-left: 4px;
              }
            }
            
            .trend-indicator {
              .trend-up { color: var(--el-color-success); }
              .trend-down { color: var(--el-color-danger); }
              .trend-stable { color: var(--el-text-color-secondary); }
            }
          }
          
          .datapoint-meta {
            display: flex;
            justify-content: space-between;
            font-size: 12px;
            color: var(--el-text-color-secondary);
          }
        }
      }
    }
    
    .chart-panel {
      height: 400px;
      
      .chart-container {
        height: 300px;
        
        .chart {
          height: 100%;
          width: 100%;
        }
      }
    }
    
    .drivers-status {
      .driver-card {
        padding: 16px;
        border: 1px solid var(--el-border-color);
        border-radius: 8px;
        background: var(--el-bg-color-light);
        
        .driver-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
          
          .driver-name {
            font-weight: 600;
          }
        }
        
        .driver-metrics {
          .metric-item {
            display: flex;
            justify-content: space-between;
            margin-bottom: 8px;
            
            .metric-label {
              font-size: 12px;
              color: var(--el-text-color-secondary);
            }
            
            .metric-value {
              font-size: 12px;
              font-weight: 500;
            }
          }
        }
      }
    }
  }
  
  .datapoint-detail {
    .detail-section {
      margin-bottom: 24px;
      
      h4 {
        margin: 0 0 16px 0;
        color: var(--el-text-color-primary);
      }
    }
  }
}
</style>