<template>
  <div class="data-comparison-panel">
    <!-- 对比配置区域 -->
    <div class="comparison-config">
      <el-card class="config-card" shadow="never">
        <template #header>
          <div class="config-header">
            <h4>对比设置</h4>
            <el-button type="primary" @click="executeComparison" :loading="comparing">
              <el-icon><TrendCharts /></el-icon>
              {{ comparing ? '分析中...' : '开始对比' }}
            </el-button>
          </div>
        </template>
        
        <div class="config-content">
          <el-row :gutter="20">
            <!-- 对比维度 -->
            <el-col :span="6">
              <div class="config-section">
                <h5>对比维度</h5>
                <el-select v-model="config.dimension" placeholder="选择对比维度">
                  <el-option label="时间对比" value="time" />
                  <el-option label="设备对比" value="device" />
                  <el-option label="标签对比" value="tag" />
                  <el-option label="分组对比" value="group" />
                </el-select>
              </div>
            </el-col>
            
            <!-- 对比类型 -->
            <el-col :span="6">
              <div class="config-section">
                <h5>对比类型</h5>
                <el-select v-model="config.type" placeholder="选择对比类型">
                  <el-option label="数值对比" value="value" />
                  <el-option label="趋势对比" value="trend" />
                  <el-option label="分布对比" value="distribution" />
                  <el-option label="相关性分析" value="correlation" />
                </el-select>
              </div>
            </el-col>
            
            <!-- 统计方法 -->
            <el-col :span="6">
              <div class="config-section">
                <h5>统计方法</h5>
                <el-select v-model="config.method" placeholder="选择统计方法">
                  <el-option label="平均值" value="avg" />
                  <el-option label="最大值" value="max" />
                  <el-option label="最小值" value="min" />
                  <el-option label="总和" value="sum" />
                  <el-option label="方差" value="variance" />
                </el-select>
              </div>
            </el-col>
            
            <!-- 时间粒度 -->
            <el-col :span="6">
              <div class="config-section">
                <h5>时间粒度</h5>
                <el-select v-model="config.granularity" placeholder="选择时间粒度">
                  <el-option label="分钟" value="minute" />
                  <el-option label="小时" value="hour" />
                  <el-option label="天" value="day" />
                  <el-option label="周" value="week" />
                  <el-option label="月" value="month" />
                </el-select>
              </div>
            </el-col>
          </el-row>
          
          <!-- 对比目标选择 -->
          <div class="comparison-targets">
            <h5>对比目标</h5>
            <div class="targets-container">
              <div
                v-for="(target, index) in config.targets"
                :key="index"
                class="target-item"
              >
                <div class="target-header">
                  <span class="target-label">目标 {{ index + 1 }}</span>
                  <el-button
                    v-if="config.targets.length > 2"
                    size="small"
                    text
                    type="danger"
                    @click="removeTarget(index)"
                  >
                    <el-icon><Close /></el-icon>
                  </el-button>
                </div>
                
                <el-row :gutter="12">
                  <el-col :span="8">
                    <el-select
                      v-model="target.deviceId"
                      placeholder="选择设备"
                      filterable
                    >
                      <el-option
                        v-for="device in devices"
                        :key="device.id"
                        :label="device.name"
                        :value="device.id"
                      />
                    </el-select>
                  </el-col>
                  
                  <el-col :span="8">
                    <el-select
                      v-model="target.tagId"
                      placeholder="选择标签"
                      filterable
                    >
                      <el-option
                        v-for="tag in tags"
                        :key="tag.id"
                        :label="tag.name"
                        :value="tag.id"
                      />
                    </el-select>
                  </el-col>
                  
                  <el-col :span="8">
                    <el-date-picker
                      v-model="target.dateRange"
                      type="datetimerange"
                      range-separator="至"
                      start-placeholder="开始时间"
                      end-placeholder="结束时间"
                      format="MM-DD HH:mm"
                      value-format="YYYY-MM-DD HH:mm:ss"
                      size="default"
                    />
                  </el-col>
                </el-row>
              </div>
              
              <el-button
                v-if="config.targets.length < 5"
                class="add-target-btn"
                dashed
                @click="addTarget"
              >
                <el-icon><Plus /></el-icon>
                添加对比目标
              </el-button>
            </div>
          </div>
        </div>
      </el-card>
    </div>
    
    <!-- 对比结果区域 -->
    <div v-if="comparisonResults" class="comparison-results">
      <!-- 结果概览 -->
      <el-card class="result-summary" shadow="never">
        <template #header>
          <h4>对比结果概览</h4>
        </template>
        
        <div class="summary-grid">
          <div class="summary-item">
            <div class="summary-icon">
              <el-icon><DataAnalysis /></el-icon>
            </div>
            <div class="summary-content">
              <div class="summary-value">{{ comparisonResults.summary.dataPoints }}</div>
              <div class="summary-label">对比数据点</div>
            </div>
          </div>
          
          <div class="summary-item">
            <div class="summary-icon">
              <el-icon><TrendCharts /></el-icon>
            </div>
            <div class="summary-content">
              <div class="summary-value">{{ comparisonResults.summary.variance }}%</div>
              <div class="summary-label">变异系数</div>
            </div>
          </div>
          
          <div class="summary-item">
            <div class="summary-icon">
              <el-icon><Warning /></el-icon>
            </div>
            <div class="summary-content">
              <div class="summary-value">{{ comparisonResults.summary.anomalies }}</div>
              <div class="summary-label">异常点</div>
            </div>
          </div>
          
          <div class="summary-item">
            <div class="summary-icon">
              <el-icon><CircleCheck /></el-icon>
            </div>
            <div class="summary-content">
              <div class="summary-value">{{ comparisonResults.summary.correlation }}%</div>
              <div class="summary-label">相关性</div>
            </div>
          </div>
        </div>
      </el-card>
      
      <!-- 对比图表 -->
      <el-card class="result-charts" shadow="never">
        <template #header>
          <div class="chart-header">
            <h4>对比图表</h4>
            <el-radio-group v-model="activeChartType" size="small">
              <el-radio-button label="overlay">叠加图</el-radio-button>
              <el-radio-button label="parallel">平行图</el-radio-button>
              <el-radio-button label="difference">差值图</el-radio-button>
            </el-radio-group>
          </div>
        </template>
        
        <div class="chart-container">
          <div ref="comparisonChartRef" class="comparison-chart"></div>
        </div>
      </el-card>
      
      <!-- 统计分析 -->
      <el-card class="result-statistics" shadow="never">
        <template #header>
          <h4>统计分析</h4>
        </template>
        
        <el-row :gutter="20">
          <el-col :span="12">
            <!-- 描述性统计 -->
            <div class="stats-section">
              <h5>描述性统计</h5>
              <el-table :data="comparisonResults.statistics" stripe style="width: 100%">
                <el-table-column prop="target" label="对比目标" width="120" />
                <el-table-column prop="mean" label="均值" width="80" />
                <el-table-column prop="median" label="中位数" width="80" />
                <el-table-column prop="std" label="标准差" width="80" />
                <el-table-column prop="cv" label="变异系数" width="80" />
                <el-table-column prop="range" label="极差" />
              </el-table>
            </div>
          </el-col>
          
          <el-col :span="12">
            <!-- 假设检验 -->
            <div class="stats-section">
              <h5>假设检验结果</h5>
              <div class="hypothesis-tests">
                <div class="test-item">
                  <div class="test-name">T检验 (均值比较)</div>
                  <div class="test-result">
                    <span class="test-statistic">t = {{ comparisonResults.tests.tTest.statistic }}</span>
                    <span class="test-pvalue">p = {{ comparisonResults.tests.tTest.pValue }}</span>
                    <el-tag :type="comparisonResults.tests.tTest.significant ? 'success' : 'info'" size="small">
                      {{ comparisonResults.tests.tTest.significant ? '显著差异' : '无显著差异' }}
                    </el-tag>
                  </div>
                </div>
                
                <div class="test-item">
                  <div class="test-name">F检验 (方差比较)</div>
                  <div class="test-result">
                    <span class="test-statistic">F = {{ comparisonResults.tests.fTest.statistic }}</span>
                    <span class="test-pvalue">p = {{ comparisonResults.tests.fTest.pValue }}</span>
                    <el-tag :type="comparisonResults.tests.fTest.significant ? 'success' : 'info'" size="small">
                      {{ comparisonResults.tests.fTest.significant ? '显著差异' : '无显著差异' }}
                    </el-tag>
                  </div>
                </div>
                
                <div class="test-item">
                  <div class="test-name">相关性检验</div>
                  <div class="test-result">
                    <span class="test-statistic">r = {{ comparisonResults.tests.correlation.coefficient }}</span>
                    <span class="test-pvalue">p = {{ comparisonResults.tests.correlation.pValue }}</span>
                    <el-tag :type="getCorrelationLevel(comparisonResults.tests.correlation.coefficient)" size="small">
                      {{ getCorrelationText(comparisonResults.tests.correlation.coefficient) }}
                    </el-tag>
                  </div>
                </div>
              </div>
            </div>
          </el-col>
        </el-row>
      </el-card>
      
      <!-- 异常检测 -->
      <el-card class="result-anomalies" shadow="never">
        <template #header>
          <div class="anomaly-header">
            <h4>异常点检测</h4>
            <el-select v-model="anomalyMethod" size="small" style="width: 150px">
              <el-option label="Z-Score" value="zscore" />
              <el-option label="IQR方法" value="iqr" />
              <el-option label="孤立森林" value="isolation" />
            </el-select>
          </div>
        </template>
        
        <div class="anomaly-content">
          <div class="anomaly-chart">
            <div ref="anomalyChartRef" class="anomaly-chart-container"></div>
          </div>
          
          <div class="anomaly-list">
            <h5>检测到的异常点</h5>
            <el-table :data="comparisonResults.anomalies" max-height="300" style="width: 100%">
              <el-table-column prop="timestamp" label="时间" width="150" />
              <el-table-column prop="target" label="目标" width="100" />
              <el-table-column prop="value" label="数值" width="100" />
              <el-table-column prop="score" label="异常分数" width="100" />
              <el-table-column prop="type" label="类型" width="100">
                <template #default="{ row }">
                  <el-tag :type="getAnomalyType(row.type)" size="small">
                    {{ row.type }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column prop="description" label="描述" />
            </el-table>
          </div>
        </div>
      </el-card>
      
      <!-- 对比结论 -->
      <el-card class="result-conclusion" shadow="never">
        <template #header>
          <h4>分析结论</h4>
        </template>
        
        <div class="conclusion-content">
          <div class="conclusion-insights">
            <h5>主要发现</h5>
            <ul>
              <li v-for="insight in comparisonResults.insights" :key="insight.id" class="insight-item">
                <el-icon class="insight-icon" :class="insight.type">
                  <component :is="getInsightIcon(insight.type)" />
                </el-icon>
                <span class="insight-text">{{ insight.text }}</span>
                <el-tag :type="insight.priority" size="small">{{ insight.priority }}</el-tag>
              </li>
            </ul>
          </div>
          
          <div class="conclusion-recommendations">
            <h5>建议措施</h5>
            <div class="recommendations-list">
              <div
                v-for="rec in comparisonResults.recommendations"
                :key="rec.id"
                class="recommendation-item"
              >
                <div class="rec-priority">
                  <el-tag :type="rec.priority" size="small">{{ rec.priority }}</el-tag>
                </div>
                <div class="rec-content">
                  <div class="rec-title">{{ rec.title }}</div>
                  <div class="rec-description">{{ rec.description }}</div>
                </div>
                <div class="rec-action">
                  <el-button size="small" @click="handleRecommendation(rec)">
                    执行
                  </el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-card>
    </div>
    
    <!-- 空状态 -->
    <el-empty v-else description="配置对比参数并开始分析" class="empty-state">
      <el-button type="primary" @click="executeComparison" :disabled="!canCompare">
        开始数据对比分析
      </el-button>
    </el-empty>
  </div>
</template>

<script setup lang="ts">
/**
 * DataComparisonPanel —— 数据对比分析面板
 *
 * 📝 Responsibilities:
 *  1. 数据对比配置和执行
 *  2. 多维度对比分析
 *  3. 统计检验和异常检测
 *  4. 结果可视化和建议
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
  TrendCharts,
  Close,
  Plus,
  DataAnalysis,
  Warning,
  CircleCheck,
  InfoFilled,
  SuccessFilled,
  WarningFilled
} from '@element-plus/icons-vue'

// ===== 响应式数据 =====
const comparing = ref(false)
const activeChartType = ref('overlay')
const anomalyMethod = ref('zscore')

// 对比配置
const config = ref({
  dimension: 'time',
  type: 'value',
  method: 'avg',
  granularity: 'hour',
  targets: [
    {
      deviceId: '',
      tagId: '',
      dateRange: []
    },
    {
      deviceId: '',
      tagId: '',
      dateRange: []
    }
  ]
})

// 图表引用
const comparisonChartRef = ref<HTMLDivElement>()
const anomalyChartRef = ref<HTMLDivElement>()
const comparisonChart = ref<echarts.ECharts>()
const anomalyChart = ref<echarts.ECharts>()

// 模拟数据
const devices = ref([
  { id: '1', name: 'PLC-01' },
  { id: '2', name: 'PLC-02' },
  { id: '3', name: 'Sensor-01' },
  { id: '4', name: 'Sensor-02' }
])

const tags = ref([
  { id: '1', name: '温度' },
  { id: '2', name: '压力' },
  { id: '3', name: '流量' },
  { id: '4', name: '功率' }
])

// 对比结果
const comparisonResults = ref(null)

// ===== 计算属性 =====
const canCompare = computed(() => {
  return config.value.targets.length >= 2 &&
         config.value.targets.every(target => 
           target.deviceId && target.tagId && target.dateRange?.length === 2
         )
})

// ===== 方法 =====

/**
 * 添加对比目标
 */
function addTarget() {
  config.value.targets.push({
    deviceId: '',
    tagId: '',
    dateRange: []
  })
}

/**
 * 移除对比目标
 */
function removeTarget(index: number) {
  config.value.targets.splice(index, 1)
}

/**
 * 执行对比分析
 */
async function executeComparison() {
  if (!canCompare.value) {
    ElMessage.warning('请完整配置对比目标')
    return
  }
  
  comparing.value = true
  
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // 生成模拟对比结果
    comparisonResults.value = generateMockResults()
    
    // 初始化图表
    nextTick(() => {
      initComparisonChart()
      initAnomalyChart()
    })
    
    ElMessage.success('对比分析完成')
    
  } catch (error) {
    console.error('对比分析失败:', error)
    ElMessage.error('对比分析失败')
  } finally {
    comparing.value = false
  }
}

/**
 * 生成模拟对比结果
 */
function generateMockResults() {
  return {
    summary: {
      dataPoints: 1440,
      variance: 12.5,
      anomalies: 8,
      correlation: 78
    },
    statistics: [
      {
        target: '目标1',
        mean: 25.6,
        median: 25.2,
        std: 3.2,
        cv: 0.125,
        range: 15.8
      },
      {
        target: '目标2',
        mean: 28.1,
        median: 27.9,
        std: 2.8,
        cv: 0.100,
        range: 12.3
      }
    ],
    tests: {
      tTest: {
        statistic: 2.34,
        pValue: 0.019,
        significant: true
      },
      fTest: {
        statistic: 1.31,
        pValue: 0.156,
        significant: false
      },
      correlation: {
        coefficient: 0.78,
        pValue: 0.001
      }
    },
    anomalies: [
      {
        timestamp: '2025-07-27 10:15:00',
        target: '目标1',
        value: 35.2,
        score: 2.8,
        type: '上限异常',
        description: '超出正常范围上限'
      },
      {
        timestamp: '2025-07-27 14:30:00',
        target: '目标2',
        value: 18.1,
        score: -2.1,
        type: '下限异常',
        description: '低于正常范围下限'
      }
    ],
    insights: [
      {
        id: '1',
        text: '目标2的平均值比目标1高出9.8%，差异具有统计学意义',
        type: 'info',
        priority: 'high'
      },
      {
        id: '2',
        text: '两个目标的数据呈现强正相关关系（r=0.78）',
        type: 'success',
        priority: 'medium'
      },
      {
        id: '3',
        text: '检测到8个异常点，主要集中在上午10-11点时段',
        type: 'warning',
        priority: 'high'
      }
    ],
    recommendations: [
      {
        id: '1',
        title: '优化设备参数',
        description: '建议调整目标1的控制参数，减少数值波动',
        priority: 'high'
      },
      {
        id: '2',
        title: '加强监控',
        description: '在10-11点时段增加数据采集频率',
        priority: 'medium'
      },
      {
        id: '3',
        title: '设置告警',
        description: '为检测到的异常模式设置自动告警规则',
        priority: 'medium'
      }
    ]
  }
}

/**
 * 初始化对比图表
 */
function initComparisonChart() {
  if (!comparisonChartRef.value) return
  
  comparisonChart.value = echarts.init(comparisonChartRef.value)
  updateComparisonChart()
}

/**
 * 更新对比图表
 */
function updateComparisonChart() {
  if (!comparisonChart.value) return
  
  // 生成模拟时间序列数据
  const times = []
  const data1 = []
  const data2 = []
  
  for (let i = 0; i < 24; i++) {
    const time = new Date()
    time.setHours(i, 0, 0, 0)
    times.push(time)
    
    data1.push(25 + Math.random() * 10 + Math.sin(i / 4) * 3)
    data2.push(28 + Math.random() * 8 + Math.cos(i / 4) * 2)
  }
  
  let option = {}
  
  if (activeChartType.value === 'overlay') {
    // 叠加图
    option = {
      animation: true,
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        top: '10%',
        containLabel: true
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: {
          type: 'cross'
        }
      },
      legend: {
        data: ['目标1', '目标2']
      },
      xAxis: {
        type: 'time',
        data: times
      },
      yAxis: {
        type: 'value'
      },
      series: [
        {
          name: '目标1',
          type: 'line',
          data: data1.map((value, index) => [times[index], value]),
          smooth: true,
          lineStyle: { color: '#409EFF' },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#409EFF40' },
              { offset: 1, color: '#409EFF10' }
            ])
          }
        },
        {
          name: '目标2',
          type: 'line',
          data: data2.map((value, index) => [times[index], value]),
          smooth: true,
          lineStyle: { color: '#67C23A' },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#67C23A40' },
              { offset: 1, color: '#67C23A10' }
            ])
          }
        }
      ]
    }
  } else if (activeChartType.value === 'parallel') {
    // 平行图
    option = {
      animation: true,
      grid: [
        { left: '7%', right: '52%', top: '10%', bottom: '10%' },
        { left: '55%', right: '7%', top: '10%', bottom: '10%' }
      ],
      tooltip: {
        trigger: 'axis'
      },
      xAxis: [
        { type: 'time', gridIndex: 0 },
        { type: 'time', gridIndex: 1 }
      ],
      yAxis: [
        { type: 'value', gridIndex: 0, name: '目标1' },
        { type: 'value', gridIndex: 1, name: '目标2' }
      ],
      series: [
        {
          name: '目标1',
          type: 'line',
          xAxisIndex: 0,
          yAxisIndex: 0,
          data: data1.map((value, index) => [times[index], value]),
          smooth: true,
          lineStyle: { color: '#409EFF' }
        },
        {
          name: '目标2',
          type: 'line',
          xAxisIndex: 1,
          yAxisIndex: 1,
          data: data2.map((value, index) => [times[index], value]),
          smooth: true,
          lineStyle: { color: '#67C23A' }
        }
      ]
    }
  } else {
    // 差值图
    const differences = data1.map((v1, index) => v1 - data2[index])
    
    option = {
      animation: true,
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        top: '10%',
        containLabel: true
      },
      tooltip: {
        trigger: 'axis'
      },
      xAxis: {
        type: 'time',
        data: times
      },
      yAxis: {
        type: 'value',
        name: '差值 (目标1 - 目标2)'
      },
      series: [
        {
          name: '差值',
          type: 'line',
          data: differences.map((value, index) => [times[index], value]),
          smooth: true,
          lineStyle: { color: '#E6A23C' },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#E6A23C40' },
              { offset: 1, color: '#E6A23C10' }
            ])
          },
          markLine: {
            data: [
              { yAxis: 0, lineStyle: { color: '#F56C6C', type: 'dashed' } }
            ]
          }
        }
      ]
    }
  }
  
  comparisonChart.value.setOption(option)
}

/**
 * 初始化异常检测图表
 */
function initAnomalyChart() {
  if (!anomalyChartRef.value) return
  
  anomalyChart.value = echarts.init(anomalyChartRef.value)
  
  // 生成包含异常点的数据
  const times = []
  const normalData = []
  const anomalyData = []
  
  for (let i = 0; i < 24; i++) {
    const time = new Date()
    time.setHours(i, 0, 0, 0)
    times.push(time)
    
    let value = 25 + Math.random() * 10 + Math.sin(i / 4) * 3
    
    // 人为加入一些异常点
    if (i === 10 || i === 14) {
      value += Math.random() > 0.5 ? 15 : -15
      anomalyData.push([time, value])
      normalData.push([time, null])
    } else {
      normalData.push([time, value])
      anomalyData.push([time, null])
    }
  }
  
  const option = {
    animation: true,
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      top: '10%',
      containLabel: true
    },
    tooltip: {
      trigger: 'axis'
    },
    legend: {
      data: ['正常数据', '异常点']
    },
    xAxis: {
      type: 'time'
    },
    yAxis: {
      type: 'value'
    },
    series: [
      {
        name: '正常数据',
        type: 'line',
        data: normalData,
        smooth: true,
        lineStyle: { color: '#409EFF' },
        symbol: 'none'
      },
      {
        name: '异常点',
        type: 'scatter',
        data: anomalyData,
        symbolSize: 10,
        itemStyle: { color: '#F56C6C' },
        emphasis: {
          scale: true,
          scaleSize: 15
        }
      }
    ]
  }
  
  anomalyChart.value.setOption(option)
}

/**
 * 获取相关性等级
 */
function getCorrelationLevel(coefficient: number): string {
  const abs = Math.abs(coefficient)
  if (abs >= 0.8) return 'success'
  if (abs >= 0.5) return 'warning'
  return 'info'
}

/**
 * 获取相关性文本
 */
function getCorrelationText(coefficient: number): string {
  const abs = Math.abs(coefficient)
  if (abs >= 0.8) return '强相关'
  if (abs >= 0.5) return '中等相关'
  if (abs >= 0.3) return '弱相关'
  return '无相关'
}

/**
 * 获取异常类型样式
 */
function getAnomalyType(type: string): string {
  if (type.includes('上限')) return 'danger'
  if (type.includes('下限')) return 'warning'
  return 'info'
}

/**
 * 获取洞察图标
 */
function getInsightIcon(type: string): string {
  const iconMap: { [key: string]: string } = {
    info: 'InfoFilled',
    success: 'SuccessFilled',
    warning: 'WarningFilled'
  }
  return iconMap[type] || 'InfoFilled'
}

/**
 * 处理建议
 */
function handleRecommendation(recommendation: any) {
  ElMessage.success(`正在执行建议: ${recommendation.title}`)
}

/**
 * 处理窗口大小变化
 */
function handleResize() {
  comparisonChart.value?.resize()
  anomalyChart.value?.resize()
}

// ===== 生命周期 =====
onMounted(() => {
  window.addEventListener('resize', handleResize)
  
  // 设置默认日期范围
  const now = new Date()
  const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000)
  
  const defaultRange = [
    yesterday.toISOString().slice(0, 19).replace('T', ' '),
    now.toISOString().slice(0, 19).replace('T', ' ')
  ]
  
  config.value.targets.forEach(target => {
    target.dateRange = [...defaultRange]
  })
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  comparisonChart.value?.dispose()
  anomalyChart.value?.dispose()
})

// ===== 监听器 =====
watch(() => activeChartType.value, () => {
  updateComparisonChart()
})

watch(() => anomalyMethod.value, () => {
  // 重新计算异常检测
  ElMessage.info(`切换到${anomalyMethod.value}异常检测方法`)
})
</script>

<style scoped lang="scss">
.data-comparison-panel {
  .comparison-config {
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
          margin-bottom: 20px;
          
          h5 {
            margin: 0 0 8px 0;
            color: #606266;
            font-size: 13px;
            font-weight: 500;
          }
        }
        
        .comparison-targets {
          margin-top: 24px;
          
          h5 {
            margin: 0 0 16px 0;
            color: #303133;
            font-size: 14px;
            font-weight: 600;
          }
          
          .targets-container {
            .target-item {
              margin-bottom: 16px;
              padding: 16px;
              background: #fafafa;
              border-radius: 6px;
              border: 1px solid #ebeef5;
              
              .target-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: 12px;
                
                .target-label {
                  font-size: 14px;
                  font-weight: 500;
                  color: #409eff;
                }
              }
            }
            
            .add-target-btn {
              width: 100%;
              border-style: dashed;
              color: #909399;
              
              &:hover {
                border-color: #409eff;
                color: #409eff;
              }
            }
          }
        }
      }
    }
  }
  
  .comparison-results {
    .result-summary {
      margin-bottom: 20px;
      
      .summary-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 20px;
        
        .summary-item {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 16px;
          background: #f8f9fa;
          border-radius: 6px;
          
          .summary-icon {
            font-size: 24px;
            color: #409eff;
          }
          
          .summary-content {
            .summary-value {
              font-size: 20px;
              font-weight: 700;
              color: #303133;
              margin-bottom: 2px;
            }
            
            .summary-label {
              font-size: 12px;
              color: #909399;
            }
          }
        }
      }
    }
    
    .result-charts {
      margin-bottom: 20px;
      
      .chart-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        
        h4 {
          margin: 0;
          color: #303133;
          font-size: 16px;
        }
      }
      
      .chart-container {
        .comparison-chart {
          height: 400px;
          width: 100%;
        }
      }
    }
    
    .result-statistics {
      margin-bottom: 20px;
      
      .stats-section {
        h5 {
          margin: 0 0 16px 0;
          color: #303133;
          font-size: 14px;
          font-weight: 600;
        }
        
        .hypothesis-tests {
          .test-item {
            margin-bottom: 16px;
            padding: 12px;
            background: #f8f9fa;
            border-radius: 6px;
            
            .test-name {
              font-size: 14px;
              font-weight: 500;
              color: #303133;
              margin-bottom: 8px;
            }
            
            .test-result {
              display: flex;
              align-items: center;
              gap: 12px;
              
              .test-statistic,
              .test-pvalue {
                font-size: 12px;
                color: #606266;
                font-family: monospace;
              }
            }
          }
        }
      }
    }
    
    .result-anomalies {
      margin-bottom: 20px;
      
      .anomaly-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        
        h4 {
          margin: 0;
          color: #303133;
          font-size: 16px;
        }
      }
      
      .anomaly-content {
        .anomaly-chart {
          .anomaly-chart-container {
            height: 300px;
            width: 100%;
            margin-bottom: 20px;
          }
        }
        
        .anomaly-list {
          h5 {
            margin: 0 0 12px 0;
            color: #303133;
            font-size: 14px;
            font-weight: 600;
          }
        }
      }
    }
    
    .result-conclusion {
      .conclusion-content {
        .conclusion-insights {
          margin-bottom: 24px;
          
          h5 {
            margin: 0 0 16px 0;
            color: #303133;
            font-size: 14px;
            font-weight: 600;
          }
          
          .insight-item {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-bottom: 8px;
            line-height: 1.5;
            
            .insight-icon {
              font-size: 16px;
              
              &.info { color: #409eff; }
              &.success { color: #67c23a; }
              &.warning { color: #e6a23c; }
            }
            
            .insight-text {
              flex: 1;
              color: #606266;
            }
          }
        }
        
        .conclusion-recommendations {
          h5 {
            margin: 0 0 16px 0;
            color: #303133;
            font-size: 14px;
            font-weight: 600;
          }
          
          .recommendations-list {
            .recommendation-item {
              display: flex;
              align-items: center;
              gap: 12px;
              padding: 12px;
              background: #f8f9fa;
              border-radius: 6px;
              margin-bottom: 8px;
              
              .rec-priority {
                flex-shrink: 0;
              }
              
              .rec-content {
                flex: 1;
                
                .rec-title {
                  font-size: 14px;
                  font-weight: 500;
                  color: #303133;
                  margin-bottom: 2px;
                }
                
                .rec-description {
                  font-size: 12px;
                  color: #606266;
                  line-height: 1.4;
                }
              }
              
              .rec-action {
                flex-shrink: 0;
              }
            }
          }
        }
      }
    }
  }
  
  .empty-state {
    margin: 60px 0;
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .data-comparison-panel {
    .result-summary .summary-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}

@media (max-width: 768px) {
  .data-comparison-panel {
    .comparison-config .config-content .comparison-targets .targets-container .target-item {
      .target-header {
        flex-direction: column;
        align-items: flex-start;
        gap: 8px;
      }
    }
    
    .result-summary .summary-grid {
      grid-template-columns: 1fr;
      gap: 12px;
    }
    
    .result-charts .chart-header {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
    }
    
    .result-anomalies .anomaly-header {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
    }
  }
}
</style>