<template>
  <div class="correlation-analysis">
    <!-- 分析配置 -->
    <div class="analysis-config">
      <el-card shadow="never">
        <template #header>
          <div class="config-header">
            <h4>相关性分析配置</h4>
            <el-button type="primary" @click="runAnalysis" :loading="analyzing">
              <el-icon><DataAnalysis /></el-icon>
              {{ analyzing ? '分析中...' : '开始分析' }}
            </el-button>
          </div>
        </template>
        
        <el-row :gutter="20">
          <el-col :span="6">
            <div class="config-item">
              <label>分析方法</label>
              <el-select v-model="config.method" placeholder="选择分析方法">
                <el-option label="皮尔逊相关" value="pearson" />
                <el-option label="斯皮尔曼相关" value="spearman" />
                <el-option label="肯德尔相关" value="kendall" />
                <el-option label="互信息" value="mutual_info" />
              </el-select>
            </div>
          </el-col>
          
          <el-col :span="6">
            <div class="config-item">
              <label>显著性水平</label>
              <el-select v-model="config.significance" placeholder="选择显著性水平">
                <el-option label="0.01" value="0.01" />
                <el-option label="0.05" value="0.05" />
                <el-option label="0.10" value="0.10" />
              </el-select>
            </div>
          </el-col>
          
          <el-col :span="6">
            <div class="config-item">
              <label>滞后分析</label>
              <el-input-number
                v-model="config.lagRange"
                :min="0"
                :max="24"
                placeholder="滞后范围（小时）"
              />
            </div>
          </el-col>
          
          <el-col :span="6">
            <div class="config-item">
              <label>平滑窗口</label>
              <el-input-number
                v-model="config.smoothWindow"
                :min="1"
                :max="100"
                placeholder="平滑窗口大小"
              />
            </div>
          </el-col>
        </el-row>
      </el-card>
    </div>
    
    <!-- 分析结果 -->
    <div v-if="analysisResults" class="analysis-results">
      <!-- 相关性矩阵热力图 -->
      <el-card class="result-heatmap" shadow="never">
        <template #header>
          <div class="heatmap-header">
            <h4>相关性矩阵</h4>
            <el-radio-group v-model="heatmapType" size="small">
              <el-radio-button label="correlation">相关系数</el-radio-button>
              <el-radio-button label="pvalue">P值</el-radio-button>
              <el-radio-button label="significance">显著性</el-radio-button>
            </el-radio-group>
          </div>
        </template>
        
        <div class="heatmap-container">
          <div ref="heatmapRef" class="correlation-heatmap"></div>
        </div>
        
        <div class="heatmap-legend">
          <div class="legend-item">
            <span class="legend-color strong-positive"></span>
            <span class="legend-text">强正相关 (≥0.7)</span>
          </div>
          <div class="legend-item">
            <span class="legend-color moderate-positive"></span>
            <span class="legend-text">中等正相关 (0.3-0.7)</span>
          </div>
          <div class="legend-item">
            <span class="legend-color weak"></span>
            <span class="legend-text">弱相关 (-0.3-0.3)</span>
          </div>
          <div class="legend-item">
            <span class="legend-color moderate-negative"></span>
            <span class="legend-text">中等负相关 (-0.7--0.3)</span>
          </div>
          <div class="legend-item">
            <span class="legend-color strong-negative"></span>
            <span class="legend-text">强负相关 (≤-0.7)</span>
          </div>
        </div>
      </el-card>
      
      <!-- 散点图矩阵 -->
      <el-card class="result-scatter" shadow="never">
        <template #header>
          <div class="scatter-header">
            <h4>散点图矩阵</h4>
            <el-select v-model="selectedPair" placeholder="选择变量对">
              <el-option
                v-for="pair in variablePairs"
                :key="pair.key"
                :label="pair.label"
                :value="pair.key"
              />
            </el-select>
          </div>
        </template>
        
        <div class="scatter-container">
          <div ref="scatterRef" class="scatter-plot"></div>
        </div>
        
        <div class="scatter-stats">
          <div class="stats-grid">
            <div class="stat-item">
              <div class="stat-label">相关系数</div>
              <div class="stat-value">{{ selectedPairStats?.correlation || '--' }}</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">P值</div>
              <div class="stat-value">{{ selectedPairStats?.pValue || '--' }}</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">R²</div>
              <div class="stat-value">{{ selectedPairStats?.rSquared || '--' }}</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">样本数</div>
              <div class="stat-value">{{ selectedPairStats?.sampleSize || '--' }}</div>
            </div>
          </div>
        </div>
      </el-card>
      
      <!-- 滞后相关分析 -->
      <el-card class="result-lag" shadow="never">
        <template #header>
          <h4>滞后相关分析</h4>
        </template>
        
        <div class="lag-container">
          <div ref="lagRef" class="lag-chart"></div>
        </div>
        
        <div class="lag-insights">
          <h5>关键发现</h5>
          <ul>
            <li v-for="insight in analysisResults.lagInsights" :key="insight.id">
              <el-icon><component :is="insight.icon" /></el-icon>
              {{ insight.text }}
            </li>
          </ul>
        </div>
      </el-card>
      
      <!-- 相关性排名 -->
      <el-card class="result-ranking" shadow="never">
        <template #header>
          <div class="ranking-header">
            <h4>相关性排名</h4>
            <el-radio-group v-model="rankingType" size="small">
              <el-radio-button label="absolute">绝对值</el-radio-button>
              <el-radio-button label="positive">正相关</el-radio-button>
              <el-radio-button label="negative">负相关</el-radio-button>
            </el-radio-group>
          </div>
        </template>
        
        <el-table :data="rankedCorrelations" style="width: 100%">
          <el-table-column prop="rank" label="排名" width="60" />
          <el-table-column prop="pair" label="变量对" width="200" />
          <el-table-column prop="correlation" label="相关系数" width="120">
            <template #default="{ row }">
              <span :class="getCorrelationClass(row.correlation)">
                {{ row.correlation.toFixed(3) }}
              </span>
            </template>
          </el-table-column>
          <el-table-column prop="pValue" label="P值" width="120">
            <template #default="{ row }">
              <el-tag :type="row.pValue < 0.05 ? 'success' : 'info'" size="small">
                {{ row.pValue.toFixed(4) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="strength" label="强度" width="100">
            <template #default="{ row }">
              <el-tag :type="getStrengthType(row.correlation)" size="small">
                {{ getStrengthText(row.correlation) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="description" label="描述" />
        </el-table>
      </el-card>
      
      <!-- 网络图 -->
      <el-card class="result-network" shadow="never">
        <template #header>
          <div class="network-header">
            <h4>相关性网络图</h4>
            <div class="network-controls">
              <el-slider
                v-model="networkThreshold"
                :min="0"
                :max="1"
                :step="0.1"
                :format-tooltip="formatThreshold"
                style="width: 200px"
              />
              <span class="threshold-label">阈值: {{ networkThreshold }}</span>
            </div>
          </div>
        </template>
        
        <div class="network-container">
          <div ref="networkRef" class="correlation-network"></div>
        </div>
        
        <div class="network-legend">
          <div class="legend-item">
            <span class="node-sample positive"></span>
            <span>正相关</span>
          </div>
          <div class="legend-item">
            <span class="node-sample negative"></span>
            <span>负相关</span>
          </div>
          <div class="legend-item">
            <span class="edge-sample thick"></span>
            <span>强相关 (|r| ≥ 0.7)</span>
          </div>
          <div class="legend-item">
            <span class="edge-sample medium"></span>
            <span>中等相关 (|r| ≥ 0.5)</span>
          </div>
          <div class="legend-item">
            <span class="edge-sample thin"></span>
            <span>弱相关 (|r| ≥ 0.3)</span>
          </div>
        </div>
      </el-card>
    </div>
    
    <!-- 空状态 -->
    <el-empty v-else description="配置参数并开始相关性分析">
      <el-button type="primary" @click="runAnalysis">
        开始分析
      </el-button>
    </el-empty>
  </div>
</template>

<script setup lang="ts">
/**
 * CorrelationAnalysis —— 相关性分析组件
 *
 * 📝 Responsibilities:
 *  1. 多变量相关性分析
 *  2. 相关矩阵热力图展示
 *  3. 散点图矩阵和滞后分析
 *  4. 相关性网络图可视化
 *
 * 📦 Dependencies:
 *  - ECharts 图表库
 *  - Element Plus UI组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import * as echarts from 'echarts'
import {
  DataAnalysis,
  InfoFilled,
  WarningFilled,
  SuccessFilled
} from '@element-plus/icons-vue'

// ===== Props =====
const props = defineProps<{
  data?: any[]
  variables?: string[]
}>()

// ===== 响应式数据 =====
const analyzing = ref(false)
const heatmapType = ref('correlation')
const selectedPair = ref('')
const rankingType = ref('absolute')
const networkThreshold = ref(0.5)

// 分析配置
const config = ref({
  method: 'pearson',
  significance: '0.05',
  lagRange: 6,
  smoothWindow: 5
})

// 图表引用
const heatmapRef = ref<HTMLDivElement>()
const scatterRef = ref<HTMLDivElement>()
const lagRef = ref<HTMLDivElement>()
const networkRef = ref<HTMLDivElement>()

const heatmapChart = ref<echarts.ECharts>()
const scatterChart = ref<echarts.ECharts>()
const lagChart = ref<echarts.ECharts>()
const networkChart = ref<echarts.ECharts>()

// 分析结果
const analysisResults = ref(null)

// ===== 计算属性 =====
const variables = computed(() => {
  return props.variables || ['温度', '压力', '流量', '功率', 'pH值', '湿度']
})

const variablePairs = computed(() => {
  const pairs = []
  for (let i = 0; i < variables.value.length; i++) {
    for (let j = i + 1; j < variables.value.length; j++) {
      pairs.push({
        key: `${i}-${j}`,
        label: `${variables.value[i]} vs ${variables.value[j]}`,
        var1: variables.value[i],
        var2: variables.value[j],
        index1: i,
        index2: j
      })
    }
  }
  return pairs
})

const selectedPairStats = computed(() => {
  if (!selectedPair.value || !analysisResults.value) return null
  
  const pair = variablePairs.value.find(p => p.key === selectedPair.value)
  if (!pair) return null
  
  return analysisResults.value.pairStats[selectedPair.value]
})

const rankedCorrelations = computed(() => {
  if (!analysisResults.value) return []
  
  let correlations = [...analysisResults.value.correlations]
  
  if (rankingType.value === 'absolute') {
    correlations.sort((a, b) => Math.abs(b.correlation) - Math.abs(a.correlation))
  } else if (rankingType.value === 'positive') {
    correlations = correlations.filter(c => c.correlation > 0)
    correlations.sort((a, b) => b.correlation - a.correlation)
  } else {
    correlations = correlations.filter(c => c.correlation < 0)
    correlations.sort((a, b) => a.correlation - b.correlation)
  }
  
  return correlations.map((item, index) => ({
    ...item,
    rank: index + 1
  }))
})

// ===== 方法 =====

/**
 * 运行相关性分析
 */
async function runAnalysis() {
  analyzing.value = true
  
  try {
    // 模拟分析过程
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // 生成模拟结果
    analysisResults.value = generateMockResults()
    
    // 初始化图表
    nextTick(() => {
      initHeatmap()
      initScatterPlot()
      initLagChart()
      initNetworkChart()
    })
    
    // 设置默认选择
    if (variablePairs.value.length > 0) {
      selectedPair.value = variablePairs.value[0].key
    }
    
    ElMessage.success('相关性分析完成')
    
  } catch (error) {
    console.error('分析失败:', error)
    ElMessage.error('相关性分析失败')
  } finally {
    analyzing.value = false
  }
}

/**
 * 生成模拟分析结果
 */
function generateMockResults() {
  const vars = variables.value
  const n = vars.length
  
  // 生成相关矩阵
  const correlationMatrix = []
  const pValueMatrix = []
  const correlations = []
  
  for (let i = 0; i < n; i++) {
    correlationMatrix[i] = []
    pValueMatrix[i] = []
    
    for (let j = 0; j < n; j++) {
      if (i === j) {
        correlationMatrix[i][j] = 1.0
        pValueMatrix[i][j] = 0.0
      } else {
        const corr = (Math.random() - 0.5) * 2 * 0.9  // -0.9 to 0.9
        const pValue = Math.random() * 0.2  // 0 to 0.2
        
        correlationMatrix[i][j] = corr
        pValueMatrix[i][j] = pValue
        
        if (i < j) {  // 避免重复
          correlations.push({
            pair: `${vars[i]} vs ${vars[j]}`,
            var1: vars[i],
            var2: vars[j],
            correlation: corr,
            pValue,
            description: generateCorrelationDescription(corr, pValue)
          })
        }
      }
    }
  }
  
  // 生成配对统计
  const pairStats: { [key: string]: any } = {}
  variablePairs.value.forEach(pair => {
    const corr = correlationMatrix[pair.index1][pair.index2]
    pairStats[pair.key] = {
      correlation: corr.toFixed(3),
      pValue: pValueMatrix[pair.index1][pair.index2].toFixed(4),
      rSquared: (corr * corr).toFixed(3),
      sampleSize: 1000 + Math.floor(Math.random() * 500)
    }
  })
  
  return {
    correlationMatrix,
    pValueMatrix,
    correlations,
    pairStats,
    lagInsights: [
      {
        id: '1',
        text: '温度与压力在滞后2小时时达到最大相关性 (r=0.85)',
        icon: 'SuccessFilled'
      },
      {
        id: '2',
        text: '流量变化领先功率变化约30分钟',
        icon: 'InfoFilled'
      },
      {
        id: '3',
        text: 'pH值与其他变量存在显著的非线性关系',
        icon: 'WarningFilled'
      }
    ]
  }
}

/**
 * 生成相关性描述
 */
function generateCorrelationDescription(corr: number, pValue: number): string {
  const strength = getStrengthText(corr)
  const significance = pValue < 0.05 ? '显著' : '不显著'
  const direction = corr > 0 ? '正相关' : '负相关'
  
  return `${strength}${direction}，统计${significance}`
}

/**
 * 初始化热力图
 */
function initHeatmap() {
  if (!heatmapRef.value || !analysisResults.value) return
  
  heatmapChart.value = echarts.init(heatmapRef.value)
  updateHeatmap()
}

/**
 * 更新热力图
 */
function updateHeatmap() {
  if (!heatmapChart.value || !analysisResults.value) return
  
  const vars = variables.value
  const matrix = heatmapType.value === 'correlation' 
    ? analysisResults.value.correlationMatrix
    : analysisResults.value.pValueMatrix
  
  const data = []
  for (let i = 0; i < vars.length; i++) {
    for (let j = 0; j < vars.length; j++) {
      data.push([j, i, matrix[i][j]])
    }
  }
  
  const option = {
    animation: true,
    grid: {
      left: '10%',
      right: '10%',
      top: '10%',
      bottom: '10%'
    },
    tooltip: {
      formatter(params: any) {
        const value = params.value[2]
        const var1 = vars[params.value[1]]
        const var2 = vars[params.value[0]]
        
        if (heatmapType.value === 'correlation') {
          return `${var1} vs ${var2}<br/>相关系数: ${value.toFixed(3)}`
        } else {
          return `${var1} vs ${var2}<br/>P值: ${value.toFixed(4)}`
        }
      }
    },
    xAxis: {
      type: 'category',
      data: vars,
      axisLabel: { rotate: 45 }
    },
    yAxis: {
      type: 'category',
      data: vars
    },
    visualMap: {
      min: heatmapType.value === 'correlation' ? -1 : 0,
      max: heatmapType.value === 'correlation' ? 1 : 0.2,
      calculable: true,
      orient: 'horizontal',
      left: 'center',
      bottom: '5%',
      inRange: {
        color: heatmapType.value === 'correlation' 
          ? ['#313695', '#4575b4', '#74add1', '#abd9e9', '#e0f3f8', '#ffffcc', '#fee090', '#fdae61', '#f46d43', '#d73027', '#a50026']
          : ['#d73027', '#f46d43', '#fdae61', '#fee090', '#ffffcc', '#e0f3f8', '#abd9e9', '#74add1', '#4575b4', '#313695']
      }
    },
    series: [{
      name: heatmapType.value === 'correlation' ? '相关系数' : 'P值',
      type: 'heatmap',
      data,
      emphasis: {
        itemStyle: {
          shadowBlur: 10,
          shadowColor: 'rgba(0, 0, 0, 0.5)'
        }
      }
    }]
  }
  
  heatmapChart.value.setOption(option, true)
}

/**
 * 初始化散点图
 */
function initScatterPlot() {
  if (!scatterRef.value) return
  
  scatterChart.value = echarts.init(scatterRef.value)
  updateScatterPlot()
}

/**
 * 更新散点图
 */
function updateScatterPlot() {
  if (!scatterChart.value || !selectedPair.value) return
  
  const pair = variablePairs.value.find(p => p.key === selectedPair.value)
  if (!pair) return
  
  // 生成模拟散点数据
  const data = []
  const n = 200
  const corr = analysisResults.value.correlationMatrix[pair.index1][pair.index2]
  
  for (let i = 0; i < n; i++) {
    const x = Math.random() * 100
    const y = x * corr + Math.random() * 30 * (1 - Math.abs(corr))
    data.push([x, y])
  }
  
  const option = {
    animation: true,
    grid: {
      left: '10%',
      right: '10%',
      top: '10%',
      bottom: '15%'
    },
    tooltip: {
      trigger: 'item',
      formatter(params: any) {
        return `${pair.var1}: ${params.value[0].toFixed(2)}<br/>${pair.var2}: ${params.value[1].toFixed(2)}`
      }
    },
    xAxis: {
      type: 'value',
      name: pair.var1,
      nameLocation: 'middle',
      nameGap: 30
    },
    yAxis: {
      type: 'value',
      name: pair.var2,
      nameLocation: 'middle',
      nameGap: 40
    },
    series: [{
      type: 'scatter',
      data,
      symbolSize: 6,
      itemStyle: {
        color: '#409EFF',
        opacity: 0.7
      },
      emphasis: {
        itemStyle: {
          color: '#66B1FF',
          opacity: 1
        }
      }
    }]
  }
  
  scatterChart.value.setOption(option, true)
}

/**
 * 初始化滞后图表
 */
function initLagChart() {
  if (!lagRef.value) return
  
  lagChart.value = echarts.init(lagRef.value)
  
  // 生成滞后相关数据
  const lagData = []
  const maxLag = config.value.lagRange
  
  for (let lag = -maxLag; lag <= maxLag; lag++) {
    const correlation = 0.6 * Math.exp(-Math.abs(lag - 2) / 3) + Math.random() * 0.1 - 0.05
    lagData.push([lag, correlation])
  }
  
  const option = {
    animation: true,
    grid: {
      left: '10%',
      right: '10%',
      top: '10%',
      bottom: '15%'
    },
    tooltip: {
      trigger: 'axis',
      formatter(params: any) {
        const lag = params[0].value[0]
        const corr = params[0].value[1]
        return `滞后: ${lag}小时<br/>相关系数: ${corr.toFixed(3)}`
      }
    },
    xAxis: {
      type: 'value',
      name: '滞后时间 (小时)',
      nameLocation: 'middle',
      nameGap: 30
    },
    yAxis: {
      type: 'value',
      name: '相关系数',
      nameLocation: 'middle',
      nameGap: 40
    },
    series: [{
      type: 'line',
      data: lagData,
      smooth: true,
      lineStyle: {
        color: '#67C23A',
        width: 3
      },
      itemStyle: {
        color: '#67C23A'
      },
      markPoint: {
        data: [
          { type: 'max', name: '最大值' }
        ],
        itemStyle: {
          color: '#E6A23C'
        }
      },
      markLine: {
        data: [
          { yAxis: 0, lineStyle: { type: 'dashed', color: '#909399' } }
        ]
      }
    }]
  }
  
  lagChart.value.setOption(option)
}

/**
 * 初始化网络图
 */
function initNetworkChart() {
  if (!networkRef.value || !analysisResults.value) return
  
  networkChart.value = echarts.init(networkRef.value)
  updateNetworkChart()
}

/**
 * 更新网络图
 */
function updateNetworkChart() {
  if (!networkChart.value || !analysisResults.value) return
  
  const vars = variables.value
  const matrix = analysisResults.value.correlationMatrix
  
  // 创建节点
  const nodes = vars.map(name => ({
    name,
    symbolSize: 30,
    itemStyle: {
      color: '#409EFF'
    }
  }))
  
  // 创建边
  const links = []
  for (let i = 0; i < vars.length; i++) {
    for (let j = i + 1; j < vars.length; j++) {
      const corr = matrix[i][j]
      if (Math.abs(corr) >= networkThreshold.value) {
        links.push({
          source: vars[i],
          target: vars[j],
          value: corr,
          lineStyle: {
            color: corr > 0 ? '#67C23A' : '#F56C6C',
            width: Math.abs(corr) * 5,
            opacity: 0.7
          }
        })
      }
    }
  }
  
  const option = {
    animation: true,
    tooltip: {
      formatter(params: any) {
        if (params.dataType === 'edge') {
          return `${params.data.source} ↔ ${params.data.target}<br/>相关系数: ${params.data.value.toFixed(3)}`
        } else {
          return params.data.name
        }
      }
    },
    series: [{
      type: 'graph',
      layout: 'circular',
      data: nodes,
      links,
      roam: true,
      label: {
        show: true,
        position: 'inside',
        fontSize: 12,
        color: '#fff'
      },
      emphasis: {
        focus: 'adjacency',
        lineStyle: {
          width: 10
        }
      }
    }]
  }
  
  networkChart.value.setOption(option, true)
}

/**
 * 获取相关性强度类型
 */
function getStrengthType(correlation: number): string {
  const abs = Math.abs(correlation)
  if (abs >= 0.7) return 'success'
  if (abs >= 0.5) return 'warning'
  if (abs >= 0.3) return 'info'
  return 'info'
}

/**
 * 获取相关性强度文本
 */
function getStrengthText(correlation: number): string {
  const abs = Math.abs(correlation)
  if (abs >= 0.7) return '强'
  if (abs >= 0.5) return '中等'
  if (abs >= 0.3) return '弱'
  return '很弱'
}

/**
 * 获取相关性样式类
 */
function getCorrelationClass(correlation: number): string {
  if (correlation > 0.7) return 'strong-positive'
  if (correlation > 0.3) return 'moderate-positive'
  if (correlation < -0.7) return 'strong-negative'
  if (correlation < -0.3) return 'moderate-negative'
  return 'weak'
}

/**
 * 格式化阈值
 */
function formatThreshold(value: number): string {
  return value.toFixed(1)
}

/**
 * 处理窗口大小变化
 */
function handleResize() {
  heatmapChart.value?.resize()
  scatterChart.value?.resize()
  lagChart.value?.resize()
  networkChart.value?.resize()
}

// ===== 生命周期 =====
onMounted(() => {
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  heatmapChart.value?.dispose()
  scatterChart.value?.dispose()
  lagChart.value?.dispose()
  networkChart.value?.dispose()
})

// ===== 监听器 =====
watch(() => heatmapType.value, () => {
  updateHeatmap()
})

watch(() => selectedPair.value, () => {
  updateScatterPlot()
})

watch(() => networkThreshold.value, () => {
  updateNetworkChart()
})
</script>

<style scoped lang="scss">
.correlation-analysis {
  .analysis-config {
    margin-bottom: 20px;
    
    .config-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      h4 {
        margin: 0;
        color: #303133;
        font-size: 16px;
      }
    }
    
    .config-item {
      label {
        display: block;
        margin-bottom: 8px;
        color: #606266;
        font-size: 13px;
        font-weight: 500;
      }
    }
  }
  
  .analysis-results {
    .result-heatmap {
      margin-bottom: 20px;
      
      .heatmap-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        
        h4 {
          margin: 0;
          color: #303133;
          font-size: 16px;
        }
      }
      
      .heatmap-container {
        .correlation-heatmap {
          height: 400px;
          width: 100%;
        }
      }
      
      .heatmap-legend {
        display: flex;
        justify-content: center;
        flex-wrap: wrap;
        gap: 16px;
        margin-top: 16px;
        
        .legend-item {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 12px;
          
          .legend-color {
            width: 16px;
            height: 16px;
            border-radius: 2px;
            
            &.strong-positive { background: #a50026; }
            &.moderate-positive { background: #f46d43; }
            &.weak { background: #ffffcc; }
            &.moderate-negative { background: #74add1; }
            &.strong-negative { background: #313695; }
          }
        }
      }
    }
    
    .result-scatter {
      margin-bottom: 20px;
      
      .scatter-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        
        h4 {
          margin: 0;
          color: #303133;
          font-size: 16px;
        }
      }
      
      .scatter-container {
        .scatter-plot {
          height: 350px;
          width: 100%;
        }
      }
      
      .scatter-stats {
        margin-top: 16px;
        
        .stats-grid {
          display: grid;
          grid-template-columns: repeat(4, 1fr);
          gap: 16px;
          
          .stat-item {
            text-align: center;
            padding: 12px;
            background: #f8f9fa;
            border-radius: 6px;
            
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
    }
    
    .result-lag {
      margin-bottom: 20px;
      
      .lag-container {
        .lag-chart {
          height: 300px;
          width: 100%;
        }
      }
      
      .lag-insights {
        margin-top: 16px;
        
        h5 {
          margin: 0 0 12px 0;
          color: #303133;
          font-size: 14px;
          font-weight: 600;
        }
        
        ul {
          margin: 0;
          padding-left: 0;
          list-style: none;
          
          li {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-bottom: 8px;
            padding: 8px;
            background: #f8f9fa;
            border-radius: 6px;
            font-size: 13px;
            color: #606266;
          }
        }
      }
    }
    
    .result-ranking {
      margin-bottom: 20px;
      
      .ranking-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        
        h4 {
          margin: 0;
          color: #303133;
          font-size: 16px;
        }
      }
      
      :deep(.el-table) {
        .strong-positive {
          color: #67c23a;
          font-weight: 600;
        }
        
        .moderate-positive {
          color: #409eff;
          font-weight: 500;
        }
        
        .strong-negative {
          color: #f56c6c;
          font-weight: 600;
        }
        
        .moderate-negative {
          color: #e6a23c;
          font-weight: 500;
        }
        
        .weak {
          color: #909399;
        }
      }
    }
    
    .result-network {
      .network-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        
        h4 {
          margin: 0;
          color: #303133;
          font-size: 16px;
        }
        
        .network-controls {
          display: flex;
          align-items: center;
          gap: 12px;
          
          .threshold-label {
            font-size: 12px;
            color: #606266;
            min-width: 60px;
          }
        }
      }
      
      .network-container {
        .correlation-network {
          height: 400px;
          width: 100%;
        }
      }
      
      .network-legend {
        display: flex;
        justify-content: center;
        flex-wrap: wrap;
        gap: 20px;
        margin-top: 16px;
        
        .legend-item {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 12px;
          color: #606266;
          
          .node-sample {
            width: 12px;
            height: 12px;
            border-radius: 50%;
            
            &.positive { background: #67C23A; }
            &.negative { background: #F56C6C; }
          }
          
          .edge-sample {
            width: 20px;
            height: 2px;
            background: #909399;
            
            &.thick { height: 4px; }
            &.medium { height: 3px; }
            &.thin { height: 2px; }
          }
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .correlation-analysis {
    .result-scatter .scatter-stats .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}

@media (max-width: 768px) {
  .correlation-analysis {
    .analysis-config {
      :deep(.el-row) {
        flex-direction: column;
        gap: 12px;
      }
    }
    
    .result-heatmap {
      .heatmap-header {
        flex-direction: column;
        gap: 12px;
        align-items: stretch;
      }
      
      .heatmap-legend {
        flex-direction: column;
        align-items: center;
        gap: 8px;
      }
    }
    
    .result-scatter {
      .scatter-header {
        flex-direction: column;
        gap: 12px;
        align-items: stretch;
      }
      
      .scatter-stats .stats-grid {
        grid-template-columns: 1fr;
        gap: 8px;
      }
    }
    
    .result-ranking .ranking-header {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
    }
    
    .result-network .network-header {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
    }
  }
}
</style>