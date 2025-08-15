<template>
  <div class="history-data-chart">
    <!-- 图表工具栏 -->
    <div class="chart-toolbar">
      <div class="toolbar-left">
        <div class="chart-title">
          <span>历史数据趋势图</span>
          <el-tag v-if="dataStats.totalPoints > 0" size="small" type="info">
            {{ dataStats.totalPoints }} 个数据点
          </el-tag>
        </div>
      </div>

      <div class="toolbar-right">
        <div class="chart-controls">
          <!-- 图表类型选择 -->
          <el-select
            v-model="chartType"
            size="small"
            style="width: 100px"
            @change="handleChartTypeChange"
          >
            <el-option label="折线图" value="line" />
            <el-option label="面积图" value="area" />
            <el-option label="柱状图" value="bar" />
            <el-option label="散点图" value="scatter" />
          </el-select>

          <!-- 图例显示控制 -->
          <el-button-group size="small">
            <el-button
              :type="showLegend ? 'primary' : 'default'"
              @click="toggleLegend"
            >
              图例
            </el-button>
            <el-button
              :type="showDataZoom ? 'primary' : 'default'"
              @click="toggleDataZoom"
            >
              缩放
            </el-button>
            <el-button
              :type="showGrid ? 'primary' : 'default'"
              @click="toggleGrid"
            >
              网格
            </el-button>
          </el-button-group>

          <!-- 操作按钮 -->
          <div class="action-buttons">
            <el-tooltip content="重置缩放">
              <el-button type="text" size="small" @click="resetZoom">
                <el-icon><RefreshLeft /></el-icon>
              </el-button>
            </el-tooltip>

            <el-tooltip content="保存图片">
              <el-button type="text" size="small" @click="saveAsImage">
                <el-icon><Picture /></el-icon>
              </el-button>
            </el-tooltip>

            <el-tooltip content="全屏显示">
              <el-button type="text" size="small" @click="toggleFullscreen">
                <el-icon><FullScreen /></el-icon>
              </el-button>
            </el-tooltip>
          </div>
        </div>
      </div>
    </div>

    <!-- 图表容器 -->
    <div
      ref="chartContainer"
      class="chart-container"
      :class="{ fullscreen: isFullscreen }"
    >
      <div v-if="loading" class="chart-loading">
        <el-skeleton animated>
          <template #template>
            <div class="loading-content">
              <el-skeleton-item
                variant="text"
                style="width: 60%; height: 20px"
              />
              <el-skeleton-item
                variant="rect"
                style="width: 100%; height: 400px; margin-top: 16px"
              />
            </div>
          </template>
        </el-skeleton>
      </div>

      <div v-else-if="!hasData" class="chart-empty">
        <el-empty description="暂无图表数据" :image-size="120">
          <template #description>
            <p>请先查询历史数据以显示趋势图表</p>
          </template>
        </el-empty>
      </div>

      <div v-else class="chart-wrapper">
        <VChart
          ref="chartRef"
          class="chart"
          :option="chartOption"
          :loading="loading"
          autoresize
          @click="handleChartClick"
          @brush="handleBrushSelect"
          @datazoom="handleDataZoom"
        />

        <!-- 数据点信息面板 -->
        <div v-if="selectedDataPoint" class="data-point-panel">
          <div class="panel-header">
            <span>数据点详情</span>
            <el-button
              type="text"
              size="small"
              @click="selectedDataPoint = null"
            >
              <el-icon><Close /></el-icon>
            </el-button>
          </div>

          <div class="panel-content">
            <div class="info-item">
              <span class="label">时间:</span>
              <span class="value">{{
                formatDateTime(selectedDataPoint.time)
              }}</span>
            </div>
            <div class="info-item">
              <span class="label">数值:</span>
              <span class="value">{{ selectedDataPoint.value }}</span>
            </div>
            <div class="info-item">
              <span class="label">标签:</span>
              <span class="value">{{ selectedDataPoint.tagName }}</span>
            </div>
            <div class="info-item">
              <span class="label">设备:</span>
              <span class="value">{{ selectedDataPoint.deviceName }}</span>
            </div>
          </div>
        </div>

        <!-- 图表统计信息 -->
        <div v-if="dataStats.totalPoints > 0" class="chart-stats">
          <div class="stats-item">
            <span class="stats-label">数据点:</span>
            <span class="stats-value">{{ dataStats.totalPoints }}</span>
          </div>
          <div class="stats-item">
            <span class="stats-label">时间跨度:</span>
            <span class="stats-value">{{ dataStats.timeSpan }}</span>
          </div>
          <div class="stats-item">
            <span class="stats-label">标签数:</span>
            <span class="stats-value">{{ dataStats.tagCount }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * HistoryDataChart —— 历史数据图表组件
 *
 * 📝 Responsibilities:
 *  1. 历史数据可视化展示
 *  2. 多种图表类型支持
 *  3. 数据点交互和详情显示
 *  4. 图表缩放和导出功能
 *  5. 实时数据统计信息
 *
 * 📦 Dependencies:
 *  - ECharts 图表库
 *  - Vue ECharts 组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  RefreshLeft,
  Picture,
  FullScreen,
  Close,
} from '@element-plus/icons-vue'
import { LineChart, BarChart, ScatterChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
  ToolboxComponent,
  BrushComponent,
} from 'echarts/components'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { ElMessage } from 'element-plus'
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import VChart from 'vue-echarts'

import { formatDateTime } from '@/utils/date'
import { formatNumber } from '@/utils/format'

// 注册ECharts组件
use([
  CanvasRenderer,
  LineChart,
  BarChart,
  ScatterChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
  ToolboxComponent,
  BrushComponent,
])

// ===== Props & Emits =====
const props = defineProps<{
  data: any[]
  loading?: boolean
  timeRange: string
  aggregation: string
}>()

const emit = defineEmits<{
  'point-click': [point: any]
}>()

// ===== 响应式数据 =====
const chartRef = ref()
const chartContainer = ref<HTMLElement>()
const isFullscreen = ref(false)
const selectedDataPoint = ref(null)

// 图表配置
const chartType = ref('line')
const showLegend = ref(true)
const showDataZoom = ref(true)
const showGrid = ref(true)

// ===== 计算属性 =====
const hasData = computed(() => {
  return props.data && props.data.length > 0
})

const dataStats = computed(() => {
  if (!hasData.value) {
    return {
      totalPoints: 0,
      timeSpan: '',
      tagCount: 0,
    }
  }

  const data = props.data
  const totalPoints = data.length

  // 计算时间跨度
  const times = data
    .map(item => new Date(item.timestamp).getTime())
    .sort((a, b) => a - b)
  const timeSpan =
    times.length > 1
      ? formatTimeSpan(new Date(times[0]), new Date(times[times.length - 1]))
      : '1分钟内'

  // 计算标签数量
  const uniqueTags = new Set(data.map(item => item.tagId || item.tagName))
  const tagCount = uniqueTags.size

  return {
    totalPoints,
    timeSpan,
    tagCount,
  }
})

const chartOption = computed(() => {
  if (!hasData.value) return {}

  // 处理数据，按标签分组
  const tagGroups = groupDataByTag(props.data)

  // 生成系列数据
  const series = Object.entries(tagGroups).map(
    ([tagName, dataPoints], index) => {
      const seriesData = dataPoints.map(point => [point.timestamp, point.value])

      const baseConfig = {
        name: tagName,
        data: seriesData,
        itemStyle: {
          color: getChartColor(index),
        },
      }

      switch (chartType.value) {
        case 'line':
          return {
            ...baseConfig,
            type: 'line',
            smooth: true,
            symbol: 'circle',
            symbolSize: 6,
            lineStyle: {
              width: 2,
            },
          }
        case 'area':
          return {
            ...baseConfig,
            type: 'line',
            smooth: true,
            symbol: 'circle',
            symbolSize: 4,
            areaStyle: {
              opacity: 0.3,
            },
            lineStyle: {
              width: 2,
            },
          }
        case 'bar':
          return {
            ...baseConfig,
            type: 'bar',
            barWidth: '60%',
          }
        case 'scatter':
          return {
            ...baseConfig,
            type: 'scatter',
            symbolSize: 8,
          }
        default:
          return {
            ...baseConfig,
            type: 'line',
          }
      }
    }
  )

  return {
    title: {
      show: false,
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross',
        label: {
          backgroundColor: '#6a7985',
        },
      },
      formatter: (params: any[]) => {
        if (!params || params.length === 0) return ''

        const time = formatDateTime(params[0].value[0])
        let content = `<div><strong>${time}</strong></div>`

        params.forEach(param => {
          const value = formatNumber(param.value[1], 2)
          content += `
            <div style="margin-top: 4px;">
              <span style="color: ${param.color}">●</span>
              ${param.seriesName}: <strong>${value}</strong>
            </div>
          `
        })

        return content
      },
    },
    legend: {
      show: showLegend.value,
      top: 10,
      type: 'scroll',
    },
    grid: {
      show: showGrid.value,
      left: '3%',
      right: '4%',
      bottom: showDataZoom.value ? '15%' : '3%',
      top: showLegend.value ? '15%' : '3%',
      containLabel: true,
    },
    xAxis: {
      type: 'time',
      boundaryGap: false,
      axisLabel: {
        formatter: (value: number) => {
          return formatDateTime(value, 'HH:mm')
        },
      },
    },
    yAxis: {
      type: 'value',
      scale: true,
      axisLabel: {
        formatter: (value: number) => formatNumber(value, 1),
      },
    },
    dataZoom: showDataZoom.value
      ? [
          {
            type: 'inside',
            start: 0,
            end: 100,
          },
          {
            type: 'slider',
            start: 0,
            end: 100,
            height: 30,
            bottom: 10,
          },
        ]
      : [],
    toolbox: {
      feature: {
        saveAsImage: {
          title: '保存图片',
          name: `历史数据_${formatDateTime(new Date(), 'YYYYMMDD_HHmmss')}`,
        },
        restore: {
          title: '重置',
        },
      },
      right: 20,
      top: 10,
    },
    brush: {
      toolbox: ['rect', 'polygon', 'clear'],
      throttleType: 'debounce',
      throttleDelay: 300,
    },
    series,
  }
})

// ===== 方法 =====

/**
 * 按标签分组数据
 */
function groupDataByTag(data: any[]): Record<string, any[]> {
  const groups: Record<string, any[]> = {}

  data.forEach(item => {
    const tagKey = item.tagName || item.tagId || 'Unknown'
    if (!groups[tagKey]) {
      groups[tagKey] = []
    }
    groups[tagKey].push(item)
  })

  // 按时间排序每个组的数据
  Object.values(groups).forEach(group => {
    group.sort(
      (a, b) =>
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
    )
  })

  return groups
}

/**
 * 获取图表颜色
 */
function getChartColor(index: number): string {
  const colors = [
    '#409eff',
    '#67c23a',
    '#e6a23c',
    '#f56c6c',
    '#909399',
    '#c471ed',
    '#36cfc9',
    '#f759ab',
    '#ff9800',
    '#795548',
    '#607d8b',
    '#9c27b0',
  ]
  return colors[index % colors.length]
}

/**
 * 格式化时间跨度
 */
function formatTimeSpan(start: Date, end: Date): string {
  const diffMs = end.getTime() - start.getTime()
  const diffMinutes = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMinutes / 60)
  const diffDays = Math.floor(diffHours / 24)

  if (diffDays > 0) {
    return `${diffDays}天${diffHours % 24}小时`
  } else if (diffHours > 0) {
    return `${diffHours}小时${diffMinutes % 60}分钟`
  } else {
    return `${diffMinutes}分钟`
  }
}

/**
 * 处理图表类型变化
 */
function handleChartTypeChange() {
  nextTick(() => {
    chartRef.value?.resize()
  })
}

/**
 * 切换图例显示
 */
function toggleLegend() {
  showLegend.value = !showLegend.value
}

/**
 * 切换数据缩放
 */
function toggleDataZoom() {
  showDataZoom.value = !showDataZoom.value
}

/**
 * 切换网格显示
 */
function toggleGrid() {
  showGrid.value = !showGrid.value
}

/**
 * 重置缩放
 */
function resetZoom() {
  if (chartRef.value) {
    chartRef.value.dispatchAction({
      type: 'dataZoom',
      start: 0,
      end: 100,
    })
  }
  ElMessage.success('缩放已重置')
}

/**
 * 保存为图片
 */
function saveAsImage() {
  if (chartRef.value) {
    const url = chartRef.value.getDataURL({
      type: 'png',
      pixelRatio: 2,
      backgroundColor: '#fff',
    })

    const link = document.createElement('a')
    link.href = url
    link.download = `历史数据图表_${formatDateTime(new Date(), 'YYYYMMDD_HHmmss')}.png`
    link.click()

    ElMessage.success('图表已保存')
  }
}

/**
 * 切换全屏
 */
function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value

  nextTick(() => {
    chartRef.value?.resize()
    window.dispatchEvent(new Event('resize'))
  })
}

/**
 * 处理图表点击
 */
function handleChartClick(params: any) {
  if (params.componentType === 'series') {
    const dataPoint = {
      time: params.value[0],
      value: params.value[1],
      tagName: params.seriesName,
      deviceName: 'Unknown', // 需要从原始数据中获取
      seriesIndex: params.seriesIndex,
      dataIndex: params.dataIndex,
    }

    selectedDataPoint.value = dataPoint
    emit('point-click', dataPoint)
  }
}

/**
 * 处理刷选事件
 */
function handleBrushSelect(params: any) {
  console.log('Brush select:', params)
  // 可以处理刷选后的数据分析
}

/**
 * 处理数据缩放事件
 */
function handleDataZoom(params: any) {
  console.log('Data zoom:', params)
  // 可以处理缩放后的数据加载
}

// ===== 生命周期 =====
onMounted(() => {
  // 监听ESC键退出全屏
  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape' && isFullscreen.value) {
      toggleFullscreen()
    }
  }

  document.addEventListener('keydown', handleKeydown)

  return () => {
    document.removeEventListener('keydown', handleKeydown)
  }
})

// ===== 监听器 =====
watch(
  () => props.data,
  () => {
    // 数据变化时重置选中的数据点
    selectedDataPoint.value = null
  },
  { deep: true }
)

watch(isFullscreen, fullscreen => {
  if (fullscreen) {
    document.body.style.overflow = 'hidden'
  } else {
    document.body.style.overflow = ''
  }
})
</script>

<style scoped lang="scss">
.history-data-chart {
  .chart-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;

    .toolbar-left {
      .chart-title {
        display: flex;
        align-items: center;
        gap: 8px;

        span {
          font-size: 16px;
          font-weight: 600;
          color: #303133;
        }
      }
    }

    .toolbar-right {
      .chart-controls {
        display: flex;
        align-items: center;
        gap: 12px;

        .action-buttons {
          display: flex;
          gap: 4px;
        }
      }
    }
  }

  .chart-container {
    position: relative;
    height: 500px;
    border: 1px solid #ebeef5;
    border-radius: 6px;
    background: white;

    &.fullscreen {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      z-index: 2000;
      height: 100vh;
      border: none;
      border-radius: 0;
    }

    .chart-loading {
      padding: 20px;

      .loading-content {
        display: flex;
        flex-direction: column;
        gap: 16px;
      }
    }

    .chart-empty {
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100%;
    }

    .chart-wrapper {
      position: relative;
      height: 100%;

      .chart {
        height: 100%;
        width: 100%;
      }

      .data-point-panel {
        position: absolute;
        top: 20px;
        right: 20px;
        width: 280px;
        background: white;
        border: 1px solid #ebeef5;
        border-radius: 6px;
        box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
        z-index: 10;

        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          border-bottom: 1px solid #ebeef5;
          background: #f5f7fa;
          border-radius: 6px 6px 0 0;

          span {
            font-weight: 600;
            color: #303133;
          }
        }

        .panel-content {
          padding: 16px;

          .info-item {
            display: flex;
            justify-content: space-between;
            margin-bottom: 8px;

            &:last-child {
              margin-bottom: 0;
            }

            .label {
              color: #909399;
              font-size: 13px;
            }

            .value {
              color: #303133;
              font-weight: 500;
              font-size: 13px;
              max-width: 180px;
              word-break: break-all;
            }
          }
        }
      }

      .chart-stats {
        position: absolute;
        bottom: 20px;
        left: 20px;
        display: flex;
        gap: 16px;
        padding: 8px 12px;
        background: rgba(255, 255, 255, 0.9);
        border-radius: 4px;
        backdrop-filter: blur(4px);

        .stats-item {
          display: flex;
          align-items: center;
          gap: 4px;

          .stats-label {
            font-size: 12px;
            color: #909399;
          }

          .stats-value {
            font-size: 12px;
            color: #303133;
            font-weight: 600;
          }
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .history-data-chart {
    .chart-toolbar {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;

      .toolbar-left,
      .toolbar-right {
        justify-content: center;
      }
    }
  }
}

@media (max-width: 768px) {
  .history-data-chart {
    .chart-toolbar .toolbar-right .chart-controls {
      flex-direction: column;
      gap: 8px;
    }

    .chart-container {
      height: 400px;

      .chart-wrapper {
        .data-point-panel {
          position: relative;
          top: auto;
          right: auto;
          width: 100%;
          margin-top: 12px;
        }

        .chart-stats {
          position: relative;
          bottom: auto;
          left: auto;
          justify-content: center;
          margin-top: 12px;
          background: white;
          border: 1px solid #ebeef5;
        }
      }
    }
  }
}
</style>
