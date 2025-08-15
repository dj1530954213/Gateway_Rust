<template>
  <div ref="containerRef" class="chart-container" :class="containerClass">
    <!-- 图表头部 -->
    <div v-if="showHeader" class="chart-header">
      <div class="header-left">
        <div class="chart-title">
          <el-icon v-if="titleIcon" :size="16">
            <component :is="titleIcon" />
          </el-icon>
          <span>{{ title }}</span>
        </div>

        <div v-if="subtitle" class="chart-subtitle">
          {{ subtitle }}
        </div>
      </div>

      <div class="header-right">
        <!-- 图表类型切换 -->
        <el-dropdown
          v-if="showTypeSwitch"
          trigger="click"
          @command="handleTypeChange"
        >
          <el-button type="text" size="small">
            <el-icon><TrendCharts /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                v-for="type in availableTypes"
                :key="type.value"
                :command="type.value"
                :icon="type.icon"
              >
                {{ type.label }}
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <!-- 刷新按钮 -->
        <el-button
          v-if="showRefresh"
          type="text"
          size="small"
          :icon="Refresh"
          :loading="refreshing"
          @click="handleRefresh"
        />

        <!-- 设置按钮 -->
        <el-button
          v-if="showSettings"
          type="text"
          size="small"
          :icon="Setting"
          @click="handleSettings"
        />

        <!-- 更多操作 -->
        <el-dropdown v-if="showActions" trigger="click" @command="handleAction">
          <el-button type="text" size="small" :icon="MoreFilled" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="export" :icon="Download">
                导出图表
              </el-dropdown-item>
              <el-dropdown-item command="fullscreen" :icon="FullScreen">
                全屏显示
              </el-dropdown-item>
              <el-dropdown-item command="reset" :icon="RefreshLeft">
                重置视图
              </el-dropdown-item>
              <el-dropdown-item command="save" :icon="Collection">
                保存配置
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- 图表内容区域 -->
    <div class="chart-content" :style="contentStyle">
      <!-- 加载状态 -->
      <LoadingCard
        v-if="loading"
        status="loading"
        :loading-text="loadingText"
        :show-progress="showProgress"
        :progress="loadingProgress"
        :animation-type="loadingAnimation"
        :min-height="chartHeight"
      />

      <!-- 错误状态 -->
      <LoadingCard
        v-else-if="error"
        status="error"
        :error-message="error"
        :show-retry="true"
        :min-height="chartHeight"
        @retry="handleRetry"
      />

      <!-- 空数据状态 -->
      <LoadingCard
        v-else-if="!hasData"
        status="empty"
        :empty-message="emptyMessage"
        :min-height="chartHeight"
      />

      <!-- 图表渲染区域 -->
      <div
        v-else
        ref="chartRef"
        class="chart-wrapper"
        :style="{ height: chartHeight }"
      >
        <!-- 这里将集成 ECharts 或其他图表库 -->
        <div class="chart-placeholder">
          <el-icon :size="48" style="color: var(--el-text-color-placeholder)">
            <TrendCharts />
          </el-icon>
          <div style="margin-top: 8px; color: var(--el-text-color-secondary)">
            {{ `${chartType}图表 - 数据点: ${dataPoints}` }}
          </div>
        </div>
      </div>
    </div>

    <!-- 图表工具栏 -->
    <div v-if="showToolbar && !loading && !error" class="chart-toolbar">
      <!-- 时间范围选择 -->
      <div v-if="showTimeRange" class="toolbar-item">
        <TimeRangePicker
          v-model="timeRange"
          size="small"
          @change="handleTimeRangeChange"
        />
      </div>

      <!-- 数据点选择 -->
      <div v-if="showDataPointSelector" class="toolbar-item">
        <el-button size="small" @click="showDataPointDialog = true">
          选择数据点 ({{ selectedDataPoints.length }})
        </el-button>
      </div>

      <!-- 图表选项 -->
      <div class="toolbar-item">
        <el-checkbox-group v-model="chartOptions" size="small">
          <el-checkbox v-if="supportLegend" label="legend">图例</el-checkbox>
          <el-checkbox v-if="supportGrid" label="grid">网格</el-checkbox>
          <el-checkbox v-if="supportTooltip" label="tooltip">提示</el-checkbox>
          <el-checkbox v-if="supportZoom" label="zoom">缩放</el-checkbox>
        </el-checkbox-group>
      </div>

      <!-- 实时更新控制 -->
      <div v-if="supportRealtime" class="toolbar-item">
        <el-switch
          v-model="realtimeEnabled"
          active-text="实时"
          size="small"
          @change="handleRealtimeToggle"
        />

        <el-select
          v-if="realtimeEnabled"
          v-model="realtimeInterval"
          size="small"
          style="width: 80px; margin-left: 8px"
          @change="handleIntervalChange"
        >
          <el-option label="1s" :value="1000" />
          <el-option label="5s" :value="5000" />
          <el-option label="10s" :value="10000" />
          <el-option label="30s" :value="30000" />
          <el-option label="1m" :value="60000" />
        </el-select>
      </div>
    </div>

    <!-- 图表配置对话框 -->
    <el-dialog
      v-model="settingsVisible"
      title="图表配置"
      width="600px"
      :before-close="handleSettingsClose"
    >
      <el-tabs v-model="settingsTab">
        <!-- 基本配置 -->
        <el-tab-pane label="基本配置" name="basic">
          <BaseForm
            v-model="chartConfig.basic"
            :fields="basicConfigFields"
            label-width="100px"
          />
        </el-tab-pane>

        <!-- 样式配置 -->
        <el-tab-pane label="样式配置" name="style">
          <BaseForm
            v-model="chartConfig.style"
            :fields="styleConfigFields"
            label-width="100px"
          />
        </el-tab-pane>

        <!-- 数据配置 -->
        <el-tab-pane label="数据配置" name="data">
          <BaseForm
            v-model="chartConfig.data"
            :fields="dataConfigFields"
            label-width="100px"
          />
        </el-tab-pane>
      </el-tabs>

      <template #footer>
        <el-button @click="settingsVisible = false">取消</el-button>
        <el-button type="primary" @click="handleConfigSave">
          保存配置
        </el-button>
      </template>
    </el-dialog>

    <!-- 数据点选择对话框 -->
    <el-dialog
      v-model="showDataPointDialog"
      title="选择数据点"
      width="800px"
      :before-close="handleDataPointDialogClose"
    >
      <DataPointSelector
        v-model="selectedDataPoints"
        :data-points="availableDataPoints"
        multiple
        @change="handleDataPointsChange"
      />

      <template #footer>
        <el-button @click="showDataPointDialog = false">取消</el-button>
        <el-button type="primary" @click="handleDataPointsConfirm">
          确定
        </el-button>
      </template>
    </el-dialog>

    <!-- 全屏显示 -->
    <el-dialog
      v-model="fullscreenVisible"
      :title="title"
      width="90%"
      top="5vh"
      :show-close="true"
      :close-on-click-modal="false"
      class="fullscreen-chart-dialog"
    >
      <div class="fullscreen-chart" :style="{ height: '70vh' }">
        <!-- 全屏图表内容 -->
        <div ref="fullscreenChartRef" style="width: 100%; height: 100%">
          <div class="chart-placeholder">
            <el-icon :size="64" style="color: var(--el-text-color-placeholder)">
              <TrendCharts />
            </el-icon>
            <div
              style="margin-top: 12px; color: var(--el-text-color-secondary)"
            >
              全屏模式 - {{ `${chartType}图表` }}
            </div>
          </div>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import {
  TrendCharts,
  Refresh,
  Setting,
  MoreFilled,
  Download,
  FullScreen,
  RefreshLeft,
  Collection,
  // PieChart,     // 这些图表图标不存在，先移除
  // BarChart,     // 这些图表图标不存在，先移除
  // Histogram     // 这些图表图标不存在，先移除
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'

import { LoadingCard, BaseForm } from '../base'

import { TimeRangePicker, DataPointSelector } from './'

export type ChartType =
  | 'line'
  | 'bar'
  | 'pie'
  | 'gauge'
  | 'scatter'
  | 'heatmap'
  | 'radar'

export interface ChartData {
  xAxis?: any[]
  yAxis?: any[]
  series: any[]
  [key: string]: any
}

export interface ChartConfig {
  basic: {
    title?: string
    subtitle?: string
    type?: ChartType
    theme?: string
  }
  style: {
    width?: string
    height?: string
    backgroundColor?: string
    color?: string[]
  }
  data: {
    dataSource?: string
    refreshInterval?: number
    maxDataPoints?: number
  }
}

interface Props {
  // 基本属性
  title?: string
  subtitle?: string
  chartType?: ChartType
  data?: ChartData
  config?: ChartConfig

  // 状态控制
  loading?: boolean
  error?: string
  loadingText?: string
  emptyMessage?: string
  showProgress?: boolean
  loadingProgress?: number
  loadingAnimation?: 'spinner' | 'dots' | 'wave' | 'pulse'

  // 功能控制
  showHeader?: boolean
  showToolbar?: boolean
  showTypeSwitch?: boolean
  showRefresh?: boolean
  showSettings?: boolean
  showActions?: boolean
  showTimeRange?: boolean
  showDataPointSelector?: boolean
  supportRealtime?: boolean
  supportLegend?: boolean
  supportGrid?: boolean
  supportTooltip?: boolean
  supportZoom?: boolean

  // 外观配置
  width?: string
  height?: string
  titleIcon?: any

  // 自定义样式
  customClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: '数据图表',
  chartType: 'line',
  loadingText: '正在加载图表数据...',
  emptyMessage: '暂无图表数据',
  showProgress: false,
  loadingProgress: null,
  loadingAnimation: 'spinner',
  showHeader: true,
  showToolbar: true,
  showTypeSwitch: true,
  showRefresh: true,
  showSettings: true,
  showActions: true,
  showTimeRange: true,
  showDataPointSelector: true,
  supportRealtime: true,
  supportLegend: true,
  supportGrid: true,
  supportTooltip: true,
  supportZoom: true,
  width: '100%',
  height: '400px',
})

interface Emits {
  'type-change': [type: ChartType]
  refresh: []
  'config-change': [config: ChartConfig]
  'time-range-change': [range: any]
  'data-points-change': [dataPoints: string[]]
  export: [format: string]
  'realtime-toggle': [enabled: boolean]
}

const emit = defineEmits<Emits>()

// 状态
const containerRef = ref()
const chartRef = ref()
const fullscreenChartRef = ref()
const refreshing = ref(false)
const settingsVisible = ref(false)
const settingsTab = ref('basic')
const showDataPointDialog = ref(false)
const fullscreenVisible = ref(false)

// 图表配置
const chartConfig = ref<ChartConfig>({
  basic: {
    title: props.title,
    type: props.chartType,
    theme: 'default',
  },
  style: {
    width: props.width,
    height: props.height,
    backgroundColor: 'transparent',
    color: ['#5470c6', '#91cc75', '#fac858', '#ee6666', '#73c0de'],
  },
  data: {
    refreshInterval: 5000,
    maxDataPoints: 1000,
  },
})

// 工具栏状态
const timeRange = ref({
  start: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
  end: new Date().toISOString(),
})

const selectedDataPoints = ref<string[]>([])
const chartOptions = ref(['legend', 'grid', 'tooltip'])
const realtimeEnabled = ref(false)
const realtimeInterval = ref(5000)

// 实时更新定时器
let realtimeTimer: NodeJS.Timeout | null = null

// 可用的图表类型
const availableTypes = [
  { value: 'line', label: '折线图', icon: TrendCharts },
  { value: 'bar', label: '柱状图', icon: BarChart },
  { value: 'pie', label: '饼图', icon: PieChart },
  { value: 'gauge', label: '仪表盘', icon: Histogram },
  { value: 'scatter', label: '散点图', icon: TrendCharts },
  { value: 'heatmap', label: '热力图', icon: TrendCharts },
  { value: 'radar', label: '雷达图', icon: TrendCharts },
]

// 模拟数据点
const availableDataPoints = ref([
  {
    id: '1',
    name: '温度传感器1',
    type: 'datapoint',
    dataType: 'number',
    address: '40001',
    unit: '°C',
    quality: 'good',
  },
  {
    id: '2',
    name: '压力传感器1',
    type: 'datapoint',
    dataType: 'number',
    address: '40002',
    unit: 'Pa',
    quality: 'good',
  },
  {
    id: '3',
    name: '流量计1',
    type: 'datapoint',
    dataType: 'number',
    address: '40003',
    unit: 'L/min',
    quality: 'good',
  },
])

// 配置表单字段
const basicConfigFields = [
  {
    key: 'title',
    label: '图表标题',
    type: 'text',
    placeholder: '输入图表标题',
  },
  {
    key: 'type',
    label: '图表类型',
    type: 'select',
    options: availableTypes.map(t => ({ label: t.label, value: t.value })),
  },
  {
    key: 'theme',
    label: '主题',
    type: 'select',
    options: [
      { label: '默认', value: 'default' },
      { label: '深色', value: 'dark' },
      { label: '浅色', value: 'light' },
    ],
  },
]

const styleConfigFields = [
  {
    key: 'backgroundColor',
    label: '背景色',
    type: 'color',
  },
  {
    key: 'width',
    label: '宽度',
    type: 'text',
    placeholder: '如: 100%, 500px',
  },
  {
    key: 'height',
    label: '高度',
    type: 'text',
    placeholder: '如: 400px, 50vh',
  },
]

const dataConfigFields = [
  {
    key: 'refreshInterval',
    label: '刷新间隔',
    type: 'number',
    unit: '毫秒',
    min: 1000,
  },
  {
    key: 'maxDataPoints',
    label: '最大数据点',
    type: 'number',
    min: 100,
    max: 10000,
  },
]

// 计算属性
const containerClass = computed(() => {
  const classes = ['chart-container']

  if (props.customClass) {
    classes.push(props.customClass)
  }

  return classes.join(' ')
})

const contentStyle = computed(() => {
  return {
    width: props.width,
    minHeight: props.height,
  }
})

const chartHeight = computed(() => {
  return props.height
})

const hasData = computed(() => {
  return props.data && props.data.series && props.data.series.length > 0
})

const dataPoints = computed(() => {
  if (!props.data || !props.data.series) return 0
  return props.data.series.reduce(
    (sum, series) => sum + (series.data?.length || 0),
    0
  )
})

// 方法
const handleTypeChange = (type: ChartType) => {
  chartConfig.value.basic.type = type
  emit('type-change', type)
}

const handleRefresh = async () => {
  refreshing.value = true
  try {
    emit('refresh')
  } finally {
    setTimeout(() => {
      refreshing.value = false
    }, 1000)
  }
}

const handleSettings = () => {
  settingsVisible.value = true
}

const handleAction = (command: string) => {
  switch (command) {
    case 'export':
      handleExport()
      break
    case 'fullscreen':
      handleFullscreen()
      break
    case 'reset':
      handleReset()
      break
    case 'save':
      handleSaveConfig()
      break
  }
}

const handleExport = () => {
  // 导出图表
  emit('export', 'png')
  ElMessage.success('图表导出成功')
}

const handleFullscreen = () => {
  fullscreenVisible.value = true
  nextTick(() => {
    // 在全屏模式下重新渲染图表
    renderFullscreenChart()
  })
}

const renderFullscreenChart = () => {
  // 这里将实现全屏图表的渲染逻辑
  console.log('渲染全屏图表')
}

const handleReset = () => {
  // 重置图表视图
  ElMessage.success('图表视图已重置')
}

const handleSaveConfig = () => {
  // 保存图表配置
  ElMessage.success('图表配置已保存')
}

const handleRetry = () => {
  handleRefresh()
}

const handleTimeRangeChange = (range: any) => {
  timeRange.value = range
  emit('time-range-change', range)
}

const handleDataPointsChange = (dataPoints: any[]) => {
  selectedDataPoints.value = dataPoints.map(dp => dp.id)
}

const handleDataPointsConfirm = () => {
  showDataPointDialog.value = false
  emit('data-points-change', selectedDataPoints.value)
}

const handleDataPointDialogClose = () => {
  showDataPointDialog.value = false
}

const handleRealtimeToggle = (enabled: boolean) => {
  realtimeEnabled.value = enabled

  if (enabled) {
    startRealtimeUpdate()
  } else {
    stopRealtimeUpdate()
  }

  emit('realtime-toggle', enabled)
}

const handleIntervalChange = (interval: number) => {
  realtimeInterval.value = interval

  if (realtimeEnabled.value) {
    stopRealtimeUpdate()
    startRealtimeUpdate()
  }
}

const startRealtimeUpdate = () => {
  if (realtimeTimer) {
    clearInterval(realtimeTimer)
  }

  realtimeTimer = setInterval(() => {
    if (!props.loading) {
      handleRefresh()
    }
  }, realtimeInterval.value)
}

const stopRealtimeUpdate = () => {
  if (realtimeTimer) {
    clearInterval(realtimeTimer)
    realtimeTimer = null
  }
}

const handleSettingsClose = () => {
  settingsVisible.value = false
}

const handleConfigSave = () => {
  emit('config-change', chartConfig.value)
  settingsVisible.value = false
  ElMessage.success('配置保存成功')
}

// 监听
watch(
  () => props.config,
  newConfig => {
    if (newConfig) {
      chartConfig.value = { ...chartConfig.value, ...newConfig }
    }
  },
  { deep: true }
)

// 生命周期
onMounted(() => {
  // 初始化图表
  nextTick(() => {
    // 这里将初始化图表实例
    console.log('初始化图表')
  })
})

onUnmounted(() => {
  stopRealtimeUpdate()
})
</script>

<style scoped lang="scss">
.chart-container {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 6px;
  overflow: hidden;
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);

  .header-left {
    .chart-title {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 16px;
      font-weight: 600;
      color: var(--el-text-color-primary);
    }

    .chart-subtitle {
      font-size: 12px;
      color: var(--el-text-color-secondary);
      margin-top: 4px;
    }
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
}

.chart-content {
  position: relative;

  .chart-wrapper {
    width: 100%;

    .chart-placeholder {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      color: var(--el-text-color-secondary);
    }
  }
}

.chart-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);
  flex-wrap: wrap;

  .toolbar-item {
    display: flex;
    align-items: center;
    gap: 8px;

    &:not(:last-child) {
      border-right: 1px solid var(--el-border-color-lighter);
      padding-right: 16px;
    }
  }
}

:deep(.fullscreen-chart-dialog) {
  .el-dialog__body {
    padding: 0;
  }

  .fullscreen-chart {
    .chart-placeholder {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      color: var(--el-text-color-secondary);
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .chart-container {
    .chart-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .chart-toolbar {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;

      .toolbar-item {
        border-right: none;
        padding-right: 0;
        width: 100%;

        &:not(:last-child) {
          border-bottom: 1px solid var(--el-border-color-lighter);
          padding-bottom: 8px;
        }
      }
    }
  }
}
</style>
