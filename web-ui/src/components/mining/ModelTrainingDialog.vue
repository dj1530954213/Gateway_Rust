<template>
  <el-dialog
    v-model="dialogVisible"
    title="模型训练配置"
    width="900px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @close="handleClose"
  >
    <div class="model-training-dialog">
      <!-- 训练向导步骤 -->
      <el-steps :active="currentStep" finish-status="success" align-center>
        <el-step title="数据配置" />
        <el-step title="模型选择" />
        <el-step title="参数调优" />
        <el-step title="训练验证" />
      </el-steps>

      <!-- 步骤内容 -->
      <div class="step-content">
        <!-- 步骤1: 数据配置 -->
        <div v-if="currentStep === 0" class="step-panel">
          <h3>数据配置</h3>

          <el-form :model="dataConfig" label-width="120px">
            <el-row :gutter="24">
              <el-col :span="12">
                <el-form-item label="数据源">
                  <el-select
                    v-model="dataConfig.dataSource"
                    @change="onDataSourceChange"
                  >
                    <el-option
                      v-for="source in dataSources"
                      :key="source.value"
                      :label="source.label"
                      :value="source.value"
                    />
                  </el-select>
                </el-form-item>

                <el-form-item label="时间范围">
                  <el-date-picker
                    v-model="dataConfig.dateRange"
                    type="datetimerange"
                    range-separator="至"
                    start-placeholder="开始时间"
                    end-placeholder="结束时间"
                    format="YYYY-MM-DD HH:mm:ss"
                    value-format="YYYY-MM-DD HH:mm:ss"
                  />
                </el-form-item>

                <el-form-item label="采样间隔">
                  <el-row :gutter="8">
                    <el-col :span="16">
                      <el-input-number
                        v-model="dataConfig.sampleInterval"
                        :min="1"
                      />
                    </el-col>
                    <el-col :span="8">
                      <el-select v-model="dataConfig.sampleUnit">
                        <el-option label="秒" value="seconds" />
                        <el-option label="分钟" value="minutes" />
                        <el-option label="小时" value="hours" />
                      </el-select>
                    </el-col>
                  </el-row>
                </el-form-item>

                <el-form-item label="数据清洗">
                  <el-checkbox-group v-model="dataConfig.cleaning">
                    <el-checkbox label="outliers">异常值处理</el-checkbox>
                    <el-checkbox label="missing">缺失值填充</el-checkbox>
                    <el-checkbox label="smooth">数据平滑</el-checkbox>
                    <el-checkbox label="normalize">标准化</el-checkbox>
                  </el-checkbox-group>
                </el-form-item>
              </el-col>

              <el-col :span="12">
                <div class="data-preview">
                  <h4>数据预览</h4>
                  <div v-if="dataPreview.loading" class="loading-state">
                    <el-icon class="is-loading"><Loading /></el-icon>
                    <span>加载数据中...</span>
                  </div>
                  <div v-else-if="dataPreview.data" class="preview-content">
                    <div class="preview-stats">
                      <div class="stat-item">
                        <span class="stat-label">总样本数:</span>
                        <span class="stat-value">{{
                          dataPreview.data.totalSamples
                        }}</span>
                      </div>
                      <div class="stat-item">
                        <span class="stat-label">特征数量:</span>
                        <span class="stat-value">{{
                          dataPreview.data.featureCount
                        }}</span>
                      </div>
                      <div class="stat-item">
                        <span class="stat-label">缺失率:</span>
                        <span class="stat-value"
                          >{{ dataPreview.data.missingRate }}%</span
                        >
                      </div>
                    </div>

                    <el-table
                      :data="dataPreview.data.samples"
                      size="small"
                      max-height="200"
                      style="margin-top: 12px"
                    >
                      <el-table-column
                        v-for="column in dataPreview.data.columns"
                        :key="column"
                        :prop="column"
                        :label="column"
                        width="100"
                        show-overflow-tooltip
                      />
                    </el-table>
                  </div>
                  <div v-else class="empty-state">
                    <el-empty description="请配置数据源" :image-size="60" />
                  </div>
                </div>
              </el-col>
            </el-row>
          </el-form>
        </div>

        <!-- 步骤2: 模型选择 -->
        <div v-if="currentStep === 1" class="step-panel">
          <h3>模型选择</h3>

          <div class="model-categories">
            <el-radio-group
              v-model="modelConfig.category"
              @change="onModelCategoryChange"
            >
              <el-radio-button
                v-for="category in modelCategories"
                :key="category.value"
                :label="category.value"
              >
                {{ category.label }}
              </el-radio-button>
            </el-radio-group>
          </div>

          <div class="model-grid">
            <div
              v-for="model in availableModels"
              :key="model.value"
              :class="[
                'model-card',
                { active: modelConfig.algorithm === model.value },
              ]"
              @click="selectModel(model)"
            >
              <div class="model-icon">
                <el-icon><component :is="model.icon" /></el-icon>
              </div>
              <div class="model-info">
                <div class="model-name">{{ model.label }}</div>
                <div class="model-description">{{ model.description }}</div>
                <div class="model-tags">
                  <el-tag
                    v-for="tag in model.tags"
                    :key="tag"
                    size="small"
                    effect="plain"
                  >
                    {{ tag }}
                  </el-tag>
                </div>
                <div class="model-metrics">
                  <div class="metric">
                    <span class="metric-label">准确性:</span>
                    <el-rate
                      v-model="model.accuracy"
                      disabled
                      :max="5"
                      size="small"
                      text-color="#99A9BF"
                    />
                  </div>
                  <div class="metric">
                    <span class="metric-label">速度:</span>
                    <el-rate
                      v-model="model.speed"
                      disabled
                      :max="5"
                      size="small"
                      text-color="#99A9BF"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 步骤3: 参数调优 -->
        <div v-if="currentStep === 2" class="step-panel">
          <h3>参数调优</h3>

          <el-form :model="trainingConfig" label-width="150px">
            <el-row :gutter="24">
              <el-col :span="12">
                <el-card shadow="never" class="params-card">
                  <template #header>
                    <h4>基础参数</h4>
                  </template>

                  <el-form-item label="训练/验证比例">
                    <el-slider
                      v-model="trainingConfig.trainTestSplit"
                      :min="50"
                      :max="90"
                      :step="5"
                      show-stops
                      show-input
                      :format-tooltip="val => `${val}% / ${100 - val}%`"
                    />
                  </el-form-item>

                  <el-form-item label="学习率">
                    <el-input-number
                      v-model="trainingConfig.learningRate"
                      :min="0.0001"
                      :max="1"
                      :step="0.0001"
                      :precision="4"
                    />
                  </el-form-item>

                  <el-form-item label="批次大小">
                    <el-select v-model="trainingConfig.batchSize">
                      <el-option :value="16" label="16" />
                      <el-option :value="32" label="32" />
                      <el-option :value="64" label="64" />
                      <el-option :value="128" label="128" />
                      <el-option :value="256" label="256" />
                    </el-select>
                  </el-form-item>

                  <el-form-item label="训练轮数">
                    <el-input-number
                      v-model="trainingConfig.epochs"
                      :min="10"
                      :max="1000"
                      :step="10"
                    />
                  </el-form-item>

                  <el-form-item label="早停策略">
                    <el-switch v-model="trainingConfig.earlyStopping" />
                    <el-input-number
                      v-if="trainingConfig.earlyStopping"
                      v-model="trainingConfig.patience"
                      :min="5"
                      :max="50"
                      style="margin-left: 8px; width: 120px"
                      placeholder="耐心值"
                    />
                  </el-form-item>
                </el-card>
              </el-col>

              <el-col :span="12">
                <el-card shadow="never" class="params-card">
                  <template #header>
                    <h4>高级参数</h4>
                  </template>

                  <el-form-item
                    v-if="modelConfig.category === 'neural'"
                    label="隐藏层数"
                  >
                    <el-input-number
                      v-model="trainingConfig.hiddenLayers"
                      :min="1"
                      :max="10"
                    />
                  </el-form-item>

                  <el-form-item
                    v-if="modelConfig.category === 'neural'"
                    label="神经元数"
                  >
                    <el-input-number
                      v-model="trainingConfig.hiddenUnits"
                      :min="16"
                      :max="512"
                      :step="16"
                    />
                  </el-form-item>

                  <el-form-item
                    v-if="modelConfig.category === 'tree'"
                    label="最大深度"
                  >
                    <el-input-number
                      v-model="trainingConfig.maxDepth"
                      :min="3"
                      :max="20"
                    />
                  </el-form-item>

                  <el-form-item
                    v-if="modelConfig.category === 'tree'"
                    label="最小样本分割"
                  >
                    <el-input-number
                      v-model="trainingConfig.minSamplesSplit"
                      :min="2"
                      :max="20"
                    />
                  </el-form-item>

                  <el-form-item label="正则化系数">
                    <el-input-number
                      v-model="trainingConfig.regularization"
                      :min="0"
                      :max="1"
                      :step="0.01"
                      :precision="3"
                    />
                  </el-form-item>

                  <el-form-item label="Dropout率">
                    <el-input-number
                      v-model="trainingConfig.dropout"
                      :min="0"
                      :max="0.8"
                      :step="0.1"
                      :precision="1"
                    />
                  </el-form-item>

                  <el-form-item label="优化器">
                    <el-select v-model="trainingConfig.optimizer">
                      <el-option value="adam" label="Adam" />
                      <el-option value="sgd" label="SGD" />
                      <el-option value="rmsprop" label="RMSprop" />
                      <el-option value="adagrad" label="Adagrad" />
                    </el-select>
                  </el-form-item>
                </el-card>
              </el-col>
            </el-row>

            <!-- 自动调参 -->
            <el-card shadow="never" class="auto-tuning-card">
              <template #header>
                <div class="card-header">
                  <h4>自动调参</h4>
                  <el-switch v-model="trainingConfig.autoTuning" />
                </div>
              </template>

              <div v-if="trainingConfig.autoTuning" class="auto-tuning-config">
                <el-row :gutter="24">
                  <el-col :span="8">
                    <el-form-item label="调参方法">
                      <el-select v-model="trainingConfig.tuningMethod">
                        <el-option value="grid" label="网格搜索" />
                        <el-option value="random" label="随机搜索" />
                        <el-option value="bayesian" label="贝叶斯优化" />
                      </el-select>
                    </el-form-item>
                  </el-col>
                  <el-col :span="8">
                    <el-form-item label="搜索次数">
                      <el-input-number
                        v-model="trainingConfig.searchIterations"
                        :min="10"
                        :max="100"
                      />
                    </el-form-item>
                  </el-col>
                  <el-col :span="8">
                    <el-form-item label="评估指标">
                      <el-select v-model="trainingConfig.evaluationMetric">
                        <el-option value="accuracy" label="准确率" />
                        <el-option value="f1" label="F1分数" />
                        <el-option value="auc" label="AUC" />
                        <el-option value="rmse" label="RMSE" />
                      </el-select>
                    </el-form-item>
                  </el-col>
                </el-row>
              </div>
            </el-card>
          </el-form>
        </div>

        <!-- 步骤4: 训练验证 -->
        <div v-if="currentStep === 3" class="step-panel">
          <h3>训练验证</h3>

          <div class="training-summary">
            <el-card shadow="never">
              <template #header>
                <h4>训练配置摘要</h4>
              </template>

              <el-descriptions :column="2" border>
                <el-descriptions-item label="数据源">
                  {{ getDataSourceName(dataConfig.dataSource) }}
                </el-descriptions-item>
                <el-descriptions-item label="样本数量">
                  {{ dataPreview.data?.totalSamples || 0 }}
                </el-descriptions-item>
                <el-descriptions-item label="模型算法">
                  {{ getModelName(modelConfig.algorithm) }}
                </el-descriptions-item>
                <el-descriptions-item label="训练比例">
                  {{ trainingConfig.trainTestSplit }}% /
                  {{ 100 - trainingConfig.trainTestSplit }}%
                </el-descriptions-item>
                <el-descriptions-item label="学习率">
                  {{ trainingConfig.learningRate }}
                </el-descriptions-item>
                <el-descriptions-item label="训练轮数">
                  {{ trainingConfig.epochs }}
                </el-descriptions-item>
                <el-descriptions-item label="批次大小">
                  {{ trainingConfig.batchSize }}
                </el-descriptions-item>
                <el-descriptions-item label="自动调参">
                  {{ trainingConfig.autoTuning ? '启用' : '禁用' }}
                </el-descriptions-item>
              </el-descriptions>
            </el-card>
          </div>

          <div class="training-validation">
            <el-card shadow="never">
              <template #header>
                <h4>验证设置</h4>
              </template>

              <el-form :model="validationConfig" label-width="120px">
                <el-row :gutter="24">
                  <el-col :span="12">
                    <el-form-item label="交叉验证">
                      <el-switch v-model="validationConfig.crossValidation" />
                      <el-input-number
                        v-if="validationConfig.crossValidation"
                        v-model="validationConfig.cvFolds"
                        :min="3"
                        :max="10"
                        style="margin-left: 8px; width: 120px"
                        placeholder="折数"
                      />
                    </el-form-item>

                    <el-form-item label="保留验证集">
                      <el-switch v-model="validationConfig.holdoutValidation" />
                      <el-input-number
                        v-if="validationConfig.holdoutValidation"
                        v-model="validationConfig.holdoutRatio"
                        :min="0.1"
                        :max="0.3"
                        :step="0.05"
                        :precision="2"
                        style="margin-left: 8px; width: 120px"
                        placeholder="比例"
                      />
                    </el-form-item>
                  </el-col>

                  <el-col :span="12">
                    <el-form-item label="评估指标">
                      <el-checkbox-group v-model="validationConfig.metrics">
                        <el-checkbox label="accuracy">准确率</el-checkbox>
                        <el-checkbox label="precision">精确率</el-checkbox>
                        <el-checkbox label="recall">召回率</el-checkbox>
                        <el-checkbox label="f1">F1分数</el-checkbox>
                        <el-checkbox label="auc">AUC</el-checkbox>
                      </el-checkbox-group>
                    </el-form-item>

                    <el-form-item label="保存策略">
                      <el-radio-group v-model="validationConfig.saveStrategy">
                        <el-radio label="best">最佳模型</el-radio>
                        <el-radio label="latest">最新模型</el-radio>
                        <el-radio label="all">全部保存</el-radio>
                      </el-radio-group>
                    </el-form-item>
                  </el-col>
                </el-row>
              </el-form>
            </el-card>
          </div>
        </div>
      </div>

      <!-- 操作按钮 -->
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="handleClose">取消</el-button>
          <el-button v-if="currentStep > 0" @click="prevStep">上一步</el-button>
          <el-button
            v-if="currentStep < 3"
            type="primary"
            :disabled="!canProceedToNext"
            @click="nextStep"
          >
            下一步
          </el-button>
          <el-button
            v-if="currentStep === 3"
            type="primary"
            :loading="isStartingTraining"
            @click="startTraining"
          >
            开始训练
          </el-button>
        </div>
      </template>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * ModelTrainingDialog —— 模型训练配置对话框
 *
 * 📝 Responsibilities:
 *  1. 提供分步骤的模型训练配置向导
 *  2. 数据源配置和预览
 *  3. 模型选择和参数调优
 *  4. 训练验证配置
 *
 * 📦 Dependencies:
 *  - Element Plus UI组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Loading,
  DataAnalysis,
  TrendCharts,
  BranchesOutlined,
  Cpu,
  SetUp,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, watch, nextTick } from 'vue'

// ===== Props & Emits =====
const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'training-started', config: any): void
}>()

// ===== 响应式数据 =====
const dialogVisible = computed({
  get: () => props.modelValue,
  set: value => emit('update:modelValue', value),
})

const currentStep = ref(0)
const isStartingTraining = ref(false)

// 数据配置
const dataConfig = ref({
  dataSource: '',
  dateRange: [],
  sampleInterval: 1,
  sampleUnit: 'minutes',
  cleaning: ['missing', 'normalize'],
})

// 模型配置
const modelConfig = ref({
  category: 'neural',
  algorithm: '',
})

// 训练配置
const trainingConfig = ref({
  trainTestSplit: 80,
  learningRate: 0.001,
  batchSize: 32,
  epochs: 100,
  earlyStopping: true,
  patience: 10,
  hiddenLayers: 2,
  hiddenUnits: 64,
  maxDepth: 10,
  minSamplesSplit: 2,
  regularization: 0.01,
  dropout: 0.2,
  optimizer: 'adam',
  autoTuning: false,
  tuningMethod: 'grid',
  searchIterations: 20,
  evaluationMetric: 'accuracy',
})

// 验证配置
const validationConfig = ref({
  crossValidation: true,
  cvFolds: 5,
  holdoutValidation: false,
  holdoutRatio: 0.2,
  metrics: ['accuracy', 'f1'],
  saveStrategy: 'best',
})

// 数据预览
const dataPreview = ref({
  loading: false,
  data: null as any,
})

// 数据源选项
const dataSources = ref([
  { value: 'realtime', label: '实时数据' },
  { value: 'historical', label: '历史数据' },
  { value: 'simulation', label: '仿真数据' },
  { value: 'upload', label: '上传文件' },
])

// 模型分类
const modelCategories = ref([
  { value: 'neural', label: '神经网络' },
  { value: 'tree', label: '树模型' },
  { value: 'linear', label: '线性模型' },
  { value: 'ensemble', label: '集成模型' },
])

// 可用模型
const availableModels = computed(() => {
  const modelMap: { [key: string]: any[] } = {
    neural: [
      {
        value: 'lstm',
        label: 'LSTM',
        description: '长短期记忆网络，适合时序预测',
        icon: 'TrendCharts',
        tags: ['时序', '深度学习'],
        accuracy: 4,
        speed: 3,
      },
      {
        value: 'gru',
        label: 'GRU',
        description: '门控循环单元，简化的LSTM',
        icon: 'TrendCharts',
        tags: ['时序', '轻量'],
        accuracy: 4,
        speed: 4,
      },
      {
        value: 'mlp',
        label: '多层感知机',
        description: '经典前馈神经网络',
        icon: 'DataAnalysis',
        tags: ['通用', '简单'],
        accuracy: 3,
        speed: 5,
      },
    ],
    tree: [
      {
        value: 'randomforest',
        label: '随机森林',
        description: '集成决策树，鲁棒性强',
        icon: 'BranchesOutlined',
        tags: ['集成', '稳定'],
        accuracy: 4,
        speed: 4,
      },
      {
        value: 'xgboost',
        label: 'XGBoost',
        description: '梯度提升树，性能优异',
        icon: 'BranchesOutlined',
        tags: ['集成', '高性能'],
        accuracy: 5,
        speed: 3,
      },
      {
        value: 'lightgbm',
        label: 'LightGBM',
        description: '轻量级梯度提升',
        icon: 'BranchesOutlined',
        tags: ['快速', '轻量'],
        accuracy: 4,
        speed: 5,
      },
    ],
    linear: [
      {
        value: 'linear',
        label: '线性回归',
        description: '简单线性关系建模',
        icon: 'DataAnalysis',
        tags: ['简单', '可解释'],
        accuracy: 2,
        speed: 5,
      },
      {
        value: 'ridge',
        label: '岭回归',
        description: 'L2正则化线性回归',
        icon: 'DataAnalysis',
        tags: ['正则化', '稳定'],
        accuracy: 3,
        speed: 5,
      },
      {
        value: 'lasso',
        label: 'Lasso回归',
        description: 'L1正则化，特征选择',
        icon: 'DataAnalysis',
        tags: ['特征选择', '稀疏'],
        accuracy: 3,
        speed: 4,
      },
    ],
    ensemble: [
      {
        value: 'voting',
        label: '投票集成',
        description: '多模型投票决策',
        icon: 'SetUp',
        tags: ['集成', '投票'],
        accuracy: 4,
        speed: 2,
      },
      {
        value: 'stacking',
        label: '堆叠集成',
        description: '分层模型集成',
        icon: 'SetUp',
        tags: ['集成', '分层'],
        accuracy: 5,
        speed: 2,
      },
    ],
  }

  return modelMap[modelConfig.value.category] || []
})

// ===== 计算属性 =====
const canProceedToNext = computed(() => {
  switch (currentStep.value) {
    case 0:
      return (
        dataConfig.value.dataSource && dataConfig.value.dateRange.length === 2
      )
    case 1:
      return modelConfig.value.algorithm
    case 2:
      return true
    case 3:
      return true
    default:
      return false
  }
})

// ===== 方法 =====

/**
 * 下一步
 */
function nextStep() {
  if (canProceedToNext.value) {
    currentStep.value++

    // 特殊处理
    if (currentStep.value === 1 && !modelConfig.value.algorithm) {
      // 自动选择第一个模型
      if (availableModels.value.length > 0) {
        selectModel(availableModels.value[0])
      }
    }
  }
}

/**
 * 上一步
 */
function prevStep() {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

/**
 * 数据源变化处理
 */
function onDataSourceChange() {
  // 加载数据预览
  loadDataPreview()
}

/**
 * 加载数据预览
 */
async function loadDataPreview() {
  if (!dataConfig.value.dataSource) {
    dataPreview.value.data = null
    return
  }

  dataPreview.value.loading = true

  try {
    // 从真实API加载数据预览
    const response = await fetch(
      `/api/v1/data-mining/preview?source=${dataConfig.value.dataSource}`,
      {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      }
    )

    if (response.ok) {
      dataPreview.value.data = await response.json()
    } else {
      throw new Error('数据加载失败')
    }
  } catch (error) {
    console.error('数据预览加载失败:', error)
    ElMessage.error('数据加载失败')
    dataPreview.value.data = null
  } finally {
    dataPreview.value.loading = false
  }
}

/**
 * 模型分类变化处理
 */
function onModelCategoryChange() {
  modelConfig.value.algorithm = ''
}

/**
 * 选择模型
 */
function selectModel(model: any) {
  modelConfig.value.algorithm = model.value
}

/**
 * 开始训练
 */
async function startTraining() {
  isStartingTraining.value = true

  try {
    // 构造训练配置
    const config = {
      data: dataConfig.value,
      model: modelConfig.value,
      training: trainingConfig.value,
      validation: validationConfig.value,
    }

    // 发射训练开始事件
    emit('training-started', config)

    // 关闭对话框
    handleClose()

    ElMessage.success('模型训练已开始')
  } catch (error) {
    ElMessage.error(`启动训练失败: ${error}`)
  } finally {
    isStartingTraining.value = false
  }
}

/**
 * 关闭对话框
 */
function handleClose() {
  // 重置步骤
  currentStep.value = 0

  // 关闭对话框
  dialogVisible.value = false
}

/**
 * 获取数据源名称
 */
function getDataSourceName(value: string): string {
  return dataSources.value.find(s => s.value === value)?.label || value
}

/**
 * 获取模型名称
 */
function getModelName(value: string): string {
  const allModels = Object.values(modelMap).flat()
  return allModels.find(m => m.value === value)?.label || value
}

// 模型映射（用于获取模型名称）
const modelMap: { [key: string]: any[] } = {
  neural: [
    { value: 'lstm', label: 'LSTM' },
    { value: 'gru', label: 'GRU' },
    { value: 'mlp', label: '多层感知机' },
  ],
  tree: [
    { value: 'randomforest', label: '随机森林' },
    { value: 'xgboost', label: 'XGBoost' },
    { value: 'lightgbm', label: 'LightGBM' },
  ],
  linear: [
    { value: 'linear', label: '线性回归' },
    { value: 'ridge', label: '岭回归' },
    { value: 'lasso', label: 'Lasso回归' },
  ],
  ensemble: [
    { value: 'voting', label: '投票集成' },
    { value: 'stacking', label: '堆叠集成' },
  ],
}

// ===== 监听器 =====
watch(
  () => props.modelValue,
  newVal => {
    if (newVal) {
      // 重置配置
      currentStep.value = 0
      dataPreview.value.data = null
    }
  }
)
</script>

<style scoped lang="scss">
.model-training-dialog {
  .step-content {
    margin: 30px 0;
    min-height: 400px;

    .step-panel {
      h3 {
        margin: 0 0 20px 0;
        color: #303133;
        font-size: 18px;
        text-align: center;
      }
    }
  }

  // 数据配置步骤
  .data-preview {
    .loading-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 200px;
      color: #909399;

      .el-icon {
        font-size: 32px;
        margin-bottom: 12px;
      }
    }

    .preview-content {
      h4 {
        margin: 0 0 12px 0;
        font-size: 14px;
        color: #303133;
      }

      .preview-stats {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 8px;
        margin-bottom: 12px;

        .stat-item {
          display: flex;
          justify-content: space-between;
          padding: 6px 8px;
          background: #f5f7fa;
          border-radius: 4px;
          font-size: 12px;

          .stat-label {
            color: #606266;
          }

          .stat-value {
            color: #303133;
            font-weight: 500;
          }
        }
      }
    }

    .empty-state {
      height: 200px;
      display: flex;
      align-items: center;
      justify-content: center;
    }
  }

  // 模型选择步骤
  .model-categories {
    margin-bottom: 20px;
    text-align: center;
  }

  .model-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 16px;

    .model-card {
      padding: 16px;
      border: 2px solid #ebeef5;
      border-radius: 8px;
      cursor: pointer;
      transition: all 0.3s;

      &:hover {
        border-color: #c6e2ff;
        background: #f0f9ff;
      }

      &.active {
        border-color: #409eff;
        background: #f0f9ff;

        .model-icon {
          color: #409eff;
        }
      }

      .model-icon {
        font-size: 32px;
        color: #909399;
        text-align: center;
        margin-bottom: 12px;
      }

      .model-info {
        .model-name {
          font-size: 16px;
          font-weight: 600;
          color: #303133;
          margin-bottom: 6px;
        }

        .model-description {
          font-size: 13px;
          color: #606266;
          line-height: 1.4;
          margin-bottom: 8px;
        }

        .model-tags {
          display: flex;
          flex-wrap: wrap;
          gap: 4px;
          margin-bottom: 12px;
        }

        .model-metrics {
          .metric {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 4px;

            .metric-label {
              font-size: 12px;
              color: #909399;
            }
          }
        }
      }
    }
  }

  // 参数调优步骤
  .params-card {
    margin-bottom: 16px;

    :deep(.el-card__header) {
      padding: 16px 20px;

      h4 {
        margin: 0;
        font-size: 14px;
        color: #303133;
      }
    }

    :deep(.el-card__body) {
      padding: 16px 20px;
    }
  }

  .auto-tuning-card {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      h4 {
        margin: 0;
        font-size: 14px;
        color: #303133;
      }
    }

    .auto-tuning-config {
      margin-top: 16px;
    }
  }

  // 训练验证步骤
  .training-summary {
    margin-bottom: 20px;
  }

  .training-validation {
    h4 {
      margin: 0;
      font-size: 14px;
      color: #303133;
    }
  }

  // 对话框底部
  .dialog-footer {
    display: flex;
    justify-content: center;
    gap: 12px;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .model-training-dialog {
    .model-grid {
      grid-template-columns: 1fr;
    }

    .preview-stats {
      grid-template-columns: 1fr !important;
    }

    .dialog-footer {
      flex-direction: column;

      .el-button {
        width: 100%;
      }
    }
  }
}
</style>
