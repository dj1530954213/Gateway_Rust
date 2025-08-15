<template>
  <div class="device-status-chart">
    <div ref="chartRef" class="chart-container"></div>

    <!-- 状态详情 -->
    <div class="status-details">
      <div
        v-for="item in data"
        :key="item.name"
        class="status-item"
        @click="handleStatusClick(item)"
      >
        <div
          class="status-indicator"
          :style="{ backgroundColor: item.color }"
        ></div>
        <div class="status-info">
          <div class="status-name">{{ item.name }}</div>
          <div class="status-count">{{ item.value }} 台</div>
          <div class="status-percent">{{ getPercentage(item.value) }}%</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * DeviceStatusChart —— 设备状态分布图表组件
 *
 * 📝 Responsibilities:
 *  1. 设备状态的饼图展示
 *  2. 状态分布统计
 *  3. 交互式状态选择
 *  4. 动态数据更新
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
    name: string
    value: number
    color: string
  }>
}>()

// ===== Emits =====
const emit = defineEmits<{
  'status-click': [status: any]
}>()

// ===== 响应式数据 =====
const chartRef = ref<HTMLDivElement>()
const chartInstance = ref<echarts.ECharts>()

// ===== 计算属性 =====
const totalDevices = computed(() => {
  return props.data?.reduce((sum, item) => sum + item.value, 0) || 0
})

const chartOptions = computed(() => ({
  animation: true,
  animationDuration: 1000,
  tooltip: {
    trigger: 'item',
    formatter(params: any) {
      return `
        <div style="margin-bottom: 5px; font-weight: bold;">${params.name}</div>
        <div>设备数量: ${params.value} 台</div>
        <div>占比: ${params.percent}%</div>
      `
    },
  },
  series: [
    {
      name: '设备状态',
      type: 'pie',
      radius: ['45%', '75%'],
      center: ['50%', '50%'],
      avoidLabelOverlap: false,
      itemStyle: {
        borderRadius: 8,
        borderColor: '#fff',
        borderWidth: 2,
      },
      label: {
        show: false,
      },
      emphasis: {
        scale: true,
        scaleSize: 5,
        itemStyle: {
          shadowBlur: 10,
          shadowOffsetX: 0,
          shadowColor: 'rgba(0, 0, 0, 0.5)',
        },
      },
      labelLine: {
        show: false,
      },
      data:
        props.data?.map(item => ({
          name: item.name,
          value: item.value,
          itemStyle: {
            color: item.color,
          },
        })) || [],
    },
  ],
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

  // 绑定点击事件
  chartInstance.value.on('click', handleChartClick)

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
  const status = props.data?.find(item => item.name === params.name)
  if (status) {
    handleStatusClick(status)
  }
}

/**
 * 处理状态点击事件
 */
function handleStatusClick(status: any) {
  emit('status-click', status)
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
 * 获取百分比
 */
function getPercentage(value: number): string {
  if (totalDevices.value === 0) return '0'
  return ((value / totalDevices.value) * 100).toFixed(1)
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
</script>

<style scoped lang="scss">
.device-status-chart {
  .chart-container {
    width: 100%;
    height: 250px;
  }

  .status-details {
    margin-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;

    .status-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 8px;
      border-radius: 6px;
      cursor: pointer;
      transition: all 0.2s;

      &:hover {
        background: #f8f9fa;
      }

      .status-indicator {
        width: 12px;
        height: 12px;
        border-radius: 50%;
        flex-shrink: 0;
      }

      .status-info {
        flex: 1;
        display: flex;
        justify-content: space-between;
        align-items: center;

        .status-name {
          font-size: 14px;
          color: #303133;
          font-weight: 500;
        }

        .status-count {
          font-size: 14px;
          color: #606266;
        }

        .status-percent {
          font-size: 12px;
          color: #909399;
          min-width: 40px;
          text-align: right;
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .device-status-chart {
    .status-details .status-item .status-info {
      flex-direction: column;
      align-items: flex-start;
      gap: 2px;
    }
  }
}
</style>
