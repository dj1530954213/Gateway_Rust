<template>
  <div class="performance-metrics-chart">
    <div 
      ref="chartRef" 
      class="chart-container"
    ></div>
    
    <!-- 性能指标摘要 -->
    <div class="metrics-summary">
      <div 
        v-for="metric in metricsConfig" 
        :key="metric.key"
        class="metric-card"
        :class="getMetricStatus(metric.key)"
      >
        <div class="metric-icon">
          <el-icon>
            <component :is="metric.icon" />
          </el-icon>
        </div>
        <div class="metric-info">
          <div class="metric-name">{{ metric.name }}</div>
          <div class="metric-value">{{ getCurrentValue(metric.key) }}</div>
          <div class="metric-status">{{ getStatusText(metric.key) }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * PerformanceMetricsChart —— 性能指标图表组件
 *
 * 📝 Responsibilities:
 *  1. 系统性能指标的可视化展示
 *  2. 性能阈值监控和告警
 *  3. 多指标对比分析
 *
 * 📦 Dependencies:
 *  - ECharts 图表库
 *  - Element Plus 图标
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import * as echarts from 'echarts'
import {
  Timer,
  TrendCharts,
  Warning,
  CircleCheck
} from '@element-plus/icons-vue'

// ===== Props =====
const props = defineProps<{
  data: Array<{
    timestamp: string
    response_time: number
    throughput: number
    error_rate: number
    availability: number
  }>
  thresholds: {
    response_time: { warning: number, critical: number }
    throughput: { warning: number, critical: number }
    error_rate: { warning: number, critical: number }
    availability: { warning: number, critical: number }
  }
}>()

// ===== 响应式数据 =====
const chartRef = ref<HTMLDivElement>()
const chartInstance = ref<echarts.ECharts>()

// 指标配置
const metricsConfig = ref([
  {
    key: 'response_time',
    name: '响应时间',
    icon: 'Timer',
    unit: 'ms',
    format: (value: number) => `${value.toFixed(1)} ms`
  },
  {
    key: 'throughput',
    name: '吞吐量',
    icon: 'TrendCharts',
    unit: '/s',
    format: (value: number) => `${value.toFixed(0)} /s`
  },
  {
    key: 'error_rate',
    name: '错误率',
    icon: 'Warning',
    unit: '%',
    format: (value: number) => `${(value * 100).toFixed(2)}%`
  },
  {
    key: 'availability',
    name: '可用性',
    icon: 'CircleCheck',
    unit: '%',
    format: (value: number) => `${(value * 100).toFixed(2)}%`
  }
])

// ===== 计算属性 =====
const chartOptions = computed(() => ({
  animation: true,
  animationDuration: 1000,
  grid: [
    { left: '7%', right: '52%', top: '7%', bottom: '25%' },
    { left: '55%', right: '7%', top: '7%', bottom: '25%' },
    { left: '7%', right: '52%', top: '52%', bottom: '7%' },
    { left: '55%', right: '7%', top: '52%', bottom: '7%' }
  ],
  tooltip: {
    trigger: 'axis',
    axisPointer: {
      type: 'cross'
    }
  },
  xAxis: [
    {
      type: 'time',
      gridIndex: 0,
      axisLabel: { show: false }
    },
    {
      type: 'time',
      gridIndex: 1,
      axisLabel: { show: false }
    },
    {
      type: 'time',
      gridIndex: 2,
      axisLabel: {
        formatter(value: number) {
          return formatTime(value)
        }
      }
    },
    {
      type: 'time',
      gridIndex: 3,
      axisLabel: {
        formatter(value: number) {
          return formatTime(value)
        }
      }
    }
  ],
  yAxis: [
    {
      type: 'value',
      gridIndex: 0,
      name: '响应时间 (ms)',
      nameTextStyle: { fontSize: 10 },
      axisLabel: { fontSize: 10 }
    },
    {
      type: 'value',
      gridIndex: 1,
      name: '吞吐量 (/s)',
      nameTextStyle: { fontSize: 10 },
      axisLabel: { fontSize: 10 }
    },
    {
      type: 'value',
      gridIndex: 2,
      name: '错误率 (%)',
      nameTextStyle: { fontSize: 10 },
      axisLabel: { 
        fontSize: 10,
        formatter(value: number) {
          return `${(value * 100).toFixed(1)}%`
        }
      }
    },
    {
      type: 'value',
      gridIndex: 3,
      name: '可用性 (%)',
      nameTextStyle: { fontSize: 10 },
      axisLabel: { 
        fontSize: 10,
        formatter(value: number) {
          return `${(value * 100).toFixed(1)}%`
        }
      }
    }
  ],
  series: [
    // 响应时间
    {
      name: '响应时间',
      type: 'line',
      xAxisIndex: 0,
      yAxisIndex: 0,
      smooth: true,
      symbol: 'none',
      lineStyle: { color: '#409EFF', width: 2 },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: '#409EFF40' },
          { offset: 1, color: '#409EFF10' }
        ])
      },
      data: props.data?.map(point => [
        new Date(point.timestamp).getTime(),
        point.response_time
      ]) || [],
      markLine: {
        data: [
          {
            name: '警告阈值',
            yAxis: props.thresholds.response_time.warning,
            lineStyle: { color: '#E6A23C', type: 'dashed' }
          },
          {
            name: '危险阈值',
            yAxis: props.thresholds.response_time.critical,
            lineStyle: { color: '#F56C6C', type: 'dashed' }
          }
        ]
      }
    },
    // 吞吐量
    {
      name: '吞吐量',
      type: 'line',
      xAxisIndex: 1,
      yAxisIndex: 1,
      smooth: true,
      symbol: 'none',
      lineStyle: { color: '#67C23A', width: 2 },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: '#67C23A40' },
          { offset: 1, color: '#67C23A10' }
        ])
      },
      data: props.data?.map(point => [
        new Date(point.timestamp).getTime(),
        point.throughput
      ]) || [],
      markLine: {
        data: [
          {
            name: '警告阈值',
            yAxis: props.thresholds.throughput.warning,
            lineStyle: { color: '#E6A23C', type: 'dashed' }
          },
          {
            name: '危险阈值',
            yAxis: props.thresholds.throughput.critical,
            lineStyle: { color: '#F56C6C', type: 'dashed' }
          }
        ]
      }
    },
    // 错误率
    {
      name: '错误率',
      type: 'line',
      xAxisIndex: 2,
      yAxisIndex: 2,
      smooth: true,
      symbol: 'none',
      lineStyle: { color: '#E6A23C', width: 2 },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: '#E6A23C40' },
          { offset: 1, color: '#E6A23C10' }
        ])
      },
      data: props.data?.map(point => [
        new Date(point.timestamp).getTime(),
        point.error_rate
      ]) || [],
      markLine: {
        data: [
          {
            name: '警告阈值',
            yAxis: props.thresholds.error_rate.warning,
            lineStyle: { color: '#E6A23C', type: 'dashed' }
          },
          {
            name: '危险阈值',
            yAxis: props.thresholds.error_rate.critical,
            lineStyle: { color: '#F56C6C', type: 'dashed' }
          }
        ]
      }
    },
    // 可用性
    {
      name: '可用性',
      type: 'line',
      xAxisIndex: 3,
      yAxisIndex: 3,
      smooth: true,
      symbol: 'none',
      lineStyle: { color: '#F56C6C', width: 2 },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: '#F56C6C40' },
          { offset: 1, color: '#F56C6C10' }
        ])
      },
      data: props.data?.map(point => [
        new Date(point.timestamp).getTime(),
        point.availability
      ]) || [],
      markLine: {
        data: [
          {
            name: '警告阈值',
            yAxis: props.thresholds.availability.warning,
            lineStyle: { color: '#E6A23C', type: 'dashed' }
          },
          {
            name: '危险阈值',
            yAxis: props.thresholds.availability.critical,
            lineStyle: { color: '#F56C6C', type: 'dashed' }
          }
        ]
      }
    }
  ]
}))

// ===== 方法 =====

/**
 * 初始化图表
 */
function initChart() {
  if (!chartRef.value) return

  chartInstance.value = echarts.init(chartRef.value)
  
  // 设置图表选项
  chartInstance.value.setOption(chartOptions.value)
  
  // 监听窗口大小变化
  window.addEventListener('resize', handleResize)
}

/**
 * 更新图表数据
 */
function updateChart() {
  if (!chartInstance.value) return
  
  chartInstance.value.setOption(chartOptions.value, false, true)
}

/**
 * 处理窗口大小变化
 */
function handleResize() {
  if (chartInstance.value) {
    chartInstance.value.resize()
  }
}

/**
 * 获取当前值
 */
function getCurrentValue(metricKey: string): string {
  if (!props.data || props.data.length === 0) return '--'
  
  const lastPoint = props.data[props.data.length - 1]
  const value = lastPoint[metricKey as keyof typeof lastPoint]
  
  if (value === undefined || value === null) return '--'
  
  const config = metricsConfig.value.find(m => m.key === metricKey)
  return config?.format(value) || value.toString()
}

/**
 * 获取指标状态
 */
function getMetricStatus(metricKey: string): string {
  if (!props.data || props.data.length === 0) return 'unknown'
  
  const lastPoint = props.data[props.data.length - 1]
  const value = lastPoint[metricKey as keyof typeof lastPoint]
  
  if (value === undefined || value === null) return 'unknown'
  
  const threshold = props.thresholds[metricKey as keyof typeof props.thresholds]
  
  if (metricKey === 'error_rate') {
    // 错误率越低越好
    if (value <= threshold.warning) return 'good'
    if (value <= threshold.critical) return 'warning'
    return 'critical'
  } else if (metricKey === 'availability') {
    // 可用性越高越好
    if (value >= threshold.warning) return 'good'
    if (value >= threshold.critical) return 'warning'
    return 'critical'
  } else if (metricKey === 'response_time') {
    // 响应时间越低越好
    if (value <= threshold.warning) return 'good'
    if (value <= threshold.critical) return 'warning'
    return 'critical'
  } else if (metricKey === 'throughput') {
    // 吞吐量越高越好
    if (value >= threshold.warning) return 'good'
    if (value >= threshold.critical) return 'warning'
    return 'critical'
  }
  
  return 'unknown'
}

/**
 * 获取状态文本
 */
function getStatusText(metricKey: string): string {
  const status = getMetricStatus(metricKey)
  
  switch (status) {
    case 'good': return '正常'
    case 'warning': return '警告'
    case 'critical': return '危险'
    default: return '未知'
  }
}

/**
 * 格式化时间
 */
function formatTime(timestamp: string | number): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('zh-CN', { 
    hour: '2-digit', 
    minute: '2-digit' 
  })
}

// ===== 生命周期 =====
onMounted(() => {
  nextTick(() => {
    initChart()
  })
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  
  if (chartInstance.value) {
    chartInstance.value.dispose()
  }
})

// ===== 监听器 =====
watch(() => props.data, () => {
  updateChart()
}, { deep: true })

watch(() => props.thresholds, () => {
  updateChart()
}, { deep: true })
</script>

<style scoped lang="scss">
.performance-metrics-chart {
  .chart-container {
    width: 100%;
    height: 250px;
  }
  
  .metrics-summary {
    margin-top: 16px;
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
    
    .metric-card {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 12px;
      border-radius: 8px;
      border: 2px solid transparent;
      background: #fafafa;
      transition: all 0.3s;
      
      &.good {
        border-color: #67C23A;
        background: #f0f9ff;
        
        .metric-icon {
          color: #67C23A;
        }
      }
      
      &.warning {
        border-color: #E6A23C;
        background: #fdf6ec;
        
        .metric-icon {
          color: #E6A23C;
        }
      }
      
      &.critical {
        border-color: #F56C6C;
        background: #fef0f0;
        
        .metric-icon {
          color: #F56C6C;
        }
      }
      
      &.unknown {
        border-color: #909399;
        background: #f4f4f5;
        
        .metric-icon {
          color: #909399;
        }
      }
      
      .metric-icon {
        font-size: 20px;
        flex-shrink: 0;
      }
      
      .metric-info {
        flex: 1;
        
        .metric-name {
          font-size: 12px;
          color: #606266;
          margin-bottom: 2px;
        }
        
        .metric-value {
          font-size: 16px;
          font-weight: 600;
          color: #303133;
          margin-bottom: 2px;
        }
        
        .metric-status {
          font-size: 10px;
          color: #909399;
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .performance-metrics-chart {
    .metrics-summary {
      grid-template-columns: 1fr;
    }
  }
}
</style>