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
        <el-select v-model="timeRange" size="small" style="width: 150px;">
          <el-option label="最近5分钟" value="5m" />
          <el-option label="最近30分钟" value="30m" />
          <el-option label="最近1小时" value="1h" />
          <el-option label="最近6小时" value="6h" />
          <el-option label="最近24小时" value="24h" />
        </el-select>
        
        <el-button 
          :type="autoRefresh ? 'primary' : 'default'"
          @click="toggleAutoRefresh"
          size="small"
        >
          <el-icon><VideoPlay v-if="!autoRefresh" /><VideoPause v-else /></el-icon>
          {{ autoRefresh ? '停止刷新' : '自动刷新' }}
        </el-button>
        
        <el-button @click="refreshMetrics" :loading="loading" size="small">
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
              :status="cpuUsage > 80 ? 'exception' : cpuUsage > 60 ? 'warning' : 'success'"
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
              :status="memoryUsage > 85 ? 'exception' : memoryUsage > 70 ? 'warning' : 'success'"
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
              :status="diskUsage > 90 ? 'exception' : diskUsage > 75 ? 'warning' : 'success'"
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
              <v-chart
                class="chart"
                :option="cpuMemoryChartOption"
                autoresize
              />
            </div>
          </el-card>
        </el-col>

        <!-- 网络流量 -->
        <el-col :span="12">
          <el-card header="网络流量" class="chart-card">
            <div class="chart-container">
              <v-chart
                class="chart"
                :option="networkChartOption"
                autoresize
              />
            </div>
          </el-card>
        </el-col>
      </el-row>

      <el-row :gutter="20" style="margin-top: 20px;">
        <!-- 磁盘I/O -->
        <el-col :span="12">
          <el-card header="磁盘I/O" class="chart-card">
            <div class="chart-container">
              <v-chart
                class="chart"
                :option="diskIOChartOption"
                autoresize
              />
            </div>
          </el-card>
        </el-col>

        <!-- 系统负载 -->
        <el-col :span="12">
          <el-card header="系统负载" class="chart-card">
            <div class="chart-container">
              <v-chart
                class="chart"
                :option="loadChartOption"
                autoresize
              />
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
        <el-table-column prop="lastUpdate" label="最后更新" show-overflow-tooltip />
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart, BarChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
} from 'echarts/components'
import VChart from 'vue-echarts'
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
const cpuUsage = ref(45)
const memoryUsage = ref(62)
const diskUsage = ref(78)
const networkSpeed = ref('125 MB/s')
const uploadSpeed = ref('45 MB/s')
const downloadSpeed = ref('80 MB/s')

// 模拟历史数据
const cpuHistory = ref<number[]>([])
const memoryHistory = ref<number[]>([])
const networkUpHistory = ref<number[]>([])
const networkDownHistory = ref<number[]>([])
const timeLabels = ref<string[]>([])

// 详细指标数据
const detailedMetrics = ref([
  {
    category: 'CPU',
    metric: 'CPU使用率',
    current: '45.2%',
    average: '38.7%',
    peak: '89.1%',
    status: '正常',
    lastUpdate: formatTime(new Date())
  },
  {
    category: 'CPU',
    metric: 'CPU温度',
    current: '65°C',
    average: '62°C',
    peak: '78°C',
    status: '正常',
    lastUpdate: formatTime(new Date())
  },
  {
    category: '内存',
    metric: '内存使用率',
    current: '62.1%',
    average: '58.3%',
    peak: '91.5%',
    status: '正常',
    lastUpdate: formatTime(new Date())
  },
  {
    category: '内存',
    metric: '可用内存',
    current: '6.1 GB',
    average: '6.7 GB',
    peak: '15.2 GB',
    status: '正常',
    lastUpdate: formatTime(new Date())
  },
  {
    category: '磁盘',
    metric: '磁盘使用率',
    current: '78.3%',
    average: '72.1%',
    peak: '89.7%',
    status: '警告',
    lastUpdate: formatTime(new Date())
  },
  {
    category: '磁盘',
    metric: '磁盘读取速度',
    current: '245 MB/s',
    average: '198 MB/s',
    peak: '512 MB/s',
    status: '正常',
    lastUpdate: formatTime(new Date())
  },
  {
    category: '网络',
    metric: '网络延迟',
    current: '12ms',
    average: '15ms',
    peak: '156ms',
    status: '正常',
    lastUpdate: formatTime(new Date())
  },
  {
    category: '网络',
    metric: '丢包率',
    current: '0.01%',
    average: '0.03%',
    peak: '2.1%',
    status: '正常',
    lastUpdate: formatTime(new Date())
  }
])

// 计算属性
const systemHealthType = computed(() => {
  const avgUsage = (cpuUsage.value + memoryUsage.value + diskUsage.value) / 3
  if (avgUsage > 80) return 'danger'
  if (avgUsage > 60) return 'warning'
  return 'success'
})

const systemHealthText = computed(() => {
  const avgUsage = (cpuUsage.value + memoryUsage.value + diskUsage.value) / 3
  if (avgUsage > 80) return '系统负载过高'
  if (avgUsage > 60) return '系统负载较高'
  return '系统运行正常'
})

const cpuTrend = computed(() => {
  const trend = Math.random() > 0.5 ? 'up' : 'down'
  return {
    class: trend === 'up' ? 'trend-up' : 'trend-down',
    icon: trend === 'up' ? 'Top' : 'Bottom',
    text: trend === 'up' ? '+2.3%' : '-1.8%'
  }
})

const memoryTrend = computed(() => {
  const trend = Math.random() > 0.5 ? 'up' : 'down'
  return {
    class: trend === 'up' ? 'trend-up' : 'trend-down',
    icon: trend === 'up' ? 'Top' : 'Bottom',
    text: trend === 'up' ? '+5.1%' : '-3.2%'
  }
})

const diskTrend = computed(() => {
  const trend = Math.random() > 0.5 ? 'up' : 'down'
  return {
    class: trend === 'up' ? 'trend-up' : 'trend-down',
    icon: trend === 'up' ? 'Top' : 'Bottom',
    text: trend === 'up' ? '+0.8%' : '-0.5%'
  }
})

const networkTrend = computed(() => {
  const trend = Math.random() > 0.5 ? 'up' : 'down'
  return {
    class: trend === 'up' ? 'trend-up' : 'trend-down',
    icon: trend === 'up' ? 'Top' : 'Bottom',
    text: trend === 'up' ? '+15 MB/s' : '-8 MB/s'
  }
})

// 图表配置
const cpuMemoryChartOption = computed(() => ({
  tooltip: {
    trigger: 'axis'
  },
  legend: {
    data: ['CPU使用率', '内存使用率']
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true
  },
  xAxis: {
    type: 'category',
    data: timeLabels.value
  },
  yAxis: {
    type: 'value',
    max: 100,
    axisLabel: {
      formatter: '{value}%'
    }
  },
  series: [
    {
      name: 'CPU使用率',
      type: 'line',
      smooth: true,
      data: cpuHistory.value,
      lineStyle: { color: '#409eff' }
    },
    {
      name: '内存使用率',
      type: 'line',
      smooth: true,
      data: memoryHistory.value,
      lineStyle: { color: '#67c23a' }
    }
  ]
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
    }
  },
  legend: {
    data: ['上传', '下载']
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true
  },
  xAxis: {
    type: 'category',
    data: timeLabels.value
  },
  yAxis: {
    type: 'value',
    axisLabel: {
      formatter: '{value} MB/s'
    }
  },
  series: [
    {
      name: '上传',
      type: 'line',
      smooth: true,
      data: networkUpHistory.value,
      lineStyle: { color: '#e6a23c' }
    },
    {
      name: '下载',
      type: 'line',
      smooth: true,
      data: networkDownHistory.value,
      lineStyle: { color: '#f56c6c' }
    }
  ]
}))

const diskIOChartOption = computed(() => ({
  tooltip: {
    trigger: 'axis'
  },
  legend: {
    data: ['读取速度', '写入速度']
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true
  },
  xAxis: {
    type: 'category',
    data: timeLabels.value
  },
  yAxis: {
    type: 'value',
    axisLabel: {
      formatter: '{value} MB/s'
    }
  },
  series: [
    {
      name: '读取速度',
      type: 'bar',
      data: Array.from({ length: 20 }, () => Math.floor(Math.random() * 300 + 100)),
      itemStyle: { color: '#909399' }
    },
    {
      name: '写入速度',
      type: 'bar',
      data: Array.from({ length: 20 }, () => Math.floor(Math.random() * 200 + 50)),
      itemStyle: { color: '#606266' }
    }
  ]
}))

const loadChartOption = computed(() => ({
  tooltip: {
    trigger: 'axis'
  },
  legend: {
    data: ['1分钟负载', '5分钟负载', '15分钟负载']
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true
  },
  xAxis: {
    type: 'category',
    data: timeLabels.value
  },
  yAxis: {
    type: 'value'
  },
  series: [
    {
      name: '1分钟负载',
      type: 'line',
      smooth: true,
      data: Array.from({ length: 20 }, () => Math.random() * 4),
      lineStyle: { color: '#409eff' }
    },
    {
      name: '5分钟负载',
      type: 'line',
      smooth: true,
      data: Array.from({ length: 20 }, () => Math.random() * 3 + 0.5),
      lineStyle: { color: '#67c23a' }
    },
    {
      name: '15分钟负载',
      type: 'line',
      smooth: true,
      data: Array.from({ length: 20 }, () => Math.random() * 2 + 1),
      lineStyle: { color: '#e6a23c' }
    }
  ]
}))

// 方法
const getMetricValueClass = (row: any) => {
  if (row.status === '异常') return 'metric-danger'
  if (row.status === '警告') return 'metric-warning'
  return 'metric-normal'
}

const getStatusType = (status: string) => {
  const types: Record<string, string> = {
    '正常': 'success',
    '警告': 'warning',
    '异常': 'danger'
  }
  return types[status] || 'info'
}

const generateMockData = () => {
  const now = new Date()
  const labels = []
  const cpu = []
  const memory = []
  const netUp = []
  const netDown = []

  for (let i = 19; i >= 0; i--) {
    const time = new Date(now.getTime() - i * 60000)
    labels.push(time.toLocaleTimeString())
    cpu.push(Math.floor(Math.random() * 30 + 30))
    memory.push(Math.floor(Math.random() * 25 + 50))
    netUp.push(Math.floor(Math.random() * 50 + 20))
    netDown.push(Math.floor(Math.random() * 100 + 40))
  }

  timeLabels.value = labels
  cpuHistory.value = cpu
  memoryHistory.value = memory
  networkUpHistory.value = netUp
  networkDownHistory.value = netDown
}

const refreshMetrics = async () => {
  loading.value = true
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // 更新指标数据
    cpuUsage.value = Math.floor(Math.random() * 40 + 30)
    memoryUsage.value = Math.floor(Math.random() * 30 + 50)
    diskUsage.value = Math.floor(Math.random() * 20 + 70)
    
    generateMockData()
    ElMessage.success('指标数据已刷新')
  } catch (error) {
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
onMounted(() => {
  generateMockData()
  startAutoRefresh()
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
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