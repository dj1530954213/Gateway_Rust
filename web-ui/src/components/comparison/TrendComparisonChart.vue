<template>
  <div class="trend-comparison-chart">
    <!-- 图表控制工具栏 -->
    <div class="chart-toolbar">
      <div class="toolbar-left">
        <div class="chart-title">
          <h4>趋势对比分析</h4>
          <span class="chart-subtitle">{{ subtitle }}</span>
        </div>
      </div>

      <div class="toolbar-right">
        <el-button-group size="small">
          <el-button
            v-for="period in timePeriods"
            :key="period.value"
            :type="selectedPeriod === period.value ? 'primary' : ''"
            @click="changePeriod(period.value)"
          >
            {{ period.label }}
          </el-button>
        </el-button-group>

        <el-divider direction="vertical" />

        <el-dropdown @command="handleChartType">
          <el-button size="small">
            图表类型
            <el-icon><ArrowDown /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="line">折线图</el-dropdown-item>
              <el-dropdown-item command="area">面积图</el-dropdown-item>
              <el-dropdown-item command="bar">柱状图</el-dropdown-item>
              <el-dropdown-item command="scatter">散点图</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <el-dropdown @command="handleAnalysisType">
          <el-button size="small">
            分析方式
            <el-icon><ArrowDown /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="absolute">绝对值对比</el-dropdown-item>
              <el-dropdown-item command="percentage"
                >百分比对比</el-dropdown-item
              >
              <el-dropdown-item command="normalized"
                >标准化对比</el-dropdown-item
              >
              <el-dropdown-item command="difference">差值分析</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <el-button size="small" @click="showSettings = true">
          <el-icon><Setting /></el-icon>
          设置
        </el-button>
      </div>
    </div>

    <!-- 主图表区域 -->
    <div class="chart-main">
      <div ref="mainChartRef" class="main-chart"></div>
    </div>

    <!-- 统计面板 -->
    <div class="statistics-panel">
      <div class="stats-grid">
        <div
          v-for="stat in statistics"
          :key="stat.key"
          class="stat-item"
          :class="stat.trend"
        >
          <div class="stat-icon">
            <el-icon><component :is="stat.icon" /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stat.value }}</div>
            <div class="stat-label">{{ stat.label }}</div>
            <div class="stat-change">
              <el-icon><component :is="getTrendIcon(stat.trend)" /></el-icon>
              {{ stat.change }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 详细对比表格 -->
    <div class="comparison-table">
      <div class="table-header">
        <h5>详细对比数据</h5>
        <div class="table-controls">
          <el-button size="small" @click="exportData">
            <el-icon><Download /></el-icon>
            导出数据
          </el-button>
          <el-button size="small" @click="toggleTable">
            <el-icon><component :is="showTable ? 'View' : 'Hide'" /></el-icon>
            {{ showTable ? '隐藏' : '显示' }}表格
          </el-button>
        </div>
      </div>

      <el-table
        v-show="showTable"
        :data="comparisonData"
        stripe
        style="width: 100%"
        max-height="300"
      >
        <el-table-column prop="timestamp" label="时间" width="150" />
        <el-table-column
          v-for="series in chartSeries"
          :key="series.name"
          :prop="series.dataKey"
          :label="series.name"
          width="120"
        >
          <template #default="{ row }">
            <span :style="{ color: series.color }">
              {{ formatValue(row[series.dataKey]) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column
          v-if="analysisType === 'difference'"
          prop="difference"
          label="差值"
          width="120"
        >
          <template #default="{ row }">
            <span :class="getDifferenceClass(row.difference)">
              {{ formatValue(row.difference) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column
          v-if="analysisType === 'percentage'"
          prop="changeRate"
          label="变化率"
          width="120"
        >
          <template #default="{ row }">
            <span :class="getChangeClass(row.changeRate)">
              {{ formatPercentage(row.changeRate) }}
            </span>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- 设置对话框 -->
    <el-dialog v-model="showSettings" title="图表设置" width="500px">
      <el-form :model="chartSettings" label-width="100px">
        <el-form-item label="平滑曲线">
          <el-switch v-model="chartSettings.smooth" />
        </el-form-item>

        <el-form-item label="显示数据点">
          <el-switch v-model="chartSettings.showSymbol" />
        </el-form-item>

        <el-form-item label="填充区域">
          <el-switch v-model="chartSettings.areaStyle" />
        </el-form-item>

        <el-form-item label="显示网格">
          <el-switch v-model="chartSettings.showGrid" />
        </el-form-item>

        <el-form-item label="动画效果">
          <el-switch v-model="chartSettings.animation" />
        </el-form-item>

        <el-form-item label="标记线">
          <el-checkbox-group v-model="chartSettings.markLines">
            <el-checkbox label="average">平均线</el-checkbox>
            <el-checkbox label="max">最大值线</el-checkbox>
            <el-checkbox label="min">最小值线</el-checkbox>
          </el-checkbox-group>
        </el-form-item>

        <el-form-item label="Y轴范围">
          <el-radio-group v-model="chartSettings.yAxisType">
            <el-radio label="auto">自动</el-radio>
            <el-radio label="zero">从零开始</el-radio>
            <el-radio label="custom">自定义</el-radio>
          </el-radio-group>
        </el-form-item>

        <el-form-item
          v-if="chartSettings.yAxisType === 'custom'"
          label="自定义范围"
        >
          <el-row :gutter="8">
            <el-col :span="12">
              <el-input-number
                v-model="chartSettings.yAxisMin"
                placeholder="最小值"
                style="width: 100%"
              />
            </el-col>
            <el-col :span="12">
              <el-input-number
                v-model="chartSettings.yAxisMax"
                placeholder="最大值"
                style="width: 100%"
              />
            </el-col>
          </el-row>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="resetSettings">重置</el-button>
        <el-button type="primary" @click="applySettings">应用</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * TrendComparisonChart —— 趋势对比图表组件
 *
 * 📝 Responsibilities:
 *  1. 多数据源趋势对比可视化
 *  2. 多种分析方式和图表类型
 *  3. 统计指标计算和展示
 *  4. 数据导出和表格展示
 *
 * 📦 Dependencies:
 *  - ECharts 图表库
 *  - Element Plus UI组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  ArrowDown,
  Setting,
  Download,
  View,
  Hide,
  TrendCharts,
  DataAnalysis,
  Warning,
  InfoFilled,
  Top,
  Bottom,
  Minus,
} from '@element-plus/icons-vue'
import * as echarts from 'echarts'
import { ElMessage } from 'element-plus'
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'

// ===== Props =====
const props = defineProps<{
  data: any[]
  series: any[]
  title?: string
}>()

// ===== 响应式数据 =====
const selectedPeriod = ref('7d')
const chartType = ref('line')
const analysisType = ref('absolute')
const showSettings = ref(false)
const showTable = ref(false)

// 图表引用
const mainChartRef = ref<HTMLDivElement>()
const mainChart = ref<echarts.ECharts>()

// 时间周期选项
const timePeriods = ref([
  { label: '1天', value: '1d' },
  { label: '7天', value: '7d' },
  { label: '30天', value: '30d' },
  { label: '90天', value: '90d' },
])

// 图表设置
const chartSettings = ref({
  smooth: true,
  showSymbol: false,
  areaStyle: true,
  showGrid: true,
  animation: true,
  markLines: [],
  yAxisType: 'auto',
  yAxisMin: null,
  yAxisMax: null,
})

// ===== 计算属性 =====
const subtitle = computed(() => {
  const periodMap: { [key: string]: string } = {
    '1d': '最近24小时',
    '7d': '最近7天',
    '30d': '最近30天',
    '90d': '最近90天',
  }
  return periodMap[selectedPeriod.value] || ''
})

const chartSeries = computed(() => {
  return (
    props.series || [
      { name: '目标1', dataKey: 'value1', color: '#409EFF' },
      { name: '目标2', dataKey: 'value2', color: '#67C23A' },
    ]
  )
})

const statistics = computed(() => {
  // 计算统计指标
  if (!props.data || props.data.length === 0) {
    return []
  }

  const stats = []

  chartSeries.value.forEach((series, index) => {
    const values = props.data
      .map(item => item[series.dataKey])
      .filter(v => v !== null && v !== undefined)

    if (values.length > 0) {
      const avg = values.reduce((sum, val) => sum + val, 0) / values.length
      const max = Math.max(...values)
      const min = Math.min(...values)
      const latest = values[values.length - 1]
      const previous = values.length > 1 ? values[values.length - 2] : latest
      const change = (((latest - previous) / previous) * 100).toFixed(1)

      stats.push({
        key: `avg_${index}`,
        label: `${series.name}平均值`,
        value: avg.toFixed(2),
        change: `${change}%`,
        trend:
          parseFloat(change) > 0
            ? 'up'
            : parseFloat(change) < 0
              ? 'down'
              : 'flat',
        icon: 'DataAnalysis',
      })
    }
  })

  return stats
})

const comparisonData = computed(() => {
  if (!props.data || props.data.length === 0) return []

  return props.data.map(item => {
    const result = { ...item }

    // 计算差值
    if (chartSeries.value.length >= 2) {
      const val1 = item[chartSeries.value[0].dataKey]
      const val2 = item[chartSeries.value[1].dataKey]

      if (val1 !== null && val2 !== null) {
        result.difference = val1 - val2
        result.changeRate = ((val1 - val2) / val2) * 100
      }
    }

    return result
  })
})

// ===== 方法 =====

/**
 * 初始化图表
 */
function initChart() {
  if (!mainChartRef.value) return

  mainChart.value = echarts.init(mainChartRef.value)
  updateChart()

  // 监听窗口大小变化
  window.addEventListener('resize', handleResize)
}

/**
 * 更新图表
 */
function updateChart() {
  if (!mainChart.value || !props.data) return

  const option = generateChartOption()
  mainChart.value.setOption(option, true)
}

/**
 * 生成图表配置
 */
function generateChartOption() {
  const xAxisData = props.data.map(item => item.timestamp)

  let series = chartSeries.value.map(seriesConfig => {
    const data = props.data.map(item => item[seriesConfig.dataKey])

    const seriesOption: any = {
      name: seriesConfig.name,
      type: chartType.value,
      data,
      lineStyle: { color: seriesConfig.color, width: 2 },
      itemStyle: { color: seriesConfig.color },
      smooth: chartSettings.value.smooth,
      symbol: chartSettings.value.showSymbol ? 'circle' : 'none',
      symbolSize: 4,
    }

    // 面积图样式
    if (chartSettings.value.areaStyle && chartType.value === 'line') {
      seriesOption.areaStyle = {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: `${seriesConfig.color}40` },
          { offset: 1, color: `${seriesConfig.color}10` },
        ]),
      }
    }

    // 标记线
    if (chartSettings.value.markLines.length > 0) {
      seriesOption.markLine = {
        data: chartSettings.value.markLines
          .map(type => {
            if (type === 'average') return { type: 'average', name: '平均值' }
            if (type === 'max') return { type: 'max', name: '最大值' }
            if (type === 'min') return { type: 'min', name: '最小值' }
            return null
          })
          .filter(Boolean),
      }
    }

    return seriesOption
  })

  // 处理不同分析类型
  if (analysisType.value === 'percentage' && chartSeries.value.length >= 2) {
    // 百分比对比
    const baseData = props.data.map(item => item[chartSeries.value[0].dataKey])
    series = chartSeries.value.slice(1).map((seriesConfig, index) => {
      const data = props.data.map((item, i) => {
        const baseValue = baseData[i]
        const currentValue = item[seriesConfig.dataKey]
        return baseValue !== 0 ? (currentValue / baseValue) * 100 - 100 : 0
      })

      return {
        name: `${seriesConfig.name} vs ${chartSeries.value[0].name}`,
        type: chartType.value,
        data,
        lineStyle: { color: seriesConfig.color, width: 2 },
        itemStyle: { color: seriesConfig.color },
        smooth: chartSettings.value.smooth,
        symbol: chartSettings.value.showSymbol ? 'circle' : 'none',
      }
    })
  } else if (
    analysisType.value === 'difference' &&
    chartSeries.value.length >= 2
  ) {
    // 差值分析
    const data1 = props.data.map(item => item[chartSeries.value[0].dataKey])
    const data2 = props.data.map(item => item[chartSeries.value[1].dataKey])

    series = [
      {
        name: `差值 (${chartSeries.value[0].name} - ${chartSeries.value[1].name})`,
        type: chartType.value,
        data: data1.map((val, index) => val - data2[index]),
        lineStyle: { color: '#E6A23C', width: 2 },
        itemStyle: { color: '#E6A23C' },
        smooth: chartSettings.value.smooth,
        symbol: chartSettings.value.showSymbol ? 'circle' : 'none',
        markLine: {
          data: [{ yAxis: 0, lineStyle: { color: '#F56C6C', type: 'dashed' } }],
        },
      },
    ]
  } else if (analysisType.value === 'normalized') {
    // 标准化对比
    series = chartSeries.value.map(seriesConfig => {
      const data = props.data.map(item => item[seriesConfig.dataKey])
      const mean = data.reduce((sum, val) => sum + val, 0) / data.length
      const std = Math.sqrt(
        data.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) /
          data.length
      )

      const normalizedData = data.map(val => (val - mean) / std)

      return {
        name: `${seriesConfig.name} (标准化)`,
        type: chartType.value,
        data: normalizedData,
        lineStyle: { color: seriesConfig.color, width: 2 },
        itemStyle: { color: seriesConfig.color },
        smooth: chartSettings.value.smooth,
        symbol: chartSettings.value.showSymbol ? 'circle' : 'none',
      }
    })
  }

  // Y轴配置
  const yAxis: any = {
    type: 'value',
    name:
      analysisType.value === 'percentage'
        ? '变化率 (%)'
        : analysisType.value === 'difference'
          ? '差值'
          : analysisType.value === 'normalized'
            ? '标准化值'
            : '数值',
    nameTextStyle: { fontSize: 12 },
    splitLine: {
      show: chartSettings.value.showGrid,
      lineStyle: { type: 'dashed', color: '#e0e6ed' },
    },
  }

  if (chartSettings.value.yAxisType === 'zero') {
    yAxis.min = 0
  } else if (chartSettings.value.yAxisType === 'custom') {
    if (chartSettings.value.yAxisMin !== null)
      yAxis.min = chartSettings.value.yAxisMin
    if (chartSettings.value.yAxisMax !== null)
      yAxis.max = chartSettings.value.yAxisMax
  }

  return {
    animation: chartSettings.value.animation,
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      top: '12%',
      containLabel: true,
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross',
        label: { backgroundColor: '#6a7985' },
      },
      formatter(params: any[]) {
        let tooltip = `<div style="margin-bottom: 5px;">${params[0].axisValue}</div>`

        params.forEach(param => {
          tooltip += `
            <div style="display: flex; align-items: center; margin-bottom: 3px;">
              <span style="display: inline-block; width: 10px; height: 10px; background-color: ${param.color}; border-radius: 50%; margin-right: 8px;"></span>
              <span style="flex: 1;">${param.seriesName}: </span>
              <span style="font-weight: bold;">${formatValue(param.value)}</span>
            </div>
          `
        })

        return tooltip
      },
    },
    legend: {
      data: series.map(s => s.name),
      top: 0,
      textStyle: { fontSize: 12 },
    },
    xAxis: {
      type: 'category',
      data: xAxisData,
      boundaryGap: false,
      axisLine: { lineStyle: { color: '#d4edda' } },
      axisTick: { show: false },
      axisLabel: {
        color: '#666',
        formatter(value: string) {
          return formatTimeLabel(value)
        },
      },
      splitLine: {
        show: chartSettings.value.showGrid,
        lineStyle: { type: 'dashed', color: '#e0e6ed' },
      },
    },
    yAxis,
    series,
  }
}

/**
 * 切换时间周期
 */
function changePeriod(period: string) {
  selectedPeriod.value = period
  // 触发数据重新加载
  ElMessage.info(`切换到${subtitle.value}数据`)
}

/**
 * 处理图表类型变更
 */
function handleChartType(command: string) {
  chartType.value = command
  updateChart()
}

/**
 * 处理分析类型变更
 */
function handleAnalysisType(command: string) {
  analysisType.value = command
  updateChart()
}

/**
 * 应用设置
 */
function applySettings() {
  showSettings.value = false
  updateChart()
  ElMessage.success('设置已应用')
}

/**
 * 重置设置
 */
function resetSettings() {
  chartSettings.value = {
    smooth: true,
    showSymbol: false,
    areaStyle: true,
    showGrid: true,
    animation: true,
    markLines: [],
    yAxisType: 'auto',
    yAxisMin: null,
    yAxisMax: null,
  }
}

/**
 * 切换表格显示
 */
function toggleTable() {
  showTable.value = !showTable.value
}

/**
 * 导出数据
 */
function exportData() {
  // 实现数据导出逻辑
  ElMessage.success('数据导出功能开发中...')
}

/**
 * 获取趋势图标
 */
function getTrendIcon(trend: string): string {
  const iconMap: { [key: string]: string } = {
    up: 'Top',
    down: 'Bottom',
    flat: 'Minus',
  }
  return iconMap[trend] || 'Minus'
}

/**
 * 格式化数值
 */
function formatValue(value: number): string {
  if (value === null || value === undefined) return '--'
  return value.toFixed(2)
}

/**
 * 格式化百分比
 */
function formatPercentage(value: number): string {
  if (value === null || value === undefined) return '--'
  return `${value.toFixed(1)}%`
}

/**
 * 格式化时间标签
 */
function formatTimeLabel(timestamp: string): string {
  const date = new Date(timestamp)
  if (selectedPeriod.value === '1d') {
    return date.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
    })
  } else {
    return date.toLocaleDateString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
    })
  }
}

/**
 * 获取差值样式类
 */
function getDifferenceClass(value: number): string {
  if (value > 0) return 'positive-diff'
  if (value < 0) return 'negative-diff'
  return 'neutral-diff'
}

/**
 * 获取变化率样式类
 */
function getChangeClass(value: number): string {
  if (value > 5) return 'high-increase'
  if (value > 0) return 'increase'
  if (value < -5) return 'high-decrease'
  if (value < 0) return 'decrease'
  return 'stable'
}

/**
 * 处理窗口大小变化
 */
function handleResize() {
  mainChart.value?.resize()
}

// ===== 生命周期 =====
onMounted(() => {
  nextTick(() => {
    initChart()
  })
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  mainChart.value?.dispose()
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
  () => props.series,
  () => {
    updateChart()
  },
  { deep: true }
)
</script>

<style scoped lang="scss">
.trend-comparison-chart {
  .chart-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 0;
    border-bottom: 1px solid #ebeef5;
    margin-bottom: 16px;

    .toolbar-left {
      .chart-title {
        h4 {
          margin: 0 0 4px 0;
          color: #303133;
          font-size: 16px;
        }

        .chart-subtitle {
          font-size: 12px;
          color: #909399;
        }
      }
    }

    .toolbar-right {
      display: flex;
      align-items: center;
      gap: 8px;
    }
  }

  .chart-main {
    .main-chart {
      height: 400px;
      width: 100%;
    }
  }

  .statistics-panel {
    margin: 20px 0;

    .stats-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 16px;

      .stat-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 16px;
        background: #f8f9fa;
        border-radius: 8px;
        border-left: 4px solid transparent;

        &.up {
          border-left-color: #67c23a;
        }

        &.down {
          border-left-color: #f56c6c;
        }

        &.flat {
          border-left-color: #909399;
        }

        .stat-icon {
          font-size: 24px;
          color: #409eff;
        }

        .stat-content {
          flex: 1;

          .stat-value {
            font-size: 18px;
            font-weight: 600;
            color: #303133;
            margin-bottom: 2px;
          }

          .stat-label {
            font-size: 12px;
            color: #606266;
            margin-bottom: 4px;
          }

          .stat-change {
            display: flex;
            align-items: center;
            gap: 4px;
            font-size: 11px;
            color: #909399;
          }
        }
      }
    }
  }

  .comparison-table {
    margin-top: 20px;

    .table-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 12px;

      h5 {
        margin: 0;
        color: #303133;
        font-size: 14px;
        font-weight: 600;
      }

      .table-controls {
        display: flex;
        gap: 8px;
      }
    }

    :deep(.el-table) {
      .positive-diff {
        color: #67c23a;
        font-weight: 500;
      }

      .negative-diff {
        color: #f56c6c;
        font-weight: 500;
      }

      .neutral-diff {
        color: #909399;
      }

      .high-increase {
        color: #67c23a;
        font-weight: 600;
      }

      .increase {
        color: #67c23a;
      }

      .high-decrease {
        color: #f56c6c;
        font-weight: 600;
      }

      .decrease {
        color: #f56c6c;
      }

      .stable {
        color: #909399;
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .trend-comparison-chart {
    .statistics-panel .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}

@media (max-width: 768px) {
  .trend-comparison-chart {
    .chart-toolbar {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;

      .toolbar-right {
        justify-content: center;
        flex-wrap: wrap;
      }
    }

    .statistics-panel .stats-grid {
      grid-template-columns: 1fr;
    }

    .comparison-table .table-header {
      flex-direction: column;
      gap: 8px;
      align-items: stretch;

      .table-controls {
        justify-content: center;
      }
    }
  }
}
</style>
