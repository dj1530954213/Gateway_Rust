<template>
  <div class="predictive-analysis-panel">
    <!-- 预测配置 -->
    <div class="prediction-config">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <h3>预测分析配置</h3>
            <div class="header-actions">
              <el-button size="small" @click="loadTemplate">
                <el-icon><Files /></el-icon>
                加载模板
              </el-button>
              <el-button size="small" @click="saveTemplate">
                <el-icon><Download /></el-icon>
                保存模板
              </el-button>
            </div>
          </div>
        </template>
        
        <el-form :model="config" label-width="120px" class="config-form">
          <el-row :gutter="24">
            <el-col :span="12">
              <el-form-item label="预测类型">
                <el-select v-model="config.predictionType" @change="onPredictionTypeChange">
                  <el-option
                    v-for="type in predictionTypes"
                    :key="type.value"
                    :label="type.label"
                    :value="type.value"
                  >
                    <div class="option-content">
                      <span>{{ type.label }}</span>
                      <small class="option-desc">{{ type.description }}</small>
                    </div>
                  </el-option>
                </el-select>
              </el-form-item>
              
              <el-form-item label="算法模型">
                <el-select v-model="config.algorithm">
                  <el-option
                    v-for="algo in algorithms"
                    :key="algo.value"
                    :label="algo.label"
                    :value="algo.value"
                  />
                </el-select>
              </el-form-item>
              
              <el-form-item label="目标变量">
                <el-select v-model="config.targetVariable" filterable>
                  <el-option
                    v-for="variable in availableVariables"
                    :key="variable.key"
                    :label="variable.name"
                    :value="variable.key"
                  />
                </el-select>
              </el-form-item>
              
              <el-form-item label="特征变量">
                <el-select
                  v-model="config.featureVariables"
                  multiple
                  filterable
                  placeholder="选择特征变量"
                >
                  <el-option
                    v-for="variable in availableVariables"
                    :key="variable.key"
                    :label="variable.name"
                    :value="variable.key"
                  />
                </el-select>
              </el-form-item>
            </el-col>
            
            <el-col :span="12">
              <el-form-item label="训练数据范围">
                <el-date-picker
                  v-model="config.trainingDateRange"
                  type="datetimerange"
                  range-separator="至"
                  start-placeholder="开始时间"
                  end-placeholder="结束时间"
                  format="YYYY-MM-DD HH:mm:ss"
                  value-format="YYYY-MM-DD HH:mm:ss"
                />
              </el-form-item>
              
              <el-form-item label="预测周期">
                <el-row :gutter="8">
                  <el-col :span="16">
                    <el-input-number
                      v-model="config.forecastHorizon"
                      :min="1"
                      :max="365"
                      style="width: 100%"
                    />
                  </el-col>
                  <el-col :span="8">
                    <el-select v-model="config.forecastUnit">
                      <el-option label="小时" value="hours" />
                      <el-option label="天" value="days" />
                      <el-option label="周" value="weeks" />
                      <el-option label="月" value="months" />
                    </el-select>
                  </el-col>
                </el-row>
              </el-form-item>
              
              <el-form-item label="置信度">
                <el-slider
                  v-model="config.confidenceLevel"
                  :min="80"
                  :max="99"
                  :step="1"
                  show-stops
                  show-input
                  :format-tooltip="(val) => `${val}%`"
                />
              </el-form-item>
              
              <el-form-item label="交叉验证">
                <el-row :gutter="8">
                  <el-col :span="12">
                    <el-switch
                      v-model="config.crossValidation"
                      active-text="启用"
                      inactive-text="禁用"
                    />
                  </el-col>
                  <el-col :span="12">
                    <el-input-number
                      v-model="config.cvFolds"
                      :min="2"
                      :max="10"
                      :disabled="!config.crossValidation"
                      placeholder="折数"
                    />
                  </el-col>
                </el-row>
              </el-form-item>
            </el-col>
          </el-row>
          
          <!-- 高级参数 -->
          <el-collapse v-model="activeAdvanced" class="advanced-params">
            <el-collapse-item title="高级参数" name="advanced">
              <el-row :gutter="24">
                <el-col :span="8">
                  <el-form-item label="季节性周期">
                    <el-input-number v-model="config.seasonality" :min="0" placeholder="自动检测" />
                  </el-form-item>
                </el-col>
                <el-col :span="8">
                  <el-form-item label="学习率">
                    <el-input-number
                      v-model="config.learningRate"
                      :min="0.001"
                      :max="1"
                      :step="0.001"
                      :precision="3"
                    />
                  </el-form-item>
                </el-col>
                <el-col :span="8">
                  <el-form-item label="最大迭代">
                    <el-input-number v-model="config.maxIterations" :min="10" :max="10000" />
                  </el-form-item>
                </el-col>
              </el-row>
              
              <el-form-item label="特征工程">
                <el-checkbox-group v-model="config.featureEngineering">
                  <el-checkbox label="lag">滞后特征</el-checkbox>
                  <el-checkbox label="rolling">滚动统计</el-checkbox>
                  <el-checkbox label="diff">差分特征</el-checkbox>
                  <el-checkbox label="fourier">傅里叶变换</el-checkbox>
                </el-checkbox-group>
              </el-form-item>
            </el-collapse-item>
          </el-collapse>
          
          <el-form-item class="form-actions">
            <el-button @click="resetConfig">重置</el-button>
            <el-button @click="validateConfig">验证配置</el-button>
            <el-button type="primary" @click="startPrediction" :loading="isTraining">
              {{ isTraining ? '训练中...' : '开始预测' }}
            </el-button>
          </el-form-item>
        </el-form>
      </el-card>
    </div>
    
    <!-- 训练进度 -->
    <div v-if="isTraining" class="training-progress">
      <el-card shadow="never">
        <template #header>
          <h3>模型训练进度</h3>
        </template>
        
        <div class="progress-content">
          <el-progress
            :percentage="trainingProgress.percentage"
            :status="trainingProgress.status"
            :stroke-width="8"
          />
          
          <div class="progress-details">
            <div class="progress-info">
              <span>当前阶段: {{ trainingProgress.stage }}</span>
              <span>耗时: {{ formatDuration(trainingProgress.elapsed) }}</span>
              <span>预计剩余: {{ formatDuration(trainingProgress.remaining) }}</span>
            </div>
            
            <div class="progress-metrics">
              <div class="metric">
                <span class="metric-label">训练损失:</span>
                <span class="metric-value">{{ trainingProgress.trainLoss }}</span>
              </div>
              <div class="metric">
                <span class="metric-label">验证损失:</span>
                <span class="metric-value">{{ trainingProgress.validLoss }}</span>
              </div>
              <div class="metric">
                <span class="metric-label">准确率:</span>
                <span class="metric-value">{{ trainingProgress.accuracy }}%</span>
              </div>
            </div>
          </div>
        </div>
      </el-card>
    </div>
    
    <!-- 预测结果 -->
    <div v-if="predictionResults" class="prediction-results">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <h3>预测结果</h3>
            <div class="header-actions">
              <el-button size="small" @click="exportResults">
                <el-icon><Download /></el-icon>
                导出结果
              </el-button>
              <el-button size="small" @click="shareResults">
                <el-icon><Share /></el-icon>
                分享结果
              </el-button>
            </div>
          </div>
        </template>
        
        <!-- 结果摘要 -->
        <div class="results-summary">
          <el-row :gutter="16">
            <el-col :span="6">
              <div class="summary-item">
                <div class="summary-icon">
                  <el-icon><TrendCharts /></el-icon>
                </div>
                <div class="summary-content">
                  <div class="summary-value">{{ predictionResults.accuracy }}%</div>
                  <div class="summary-label">预测准确率</div>
                </div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="summary-item">
                <div class="summary-icon">
                  <el-icon><Warning /></el-icon>
                </div>
                <div class="summary-content">
                  <div class="summary-value">{{ predictionResults.mape }}%</div>
                  <div class="summary-label">平均绝对误差</div>
                </div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="summary-item">
                <div class="summary-icon">
                  <el-icon><DataAnalysis /></el-icon>
                </div>
                <div class="summary-content">
                  <div class="summary-value">{{ predictionResults.r2Score }}</div>
                  <div class="summary-label">R²决定系数</div>
                </div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="summary-item">
                <div class="summary-icon">
                  <el-icon><Clock /></el-icon>
                </div>
                <div class="summary-content">
                  <div class="summary-value">{{ predictionResults.trainTime }}s</div>
                  <div class="summary-label">训练耗时</div>
                </div>
              </div>
            </el-col>
          </el-row>
        </div>
        
        <!-- 预测图表 -->
        <div class="prediction-chart">
          <div ref="predictionChartRef" class="chart-container"></div>
        </div>
        
        <!-- 模型评估 -->
        <div class="model-evaluation">
          <el-tabs v-model="activeEvalTab">
            <el-tab-pane label="残差分析" name="residuals">
              <div ref="residualsChartRef" class="eval-chart"></div>
            </el-tab-pane>
            
            <el-tab-pane label="特征重要性" name="importance">
              <div ref="importanceChartRef" class="eval-chart"></div>
            </el-tab-pane>
            
            <el-tab-pane label="预测区间" name="intervals">
              <div ref="intervalsChartRef" class="eval-chart"></div>
            </el-tab-pane>
            
            <el-tab-pane label="模型诊断" name="diagnostics">
              <div class="diagnostics-content">
                <el-descriptions :column="2" border>
                  <el-descriptions-item label="模型类型">
                    {{ getAlgorithmName(config.algorithm) }}
                  </el-descriptions-item>
                  <el-descriptions-item label="训练样本数">
                    {{ predictionResults.sampleCount }}
                  </el-descriptions-item>
                  <el-descriptions-item label="特征数量">
                    {{ config.featureVariables.length }}
                  </el-descriptions-item>
                  <el-descriptions-item label="预测周期">
                    {{ config.forecastHorizon }} {{ config.forecastUnit }}
                  </el-descriptions-item>
                  <el-descriptions-item label="置信水平">
                    {{ config.confidenceLevel }}%
                  </el-descriptions-item>
                  <el-descriptions-item label="交叉验证">
                    {{ config.crossValidation ? `${config.cvFolds}折` : '未启用' }}
                  </el-descriptions-item>
                </el-descriptions>
                
                <div class="metrics-table">
                  <h4>评估指标</h4>
                  <el-table :data="predictionResults.metrics" stripe>
                    <el-table-column prop="metric" label="指标" width="120" />
                    <el-table-column prop="value" label="数值" width="100" />
                    <el-table-column prop="description" label="说明" />
                  </el-table>
                </div>
              </div>
            </el-tab-pane>
          </el-tabs>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * PredictiveAnalysisPanel —— 预测分析面板
 *
 * 📝 Responsibilities:
 *  1. 预测模型配置和参数设置
 *  2. 模型训练过程监控
 *  3. 预测结果可视化
 *  4. 模型评估和诊断
 *
 * 📦 Dependencies:
 *  - ECharts 图表库
 *  - Element Plus UI组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import * as echarts from 'echarts'
import {
  Files,
  Download,
  Share,
  TrendCharts,
  Warning,
  DataAnalysis,
  Clock
} from '@element-plus/icons-vue'

// ===== 响应式数据 =====
const activeAdvanced = ref([])
const activeEvalTab = ref('residuals')
const isTraining = ref(false)

// 图表引用
const predictionChartRef = ref<HTMLDivElement>()
const residualsChartRef = ref<HTMLDivElement>()
const importanceChartRef = ref<HTMLDivElement>()
const intervalsChartRef = ref<HTMLDivElement>()

const predictionChart = ref<echarts.ECharts>()
const residualsChart = ref<echarts.ECharts>()
const importanceChart = ref<echarts.ECharts>()
const intervalsChart = ref<echarts.ECharts>()

// 预测配置
const config = ref({
  predictionType: 'timeseries',
  algorithm: 'lstm',
  targetVariable: '',
  featureVariables: [],
  trainingDateRange: [],
  forecastHorizon: 24,
  forecastUnit: 'hours',
  confidenceLevel: 95,
  crossValidation: true,
  cvFolds: 5,
  seasonality: null,
  learningRate: 0.001,
  maxIterations: 1000,
  featureEngineering: ['lag', 'rolling']
})

// 预测类型选项
const predictionTypes = ref([
  {
    value: 'timeseries',
    label: '时间序列预测',
    description: '基于历史时间序列数据预测未来趋势'
  },
  {
    value: 'regression',
    label: '回归预测',
    description: '基于特征变量预测连续数值'
  },
  {
    value: 'classification',
    label: '分类预测',
    description: '预测离散类别或状态'
  },
  {
    value: 'anomaly',
    label: '异常预测',
    description: '预测异常事件发生概率'
  }
])

// 算法选项
const algorithms = computed(() => {
  const algorithmMap: { [key: string]: any[] } = {
    timeseries: [
      { value: 'lstm', label: 'LSTM神经网络' },
      { value: 'arima', label: 'ARIMA模型' },
      { value: 'prophet', label: 'Prophet预测' },
      { value: 'gru', label: 'GRU神经网络' }
    ],
    regression: [
      { value: 'linear', label: '线性回归' },
      { value: 'ridge', label: '岭回归' },
      { value: 'lasso', label: 'Lasso回归' },
      { value: 'randomforest', label: '随机森林' },
      { value: 'xgboost', label: 'XGBoost' }
    ],
    classification: [
      { value: 'logistic', label: '逻辑回归' },
      { value: 'svm', label: '支持向量机' },
      { value: 'randomforest', label: '随机森林' },
      { value: 'neuralnet', label: '神经网络' }
    ],
    anomaly: [
      { value: 'isolation', label: '孤立森林' },
      { value: 'ocsvm', label: '一类SVM' },
      { value: 'autoencoder', label: '自编码器' },
      { value: 'lof', label: '局部异常因子' }
    ]
  }
  
  return algorithmMap[config.value.predictionType] || []
})

// 可用变量
const availableVariables = ref([
  { key: 'temperature', name: '温度' },
  { key: 'pressure', name: '压力' },
  { key: 'flow', name: '流量' },
  { key: 'power', name: '功率' },
  { key: 'vibration', name: '振动' },
  { key: 'efficiency', name: '效率' },
  { key: 'quality', name: '质量指标' },
  { key: 'energy', name: '能耗' }
])

// 训练进度
const trainingProgress = ref({
  percentage: 0,
  status: '',
  stage: '',
  elapsed: 0,
  remaining: 0,
  trainLoss: 0,
  validLoss: 0,
  accuracy: 0
})

// 预测结果
const predictionResults = ref(null as any)

// ===== 方法 =====

/**
 * 预测类型变化处理
 */
function onPredictionTypeChange() {
  // 重置算法选择
  config.value.algorithm = algorithms.value[0]?.value || ''
  
  // 重置特征变量
  config.value.featureVariables = []
}

/**
 * 开始预测
 */
async function startPrediction() {
  // 验证配置
  if (!validateConfiguration()) {
    return
  }
  
  isTraining.value = true
  trainingProgress.value = {
    percentage: 0,
    status: 'active',
    stage: '数据预处理',
    elapsed: 0,
    remaining: 0,
    trainLoss: 0,
    validLoss: 0,
    accuracy: 0
  }
  
  try {
    // 模拟训练过程
    await simulateTraining()
    
    // 生成预测结果
    await generatePredictionResults()
    
    ElMessage.success('预测分析完成')
  } catch (error) {
    ElMessage.error(`预测分析失败: ${  error}`)
    trainingProgress.value.status = 'exception'
  } finally {
    isTraining.value = false
  }
}

/**
 * 验证配置
 */
function validateConfiguration(): boolean {
  if (!config.value.targetVariable) {
    ElMessage.warning('请选择目标变量')
    return false
  }
  
  if (config.value.featureVariables.length === 0) {
    ElMessage.warning('请至少选择一个特征变量')
    return false
  }
  
  if (!config.value.trainingDateRange.length) {
    ElMessage.warning('请选择训练数据时间范围')
    return false
  }
  
  return true
}

/**
 * 模拟训练过程
 */
async function simulateTraining(): Promise<void> {
  const stages = [
    '数据预处理',
    '特征工程',
    '模型训练',
    '交叉验证',
    '模型优化'
  ]
  
  for (let i = 0; i < stages.length; i++) {
    trainingProgress.value.stage = stages[i]
    
    for (let j = 0; j <= 20; j++) {
      trainingProgress.value.percentage = (i * 20) + j
      trainingProgress.value.elapsed = trainingProgress.value.percentage * 100
      trainingProgress.value.remaining = (100 - trainingProgress.value.percentage) * 100
      trainingProgress.value.trainLoss = Math.max(0.1, 1 - trainingProgress.value.percentage / 120)
      trainingProgress.value.validLoss = Math.max(0.15, 1.2 - trainingProgress.value.percentage / 110)
      trainingProgress.value.accuracy = Math.min(99, trainingProgress.value.percentage * 0.9)
      
      await new Promise(resolve => setTimeout(resolve, 50))
    }
  }
  
  trainingProgress.value.percentage = 100
  trainingProgress.value.status = 'success'
  trainingProgress.value.stage = '训练完成'
}

/**
 * 生成预测结果
 */
async function generatePredictionResults() {
  // 模拟预测结果数据
  const results = {
    accuracy: 92.5,
    mape: 5.8,
    r2Score: 0.89,
    trainTime: 45.2,
    sampleCount: 1250,
    predictions: generateMockPredictions(),
    metrics: [
      { metric: 'MAE', value: '2.34', description: '平均绝对误差' },
      { metric: 'RMSE', value: '3.12', description: '均方根误差' },
      { metric: 'MAPE', value: '5.8%', description: '平均绝对百分比误差' },
      { metric: 'R²', value: '0.89', description: '决定系数' }
    ]
  }
  
  predictionResults.value = results
  
  // 初始化图表
  await nextTick()
  initPredictionCharts()
}

/**
 * 生成模拟预测数据
 */
function generateMockPredictions() {
  const data = {
    historical: [],
    predictions: [],
    confidence: {
      upper: [],
      lower: []
    }
  }
  
  const now = new Date()
  
  // 历史数据 (过去30天)
  for (let i = 30; i >= 0; i--) {
    const time = new Date(now.getTime() - i * 24 * 60 * 60 * 1000)
    const value = 100 + Math.sin(i / 5) * 20 + Math.random() * 10
    
    data.historical.push({
      timestamp: time.toISOString().slice(0, 19).replace('T', ' '),
      value
    })
  }
  
  // 预测数据 (未来7天)
  for (let i = 1; i <= 7; i++) {
    const time = new Date(now.getTime() + i * 24 * 60 * 60 * 1000)
    const baseValue = 100 + Math.sin((30 + i) / 5) * 20
    const prediction = baseValue + (Math.random() - 0.5) * 5
    
    data.predictions.push({
      timestamp: time.toISOString().slice(0, 19).replace('T', ' '),
      value: prediction
    })
    
    data.confidence.upper.push(prediction + 10)
    data.confidence.lower.push(prediction - 10)
  }
  
  return data
}

/**
 * 初始化预测图表
 */
function initPredictionCharts() {
  if (predictionChartRef.value) {
    predictionChart.value = echarts.init(predictionChartRef.value)
    updatePredictionChart()
  }
  
  if (residualsChartRef.value) {
    residualsChart.value = echarts.init(residualsChartRef.value)
    updateResidualsChart()
  }
  
  if (importanceChartRef.value) {
    importanceChart.value = echarts.init(importanceChartRef.value)
    updateImportanceChart()
  }
  
  if (intervalsChartRef.value) {
    intervalsChart.value = echarts.init(intervalsChartRef.value)
    updateIntervalsChart()
  }
}

/**
 * 更新预测图表
 */
function updatePredictionChart() {
  if (!predictionChart.value || !predictionResults.value) return
  
  const data = predictionResults.value.predictions
  
  const option = {
    title: {
      text: '预测结果',
      left: 'center',
      textStyle: { fontSize: 14 }
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'cross' }
    },
    legend: {
      data: ['历史数据', '预测值', '置信区间'],
      bottom: 0
    },
    xAxis: {
      type: 'category',
      data: [...data.historical, ...data.predictions].map(item => item.timestamp),
      axisLabel: {
        formatter(value: string) {
          return new Date(value).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' })
        }
      }
    },
    yAxis: {
      type: 'value',
      name: '数值'
    },
    series: [
      {
        name: '历史数据',
        type: 'line',
        data: data.historical.map(item => item.value),
        lineStyle: { color: '#409EFF', width: 2 },
        itemStyle: { color: '#409EFF' },
        symbol: 'circle',
        symbolSize: 4
      },
      {
        name: '预测值',
        type: 'line',
        data: Array(data.historical.length).fill(null).concat(data.predictions.map(item => item.value)),
        lineStyle: { color: '#67C23A', width: 2, type: 'dashed' },
        itemStyle: { color: '#67C23A' },
        symbol: 'circle',
        symbolSize: 4
      },
      {
        name: '置信区间',
        type: 'line',
        data: Array(data.historical.length).fill(null).concat(data.confidence.upper),
        lineStyle: { width: 0 },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(103, 194, 58, 0.2)' },
            { offset: 1, color: 'rgba(103, 194, 58, 0.1)' }
          ])
        },
        stack: 'confidence',
        symbol: 'none'
      },
      {
        name: '置信区间下界',
        type: 'line',
        data: Array(data.historical.length).fill(null).concat(data.confidence.lower),
        lineStyle: { width: 0 },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(255, 255, 255, 0.8)' },
            { offset: 1, color: 'rgba(255, 255, 255, 0.8)' }
          ])
        },
        stack: 'confidence',
        symbol: 'none',
        showInLegend: false
      }
    ]
  }
  
  predictionChart.value.setOption(option)
}

/**
 * 更新残差图表
 */
function updateResidualsChart() {
  if (!residualsChart.value) return
  
  // 生成模拟残差数据
  const residuals = Array.from({ length: 50 }, (_, i) => ({
    predicted: 90 + Math.random() * 20,
    residual: (Math.random() - 0.5) * 8
  }))
  
  const option = {
    title: {
      text: '残差分析',
      left: 'center',
      textStyle: { fontSize: 14 }
    },
    tooltip: {
      trigger: 'item',
      formatter: 'Predicted: {c[0]}<br/>Residual: {c[1]}'
    },
    xAxis: {
      type: 'value',
      name: '预测值'
    },
    yAxis: {
      type: 'value',
      name: '残差'
    },
    series: [{
      type: 'scatter',
      data: residuals.map(item => [item.predicted, item.residual]),
      itemStyle: { color: '#E6A23C' },
      symbolSize: 6
    }, {
      type: 'line',
      data: [[85, 0], [115, 0]],
      lineStyle: { color: '#F56C6C', type: 'dashed' },
      symbol: 'none',
      silent: true
    }]
  }
  
  residualsChart.value.setOption(option)
}

/**
 * 更新特征重要性图表
 */
function updateImportanceChart() {
  if (!importanceChart.value) return
  
  const features = config.value.featureVariables.map(key => {
    const variable = availableVariables.value.find(v => v.key === key)
    return {
      name: variable?.name || key,
      importance: Math.random()
    }
  }).sort((a, b) => b.importance - a.importance)
  
  const option = {
    title: {
      text: '特征重要性',
      left: 'center',
      textStyle: { fontSize: 14 }
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' }
    },
    xAxis: {
      type: 'value',
      name: '重要性'
    },
    yAxis: {
      type: 'category',
      data: features.map(f => f.name)
    },
    series: [{
      type: 'bar',
      data: features.map(f => f.importance),
      itemStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 1, 0, [
          { offset: 0, color: '#91CC75' },
          { offset: 1, color: '#67C23A' }
        ])
      }
    }]
  }
  
  importanceChart.value.setOption(option)
}

/**
 * 更新预测区间图表
 */
function updateIntervalsChart() {
  if (!intervalsChart.value) return
  
  const data = predictionResults.value.predictions
  const intervals = [0.8, 0.9, 0.95, 0.99].map(level => ({
    level,
    name: `${Math.round(level * 100)}%置信区间`,
    upper: data.predictions.map((_, i) => data.predictions[i].value + (1 - level) * 15),
    lower: data.predictions.map((_, i) => data.predictions[i].value - (1 - level) * 15)
  }))
  
  const option = {
    title: {
      text: '预测区间',
      left: 'center',
      textStyle: { fontSize: 14 }
    },
    tooltip: {
      trigger: 'axis'
    },
    legend: {
      data: ['预测值', ...intervals.map(i => i.name)],
      bottom: 0
    },
    xAxis: {
      type: 'category',
      data: data.predictions.map(item => item.timestamp)
    },
    yAxis: {
      type: 'value'
    },
    series: [
      {
        name: '预测值',
        type: 'line',
        data: data.predictions.map(item => item.value),
        lineStyle: { color: '#409EFF', width: 2 },
        symbol: 'circle',
        symbolSize: 4,
        z: 10
      },
      ...intervals.map((interval, index) => ({
        name: interval.name,
        type: 'line',
        data: interval.upper,
        lineStyle: { width: 0 },
        areaStyle: {
          color: `rgba(103, 194, 58, ${0.3 - index * 0.05})`
        },
        stack: `interval_${index}`,
        symbol: 'none'
      })),
      ...intervals.map((interval, index) => ({
        name: `${interval.name}_lower`,
        type: 'line',
        data: interval.lower,
        lineStyle: { width: 0 },
        areaStyle: {
          color: 'rgba(255, 255, 255, 0.8)'
        },
        stack: `interval_${index}`,
        symbol: 'none',
        showInLegend: false
      }))
    ]
  }
  
  intervalsChart.value.setOption(option)
}

/**
 * 获取算法名称
 */
function getAlgorithmName(value: string): string {
  const allAlgorithms = [
    ...algorithms.value,
    ...predictionTypes.value.flatMap(type => 
      algorithmMap[type.value] || []
    )
  ]
  
  return allAlgorithms.find(algo => algo.value === value)?.label || value
}

/**
 * 重置配置
 */
function resetConfig() {
  config.value = {
    predictionType: 'timeseries',
    algorithm: 'lstm',
    targetVariable: '',
    featureVariables: [],
    trainingDateRange: [],
    forecastHorizon: 24,
    forecastUnit: 'hours',
    confidenceLevel: 95,
    crossValidation: true,
    cvFolds: 5,
    seasonality: null,
    learningRate: 0.001,
    maxIterations: 1000,
    featureEngineering: ['lag', 'rolling']
  }
}

/**
 * 验证配置
 */
function validateConfig() {
  if (validateConfiguration()) {
    ElMessage.success('配置验证通过')
  }
}

/**
 * 加载模板
 */
function loadTemplate() {
  ElMessage.info('模板加载功能开发中...')
}

/**
 * 保存模板
 */
function saveTemplate() {
  ElMessage.info('模板保存功能开发中...')
}

/**
 * 导出结果
 */
function exportResults() {
  ElMessage.info('结果导出功能开发中...')
}

/**
 * 分享结果
 */
function shareResults() {
  ElMessage.info('结果分享功能开发中...')
}

/**
 * 格式化时长
 */
function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60)
  const secs = seconds % 60
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

// 算法映射(用于获取算法名称)
const algorithmMap: { [key: string]: any[] } = {
  timeseries: [
    { value: 'lstm', label: 'LSTM神经网络' },
    { value: 'arima', label: 'ARIMA模型' },
    { value: 'prophet', label: 'Prophet预测' },
    { value: 'gru', label: 'GRU神经网络' }
  ],
  regression: [
    { value: 'linear', label: '线性回归' },
    { value: 'ridge', label: '岭回归' },
    { value: 'lasso', label: 'Lasso回归' },
    { value: 'randomforest', label: '随机森林' },
    { value: 'xgboost', label: 'XGBoost' }
  ],
  classification: [
    { value: 'logistic', label: '逻辑回归' },
    { value: 'svm', label: '支持向量机' },
    { value: 'randomforest', label: '随机森林' },
    { value: 'neuralnet', label: '神经网络' }
  ],
  anomaly: [
    { value: 'isolation', label: '孤立森林' },
    { value: 'ocsvm', label: '一类SVM' },
    { value: 'autoencoder', label: '自编码器' },
    { value: 'lof', label: '局部异常因子' }
  ]
}

// ===== 生命周期 =====
onMounted(() => {
  // 监听窗口大小变化
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  predictionChart.value?.dispose()
  residualsChart.value?.dispose()
  importanceChart.value?.dispose()
  intervalsChart.value?.dispose()
})

function handleResize() {
  predictionChart.value?.resize()
  residualsChart.value?.resize()
  importanceChart.value?.resize()
  intervalsChart.value?.resize()
}

// ===== 监听器 =====
watch(() => activeEvalTab.value, () => {
  nextTick(() => {
    handleResize()
  })
})
</script>

<style scoped lang="scss">
.predictive-analysis-panel {
  .prediction-config {
    margin-bottom: 24px;
    
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      h3 {
        margin: 0;
        color: #303133;
        font-size: 16px;
      }
      
      .header-actions {
        display: flex;
        gap: 8px;
      }
    }
    
    .config-form {
      .advanced-params {
        margin: 20px 0;
      }
      
      .form-actions {
        margin-top: 24px;
        text-align: center;
      }
      
      :deep(.el-collapse-item__header) {
        font-weight: 500;
      }
    }
    
    .option-content {
      width: 100%;
      
      .option-desc {
        display: block;
        color: #909399;
        font-size: 12px;
        margin-top: 2px;
      }
    }
  }
  
  .training-progress {
    margin-bottom: 24px;
    
    .progress-content {
      .progress-details {
        margin-top: 16px;
        
        .progress-info {
          display: flex;
          justify-content: space-between;
          margin-bottom: 12px;
          font-size: 13px;
          color: #606266;
        }
        
        .progress-metrics {
          display: flex;
          gap: 24px;
          
          .metric {
            display: flex;
            flex-direction: column;
            
            .metric-label {
              font-size: 12px;
              color: #909399;
              margin-bottom: 4px;
            }
            
            .metric-value {
              font-size: 16px;
              font-weight: 600;
              color: #303133;
            }
          }
        }
      }
    }
  }
  
  .prediction-results {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      h3 {
        margin: 0;
        color: #303133;
        font-size: 16px;
      }
      
      .header-actions {
        display: flex;
        gap: 8px;
      }
    }
    
    .results-summary {
      margin-bottom: 24px;
      
      .summary-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 16px;
        background: #f8f9fa;
        border-radius: 8px;
        border-left: 4px solid #409eff;
        
        .summary-icon {
          font-size: 24px;
          color: #409eff;
        }
        
        .summary-content {
          .summary-value {
            font-size: 20px;
            font-weight: 600;
            color: #303133;
            margin-bottom: 2px;
          }
          
          .summary-label {
            font-size: 12px;
            color: #606266;
          }
        }
      }
    }
    
    .prediction-chart {
      margin-bottom: 24px;
      
      .chart-container {
        height: 400px;
        width: 100%;
      }
    }
    
    .model-evaluation {
      .eval-chart {
        height: 300px;
        width: 100%;
      }
      
      .diagnostics-content {
        .metrics-table {
          margin-top: 20px;
          
          h4 {
            margin: 0 0 12px 0;
            font-size: 14px;
            color: #303133;
          }
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .predictive-analysis-panel {
    .results-summary {
      :deep(.el-col) {
        margin-bottom: 16px;
      }
    }
    
    .progress-metrics {
      flex-wrap: wrap;
      gap: 16px !important;
    }
  }
}

@media (max-width: 768px) {
  .predictive-analysis-panel {
    .card-header {
      flex-direction: column;
      gap: 12px;
      align-items: stretch !important;
      
      .header-actions {
        justify-content: center;
      }
    }
    
    .progress-info {
      flex-direction: column !important;
      gap: 8px;
    }
    
    .progress-metrics {
      justify-content: center !important;
    }
  }
}
</style>