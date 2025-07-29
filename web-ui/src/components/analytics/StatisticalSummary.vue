<template>
  <div class="statistical-summary">
    <div class="summary-header">
      <h4>统计摘要</h4>
      <div class="summary-info">
        <span>数据点: {{ data?.count || 0 }}</span>
        <span>置信度: 95%</span>
      </div>
    </div>
    
    <!-- 基础统计指标 -->
    <div class="basic-stats">
      <div class="stats-grid">
        <div class="stat-item">
          <div class="stat-label">均值</div>
          <div class="stat-value">{{ formatNumber(data?.mean) }}</div>
        </div>
        
        <div class="stat-item">
          <div class="stat-label">中位数</div>
          <div class="stat-value">{{ formatNumber(data?.median) }}</div>
        </div>
        
        <div class="stat-item">
          <div class="stat-label">标准差</div>
          <div class="stat-value">{{ formatNumber(data?.std) }}</div>
        </div>
        
        <div class="stat-item">
          <div class="stat-label">变异系数</div>
          <div class="stat-value">{{ formatPercent(data?.cv) }}</div>
        </div>
        
        <div class="stat-item">
          <div class="stat-label">最小值</div>
          <div class="stat-value">{{ formatNumber(data?.min) }}</div>
        </div>
        
        <div class="stat-item">
          <div class="stat-label">最大值</div>
          <div class="stat-value">{{ formatNumber(data?.max) }}</div>
        </div>
        
        <div class="stat-item">
          <div class="stat-label">第一四分位数</div>
          <div class="stat-value">{{ formatNumber(data?.q1) }}</div>
        </div>
        
        <div class="stat-item">
          <div class="stat-label">第三四分位数</div>
          <div class="stat-value">{{ formatNumber(data?.q3) }}</div>
        </div>
      </div>
    </div>
    
    <!-- 分布图表 -->
    <div class="distribution-chart">
      <h5>数据分布</h5>
      <div ref="histogramRef" class="histogram-container"></div>
    </div>
    
    <!-- 箱线图 -->
    <div class="boxplot-chart">
      <h5>箱线图</h5>
      <div ref="boxplotRef" class="boxplot-container"></div>
    </div>
    
    <!-- 正态性检验 -->
    <div class="normality-test">
      <h5>正态性检验</h5>
      <div class="test-results">
        <div class="test-item">
          <span class="test-name">Shapiro-Wilk 检验:</span>
          <span class="test-value">p = {{ formatNumber(normalityTest?.shapiro_p) }}</span>
          <el-tag :type="normalityTest?.shapiro_p > 0.05 ? 'success' : 'danger'" size="small">
            {{ normalityTest?.shapiro_p > 0.05 ? '正态分布' : '非正态分布' }}
          </el-tag>
        </div>
        
        <div class="test-item">
          <span class="test-name">偏度系数:</span>
          <span class="test-value">{{ formatNumber(normalityTest?.skewness) }}</span>
          <el-tag :type="getSkewnessType(normalityTest?.skewness)" size="small">
            {{ getSkewnessText(normalityTest?.skewness) }}
          </el-tag>
        </div>
        
        <div class="test-item">
          <span class="test-name">峰度系数:</span>
          <span class="test-value">{{ formatNumber(normalityTest?.kurtosis) }}</span>
          <el-tag :type="getKurtosisType(normalityTest?.kurtosis)" size="small">
            {{ getKurtosisText(normalityTest?.kurtosis) }}
          </el-tag>
        </div>
      </div>
    </div>
    
    <!-- 异常值检测 -->
    <div class="outlier-detection">
      <h5>异常值检测</h5>
      <div class="outlier-info">
        <div class="outlier-stats">
          <span>检测到 {{ outliers?.length || 0 }} 个异常值</span>
          <span>占比: {{ formatPercent(outlierRatio) }}</span>
        </div>
        
        <el-button v-if="outliers?.length > 0" size="small" @click="showOutlierDetails = true">
          查看详情
        </el-button>
      </div>
      
      <!-- 异常值列表 -->
      <el-dialog
        v-model="showOutlierDetails"
        title="异常值详情"
        width="600px"
      >
        <el-table :data="outliers" height="300">
          <el-table-column prop="index" label="索引" width="80" />
          <el-table-column prop="value" label="数值" width="100" :formatter="formatTableValue" />
          <el-table-column prop="z_score" label="Z分数" width="100" :formatter="formatTableValue" />
          <el-table-column prop="distance" label="距离" width="100" :formatter="formatTableValue" />
          <el-table-column prop="type" label="类型" width="100">
            <template #default="{ row }">
              <el-tag :type="row.type === 'extreme' ? 'danger' : 'warning'" size="small">
                {{ row.type === 'extreme' ? '极端值' : '离群值' }}
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
      </el-dialog>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * StatisticalSummary —— 统计摘要组件
 *
 * 📝 Responsibilities:
 *  1. 基础统计指标展示
 *  2. 数据分布可视化
 *  3. 正态性检验
 *  4. 异常值检测
 *
 * 📦 Dependencies:
 *  - ECharts 图表库
 *  - Element Plus UI组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import * as echarts from 'echarts'

// ===== Props =====
const props = defineProps<{
  data?: {
    count: number
    mean: number
    median: number
    std: number
    cv?: number
    min: number
    max: number
    q1: number
    q3: number
    variance?: number
    range?: number
  }
  rawData?: number[]
}>()

// ===== 响应式数据 =====
const histogramRef = ref<HTMLDivElement>()
const boxplotRef = ref<HTMLDivElement>()
const histogramChart = ref<echarts.ECharts>()
const boxplotChart = ref<echarts.ECharts>()
const showOutlierDetails = ref(false)

// 模拟正态性检验结果
const normalityTest = computed(() => ({
  shapiro_p: 0.15, // p值
  skewness: -0.23, // 偏度
  kurtosis: 0.89   // 峰度
}))

// 模拟异常值数据
const outliers = computed(() => [
  { index: 15, value: 95.7, z_score: 3.2, distance: 2.1, type: 'extreme' },
  { index: 42, value: 12.3, z_score: -2.8, distance: 1.9, type: 'outlier' },
  { index: 78, value: 88.9, z_score: 2.6, distance: 1.7, type: 'outlier' }
])

const outlierRatio = computed(() => {
  if (!props.data?.count || !outliers.value?.length) return 0
  return outliers.value.length / props.data.count
})

// ===== 方法 =====

/**
 * 初始化图表
 */
function initCharts() {
  nextTick(() => {
    initHistogram()
    initBoxplot()
  })
}

/**
 * 初始化直方图
 */
function initHistogram() {
  if (!histogramRef.value) return

  histogramChart.value = echarts.init(histogramRef.value)
  
  // 生成直方图数据
  const histogramData = generateHistogramData()
  
  const option = {
    animation: true,
    grid: {
      left: '8%',
      right: '8%',
      bottom: '15%',
      top: '10%'
    },
    tooltip: {
      trigger: 'axis',
      formatter(params: any[]) {
        const param = params[0]
        return `
          <div>区间: ${param.name}</div>
          <div>频数: ${param.value}</div>
          <div>频率: ${((param.value / (props.data?.count || 1)) * 100).toFixed(1)}%</div>
        `
      }
    },
    xAxis: {
      type: 'category',
      data: histogramData.bins,
      axisLabel: {
        rotate: 45,
        fontSize: 10
      }
    },
    yAxis: {
      type: 'value',
      name: '频数',
      nameTextStyle: {
        fontSize: 10
      }
    },
    series: [
      {
        name: '频数',
        type: 'bar',
        data: histogramData.frequencies,
        itemStyle: {
          color: '#409EFF',
          borderRadius: [2, 2, 0, 0]
        },
        emphasis: {
          itemStyle: {
            color: '#66B1FF'
          }
        }
      }
    ]
  }
  
  histogramChart.value.setOption(option)
}

/**
 * 初始化箱线图
 */
function initBoxplot() {
  if (!boxplotRef.value || !props.data) return

  boxplotChart.value = echarts.init(boxplotRef.value)
  
  const option = {
    animation: true,
    grid: {
      left: '10%',
      right: '10%',
      bottom: '15%',
      top: '10%'
    },
    tooltip: {
      trigger: 'item',
      formatter(params: any) {
        const data = params.data
        return `
          <div>最小值: ${data[0].toFixed(2)}</div>
          <div>Q1: ${data[1].toFixed(2)}</div>
          <div>中位数: ${data[2].toFixed(2)}</div>
          <div>Q3: ${data[3].toFixed(2)}</div>
          <div>最大值: ${data[4].toFixed(2)}</div>
        `
      }
    },
    xAxis: {
      type: 'category',
      data: ['数据分布'],
      axisLabel: {
        fontSize: 10
      }
    },
    yAxis: {
      type: 'value',
      name: '数值',
      nameTextStyle: {
        fontSize: 10
      }
    },
    series: [
      {
        name: '箱线图',
        type: 'boxplot',
        data: [[
          props.data.min,
          props.data.q1,
          props.data.median,
          props.data.q3,
          props.data.max
        ]],
        itemStyle: {
          color: '#67C23A',
          borderColor: '#5DAF34'
        },
        emphasis: {
          itemStyle: {
            color: '#85CE61'
          }
        }
      }
    ]
  }
  
  boxplotChart.value.setOption(option)
}

/**
 * 生成直方图数据
 */
function generateHistogramData() {
  if (!props.data) {
    return { bins: [], frequencies: [] }
  }
  
  const { min, max } = props.data
  const binCount = Math.min(Math.ceil(Math.sqrt(props.data.count)), 20)
  const binWidth = (max - min) / binCount
  
  const bins = []
  const frequencies = []
  
  for (let i = 0; i < binCount; i++) {
    const binStart = min + i * binWidth
    const binEnd = min + (i + 1) * binWidth
    bins.push(`${binStart.toFixed(1)}-${binEnd.toFixed(1)}`)
    
    // 模拟频数（正态分布）
    const binCenter = (binStart + binEnd) / 2
    const normalizedDistance = Math.abs(binCenter - props.data.mean) / props.data.std
    const frequency = Math.round(
      props.data.count * 0.1 * Math.exp(-0.5 * normalizedDistance * normalizedDistance)
    )
    frequencies.push(frequency)
  }
  
  return { bins, frequencies }
}

/**
 * 格式化数字
 */
function formatNumber(value?: number): string {
  if (value === undefined || value === null) return '--'
  return value.toFixed(2)
}

/**
 * 格式化百分比
 */
function formatPercent(value?: number): string {
  if (value === undefined || value === null) return '--'
  return `${(value * 100).toFixed(1)}%`
}

/**
 * 格式化表格数值
 */
function formatTableValue(row: any, column: any, cellValue: number): string {
  return cellValue?.toFixed(2) || '--'
}

/**
 * 获取偏度类型
 */
function getSkewnessType(skewness?: number): string {
  if (!skewness) return 'info'
  const abs = Math.abs(skewness)
  if (abs < 0.5) return 'success'
  if (abs < 1) return 'warning'
  return 'danger'
}

/**
 * 获取偏度文本
 */
function getSkewnessText(skewness?: number): string {
  if (!skewness) return '未知'
  if (Math.abs(skewness) < 0.5) return '近似对称'
  if (skewness > 0) return '右偏'
  return '左偏'
}

/**
 * 获取峰度类型
 */
function getKurtosisType(kurtosis?: number): string {
  if (!kurtosis) return 'info'
  const abs = Math.abs(kurtosis)
  if (abs < 1) return 'success'
  if (abs < 2) return 'warning'
  return 'danger'
}

/**
 * 获取峰度文本
 */
function getKurtosisText(kurtosis?: number): string {
  if (!kurtosis) return '未知'
  if (Math.abs(kurtosis) < 1) return '正常峰度'
  if (kurtosis > 0) return '尖峰分布'
  return '平峰分布'
}

/**
 * 处理窗口大小变化
 */
function handleResize() {
  histogramChart.value?.resize()
  boxplotChart.value?.resize()
}

// ===== 生命周期 =====
onMounted(() => {
  initCharts()
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  histogramChart.value?.dispose()
  boxplotChart.value?.dispose()
})

// ===== 监听器 =====
watch(() => props.data, () => {
  initCharts()
}, { deep: true })
</script>

<style scoped lang="scss">
.statistical-summary {
  .summary-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    
    h4 {
      margin: 0;
      font-size: 16px;
      color: #303133;
    }
    
    .summary-info {
      display: flex;
      gap: 16px;
      font-size: 12px;
      color: #909399;
    }
  }
  
  .basic-stats {
    margin-bottom: 24px;
    
    .stats-grid {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 16px;
      
      .stat-item {
        padding: 12px;
        background: #f8f9fa;
        border-radius: 6px;
        text-align: center;
        
        .stat-label {
          font-size: 12px;
          color: #606266;
          margin-bottom: 4px;
        }
        
        .stat-value {
          font-size: 18px;
          font-weight: 600;
          color: #303133;
        }
      }
    }
  }
  
  .distribution-chart,
  .boxplot-chart {
    margin-bottom: 24px;
    
    h5 {
      margin: 0 0 12px 0;
      font-size: 14px;
      color: #303133;
    }
    
    .histogram-container,
    .boxplot-container {
      height: 200px;
      border: 1px solid #ebeef5;
      border-radius: 6px;
    }
  }
  
  .normality-test,
  .outlier-detection {
    margin-bottom: 24px;
    
    h5 {
      margin: 0 0 12px 0;
      font-size: 14px;
      color: #303133;
    }
    
    .test-results {
      display: flex;
      flex-direction: column;
      gap: 8px;
      
      .test-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 8px;
        background: #f8f9fa;
        border-radius: 6px;
        
        .test-name {
          font-size: 13px;
          color: #606266;
          min-width: 120px;
        }
        
        .test-value {
          font-size: 13px;
          color: #303133;
          font-weight: 500;
          min-width: 80px;
        }
      }
    }
    
    .outlier-info {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px;
      background: #f8f9fa;
      border-radius: 6px;
      
      .outlier-stats {
        display: flex;
        gap: 16px;
        font-size: 13px;
        color: #606266;
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .statistical-summary {
    .summary-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }
    
    .basic-stats .stats-grid {
      grid-template-columns: repeat(2, 1fr);
      gap: 8px;
    }
    
    .normality-test .test-results .test-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 4px;
    }
    
    .outlier-detection .outlier-info {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }
  }
}
</style>