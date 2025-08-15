<template>
  <div class="data-quality-chart">
    <div ref="chartRef" class="chart-container"></div>

    <!-- 质量指标 -->
    <div class="quality-metrics">
      <div v-for="(value, key) in metrics" :key="key" class="metric-item">
        <div class="metric-label">{{ getMetricLabel(key) }}</div>
        <div class="metric-value" :class="getMetricClass(value)">
          {{ value.toFixed(1) }}%
        </div>
        <div class="metric-bar">
          <div
            class="metric-progress"
            :class="getMetricClass(value)"
            :style="{ width: `${value}%` }"
          ></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * DataQualityChart —— 数据质量分析图表组件
 *
 * 📝 Responsibilities:
 *  1. 数据质量趋势的可视化展示
 *  2. 多维度质量指标监控
 *  3. 质量阈值告警显示
 *
 * 📦 Dependencies:
 *  - ECharts 图表库
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import * as echarts from 'echarts'
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'

// ===== Props =====
const props = defineProps<{
  data: Array<{
    timestamp: string
    completeness: number
    accuracy: number
    consistency: number
    timeliness: number
  }>
  metrics: {
    completeness: number
    accuracy: number
    consistency: number
    timeliness: number
  }
}>()

// ===== 响应式数据 =====
const chartRef = ref<HTMLDivElement>()
const chartInstance = ref<echarts.ECharts>()

// ===== 计算属性 =====
const chartOptions = computed(() => ({
  animation: true,
  animationDuration: 1000,
  grid: {
    left: '3%',
    right: '4%',
    bottom: '8%',
    top: '10%',
    containLabel: true,
  },
  tooltip: {
    trigger: 'axis',
    axisPointer: {
      type: 'cross',
    },
    formatter(params: any[]) {
      let tooltip = `<div style="margin-bottom: 5px;">${formatTime(params[0].axisValue)}</div>`

      params.forEach(param => {
        tooltip += `
          <div style="display: flex; align-items: center; margin-bottom: 3px;">
            <span style="display: inline-block; width: 10px; height: 10px; background-color: ${param.color}; border-radius: 50%; margin-right: 8px;"></span>
            <span style="flex: 1;">${param.seriesName}: </span>
            <span style="font-weight: bold;">${param.value.toFixed(1)}%</span>
          </div>
        `
      })

      return tooltip
    },
  },
  legend: {
    data: ['完整性', '准确性', '一致性', '及时性'],
    top: 0,
    textStyle: {
      fontSize: 12,
    },
  },
  xAxis: {
    type: 'time',
    boundaryGap: false,
    axisLabel: {
      formatter(value: number) {
        return formatTime(value)
      },
    },
  },
  yAxis: {
    type: 'value',
    name: '质量得分 (%)',
    min: 80,
    max: 100,
    axisLabel: {
      formatter: '{value}%',
    },
    splitLine: {
      lineStyle: {
        type: 'dashed',
      },
    },
  },
  series: [
    {
      name: '完整性',
      type: 'line',
      smooth: true,
      symbol: 'circle',
      symbolSize: 4,
      lineStyle: {
        color: '#67C23A',
        width: 2,
      },
      itemStyle: {
        color: '#67C23A',
      },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: '#67C23A40' },
          { offset: 1, color: '#67C23A10' },
        ]),
      },
      data:
        props.data?.map(point => [
          new Date(point.timestamp).getTime(),
          point.completeness,
        ]) || [],
    },
    {
      name: '准确性',
      type: 'line',
      smooth: true,
      symbol: 'circle',
      symbolSize: 4,
      lineStyle: {
        color: '#409EFF',
        width: 2,
      },
      itemStyle: {
        color: '#409EFF',
      },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: '#409EFF40' },
          { offset: 1, color: '#409EFF10' },
        ]),
      },
      data:
        props.data?.map(point => [
          new Date(point.timestamp).getTime(),
          point.accuracy,
        ]) || [],
    },
    {
      name: '一致性',
      type: 'line',
      smooth: true,
      symbol: 'circle',
      symbolSize: 4,
      lineStyle: {
        color: '#E6A23C',
        width: 2,
      },
      itemStyle: {
        color: '#E6A23C',
      },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: '#E6A23C40' },
          { offset: 1, color: '#E6A23C10' },
        ]),
      },
      data:
        props.data?.map(point => [
          new Date(point.timestamp).getTime(),
          point.consistency,
        ]) || [],
    },
    {
      name: '及时性',
      type: 'line',
      smooth: true,
      symbol: 'circle',
      symbolSize: 4,
      lineStyle: {
        color: '#F56C6C',
        width: 2,
      },
      itemStyle: {
        color: '#F56C6C',
      },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: '#F56C6C40' },
          { offset: 1, color: '#F56C6C10' },
        ]),
      },
      data:
        props.data?.map(point => [
          new Date(point.timestamp).getTime(),
          point.timeliness,
        ]) || [],
    },
  ],
  // 添加质量阈值线
  markLine: {
    data: [
      {
        name: '警告阈值',
        yAxis: 90,
        lineStyle: {
          color: '#E6A23C',
          type: 'dashed',
        },
        label: {
          formatter: '警告: 90%',
        },
      },
      {
        name: '危险阈值',
        yAxis: 85,
        lineStyle: {
          color: '#F56C6C',
          type: 'dashed',
        },
        label: {
          formatter: '危险: 85%',
        },
      },
    ],
  },
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
 * 获取指标标签
 */
function getMetricLabel(key: string): string {
  const labels: { [key: string]: string } = {
    completeness: '完整性',
    accuracy: '准确性',
    consistency: '一致性',
    timeliness: '及时性',
  }
  return labels[key] || key
}

/**
 * 获取指标样式类
 */
function getMetricClass(value: number): string {
  if (value >= 95) return 'excellent'
  if (value >= 90) return 'good'
  if (value >= 85) return 'warning'
  return 'danger'
}

/**
 * 格式化时间
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
  () => props.metrics,
  () => {
    updateChart()
  },
  { deep: true }
)
</script>

<style scoped lang="scss">
.data-quality-chart {
  .chart-container {
    width: 100%;
    height: 200px;
  }

  .quality-metrics {
    margin-top: 16px;
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;

    .metric-item {
      padding: 8px;
      border-radius: 6px;
      background: #fafafa;

      .metric-label {
        font-size: 12px;
        color: #606266;
        margin-bottom: 4px;
      }

      .metric-value {
        font-size: 18px;
        font-weight: 600;
        margin-bottom: 6px;

        &.excellent {
          color: #67c23a;
        }

        &.good {
          color: #409eff;
        }

        &.warning {
          color: #e6a23c;
        }

        &.danger {
          color: #f56c6c;
        }
      }

      .metric-bar {
        height: 4px;
        background: #e4e7ed;
        border-radius: 2px;
        overflow: hidden;

        .metric-progress {
          height: 100%;
          border-radius: 2px;
          transition: width 0.3s ease;

          &.excellent {
            background: #67c23a;
          }

          &.good {
            background: #409eff;
          }

          &.warning {
            background: #e6a23c;
          }

          &.danger {
            background: #f56c6c;
          }
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .data-quality-chart {
    .quality-metrics {
      grid-template-columns: 1fr;
    }
  }
}
</style>
