<template>
  <div class="data-mining-panel">
    <!-- 挖掘配置区域 -->
    <div class="mining-config">
      <el-card class="config-card" shadow="never">
        <template #header>
          <div class="config-header">
            <h4>数据挖掘配置</h4>
            <el-button type="primary" :loading="mining" @click="startMining">
              <el-icon><Cpu /></el-icon>
              {{ mining ? '挖掘中...' : '开始挖掘' }}
            </el-button>
          </div>
        </template>

        <div class="config-content">
          <el-row :gutter="20">
            <!-- 挖掘任务类型 -->
            <el-col :span="6">
              <div class="config-section">
                <h5>挖掘任务</h5>
                <el-select v-model="config.taskType" placeholder="选择挖掘任务">
                  <el-option label="模式发现" value="pattern" />
                  <el-option label="异常检测" value="anomaly" />
                  <el-option label="聚类分析" value="clustering" />
                  <el-option label="关联规则" value="association" />
                  <el-option label="频繁项集" value="frequent" />
                </el-select>
              </div>
            </el-col>

            <!-- 算法选择 -->
            <el-col :span="6">
              <div class="config-section">
                <h5>挖掘算法</h5>
                <el-select v-model="config.algorithm" placeholder="选择算法">
                  <el-option
                    v-for="algo in availableAlgorithms"
                    :key="algo.value"
                    :label="algo.label"
                    :value="algo.value"
                  />
                </el-select>
              </div>
            </el-col>

            <!-- 数据源选择 -->
            <el-col :span="6">
              <div class="config-section">
                <h5>数据源</h5>
                <el-select v-model="config.dataSource" placeholder="选择数据源">
                  <el-option label="实时数据" value="realtime" />
                  <el-option label="历史数据" value="historical" />
                  <el-option label="事件日志" value="events" />
                  <el-option label="设备状态" value="status" />
                </el-select>
              </div>
            </el-col>

            <!-- 时间窗口 -->
            <el-col :span="6">
              <div class="config-section">
                <h5>时间窗口</h5>
                <el-select
                  v-model="config.timeWindow"
                  placeholder="选择时间窗口"
                >
                  <el-option label="最近1小时" value="1h" />
                  <el-option label="最近24小时" value="24h" />
                  <el-option label="最近7天" value="7d" />
                  <el-option label="最近30天" value="30d" />
                </el-select>
              </div>
            </el-col>
          </el-row>

          <!-- 高级参数 -->
          <div class="advanced-params">
            <el-collapse v-model="activeCollapse">
              <el-collapse-item title="算法参数" name="algorithm">
                <el-row :gutter="16">
                  <el-col :span="8">
                    <el-form-item label="支持度阈值">
                      <el-slider
                        v-model="config.params.support"
                        :min="0.1"
                        :max="1"
                        :step="0.1"
                        show-input
                      />
                    </el-form-item>
                  </el-col>
                  <el-col :span="8">
                    <el-form-item label="置信度阈值">
                      <el-slider
                        v-model="config.params.confidence"
                        :min="0.1"
                        :max="1"
                        :step="0.1"
                        show-input
                      />
                    </el-form-item>
                  </el-col>
                  <el-col :span="8">
                    <el-form-item label="最小支持数">
                      <el-input-number
                        v-model="config.params.minSupport"
                        :min="1"
                        :max="1000"
                      />
                    </el-form-item>
                  </el-col>
                </el-row>
              </el-collapse-item>

              <el-collapse-item title="数据过滤" name="filter">
                <el-row :gutter="16">
                  <el-col :span="12">
                    <el-form-item label="设备过滤">
                      <el-select
                        v-model="config.filters.devices"
                        multiple
                        placeholder="选择设备"
                        style="width: 100%"
                      >
                        <el-option
                          v-for="device in devices"
                          :key="device.id"
                          :label="device.name"
                          :value="device.id"
                        />
                      </el-select>
                    </el-form-item>
                  </el-col>
                  <el-col :span="12">
                    <el-form-item label="标签过滤">
                      <el-select
                        v-model="config.filters.tags"
                        multiple
                        placeholder="选择标签"
                        style="width: 100%"
                      >
                        <el-option
                          v-for="tag in tags"
                          :key="tag.id"
                          :label="tag.name"
                          :value="tag.id"
                        />
                      </el-select>
                    </el-form-item>
                  </el-col>
                </el-row>
              </el-collapse-item>
            </el-collapse>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 挖掘结果区域 -->
    <div v-if="miningResults" class="mining-results">
      <!-- 结果概览 -->
      <el-card class="result-overview" shadow="never">
        <template #header>
          <h4>挖掘结果概览</h4>
        </template>

        <div class="overview-grid">
          <div class="overview-item">
            <div class="overview-icon">
              <el-icon><DataBoard /></el-icon>
            </div>
            <div class="overview-content">
              <div class="overview-value">
                {{ miningResults.summary.patterns }}
              </div>
              <div class="overview-label">发现模式</div>
            </div>
          </div>

          <div class="overview-item">
            <div class="overview-icon">
              <el-icon><Warning /></el-icon>
            </div>
            <div class="overview-content">
              <div class="overview-value">
                {{ miningResults.summary.anomalies }}
              </div>
              <div class="overview-label">异常点</div>
            </div>
          </div>

          <div class="overview-item">
            <div class="overview-icon">
              <el-icon><Connection /></el-icon>
            </div>
            <div class="overview-content">
              <div class="overview-value">
                {{ miningResults.summary.associations }}
              </div>
              <div class="overview-label">关联规则</div>
            </div>
          </div>

          <div class="overview-item">
            <div class="overview-icon">
              <el-icon><PieChart /></el-icon>
            </div>
            <div class="overview-content">
              <div class="overview-value">
                {{ miningResults.summary.clusters }}
              </div>
              <div class="overview-label">聚类数量</div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 模式发现结果 -->
      <el-card
        v-if="config.taskType === 'pattern'"
        class="result-patterns"
        shadow="never"
      >
        <template #header>
          <div class="patterns-header">
            <h4>发现的模式</h4>
            <el-select v-model="patternSortBy" size="small">
              <el-option label="按支持度排序" value="support" />
              <el-option label="按置信度排序" value="confidence" />
              <el-option label="按频率排序" value="frequency" />
            </el-select>
          </div>
        </template>

        <div class="patterns-content">
          <div class="patterns-visualization">
            <div ref="patternsChartRef" class="patterns-chart"></div>
          </div>

          <div class="patterns-list">
            <el-table
              :data="sortedPatterns"
              style="width: 100%"
              max-height="400"
            >
              <el-table-column prop="pattern" label="模式" width="300">
                <template #default="{ row }">
                  <div class="pattern-display">
                    <el-tag
                      v-for="item in row.items"
                      :key="item"
                      size="small"
                      effect="plain"
                      style="margin-right: 4px"
                    >
                      {{ item }}
                    </el-tag>
                  </div>
                </template>
              </el-table-column>
              <el-table-column prop="support" label="支持度" width="100">
                <template #default="{ row }">
                  <el-progress
                    :percentage="row.support * 100"
                    :stroke-width="6"
                    :show-text="false"
                    :color="getSupportColor(row.support)"
                  />
                  <span class="support-text"
                    >{{ (row.support * 100).toFixed(1) }}%</span
                  >
                </template>
              </el-table-column>
              <el-table-column prop="confidence" label="置信度" width="100">
                <template #default="{ row }">
                  <span :class="getConfidenceClass(row.confidence)">
                    {{ (row.confidence * 100).toFixed(1) }}%
                  </span>
                </template>
              </el-table-column>
              <el-table-column prop="frequency" label="频率" width="80" />
              <el-table-column prop="significance" label="显著性" width="100">
                <template #default="{ row }">
                  <el-tag
                    :type="getSignificanceType(row.significance)"
                    size="small"
                  >
                    {{ row.significance }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column prop="description" label="描述" />
            </el-table>
          </div>
        </div>
      </el-card>

      <!-- 聚类分析结果 -->
      <el-card
        v-if="config.taskType === 'clustering'"
        class="result-clustering"
        shadow="never"
      >
        <template #header>
          <div class="clustering-header">
            <h4>聚类分析结果</h4>
            <el-radio-group v-model="clusterView" size="small">
              <el-radio-button label="scatter">散点图</el-radio-button>
              <el-radio-button label="parallel">平行坐标</el-radio-button>
              <el-radio-button label="heatmap">热力图</el-radio-button>
            </el-radio-group>
          </div>
        </template>

        <div class="clustering-content">
          <div class="clustering-visualization">
            <div ref="clusteringChartRef" class="clustering-chart"></div>
          </div>

          <div class="clustering-summary">
            <h5>聚类摘要</h5>
            <el-table
              :data="miningResults.clustering.clusters"
              style="width: 100%"
            >
              <el-table-column prop="id" label="聚类ID" width="80" />
              <el-table-column prop="size" label="样本数" width="100" />
              <el-table-column prop="center" label="中心点">
                <template #default="{ row }">
                  <span>{{ formatClusterCenter(row.center) }}</span>
                </template>
              </el-table-column>
              <el-table-column prop="inertia" label="惯性" width="100">
                <template #default="{ row }">
                  {{ row.inertia.toFixed(2) }}
                </template>
              </el-table-column>
              <el-table-column prop="characteristics" label="特征" />
            </el-table>
          </div>
        </div>
      </el-card>

      <!-- 关联规则结果 -->
      <el-card
        v-if="config.taskType === 'association'"
        class="result-association"
        shadow="never"
      >
        <template #header>
          <h4>关联规则</h4>
        </template>

        <div class="association-content">
          <div class="association-network">
            <div ref="associationChartRef" class="association-chart"></div>
          </div>

          <div class="association-rules">
            <h5>强关联规则</h5>
            <div class="rules-list">
              <div
                v-for="rule in miningResults.association.rules"
                :key="rule.id"
                class="rule-item"
              >
                <div class="rule-expression">
                  <div class="rule-antecedent">
                    <el-tag
                      v-for="item in rule.antecedent"
                      :key="item"
                      size="small"
                      type="primary"
                      effect="plain"
                    >
                      {{ item }}
                    </el-tag>
                  </div>
                  <div class="rule-arrow">
                    <el-icon><Right /></el-icon>
                  </div>
                  <div class="rule-consequent">
                    <el-tag
                      v-for="item in rule.consequent"
                      :key="item"
                      size="small"
                      type="success"
                      effect="plain"
                    >
                      {{ item }}
                    </el-tag>
                  </div>
                </div>

                <div class="rule-metrics">
                  <div class="metric">
                    <span class="metric-label">支持度:</span>
                    <span class="metric-value"
                      >{{ (rule.support * 100).toFixed(1) }}%</span
                    >
                  </div>
                  <div class="metric">
                    <span class="metric-label">置信度:</span>
                    <span class="metric-value"
                      >{{ (rule.confidence * 100).toFixed(1) }}%</span
                    >
                  </div>
                  <div class="metric">
                    <span class="metric-label">提升度:</span>
                    <span class="metric-value">{{ rule.lift.toFixed(2) }}</span>
                  </div>
                </div>

                <div class="rule-interpretation">{{ rule.interpretation }}</div>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 挖掘洞察 -->
      <el-card class="result-insights" shadow="never">
        <template #header>
          <h4>挖掘洞察</h4>
        </template>

        <div class="insights-content">
          <div class="insights-list">
            <div
              v-for="insight in miningResults.insights"
              :key="insight.id"
              class="insight-item"
            >
              <div class="insight-header">
                <el-icon class="insight-icon" :class="insight.type">
                  <component :is="getInsightIcon(insight.type)" />
                </el-icon>
                <span class="insight-title">{{ insight.title }}</span>
                <el-tag :type="insight.priority" size="small">{{
                  insight.priority
                }}</el-tag>
              </div>
              <div class="insight-content">{{ insight.content }}</div>
              <div class="insight-actions">
                <el-button size="small" @click="applyInsight(insight)">
                  应用建议
                </el-button>
                <el-button size="small" text @click="saveInsight(insight)">
                  保存
                </el-button>
              </div>
            </div>
          </div>

          <div class="insights-summary">
            <h5>关键发现总结</h5>
            <ul>
              <li
                v-for="finding in miningResults.keyFindings"
                :key="finding.id"
              >
                {{ finding.text }}
              </li>
            </ul>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 空状态 -->
    <el-empty v-else description="配置参数并开始数据挖掘">
      <el-button
        type="primary"
        :disabled="!canStartMining"
        @click="startMining"
      >
        开始数据挖掘
      </el-button>
    </el-empty>
  </div>
</template>

<script setup lang="ts">
/**
 * DataMiningPanel —— 数据挖掘面板
 *
 * 📝 Responsibilities:
 *  1. 数据挖掘任务配置和执行
 *  2. 多种挖掘算法支持
 *  3. 模式发现和关联规则挖掘
 *  4. 聚类分析和异常检测
 *
 * 📦 Dependencies:
 *  - ECharts 图表库
 *  - Element Plus UI组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Cpu,
  DataBoard,
  Warning,
  Connection,
  PieChart,
  Right,
  InfoFilled,
  SuccessFilled,
  WarningFilled,
} from '@element-plus/icons-vue'
import * as echarts from 'echarts'
import { ElMessage } from 'element-plus'
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'

// ===== 响应式数据 =====
const mining = ref(false)
const activeCollapse = ref(['algorithm'])
const patternSortBy = ref('support')
const clusterView = ref('scatter')

// 挖掘配置
const config = ref({
  taskType: 'pattern',
  algorithm: 'apriori',
  dataSource: 'historical',
  timeWindow: '24h',
  params: {
    support: 0.3,
    confidence: 0.7,
    minSupport: 10,
  },
  filters: {
    devices: [],
    tags: [],
  },
})

// 图表引用
const patternsChartRef = ref<HTMLDivElement>()
const clusteringChartRef = ref<HTMLDivElement>()
const associationChartRef = ref<HTMLDivElement>()

const patternsChart = ref<echarts.ECharts>()
const clusteringChart = ref<echarts.ECharts>()
const associationChart = ref<echarts.ECharts>()

// 模拟数据
const devices = ref([
  { id: '1', name: 'PLC-01' },
  { id: '2', name: 'PLC-02' },
  { id: '3', name: 'Sensor-01' },
  { id: '4', name: 'Sensor-02' },
])

const tags = ref([
  { id: '1', name: '温度' },
  { id: '2', name: '压力' },
  { id: '3', name: '流量' },
  { id: '4', name: '功率' },
])

// 挖掘结果
const miningResults = ref(null)

// ===== 计算属性 =====
const availableAlgorithms = computed(() => {
  const algorithmMap = {
    pattern: [
      { label: 'Apriori', value: 'apriori' },
      { label: 'FP-Growth', value: 'fpgrowth' },
      { label: 'ECLAT', value: 'eclat' },
    ],
    clustering: [
      { label: 'K-Means', value: 'kmeans' },
      { label: 'DBSCAN', value: 'dbscan' },
      { label: '层次聚类', value: 'hierarchical' },
    ],
    association: [
      { label: 'Apriori', value: 'apriori' },
      { label: 'FP-Growth', value: 'fpgrowth' },
    ],
    anomaly: [
      { label: '孤立森林', value: 'isolation_forest' },
      { label: 'One-Class SVM', value: 'one_class_svm' },
      { label: 'LOF', value: 'lof' },
    ],
  }

  return algorithmMap[config.value.taskType] || []
})

const canStartMining = computed(() => {
  return (
    config.value.taskType && config.value.algorithm && config.value.dataSource
  )
})

const sortedPatterns = computed(() => {
  if (!miningResults.value?.patterns) return []

  const patterns = [...miningResults.value.patterns]
  patterns.sort((a, b) => b[patternSortBy.value] - a[patternSortBy.value])

  return patterns
})

// ===== 方法 =====

/**
 * 开始挖掘
 */
async function startMining() {
  if (!canStartMining.value) {
    ElMessage.warning('请完善挖掘配置')
    return
  }

  mining.value = true

  try {
    // 调用真实的数据挖掘API
    const response = await fetch('/api/v1/analytics/mining', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        taskType: config.value.taskType,
        algorithm: config.value.algorithm,
        parameters: config.value.parameters,
        dataSource: config.value.dataSource,
        timeRange: config.value.timeRange,
        filters: config.value.filters,
      }),
    })

    if (!response.ok) {
      throw new Error('数据挖掘请求失败')
    }

    miningResults.value = await response.json()

    // 初始化图表
    nextTick(() => {
      if (config.value.taskType === 'pattern') {
        initPatternsChart()
      } else if (config.value.taskType === 'clustering') {
        initClusteringChart()
      } else if (config.value.taskType === 'association') {
        initAssociationChart()
      }
    })

    ElMessage.success('数据挖掘完成')
  } catch (error) {
    console.error('挖掘失败:', error)
    ElMessage.error('数据挖掘失败')
  } finally {
    mining.value = false
  }
}

/**
 * 初始化模式图表
 */
function initPatternsChart() {
  if (!patternsChartRef.value || !miningResults.value?.patterns) return

  patternsChart.value = echarts.init(patternsChartRef.value)

  const data = miningResults.value.patterns.map(pattern => ({
    name: pattern.items.join(' + '),
    value: pattern.support,
    confidence: pattern.confidence,
    frequency: pattern.frequency,
  }))

  const option = {
    animation: true,
    tooltip: {
      trigger: 'item',
      formatter(params: any) {
        return `
          <div><strong>${params.name}</strong></div>
          <div>支持度: ${(params.value * 100).toFixed(1)}%</div>
          <div>置信度: ${(params.data.confidence * 100).toFixed(1)}%</div>
          <div>频率: ${params.data.frequency}</div>
        `
      },
    },
    series: [
      {
        type: 'treemap',
        data,
        roam: false,
        nodeClick: false,
        breadcrumb: { show: false },
        itemStyle: {
          borderColor: '#fff',
          borderWidth: 2,
          borderRadius: 4,
        },
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowColor: 'rgba(0,0,0,0.3)',
          },
        },
        levels: [
          {
            itemStyle: {
              borderWidth: 3,
              gapWidth: 3,
            },
          },
        ],
      },
    ],
  }

  patternsChart.value.setOption(option)
}

/**
 * 初始化聚类图表
 */
function initClusteringChart() {
  if (!clusteringChartRef.value || !miningResults.value?.clustering) return

  clusteringChart.value = echarts.init(clusteringChartRef.value)

  if (clusterView.value === 'scatter') {
    // 生成模拟散点数据
    const series = miningResults.value.clustering.clusters.map(
      (cluster, index) => {
        const data = []
        const colors = ['#409EFF', '#67C23A', '#E6A23C', '#F56C6C', '#909399']

        for (let i = 0; i < cluster.size / 10; i++) {
          data.push([
            cluster.center[0] + (Math.random() - 0.5) * 10,
            cluster.center[1] + (Math.random() - 0.5) * 0.5,
            cluster.center[2] + (Math.random() - 0.5) * 50,
          ])
        }

        return {
          name: `聚类 ${cluster.id}`,
          type: 'scatter3D',
          data,
          itemStyle: {
            color: colors[index % colors.length],
          },
          symbolSize: 6,
        }
      }
    )

    const option = {
      animation: true,
      grid3D: {
        boxWidth: 100,
        boxDepth: 100,
        boxHeight: 100,
      },
      xAxis3D: { name: '温度' },
      yAxis3D: { name: '压力' },
      zAxis3D: { name: '流量' },
      tooltip: {
        trigger: 'item',
      },
      legend: {
        data: series.map(s => s.name),
      },
      series,
    }

    patternsChart.value.setOption(option)
  }
}

/**
 * 初始化关联图表
 */
function initAssociationChart() {
  if (!associationChartRef.value || !miningResults.value?.association) return

  associationChart.value = echarts.init(associationChartRef.value)

  // 创建节点和连接
  const nodes = new Set()
  const links = []

  miningResults.value.association.rules.forEach(rule => {
    rule.antecedent.forEach(item => nodes.add(item))
    rule.consequent.forEach(item => nodes.add(item))

    rule.antecedent.forEach(ant => {
      rule.consequent.forEach(con => {
        links.push({
          source: ant,
          target: con,
          value: rule.confidence,
          lineStyle: {
            width: rule.confidence * 5,
            color: rule.confidence > 0.8 ? '#67C23A' : '#409EFF',
          },
        })
      })
    })
  })

  const nodeData = Array.from(nodes).map(name => ({
    name,
    symbolSize: 30,
    itemStyle: {
      color: '#409EFF',
    },
  }))

  const option = {
    animation: true,
    tooltip: {
      formatter(params: any) {
        if (params.dataType === 'edge') {
          return `${params.data.source} → ${params.data.target}<br/>置信度: ${(params.data.value * 100).toFixed(1)}%`
        } else {
          return params.data.name
        }
      },
    },
    series: [
      {
        type: 'graph',
        layout: 'force',
        data: nodeData,
        links,
        roam: true,
        force: {
          repulsion: 200,
          edgeLength: 100,
        },
        label: {
          show: true,
          position: 'inside',
          fontSize: 12,
          color: '#fff',
        },
        emphasis: {
          focus: 'adjacency',
          lineStyle: {
            width: 8,
          },
        },
      },
    ],
  }

  associationChart.value.setOption(option)
}

/**
 * 获取支持度颜色
 */
function getSupportColor(support: number): string {
  if (support >= 0.5) return '#67C23A'
  if (support >= 0.3) return '#E6A23C'
  return '#F56C6C'
}

/**
 * 获取置信度样式类
 */
function getConfidenceClass(confidence: number): string {
  if (confidence >= 0.8) return 'high-confidence'
  if (confidence >= 0.6) return 'medium-confidence'
  return 'low-confidence'
}

/**
 * 获取显著性类型
 */
function getSignificanceType(significance: string): string {
  if (significance === '高') return 'success'
  if (significance === '中') return 'warning'
  return 'info'
}

/**
 * 格式化聚类中心
 */
function formatClusterCenter(center: number[]): string {
  return `(${center.map(c => c.toFixed(1)).join(', ')})`
}

/**
 * 获取洞察图标
 */
function getInsightIcon(type: string): string {
  const iconMap = {
    info: 'InfoFilled',
    success: 'SuccessFilled',
    warning: 'WarningFilled',
  }
  return iconMap[type] || 'InfoFilled'
}

/**
 * 应用洞察
 */
function applyInsight(insight: any) {
  ElMessage.success(`正在应用洞察: ${insight.title}`)
}

/**
 * 保存洞察
 */
function saveInsight(insight: any) {
  ElMessage.success(`洞察已保存: ${insight.title}`)
}

/**
 * 处理窗口大小变化
 */
function handleResize() {
  patternsChart.value?.resize()
  clusteringChart.value?.resize()
  associationChart.value?.resize()
}

// ===== 生命周期 =====
onMounted(() => {
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  patternsChart.value?.dispose()
  clusteringChart.value?.dispose()
  associationChart.value?.dispose()
})

// ===== 监听器 =====
watch(
  () => config.value.taskType,
  () => {
    // 切换任务类型时重置算法
    if (availableAlgorithms.value.length > 0) {
      config.value.algorithm = availableAlgorithms.value[0].value
    }
  }
)

watch(
  () => clusterView.value,
  () => {
    if (config.value.taskType === 'clustering' && clusteringChart.value) {
      initClusteringChart()
    }
  }
)
</script>

<style scoped lang="scss">
.data-mining-panel {
  .mining-config {
    margin-bottom: 20px;

    .config-card {
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

      .config-content {
        .config-section {
          margin-bottom: 16px;

          h5 {
            margin: 0 0 8px 0;
            color: #606266;
            font-size: 13px;
            font-weight: 500;
          }
        }

        .advanced-params {
          margin-top: 20px;
        }
      }
    }
  }

  .mining-results {
    .result-overview {
      margin-bottom: 20px;

      .overview-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 20px;

        .overview-item {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 16px;
          background: #f8f9fa;
          border-radius: 6px;

          .overview-icon {
            font-size: 24px;
            color: #409eff;
          }

          .overview-content {
            .overview-value {
              font-size: 20px;
              font-weight: 700;
              color: #303133;
              margin-bottom: 2px;
            }

            .overview-label {
              font-size: 12px;
              color: #909399;
            }
          }
        }
      }
    }

    .result-patterns {
      margin-bottom: 20px;

      .patterns-header {
        display: flex;
        justify-content: space-between;
        align-items: center;

        h4 {
          margin: 0;
          color: #303133;
          font-size: 16px;
        }
      }

      .patterns-content {
        .patterns-visualization {
          .patterns-chart {
            height: 300px;
            width: 100%;
            margin-bottom: 20px;
          }
        }

        .patterns-list {
          .pattern-display {
            display: flex;
            flex-wrap: wrap;
            gap: 4px;
          }

          .support-text {
            margin-left: 8px;
            font-size: 12px;
            color: #606266;
          }

          .high-confidence {
            color: #67c23a;
            font-weight: 600;
          }

          .medium-confidence {
            color: #e6a23c;
            font-weight: 500;
          }

          .low-confidence {
            color: #f56c6c;
          }
        }
      }
    }

    .result-clustering {
      margin-bottom: 20px;

      .clustering-header {
        display: flex;
        justify-content: space-between;
        align-items: center;

        h4 {
          margin: 0;
          color: #303133;
          font-size: 16px;
        }
      }

      .clustering-content {
        .clustering-visualization {
          .clustering-chart {
            height: 400px;
            width: 100%;
            margin-bottom: 20px;
          }
        }

        .clustering-summary {
          h5 {
            margin: 0 0 12px 0;
            color: #303133;
            font-size: 14px;
            font-weight: 600;
          }
        }
      }
    }

    .result-association {
      margin-bottom: 20px;

      .association-content {
        .association-network {
          .association-chart {
            height: 300px;
            width: 100%;
            margin-bottom: 20px;
          }
        }

        .association-rules {
          h5 {
            margin: 0 0 16px 0;
            color: #303133;
            font-size: 14px;
            font-weight: 600;
          }

          .rules-list {
            .rule-item {
              margin-bottom: 16px;
              padding: 16px;
              background: #f8f9fa;
              border-radius: 6px;
              border-left: 4px solid #409eff;

              .rule-expression {
                display: flex;
                align-items: center;
                gap: 12px;
                margin-bottom: 12px;

                .rule-antecedent,
                .rule-consequent {
                  display: flex;
                  gap: 4px;
                  flex-wrap: wrap;
                }

                .rule-arrow {
                  font-size: 16px;
                  color: #909399;
                }
              }

              .rule-metrics {
                display: flex;
                gap: 20px;
                margin-bottom: 8px;

                .metric {
                  display: flex;
                  gap: 4px;
                  font-size: 13px;

                  .metric-label {
                    color: #606266;
                  }

                  .metric-value {
                    color: #303133;
                    font-weight: 500;
                  }
                }
              }

              .rule-interpretation {
                font-size: 12px;
                color: #606266;
                font-style: italic;
              }
            }
          }
        }
      }
    }

    .result-insights {
      .insights-content {
        .insights-list {
          margin-bottom: 24px;

          .insight-item {
            margin-bottom: 16px;
            padding: 16px;
            background: #f8f9fa;
            border-radius: 6px;

            .insight-header {
              display: flex;
              align-items: center;
              gap: 8px;
              margin-bottom: 8px;

              .insight-icon {
                font-size: 16px;

                &.info {
                  color: #409eff;
                }
                &.success {
                  color: #67c23a;
                }
                &.warning {
                  color: #e6a23c;
                }
              }

              .insight-title {
                flex: 1;
                font-weight: 500;
                color: #303133;
              }
            }

            .insight-content {
              margin-bottom: 12px;
              color: #606266;
              line-height: 1.5;
            }

            .insight-actions {
              display: flex;
              gap: 8px;
            }
          }
        }

        .insights-summary {
          h5 {
            margin: 0 0 12px 0;
            color: #303133;
            font-size: 14px;
            font-weight: 600;
          }

          ul {
            margin: 0;
            padding-left: 20px;

            li {
              margin-bottom: 6px;
              color: #606266;
              line-height: 1.5;
            }
          }
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .data-mining-panel {
    .result-overview .overview-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}

@media (max-width: 768px) {
  .data-mining-panel {
    .mining-config .config-content {
      :deep(.el-row) {
        flex-direction: column;
        gap: 12px;
      }
    }

    .result-overview .overview-grid {
      grid-template-columns: 1fr;
      gap: 12px;
    }

    .result-patterns .patterns-header,
    .result-clustering .clustering-header {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
    }

    .result-association .association-rules .rules-list .rule-item {
      .rule-expression {
        flex-direction: column;
        align-items: flex-start;
        gap: 8px;
      }

      .rule-metrics {
        flex-direction: column;
        gap: 8px;
      }
    }
  }
}
</style>
