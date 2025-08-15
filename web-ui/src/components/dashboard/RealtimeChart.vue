<template>
  <div class="realtime-chart">
    <!-- 图表控制栏 -->
    <div class="chart-toolbar">
      <div class="tag-selector">
        <el-select
          v-model="selectedTags"
          multiple
          placeholder="选择要显示的点位"
          style="width: 300px"
          @change="handleTagChange"
        >
          <el-option
            v-for="tag in availableTags"
            :key="tag.id"
            :label="tag.name"
            :value="tag.id"
          >
            <span class="tag-option">
              <span class="tag-name">{{ tag.name }}</span>
              <span class="tag-address">{{ tag.address }}</span>
            </span>
          </el-option>
        </el-select>
      </div>

      <div class="chart-actions">
        <el-button-group size="small">
          <el-button
            :type="refreshing ? 'primary' : 'default'"
            :loading="refreshing"
            @click="refreshData"
          >
            <el-icon><Refresh /></el-icon>
          </el-button>
          <el-button @click="exportChart">
            <el-icon><Download /></el-icon>
          </el-button>
          <el-button @click="toggleFullscreen">
            <el-icon><FullScreen /></el-icon>
          </el-button>
        </el-button-group>
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
                style="width: 100%; height: 300px; margin-top: 16px"
              />
            </div>
          </template>
        </el-skeleton>
      </div>

      <div v-else-if="!hasData" class="chart-empty">
        <el-empty description="暂无数据" :image-size="120">
          <template #description>
            <p>请选择要显示的数据点位</p>
          </template>
          <el-button type="primary" @click="selectDefaultTags">
            选择默认点位
          </el-button>
        </el-empty>
      </div>

      <div v-else class="chart-wrapper">
        <VChart
          class="chart"
          :option="chartOption"
          :loading="refreshing"
          autoresize
          @click="handleChartClick"
          @legendselectchanged="handleLegendChange"
        />

        <!-- 数据点信息面板 -->
        <div v-if="selectedPoint" class="point-info-panel">
          <div class="panel-header">
            <span>{{ selectedPoint.name }}</span>
            <el-button type="text" size="small" @click="selectedPoint = null">
              <el-icon><Close /></el-icon>
            </el-button>
          </div>
          <div class="panel-content">
            <div class="info-item">
              <span class="label">当前值:</span>
              <span class="value">{{ selectedPoint.currentValue }}</span>
            </div>
            <div class="info-item">
              <span class="label">单位:</span>
              <span class="value">{{ selectedPoint.unit || '无' }}</span>
            </div>
            <div class="info-item">
              <span class="label">更新时间:</span>
              <span class="value">{{
                formatDateTime(selectedPoint.updateTime)
              }}</span>
            </div>
            <div class="info-item">
              <span class="label">地址:</span>
              <span class="value">{{ selectedPoint.address }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * RealtimeChart —— 实时数据图表组件
 *
 * 📝 Responsibilities:
 *  1. 显示多个点位的实时数据趋势
 *  2. 支持点位选择和图例控制
 *  3. 提供数据导出功能
 *  4. 响应WebSocket实时数据更新
 *  5. 支持全屏显示
 *
 * 📦 Dependencies:
 *  - ECharts 图表库
 *  - WebSocket 实时数据
 *  - 标签Store
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { Refresh, Download, FullScreen, Close } from '@element-plus/icons-vue'
import { LineChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
  ToolboxComponent,
} from 'echarts/components'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { ElMessage } from 'element-plus'
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import VChart from 'vue-echarts'

import { historyApi } from '@/api'
import type { TagVO } from '@/api/tags'
import { useTagsStore } from '@/stores'
import { formatDateTime } from '@/utils/date'

// 注册ECharts组件
use([
  CanvasRenderer,
  LineChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
  ToolboxComponent,
])

// ===== Props & Emits =====
const props = defineProps<{
  timeRange: string
  selectedTags: string[]
  loading?: boolean
}>()

const emit = defineEmits<{
  'tag-change': [tags: string[]]
}>()

// ===== Store =====
const tagsStore = useTagsStore()

// ===== 响应式数据 =====
const chartContainer = ref<HTMLElement>()
const selectedTags = ref<string[]>([])
const refreshing = ref(false)
const isFullscreen = ref(false)
const selectedPoint = ref<any>(null)

// 图表数据
const chartData = ref<Record<string, Array<{ time: string; value: number }>>>(
  {}
)
const availableTags = ref<TagVO[]>([])

// ===== 计算属性 =====
const hasData = computed(() => {
  return (
    selectedTags.value.length > 0 && Object.keys(chartData.value).length > 0
  )
})

const chartOption = computed(() => {
  if (!hasData.value) return {}

  // 生成时间轴数据
  const timeLabels = generateTimeLabels()

  // 生成系列数据
  const series = selectedTags.value.map((tagId, index) => {
    const tag = availableTags.value.find(t => t.id === tagId)
    const data = chartData.value[tagId] || []

    return {
      name: tag?.name || tagId,
      type: 'line',
      smooth: true,
      symbol: 'circle',
      symbolSize: 4,
      data: data.map(item => item.value),
      itemStyle: {
        color: getChartColor(index),
      },
      lineStyle: {
        width: 2,
      },
      areaStyle: {
        opacity: 0.1,
      },
    }
  })

  return {
    title: {
      text: '实时数据趋势',
      left: 'center',
      textStyle: {
        fontSize: 16,
        fontWeight: 'normal',
      },
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

        const time = timeLabels[params[0].dataIndex]
        let content = `<div><strong>${time}</strong></div>`

        params.forEach(param => {
          const tag = availableTags.value.find(t => t.name === param.seriesName)
          const unit = tag?.unit || ''
          content += `
            <div style="margin-top: 4px;">
              <span style="color: ${param.color}">●</span>
              ${param.seriesName}: <strong>${param.value}${unit}</strong>
            </div>
          `
        })

        return content
      },
    },
    legend: {
      top: 30,
      type: 'scroll',
      data: series.map(s => s.name),
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '10%',
      top: '15%',
      containLabel: true,
    },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: timeLabels,
      axisLabel: {
        formatter: (value: string) => {
          return value.split(' ')[1] // 只显示时间部分
        },
      },
    },
    yAxis: {
      type: 'value',
      scale: true,
      axisLabel: {
        formatter: '{value}',
      },
    },
    dataZoom: [
      {
        type: 'inside',
        start: 70,
        end: 100,
      },
      {
        type: 'slider',
        start: 70,
        end: 100,
        height: 20,
        bottom: 20,
      },
    ],
    toolbox: {
      feature: {
        saveAsImage: {
          title: '保存图片',
        },
      },
      right: 20,
    },
    series,
  }
})

// ===== 方法 =====

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
  ]
  return colors[index % colors.length]
}

/**
 * 生成时间标签
 */
function generateTimeLabels(): string[] {
  const labels: string[] = []
  const now = new Date()
  const interval = getTimeInterval()
  const count = getDataPointCount()

  for (let i = count - 1; i >= 0; i--) {
    const time = new Date(now.getTime() - i * interval)
    labels.push(formatDateTime(time))
  }

  return labels
}

/**
 * 获取时间间隔（毫秒）
 */
function getTimeInterval(): number {
  switch (props.timeRange) {
    case '1h':
      return 5 * 60 * 1000 // 5分钟
    case '6h':
      return 30 * 60 * 1000 // 30分钟
    case '24h':
      return 60 * 60 * 1000 // 1小时
    default:
      return 5 * 60 * 1000
  }
}

/**
 * 获取数据点数量
 */
function getDataPointCount(): number {
  switch (props.timeRange) {
    case '1h':
      return 12 // 1小时/5分钟
    case '6h':
      return 12 // 6小时/30分钟
    case '24h':
      return 24 // 24小时/1小时
    default:
      return 12
  }
}

/**
 * 获取时间范围毫秒数
 */
function getTimeRangeMs(): number {
  switch (props.timeRange) {
    case '1h':
      return 60 * 60 * 1000
    case '6h':
      return 6 * 60 * 60 * 1000
    case '24h':
      return 24 * 60 * 60 * 1000
    default:
      return 60 * 60 * 1000
  }
}

/**
 * 处理标签变更
 */
function handleTagChange() {
  emit('tag-change', selectedTags.value)
  fetchRealTimeData()
}

/**
 * 刷新数据
 */
async function refreshData() {
  refreshing.value = true
  try {
    await fetchRealTimeData()
    ElMessage.success('数据已刷新')
  } catch (error) {
    console.error('刷新数据失败:', error)
    ElMessage.error('刷新数据失败')
  } finally {
    refreshing.value = false
  }
}

/**
 * 导出图表
 */
function exportChart() {
  ElMessage.info('图表导出功能开发中...')
}

/**
 * 切换全屏
 */
function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value

  nextTick(() => {
    // 触发图表重新渲染
    if (chartContainer.value) {
      const resizeEvent = new Event('resize')
      window.dispatchEvent(resizeEvent)
    }
  })
}

/**
 * 处理图表点击
 */
function handleChartClick(params: any) {
  if (params.componentType === 'series') {
    const tagId = selectedTags.value[params.seriesIndex]
    const tag = availableTags.value.find(t => t.id === tagId)

    if (tag) {
      selectedPoint.value = {
        id: tag.id,
        name: tag.name,
        address: tag.address,
        unit: tag.unit,
        currentValue: params.value,
        updateTime: new Date().toISOString(),
      }
    }
  }
}

/**
 * 处理图例变更
 */
function handleLegendChange(params: any) {
  console.log('Legend changed:', params)
}

/**
 * 选择默认标签
 */
function selectDefaultTags() {
  if (availableTags.value.length > 0) {
    selectedTags.value = availableTags.value.slice(0, 3).map(tag => tag.id)
    handleTagChange()
  }
}

/**
 * 获取实时数据
 */
async function fetchRealTimeData() {
  if (selectedTags.value.length === 0) {
    chartData.value = {}
    return
  }

  try {
    const endTime = new Date()
    const startTime = new Date(endTime.getTime() - getTimeRangeMs())

    const queryParams = {
      tag_ids: selectedTags.value,
      start_time: startTime.toISOString(),
      end_time: endTime.toISOString(),
      limit: getDataPointCount(),
    }

    const response = await historyApi.getRealtimeData(queryParams)

    const newData: Record<string, Array<{ time: string; value: number }>> = {}

    response.data.forEach((item: any) => {
      if (!newData[item.tag_id]) {
        newData[item.tag_id] = []
      }
      newData[item.tag_id].push({
        time: item.timestamp,
        value: item.value,
      })
    })

    chartData.value = newData
  } catch (error) {
    console.error('获取实时数据失败:', error)
    ElMessage.error('获取实时数据失败')
  }
}

/**
 * 加载可用标签
 */
async function loadAvailableTags() {
  try {
    await tagsStore.fetchTags({ page: 1, size: 100 })
    availableTags.value = tagsStore.enabledTags.slice(0, 20)
  } catch (error) {
    console.error('加载标签失败:', error)
  }
}

// ===== 生命周期 =====
onMounted(async () => {
  await loadAvailableTags()

  // 监听ESC键退出全屏
  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape' && isFullscreen.value) {
      toggleFullscreen()
    }
  }

  document.addEventListener('keydown', handleKeydown)

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })
})

// ===== 监听器 =====
watch(
  () => props.selectedTags,
  newTags => {
    selectedTags.value = [...newTags]
    fetchRealTimeData()
  },
  { immediate: true }
)

watch(
  () => props.timeRange,
  () => {
    fetchRealTimeData()
  }
)
</script>

<style scoped lang="scss">
.realtime-chart {
  .chart-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    padding: 0 16px;

    .tag-selector {
      .tag-option {
        display: flex;
        justify-content: space-between;
        align-items: center;
        width: 100%;

        .tag-name {
          font-weight: 500;
        }

        .tag-address {
          font-size: 12px;
          color: #909399;
          font-family: monospace;
        }
      }
    }

    .chart-actions {
      display: flex;
      gap: 8px;
    }
  }

  .chart-container {
    position: relative;
    height: 400px;
    border: 1px solid #ebeef5;
    border-radius: 4px;

    &.fullscreen {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      z-index: 2000;
      background: white;
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

      .point-info-panel {
        position: absolute;
        top: 20px;
        right: 20px;
        width: 280px;
        background: white;
        border: 1px solid #ebeef5;
        border-radius: 4px;
        box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
        z-index: 10;

        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          border-bottom: 1px solid #ebeef5;
          background: #f5f7fa;

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
            }
          }
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .realtime-chart {
    .chart-toolbar {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;

      .tag-selector {
        width: 100%;

        :deep(.el-select) {
          width: 100% !important;
        }
      }

      .chart-actions {
        justify-content: center;
      }
    }

    .chart-container {
      height: 300px;

      .point-info-panel {
        position: relative;
        top: auto;
        right: auto;
        width: 100%;
        margin-top: 12px;
      }
    }
  }
}
</style>
