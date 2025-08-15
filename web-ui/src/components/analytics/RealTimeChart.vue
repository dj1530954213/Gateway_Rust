<template>
  <div class="real-time-chart">
    <div v-if="loading" class="chart-loading">
      <el-icon class="loading-icon"><Loading /></el-icon>
      <span>加载图表数据...</span>
    </div>

    <div
      v-else
      ref="chartRef"
      class="chart-container"
      :style="{ height: chartHeight }"
    ></div>

    <!-- 图表工具栏 -->
    <div v-if="!loading" class="chart-toolbar">
      <div class="chart-info">
        <span class="data-points">数据点: {{ dataPointsCount }}</span>
        <span class="update-time">更新时间: {{ lastUpdateTime }}</span>
      </div>

      <div class="chart-tools">
        <el-button-group size="small">
          <el-button :disabled="zoomLevel >= maxZoom" @click="zoomIn">
            <el-icon><ZoomIn /></el-icon>
          </el-button>
          <el-button :disabled="zoomLevel <= minZoom" @click="zoomOut">
            <el-icon><ZoomOut /></el-icon>
          </el-button>
          <el-button @click="resetZoom">
            <el-icon><FullScreen /></el-icon>
          </el-button>
        </el-button-group>

        <el-button
          size="small"
          :type="autoRefresh ? 'primary' : ''"
          @click="toggleAutoRefresh"
        >
          <el-icon><Refresh /></el-icon>
          {{ autoRefresh ? '停止' : '自动' }}刷新
        </el-button>

        <el-dropdown @command="handleExportChart">
          <el-button size="small">
            <el-icon><Download /></el-icon>
            导出
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="png">PNG图片</el-dropdown-item>
              <el-dropdown-item command="jpg">JPG图片</el-dropdown-item>
              <el-dropdown-item command="svg">SVG矢量图</el-dropdown-item>
              <el-dropdown-item command="pdf">PDF文档</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- 图例控制 -->
    <div v-if="!loading && showLegendControls" class="legend-controls">
      <div class="legend-title">数据系列:</div>
      <div class="legend-items">
        <div
          v-for="series in seriesConfig"
          :key="series.name"
          :class="['legend-item', { disabled: !series.visible }]"
          @click="toggleSeries(series.name)"
        >
          <div
            class="legend-color"
            :style="{ backgroundColor: series.visible ? series.color : '#ccc' }"
          ></div>
          <span class="legend-name">{{ series.label }}</span>
          <span class="legend-value">{{ getCurrentValue(series.name) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * RealTimeChart —— 实时数据图表组件
 *
 * 📝 Responsibilities:
 *  1. 实时数据的折线图展示
 *  2. 多数据系列的同时显示
 *  3. 图表交互操作（缩放、平移、导出）
 *  4. 自动刷新和实时更新
 *  5. 图例控制和数据切换
 *
 * 📦 Dependencies:
 *  - ECharts 图表库
 *  - Element Plus UI组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Loading,
  ZoomIn,
  ZoomOut,
  FullScreen,
  Refresh,
  Download,
} from '@element-plus/icons-vue'
import * as echarts from 'echarts'
import { ElMessage } from 'element-plus'
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'

// ===== Props =====
const props = defineProps<{
  data: any[]
  loading?: boolean
  options?: any
  height?: string
  autoRefresh?: boolean
  refreshInterval?: number
  showLegendControls?: boolean
}>()

// ===== Emits =====
const emit = defineEmits<{
  'point-click': [point: any]
  'zoom-change': [zoom: { start: number; end: number }]
  'series-toggle': [series: string, visible: boolean]
  'export-complete': [type: string, success: boolean]
}>()

// ===== 响应式数据 =====
const chartRef = ref<HTMLDivElement>()
const chartInstance = ref<echarts.ECharts>()
const autoRefresh = ref(props.autoRefresh ?? false)
const refreshTimer = ref<number>()

// 图表控制
const zoomLevel = ref(1)
const minZoom = ref(0.1)
const maxZoom = ref(10)

// 数据系列配置
const seriesConfig = ref([
  {
    name: 'temperature',
    label: '温度',
    color: '#ff7675',
    visible: true,
    unit: '°C',
  },
  {
    name: 'pressure',
    label: '压力',
    color: '#74b9ff',
    visible: true,
    unit: 'Pa',
  },
  {
    name: 'flow',
    label: '流量',
    color: '#00cec9',
    visible: true,
    unit: 'L/min',
  },
  { name: 'power', label: '功率', color: '#fdcb6e', visible: true, unit: 'W' },
])

// ===== 计算属性 =====
const chartHeight = computed(() => props.height || '400px')

const dataPointsCount = computed(() => props.data?.length || 0)

const lastUpdateTime = computed(() => {
  if (!props.data || props.data.length === 0) return '--'

  const lastPoint = props.data[props.data.length - 1]
  return new Date(lastPoint.timestamp).toLocaleTimeString('zh-CN')
})

const chartOptions = computed(() => {
  const baseOptions = {
    animation: true,
    animationDuration: 300,
    grid: {
      left: '3%',
      right: '8%',
      bottom: '8%',
      top: '12%',
      containLabel: true,
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross',
        label: {
          backgroundColor: '#6a7985',
        },
      },
      formatter(params: any[]) {
        let tooltip = `<div style="margin-bottom: 5px;">${formatTime(params[0].axisValue)}</div>`

        params.forEach(param => {
          const series = seriesConfig.value.find(
            s => s.name === param.seriesName
          )
          const unit = series?.unit || ''
          tooltip += `
            <div style="display: flex; align-items: center; margin-bottom: 3px;">
              <span style="display: inline-block; width: 10px; height: 10px; background-color: ${param.color}; border-radius: 50%; margin-right: 8px;"></span>
              <span style="flex: 1;">${param.seriesName}: </span>
              <span style="font-weight: bold;">${param.value.toFixed(2)} ${unit}</span>
            </div>
          `
        })

        return tooltip
      },
    },
    legend: {
      show: false, // 使用自定义图例
    },
    xAxis: {
      type: 'time',
      boundaryGap: false,
      axisLine: {
        lineStyle: {
          color: '#ddd',
        },
      },
      axisTick: {
        show: false,
      },
      axisLabel: {
        color: '#666',
        formatter(value: number) {
          return formatTime(value)
        },
      },
      splitLine: {
        show: true,
        lineStyle: {
          color: '#f0f0f0',
          type: 'dashed',
        },
      },
    },
    yAxis: [
      {
        type: 'value',
        name: '数值',
        position: 'left',
        axisLine: {
          lineStyle: {
            color: '#ddd',
          },
        },
        axisTick: {
          show: false,
        },
        axisLabel: {
          color: '#666',
        },
        splitLine: {
          lineStyle: {
            color: '#f0f0f0',
            type: 'dashed',
          },
        },
      },
    ],
    dataZoom: [
      {
        type: 'inside',
        start: 0,
        end: 100,
        minValueSpan: 3600000, // 最小1小时
      },
      {
        type: 'slider',
        start: 0,
        end: 100,
        height: 20,
        bottom: 25,
        borderColor: 'transparent',
        fillerColor: 'rgba(64, 158, 255, 0.2)',
        handleStyle: {
          color: '#409EFF',
          borderColor: '#409EFF',
        },
        moveHandleStyle: {
          color: '#409EFF',
        },
        selectedDataBackground: {
          lineStyle: {
            color: '#409EFF',
          },
          areaStyle: {
            color: 'rgba(64, 158, 255, 0.1)',
          },
        },
      },
    ],
    series: generateSeries(),
  }

  // 合并自定义选项
  return { ...baseOptions, ...props.options }
})

// ===== 方法 =====

/**
 * 生成图表系列数据
 */
function generateSeries() {
  return seriesConfig.value
    .filter(config => config.visible)
    .map(config => ({
      name: config.label,
      type: 'line',
      smooth: true,
      symbol: 'none',
      lineStyle: {
        width: 2,
        color: config.color,
      },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: `${config.color}40` },
          { offset: 1, color: `${config.color}10` },
        ]),
      },
      emphasis: {
        focus: 'series',
      },
      data:
        props.data?.map(point => [
          new Date(point.timestamp).getTime(),
          point[config.name] || 0,
        ]) || [],
    }))
}

/**
 * 初始化图表
 */
function initChart() {
  if (!chartRef.value) return

  chartInstance.value = echarts.init(chartRef.value)

  // 设置图表选项
  chartInstance.value.setOption(chartOptions.value)

  // 绑定事件
  chartInstance.value.on('click', handleChartClick)
  chartInstance.value.on('datazoom', handleDataZoom)

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
 * 处理图表点击事件
 */
function handleChartClick(params: any) {
  const point = {
    timestamp: new Date(params.value[0]),
    value: params.value[1],
    series: params.seriesName,
    dataIndex: params.dataIndex,
  }

  emit('point-click', point)
}

/**
 * 处理数据缩放事件
 */
function handleDataZoom(params: any) {
  if (params.batch) {
    const zoom = params.batch[0]
    emit('zoom-change', { start: zoom.start, end: zoom.end })
  }
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
 * 缩放操作
 */
function zoomIn() {
  if (zoomLevel.value >= maxZoom.value) return

  zoomLevel.value = Math.min(zoomLevel.value * 1.2, maxZoom.value)
  applyZoom()
}

function zoomOut() {
  if (zoomLevel.value <= minZoom.value) return

  zoomLevel.value = Math.max(zoomLevel.value / 1.2, minZoom.value)
  applyZoom()
}

function resetZoom() {
  zoomLevel.value = 1

  if (chartInstance.value) {
    chartInstance.value.dispatchAction({
      type: 'dataZoom',
      start: 0,
      end: 100,
    })
  }
}

function applyZoom() {
  if (!chartInstance.value) return

  const centerPercent = 50
  const zoomPercent = 100 / zoomLevel.value
  const start = Math.max(0, centerPercent - zoomPercent / 2)
  const end = Math.min(100, centerPercent + zoomPercent / 2)

  chartInstance.value.dispatchAction({
    type: 'dataZoom',
    start,
    end,
  })
}

/**
 * 自动刷新控制
 */
function toggleAutoRefresh() {
  autoRefresh.value = !autoRefresh.value

  if (autoRefresh.value) {
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
}

function startAutoRefresh() {
  if (refreshTimer.value) return

  const interval = props.refreshInterval || 5000
  refreshTimer.value = window.setInterval(() => {
    // 这里可以触发数据更新
    // 实际应用中会调用父组件的刷新方法
  }, interval)
}

function stopAutoRefresh() {
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
    refreshTimer.value = undefined
  }
}

/**
 * 系列控制
 */
function toggleSeries(seriesName: string) {
  const series = seriesConfig.value.find(s => s.name === seriesName)
  if (series) {
    series.visible = !series.visible
    updateChart()
    emit('series-toggle', seriesName, series.visible)
  }
}

function getCurrentValue(seriesName: string): string {
  if (!props.data || props.data.length === 0) return '--'

  const lastPoint = props.data[props.data.length - 1]
  const value = lastPoint[seriesName]

  if (value === undefined || value === null) return '--'

  const series = seriesConfig.value.find(s => s.name === seriesName)
  const unit = series?.unit || ''

  return `${value.toFixed(2)} ${unit}`
}

/**
 * 导出图表
 */
function handleExportChart(command: string) {
  if (!chartInstance.value) {
    ElMessage.error('图表未初始化')
    return
  }

  try {
    let dataURL: string

    switch (command) {
      case 'png':
        dataURL = chartInstance.value.getDataURL({
          type: 'png',
          pixelRatio: 2,
          backgroundColor: '#fff',
        })
        downloadImage(dataURL, 'chart.png')
        break

      case 'jpg':
        dataURL = chartInstance.value.getDataURL({
          type: 'jpg',
          pixelRatio: 2,
          backgroundColor: '#fff',
        })
        downloadImage(dataURL, 'chart.jpg')
        break

      case 'svg':
        dataURL = chartInstance.value.getDataURL({
          type: 'svg',
        })
        downloadImage(dataURL, 'chart.svg')
        break

      case 'pdf':
        // PDF导出需要额外处理
        exportToPDF()
        break
    }

    emit('export-complete', command, true)
    ElMessage.success(`图表导出${command.toUpperCase()}成功`)
  } catch (error) {
    console.error('导出图表失败:', error)
    emit('export-complete', command, false)
    ElMessage.error('导出图表失败')
  }
}

function downloadImage(dataURL: string, filename: string) {
  const link = document.createElement('a')
  link.download = filename
  link.href = dataURL
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
}

function exportToPDF() {
  // 这里可以集成PDF生成库，如jsPDF
  ElMessage.info('PDF导出功能开发中...')
}

/**
 * 工具函数
 */
function formatTime(timestamp: string | number): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  })
}

// ===== 生命周期 =====
onMounted(() => {
  nextTick(() => {
    initChart()
  })
})

onUnmounted(() => {
  stopAutoRefresh()
  window.removeEventListener('resize', handleResize)

  if (chartInstance.value) {
    chartInstance.value.dispose()
  }
})

// ===== 监听器 =====
watch(
  () => props.data,
  () => {
    updateChart()
  },
  { deep: true }
)

watch(
  () => props.loading,
  loading => {
    if (!loading && chartRef.value && !chartInstance.value) {
      nextTick(() => {
        initChart()
      })
    }
  }
)

watch(
  () => props.options,
  () => {
    updateChart()
  },
  { deep: true }
)
</script>

<style scoped lang="scss">
.real-time-chart {
  position: relative;

  .chart-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 400px;
    color: #909399;

    .loading-icon {
      font-size: 32px;
      margin-bottom: 12px;
      animation: rotate 1s linear infinite;
    }

    @keyframes rotate {
      from {
        transform: rotate(0deg);
      }
      to {
        transform: rotate(360deg);
      }
    }
  }

  .chart-container {
    width: 100%;
    transition: all 0.3s ease;
  }

  .chart-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 0;
    border-top: 1px solid #ebeef5;
    margin-top: 12px;

    .chart-info {
      display: flex;
      gap: 16px;
      font-size: 12px;
      color: #909399;

      .data-points,
      .update-time {
        display: flex;
        align-items: center;
      }
    }

    .chart-tools {
      display: flex;
      gap: 8px;
      align-items: center;
    }
  }

  .legend-controls {
    margin-top: 16px;
    padding: 12px;
    background: #fafafa;
    border-radius: 6px;

    .legend-title {
      font-size: 12px;
      color: #606266;
      margin-bottom: 8px;
      font-weight: 500;
    }

    .legend-items {
      display: flex;
      flex-wrap: wrap;
      gap: 16px;

      .legend-item {
        display: flex;
        align-items: center;
        gap: 6px;
        cursor: pointer;
        padding: 4px 8px;
        border-radius: 4px;
        transition: all 0.2s;

        &:hover {
          background: #f0f0f0;
        }

        &.disabled {
          opacity: 0.5;
        }

        .legend-color {
          width: 12px;
          height: 12px;
          border-radius: 50%;
          flex-shrink: 0;
        }

        .legend-name {
          font-size: 12px;
          color: #606266;
          min-width: 40px;
        }

        .legend-value {
          font-size: 12px;
          color: #303133;
          font-weight: 500;
          min-width: 60px;
          text-align: right;
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .real-time-chart {
    .chart-toolbar {
      flex-direction: column;
      gap: 8px;
      align-items: stretch;

      .chart-info {
        justify-content: center;
      }

      .chart-tools {
        justify-content: center;
        flex-wrap: wrap;
      }
    }

    .legend-controls .legend-items {
      justify-content: center;
      gap: 8px;

      .legend-item {
        flex-direction: column;
        text-align: center;
        gap: 2px;
        min-width: 60px;
      }
    }
  }
}
</style>
