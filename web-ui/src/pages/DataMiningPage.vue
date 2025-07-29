<template>
  <div class="data-mining-page">
    <div class="page-header">
      <div class="header-content">
        <h1>数据挖掘与预测分析</h1>
        <p>利用机器学习和数据挖掘技术发现数据中的模式和规律，进行智能预测</p>
      </div>
      <div class="header-actions">
        <el-button @click="showTemplatesDialog = true">
          <el-icon><Grid /></el-icon>
          分析模板
        </el-button>
        <el-button @click="showModelsDialog = true">
          <el-icon><Cpu /></el-icon>
          模型管理
        </el-button>
        <el-button type="primary" @click="showTrainingDialog = true">
          <el-icon><Plus /></el-icon>
          新建模型
        </el-button>
      </div>
    </div>
    
    <div class="page-content">
      <!-- 功能导航 -->
      <div class="function-nav">
        <el-card shadow="never">
          <div class="nav-items">
            <div
              v-for="func in functions"
              :key="func.key"
              :class="['nav-item', { active: selectedFunction === func.key }]"
              @click="selectFunction(func.key)"
            >
              <div class="nav-icon">
                <el-icon><component :is="func.icon" /></el-icon>
              </div>
              <div class="nav-info">
                <div class="nav-title">{{ func.title }}</div>
                <div class="nav-description">{{ func.description }}</div>
              </div>
              <div class="nav-badge" v-if="func.badge">
                <el-badge :value="func.badge" :type="func.badgeType || 'primary'" />
              </div>
            </div>
          </div>
        </el-card>
      </div>
      
      <!-- 数据挖掘面板 -->
      <div v-if="selectedFunction === 'mining'" class="function-panel">
        <DataMiningPanel
          :tasks="miningTasks"
          @task-created="onMiningTaskCreated"
          @task-executed="onMiningTaskExecuted"
        />
      </div>
      
      <!-- 预测分析面板 -->
      <div v-if="selectedFunction === 'prediction'" class="function-panel">
        <PredictiveAnalysisPanel
          @prediction-completed="onPredictionCompleted"
        />
      </div>
      
      <!-- 模式发现面板 -->
      <div v-if="selectedFunction === 'patterns'" class="function-panel">
        <PatternDiscoveryPanel
          :patterns="discoveredPatterns"
          @pattern-discovered="onPatternDiscovered"
        />
      </div>
      
      <!-- 异常检测面板 -->
      <div v-if="selectedFunction === 'anomaly'" class="function-panel">
        <AnomalyDetectionPanel
          :anomalies="detectedAnomalies"
          @anomaly-detected="onAnomalyDetected"
        />
      </div>
      
      <!-- 趋势预测面板 -->
      <div v-if="selectedFunction === 'forecast'" class="function-panel">
        <ForecastPanel
          :forecasts="forecastResults"
          @forecast-generated="onForecastGenerated"
        />
      </div>
      
      <!-- 关联分析面板 -->
      <div v-if="selectedFunction === 'association'" class="function-panel">
        <AssociationAnalysisPanel
          :rules="associationRules"
          @rules-generated="onAssociationRulesGenerated"
        />
      </div>
    </div>
    
    <!-- 分析模板对话框 -->
    <el-dialog v-model="showTemplatesDialog" title="分析模板" width="1000px">
      <div class="templates-content">
        <div class="template-categories">
          <el-radio-group v-model="selectedTemplateCategory">
            <el-radio-button
              v-for="category in templateCategories"
              :key="category.value"
              :label="category.value"
            >
              {{ category.label }}
            </el-radio-button>
          </el-radio-group>
        </div>
        
        <div class="templates-grid">
          <div
            v-for="template in filteredTemplates"
            :key="template.id"
            class="template-card"
            @click="applyTemplate(template)"
          >
            <div class="template-icon">
              <el-icon><component :is="template.icon" /></el-icon>
            </div>
            <div class="template-info">
              <div class="template-name">{{ template.name }}</div>
              <div class="template-description">{{ template.description }}</div>
              <div class="template-tags">
                <el-tag
                  v-for="tag in template.tags"
                  :key="tag"
                  size="small"
                  effect="plain"
                >
                  {{ tag }}
                </el-tag>
              </div>
              <div class="template-stats">
                <span>使用次数: {{ template.useCount }}</span>
                <span>成功率: {{ template.successRate }}%</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </el-dialog>
    
    <!-- 模型管理对话框 -->
    <el-dialog v-model="showModelsDialog" title="模型管理" width="1200px">
      <div class="models-content">
        <div class="models-toolbar">
          <el-input
            v-model="modelSearchQuery"
            placeholder="搜索模型..."
            style="width: 300px"
            clearable
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
          
          <div class="toolbar-actions">
            <el-button @click="refreshModels">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
            <el-button @click="importModel">
              <el-icon><Upload /></el-icon>
              导入模型
            </el-button>
            <el-button type="primary" @click="showTrainingDialog = true">
              <el-icon><Plus /></el-icon>
              训练新模型
            </el-button>
          </div>
        </div>
        
        <el-table :data="filteredModels" stripe style="width: 100%">
          <el-table-column prop="name" label="模型名称" width="200" />
          <el-table-column prop="algorithm" label="算法" width="120" />
          <el-table-column prop="accuracy" label="准确率" width="100">
            <template #default="{ row }">
              <span :class="getAccuracyClass(row.accuracy)">
                {{ row.accuracy }}%
              </span>
            </template>
          </el-table-column>
          <el-table-column prop="status" label="状态" width="100">
            <template #default="{ row }">
              <el-tag :type="getStatusType(row.status)" size="small">
                {{ row.status }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="trainingTime" label="训练时间" width="150" />
          <el-table-column prop="lastUsed" label="最后使用" width="150" />
          <el-table-column label="操作" width="200">
            <template #default="{ row }">
              <el-button size="small" @click="deployModel(row)">部署</el-button>
              <el-button size="small" @click="evaluateModel(row)">评估</el-button>
              <el-dropdown @command="(cmd) => handleModelAction(cmd, row)">
                <el-button size="small">
                  更多<el-icon><ArrowDown /></el-icon>
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item command="export">导出</el-dropdown-item>
                    <el-dropdown-item command="copy">复制</el-dropdown-item>
                    <el-dropdown-item command="retrain">重新训练</el-dropdown-item>
                    <el-dropdown-item command="delete" divided>删除</el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-dialog>
    
    <!-- 模型训练对话框 -->
    <ModelTrainingDialog
      v-model="showTrainingDialog"
      @training-started="onTrainingStarted"
    />
    
    <!-- 结果统计卡片 -->
    <div class="results-overview">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <h3>分析结果概览</h3>
            <el-button size="small" @click="exportAllResults">
              <el-icon><Download /></el-icon>
              导出全部
            </el-button>
          </div>
        </template>
        
        <div class="overview-stats">
          <el-row :gutter="16">
            <el-col :span="4">
              <div class="stat-item">
                <div class="stat-icon mining">
                  <el-icon><DataAnalysis /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-value">{{ stats.miningTasks }}</div>
                  <div class="stat-label">挖掘任务</div>
                </div>
              </div>
            </el-col>
            <el-col :span="4">
              <div class="stat-item">
                <div class="stat-icon prediction">
                  <el-icon><TrendCharts /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-value">{{ stats.predictions }}</div>
                  <div class="stat-label">预测模型</div>
                </div>
              </div>
            </el-col>
            <el-col :span="4">
              <div class="stat-item">
                <div class="stat-icon patterns">
                  <el-icon><ScaleToOriginal /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-value">{{ stats.patterns }}</div>
                  <div class="stat-label">发现模式</div>
                </div>
              </div>
            </el-col>
            <el-col :span="4">
              <div class="stat-item">
                <div class="stat-icon anomalies">
                  <el-icon><Warning /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-value">{{ stats.anomalies }}</div>
                  <div class="stat-label">检测异常</div>
                </div>
              </div>
            </el-col>
            <el-col :span="4">
              <div class="stat-item">
                <div class="stat-icon forecasts">
                  <el-icon><Opportunity /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-value">{{ stats.forecasts }}</div>
                  <div class="stat-label">趋势预测</div>
                </div>
              </div>
            </el-col>
            <el-col :span="4">
              <div class="stat-item">
                <div class="stat-icon associations">
                  <el-icon><Share /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-value">{{ stats.associations }}</div>
                  <div class="stat-label">关联规则</div>
                </div>
              </div>
            </el-col>
          </el-row>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * DataMiningPage —— 数据挖掘与预测分析页面
 *
 * 📝 Responsibilities:
 *  1. 数据挖掘和预测分析的统一入口
 *  2. 多种分析功能的导航和切换
 *  3. 模型管理和训练配置
 *  4. 分析结果的整合展示
 *
 * 📦 Dependencies:
 *  - DataMiningPanel 数据挖掘面板
 *  - PredictiveAnalysisPanel 预测分析面板
 *  - ModelTrainingDialog 模型训练对话框
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Grid,
  Cpu,
  Plus,
  Search,
  Refresh,
  Upload,
  Download,
  ArrowDown,
  DataAnalysis,
  TrendCharts,
  ScaleToOriginal,
  Warning,
  Opportunity,
  Share,
  Aim,
  MagicStick,
  Monitor,
  BranchesOutlined
} from '@element-plus/icons-vue'

import DataMiningPanel from '../components/mining/DataMiningPanel.vue'
import PredictiveAnalysisPanel from '../components/mining/PredictiveAnalysisPanel.vue'
import ModelTrainingDialog from '../components/mining/ModelTrainingDialog.vue'

// 这些组件暂时注释掉，因为还没有创建
// import PatternDiscoveryPanel from '../components/mining/PatternDiscoveryPanel.vue'
// import AnomalyDetectionPanel from '../components/mining/AnomalyDetectionPanel.vue'
// import ForecastPanel from '../components/mining/ForecastPanel.vue'
// import AssociationAnalysisPanel from '../components/mining/AssociationAnalysisPanel.vue'

// ===== 响应式数据 =====
const selectedFunction = ref('mining')
const showTemplatesDialog = ref(false)
const showModelsDialog = ref(false)
const showTrainingDialog = ref(false)
const selectedTemplateCategory = ref('all')
const modelSearchQuery = ref('')

// 功能列表
const functions = ref([
  {
    key: 'mining',
    title: '数据挖掘',
    description: '关联规则、频繁项集、聚类分析',
    icon: 'DataAnalysis',
    badge: 5,
    badgeType: 'primary'
  },
  {
    key: 'prediction',
    title: '预测分析',
    description: '时间序列预测、回归分析',
    icon: 'TrendCharts',
    badge: 3,
    badgeType: 'success'
  },
  {
    key: 'patterns',
    title: '模式发现',
    description: '模式识别、特征提取',
    icon: 'ScaleToOriginal',
    badge: 8,
    badgeType: 'warning'
  },
  {
    key: 'anomaly',
    title: '异常检测',
    description: '异常点检测、离群值分析',
    icon: 'Warning',
    badge: 2,
    badgeType: 'danger'
  },
  {
    key: 'forecast',
    title: '趋势预测',
    description: '长期趋势、季节性分析',
    icon: 'Opportunity',
    badge: 4,
    badgeType: 'info'
  },
  {
    key: 'association',
    title: '关联分析',
    description: '变量关联、因果关系',
    icon: 'Share',
    badge: 6,
    badgeType: 'primary'
  }
])

// 模板分类
const templateCategories = ref([
  { value: 'all', label: '全部' },
  { value: 'mining', label: '数据挖掘' },
  { value: 'prediction', label: '预测分析' },
  { value: 'patterns', label: '模式发现' },
  { value: 'anomaly', label: '异常检测' }
])

// 分析模板
const analysisTemplates = ref([
  {
    id: '1',
    name: '设备故障预测',
    description: '基于设备运行数据预测故障发生概率',
    category: 'prediction',
    icon: 'Monitor',
    tags: ['设备', '故障', '预测'],
    useCount: 245,
    successRate: 87
  },
  {
    id: '2',
    name: '生产质量关联分析',
    description: '分析工艺参数与产品质量的关联关系',
    category: 'mining',
    icon: 'ScaleToOriginal',
    tags: ['质量', '工艺', '关联'],
    useCount: 156,
    successRate: 92
  },
  {
    id: '3',
    name: '能耗异常检测',
    description: '检测设备异常能耗模式',
    category: 'anomaly',
    icon: 'Warning',
    tags: ['能耗', '异常', '检测'],
    useCount: 89,
    successRate: 94
  },
  {
    id: '4',
    name: '生产模式发现',
    description: '发现不同生产条件下的运行模式',
    category: 'patterns',
    icon: 'Aim',
    tags: ['生产', '模式', '发现'],
    useCount: 123,
    successRate: 88
  },
  {
    id: '5',
    name: '需求预测',
    description: '基于历史数据预测产品需求趋势',
    category: 'prediction',
    icon: 'TrendCharts',
    tags: ['需求', '预测', '趋势'],
    useCount: 67,
    successRate: 85
  },
  {
    id: '6',
    name: '设备聚类分析',
    description: '根据运行特征对设备进行聚类分析',
    category: 'mining',
    icon: 'BranchesOutlined',
    tags: ['设备', '聚类', '特征'],
    useCount: 78,
    successRate: 91
  }
])

// 模型列表
const models = ref([
  {
    id: '1',
    name: '温度预测LSTM',
    algorithm: 'LSTM',
    accuracy: 94.2,
    status: '已部署',
    trainingTime: '2025-07-26 14:30:00',
    lastUsed: '2025-07-27 09:15:00'
  },
  {
    id: '2',
    name: '压力异常检测',
    algorithm: '孤立森林',
    accuracy: 89.7,
    status: '训练中',
    trainingTime: '2025-07-27 10:20:00',
    lastUsed: '2025-07-27 08:45:00'
  },
  {
    id: '3',
    name: '设备故障分类',
    algorithm: '随机森林',
    accuracy: 92.8,
    status: '已停用',
    trainingTime: '2025-07-25 16:15:00',
    lastUsed: '2025-07-26 11:30:00'
  },
  {
    id: '4',
    name: '能耗预测XGBoost',
    algorithm: 'XGBoost',
    accuracy: 96.1,
    status: '已部署',
    trainingTime: '2025-07-27 08:00:00',
    lastUsed: '2025-07-27 09:30:00'
  }
])

// 统计数据
const stats = ref({
  miningTasks: 12,
  predictions: 8,
  patterns: 15,
  anomalies: 6,
  forecasts: 9,
  associations: 11
})

// 分析结果数据
const miningTasks = ref([])
const discoveredPatterns = ref([])
const detectedAnomalies = ref([])
const forecastResults = ref([])
const associationRules = ref([])

// ===== 计算属性 =====
const filteredTemplates = computed(() => {
  if (selectedTemplateCategory.value === 'all') {
    return analysisTemplates.value
  }
  return analysisTemplates.value.filter(t => t.category === selectedTemplateCategory.value)
})

const filteredModels = computed(() => {
  if (!modelSearchQuery.value) {
    return models.value
  }
  return models.value.filter(m => 
    m.name.toLowerCase().includes(modelSearchQuery.value.toLowerCase()) ||
    m.algorithm.toLowerCase().includes(modelSearchQuery.value.toLowerCase())
  )
})

// ===== 方法 =====

/**
 * 选择功能
 */
function selectFunction(func: string) {
  selectedFunction.value = func
  
  const functionName = functions.value.find(f => f.key === func)?.title
  ElMessage.success(`已切换到${functionName}`)
}

/**
 * 应用模板
 */
function applyTemplate(template: any) {
  showTemplatesDialog.value = false
  
  // 根据模板类型切换到对应功能
  selectedFunction.value = template.category
  
  ElMessage.success(`已应用模板: ${template.name}`)
}

/**
 * 模型训练开始
 */
function onTrainingStarted(config: any) {
  ElMessage.success('模型训练已开始，请在模型管理中查看进度')
  
  // 添加新模型到列表
  const newModel = {
    id: Date.now().toString(),
    name: `新模型_${Date.now()}`,
    algorithm: config.model.algorithm,
    accuracy: 0,
    status: '训练中',
    trainingTime: new Date().toLocaleString('zh-CN'),
    lastUsed: '--'
  }
  
  models.value.unshift(newModel)
}

/**
 * 部署模型
 */
function deployModel(model: any) {
  ElMessageBox.confirm(
    `确定要部署模型 "${model.name}" 吗？`,
    '部署确认',
    {
      confirmButtonText: '部署',
      cancelButtonText: '取消',
      type: 'info'
    }
  ).then(() => {
    model.status = '已部署'
    ElMessage.success('模型部署成功')
  }).catch(() => {
    // 取消操作
  })
}

/**
 * 评估模型
 */
function evaluateModel(model: any) {
  ElMessage.info(`模型 "${model.name}" 评估功能开发中...`)
}

/**
 * 处理模型操作
 */
function handleModelAction(command: string, model: any) {
  switch (command) {
    case 'export':
      ElMessage.info(`导出模型 "${model.name}" 功能开发中...`)
      break
    case 'copy':
      const copy = { ...model, id: Date.now().toString(), name: `${model.name}_副本` }
      models.value.unshift(copy)
      ElMessage.success('模型已复制')
      break
    case 'retrain':
      model.status = '训练中'
      ElMessage.success('已开始重新训练模型')
      break
    case 'delete':
      ElMessageBox.confirm(
        `确定要删除模型 "${model.name}" 吗？`,
        '删除确认',
        {
          confirmButtonText: '删除',
          cancelButtonText: '取消',
          type: 'warning'
        }
      ).then(() => {
        const index = models.value.findIndex(m => m.id === model.id)
        if (index !== -1) {
          models.value.splice(index, 1)
          ElMessage.success('模型已删除')
        }
      }).catch(() => {
        // 取消操作
      })
      break
  }
}

/**
 * 刷新模型列表
 */
function refreshModels() {
  ElMessage.success('模型列表已刷新')
}

/**
 * 导入模型
 */
function importModel() {
  ElMessage.info('模型导入功能开发中...')
}

/**
 * 导出全部结果
 */
function exportAllResults() {
  ElMessage.info('全部结果导出功能开发中...')
}

/**
 * 获取准确率样式类
 */
function getAccuracyClass(accuracy: number): string {
  if (accuracy >= 90) return 'accuracy-high'
  if (accuracy >= 80) return 'accuracy-medium'
  return 'accuracy-low'
}

/**
 * 获取状态类型
 */
function getStatusType(status: string): string {
  if (status === '已部署') return 'success'
  if (status === '训练中') return 'warning'
  if (status === '已停用') return 'info'
  return 'danger'
}

// 事件处理函数
function onMiningTaskCreated(task: any) {
  miningTasks.value.push(task)
  stats.value.miningTasks++
}

function onMiningTaskExecuted(result: any) {
  ElMessage.success('数据挖掘任务执行完成')
}

function onPredictionCompleted(result: any) {
  stats.value.predictions++
  ElMessage.success('预测分析完成')
}

function onPatternDiscovered(patterns: any[]) {
  discoveredPatterns.value.push(...patterns)
  stats.value.patterns += patterns.length
}

function onAnomalyDetected(anomalies: any[]) {
  detectedAnomalies.value.push(...anomalies)
  stats.value.anomalies += anomalies.length
}

function onForecastGenerated(forecast: any) {
  forecastResults.value.push(forecast)
  stats.value.forecasts++
}

function onAssociationRulesGenerated(rules: any[]) {
  associationRules.value.push(...rules)
  stats.value.associations += rules.length
}

// ===== 生命周期 =====
onMounted(() => {
  // 初始化默认功能
  selectedFunction.value = 'mining'
})
</script>

<style scoped lang="scss">
.data-mining-page {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;
    
    .header-content {
      h1 {
        margin: 0 0 8px 0;
        font-size: 24px;
        color: #303133;
      }
      
      p {
        margin: 0;
        color: #606266;
        font-size: 14px;
        line-height: 1.5;
      }
    }
    
    .header-actions {
      display: flex;
      gap: 12px;
    }
  }
  
  .page-content {
    .function-nav {
      margin-bottom: 24px;
      
      .nav-items {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 16px;
        
        .nav-item {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 16px;
          border: 2px solid #ebeef5;
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.3s;
          position: relative;
          
          &:hover {
            border-color: #c6e2ff;
            background: #f0f9ff;
          }
          
          &.active {
            border-color: #409eff;
            background: #f0f9ff;
            
            .nav-icon {
              color: #409eff;
            }
          }
          
          .nav-icon {
            font-size: 28px;
            color: #909399;
            flex-shrink: 0;
          }
          
          .nav-info {
            flex: 1;
            
            .nav-title {
              font-size: 16px;
              font-weight: 600;
              color: #303133;
              margin-bottom: 4px;
            }
            
            .nav-description {
              font-size: 13px;
              color: #606266;
              line-height: 1.4;
            }
          }
          
          .nav-badge {
            position: absolute;
            top: -8px;
            right: -8px;
          }
        }
      }
    }
    
    .function-panel {
      background: white;
      border-radius: 8px;
      border: 1px solid #ebeef5;
      min-height: 500px;
    }
  }
  
  .results-overview {
    margin-top: 24px;
    
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      h3 {
        margin: 0;
        color: #303133;
        font-size: 16px;
      }
    }
    
    .overview-stats {
      .stat-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 20px 16px;
        background: #f8f9fa;
        border-radius: 8px;
        border-left: 4px solid #409eff;
        
        .stat-icon {
          width: 48px;
          height: 48px;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 24px;
          color: white;
          
          &.mining { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); }
          &.prediction { background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%); }
          &.patterns { background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%); }
          &.anomalies { background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%); }
          &.forecasts { background: linear-gradient(135deg, #fa709a 0%, #fee140 100%); }
          &.associations { background: linear-gradient(135deg, #a8edea 0%, #fed6e3 100%); }
        }
        
        .stat-content {
          .stat-value {
            font-size: 24px;
            font-weight: 600;
            color: #303133;
            margin-bottom: 4px;
          }
          
          .stat-label {
            font-size: 12px;
            color: #606266;
          }
        }
      }
    }
  }
}

// 对话框样式
.templates-content {
  .template-categories {
    margin-bottom: 20px;
    text-align: center;
  }
  
  .templates-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
    
    .template-card {
      padding: 16px;
      border: 1px solid #ebeef5;
      border-radius: 8px;
      cursor: pointer;
      transition: all 0.3s;
      
      &:hover {
        border-color: #c6e2ff;
        background: #f0f9ff;
      }
      
      .template-icon {
        font-size: 32px;
        color: #409eff;
        text-align: center;
        margin-bottom: 12px;
      }
      
      .template-info {
        .template-name {
          font-size: 16px;
          font-weight: 600;
          color: #303133;
          margin-bottom: 6px;
        }
        
        .template-description {
          font-size: 13px;
          color: #606266;
          line-height: 1.4;
          margin-bottom: 8px;
        }
        
        .template-tags {
          display: flex;
          flex-wrap: wrap;
          gap: 4px;
          margin-bottom: 8px;
        }
        
        .template-stats {
          display: flex;
          justify-content: space-between;
          font-size: 12px;
          color: #909399;
        }
      }
    }
  }
}

.models-content {
  .models-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    
    .toolbar-actions {
      display: flex;
      gap: 8px;
    }
  }
  
  :deep(.el-table) {
    .accuracy-high { color: #67c23a; font-weight: 500; }
    .accuracy-medium { color: #e6a23c; font-weight: 500; }
    .accuracy-low { color: #f56c6c; font-weight: 500; }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .data-mining-page {
    .function-nav .nav-items {
      grid-template-columns: repeat(2, 1fr);
    }
    
    .overview-stats {
      :deep(.el-col) {
        margin-bottom: 16px;
      }
    }
  }
  
  .templates-grid {
    grid-template-columns: 1fr !important;
  }
}

@media (max-width: 768px) {
  .data-mining-page {
    .page-header {
      flex-direction: column;
      gap: 16px;
      align-items: stretch;
      
      .header-actions {
        justify-content: center;
        flex-wrap: wrap;
      }
    }
    
    .function-nav .nav-items {
      grid-template-columns: 1fr;
    }
    
    .models-toolbar {
      flex-direction: column !important;
      gap: 12px;
      align-items: stretch !important;
      
      .toolbar-actions {
        justify-content: center;
      }
    }
  }
}
</style>