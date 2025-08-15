<template>
  <div class="data-comparison-page">
    <div class="page-header">
      <div class="header-content">
        <h1>数据对比分析</h1>
        <p>多维度数据对比、趋势分析和相关性分析</p>
      </div>
      <div class="header-actions">
        <el-button @click="showTemplateDialog = true">
          <el-icon><Grid /></el-icon>
          分析模板
        </el-button>
        <el-button @click="showHistoryDialog = true">
          <el-icon><Clock /></el-icon>
          历史分析
        </el-button>
        <el-button type="primary" @click="createNewAnalysis">
          <el-icon><Plus /></el-icon>
          新建分析
        </el-button>
      </div>
    </div>

    <div class="page-content">
      <!-- 分析类型选择 -->
      <div class="analysis-selector">
        <el-card shadow="never">
          <div class="selector-header">
            <h3>选择分析类型</h3>
            <div class="selector-tips">
              <el-tooltip content="选择不同的分析类型来探索数据间的关系和模式">
                <el-icon><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
          </div>

          <div class="analysis-types">
            <div
              v-for="type in analysisTypes"
              :key="type.key"
              :class="[
                'analysis-type',
                { active: selectedAnalysisType === type.key },
              ]"
              @click="selectAnalysisType(type.key)"
            >
              <div class="type-icon">
                <el-icon><component :is="type.icon" /></el-icon>
              </div>
              <div class="type-info">
                <div class="type-name">{{ type.name }}</div>
                <div class="type-description">{{ type.description }}</div>
              </div>
              <div class="type-features">
                <el-tag
                  v-for="feature in type.features"
                  :key="feature"
                  size="small"
                  effect="plain"
                >
                  {{ feature }}
                </el-tag>
              </div>
            </div>
          </div>
        </el-card>
      </div>

      <!-- 数据对比分析面板 -->
      <div v-if="selectedAnalysisType === 'comparison'" class="analysis-panel">
        <DataComparisonPanel
          :data="analysisData"
          :variables="selectedVariables"
        />
      </div>

      <!-- 趋势对比图表 -->
      <div v-if="selectedAnalysisType === 'trend'" class="analysis-panel">
        <TrendComparisonChart
          :data="trendData"
          :series="trendSeries"
          :title="'多变量趋势对比分析'"
        />
      </div>

      <!-- 相关性分析 -->
      <div v-if="selectedAnalysisType === 'correlation'" class="analysis-panel">
        <CorrelationAnalysis
          :data="correlationData"
          :variables="selectedVariables"
        />
      </div>

      <!-- 分布对比分析 -->
      <div
        v-if="selectedAnalysisType === 'distribution'"
        class="analysis-panel"
      >
        <DistributionComparison
          :data="distributionData"
          :variables="selectedVariables"
        />
      </div>

      <!-- 空状态 -->
      <el-empty
        v-if="!selectedAnalysisType"
        description="请选择分析类型开始探索数据"
      >
        <div class="empty-actions">
          <el-button
            v-for="type in analysisTypes.slice(0, 3)"
            :key="type.key"
            @click="selectAnalysisType(type.key)"
          >
            {{ type.name }}
          </el-button>
        </div>
      </el-empty>
    </div>

    <!-- 分析模板对话框 -->
    <el-dialog v-model="showTemplateDialog" title="分析模板" width="800px">
      <div class="template-grid">
        <div
          v-for="template in analysisTemplates"
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
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 历史分析对话框 -->
    <el-dialog v-model="showHistoryDialog" title="历史分析" width="900px">
      <el-table :data="analysisHistory" style="width: 100%">
        <el-table-column prop="name" label="分析名称" width="200" />
        <el-table-column prop="type" label="类型" width="120">
          <template #default="{ row }">
            <el-tag :type="getAnalysisTypeTag(row.type)" size="small">
              {{ getAnalysisTypeName(row.type) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="variables" label="变量" width="150">
          <template #default="{ row }">
            <span>{{ row.variables.join(', ') }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="createTime" label="创建时间" width="150" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)" size="small">
              {{ row.status }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150">
          <template #default="{ row }">
            <el-button size="small" text @click="loadAnalysis(row)">
              加载
            </el-button>
            <el-button size="small" text @click="copyAnalysis(row)">
              复制
            </el-button>
            <el-button
              size="small"
              text
              type="danger"
              @click="deleteAnalysis(row)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>

    <!-- 新建分析对话框 -->
    <el-dialog v-model="showNewAnalysisDialog" title="新建分析" width="600px">
      <el-form :model="newAnalysisForm" label-width="120px">
        <el-form-item label="分析名称">
          <el-input
            v-model="newAnalysisForm.name"
            placeholder="请输入分析名称"
          />
        </el-form-item>

        <el-form-item label="分析类型">
          <el-select v-model="newAnalysisForm.type" placeholder="选择分析类型">
            <el-option
              v-for="type in analysisTypes"
              :key="type.key"
              :label="type.name"
              :value="type.key"
            />
          </el-select>
        </el-form-item>

        <el-form-item label="选择变量">
          <el-select
            v-model="newAnalysisForm.variables"
            multiple
            placeholder="选择要分析的变量"
            style="width: 100%"
          >
            <el-option
              v-for="variable in availableVariables"
              :key="variable.key"
              :label="variable.name"
              :value="variable.key"
            />
          </el-select>
        </el-form-item>

        <el-form-item label="时间范围">
          <el-date-picker
            v-model="newAnalysisForm.dateRange"
            type="datetimerange"
            range-separator="至"
            start-placeholder="开始时间"
            end-placeholder="结束时间"
            format="YYYY-MM-DD HH:mm:ss"
            value-format="YYYY-MM-DD HH:mm:ss"
          />
        </el-form-item>

        <el-form-item label="描述">
          <el-input
            v-model="newAnalysisForm.description"
            type="textarea"
            :rows="3"
            placeholder="分析描述（可选）"
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showNewAnalysisDialog = false">取消</el-button>
        <el-button type="primary" @click="saveNewAnalysis">创建分析</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * DataComparisonPage —— 数据对比分析页面
 *
 * 📝 Responsibilities:
 *  1. 数据对比分析的统一入口
 *  2. 多种分析类型选择和切换
 *  3. 分析模板和历史管理
 *  4. 分析结果的整合展示
 *
 * 📦 Dependencies:
 *  - DataComparisonPanel 数据对比面板
 *  - TrendComparisonChart 趋势对比图表
 *  - CorrelationAnalysis 相关性分析
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Grid,
  Clock,
  Plus,
  QuestionFilled,
  TrendCharts,
  DataAnalysis,
  PieChart,
  ScaleToOriginal,
  Monitor,
  Setting,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, onMounted } from 'vue'

import CorrelationAnalysis from '../components/comparison/CorrelationAnalysis.vue'
import DataComparisonPanel from '../components/comparison/DataComparisonPanel.vue'
import DistributionComparison from '../components/comparison/DistributionComparison.vue'
import TrendComparisonChart from '../components/comparison/TrendComparisonChart.vue'

// ===== 响应式数据 =====
const selectedAnalysisType = ref('')
const showTemplateDialog = ref(false)
const showHistoryDialog = ref(false)
const showNewAnalysisDialog = ref(false)

// 分析类型配置
const analysisTypes = ref([
  {
    key: 'comparison',
    name: '多维度对比',
    description: '多个数据源的综合对比分析，支持统计检验和异常检测',
    icon: 'DataAnalysis',
    features: ['统计检验', '异常检测', '多目标对比'],
  },
  {
    key: 'trend',
    name: '趋势对比',
    description: '时间序列数据的趋势对比，支持多种分析方法和可视化',
    icon: 'TrendCharts',
    features: ['时间序列', '趋势分析', '预测对比'],
  },
  {
    key: 'correlation',
    name: '相关性分析',
    description: '变量间相关性分析，包括相关矩阵、网络图和滞后分析',
    icon: 'ScaleToOriginal',
    features: ['相关矩阵', '网络分析', '滞后分析'],
  },
  {
    key: 'distribution',
    name: '分布对比',
    description: '数据分布特征对比，包括概率分布、统计特征等',
    icon: 'PieChart',
    features: ['分布拟合', '特征对比', '假设检验'],
  },
])

// 分析模板
const analysisTemplates = ref([
  {
    id: '1',
    name: '设备性能对比',
    description: '对比不同设备的性能指标和运行状态',
    icon: 'Monitor',
    tags: ['设备', '性能', '对比'],
    type: 'comparison',
    variables: ['temperature', 'pressure', 'efficiency'],
  },
  {
    id: '2',
    name: '工艺参数相关性',
    description: '分析工艺参数之间的相关关系',
    icon: 'ScaleToOriginal',
    tags: ['工艺', '参数', '相关性'],
    type: 'correlation',
    variables: ['temperature', 'pressure', 'flow', 'ph'],
  },
  {
    id: '3',
    name: '质量趋势分析',
    description: '对比不同时期的质量指标趋势',
    icon: 'TrendCharts',
    tags: ['质量', '趋势', '时间'],
    type: 'trend',
    variables: ['quality_score', 'defect_rate', 'compliance'],
  },
  {
    id: '4',
    name: '能耗分布对比',
    description: '对比不同时段的能耗分布特征',
    icon: 'PieChart',
    tags: ['能耗', '分布', '对比'],
    type: 'distribution',
    variables: ['power_consumption', 'energy_efficiency'],
  },
])

// 可用变量
const availableVariables = ref([
  { key: 'temperature', name: '温度' },
  { key: 'pressure', name: '压力' },
  { key: 'flow', name: '流量' },
  { key: 'power', name: '功率' },
  { key: 'ph', name: 'pH值' },
  { key: 'humidity', name: '湿度' },
  { key: 'efficiency', name: '效率' },
  { key: 'quality_score', name: '质量得分' },
  { key: 'defect_rate', name: '缺陷率' },
  { key: 'energy_efficiency', name: '能效比' },
])

// 新建分析表单
const newAnalysisForm = ref({
  name: '',
  type: '',
  variables: [],
  dateRange: [],
  description: '',
})

// 分析历史
const analysisHistory = ref([
  {
    id: '1',
    name: '温度压力关联分析',
    type: 'correlation',
    variables: ['温度', '压力'],
    createTime: '2025-07-26 14:30:00',
    status: '已完成',
  },
  {
    id: '2',
    name: '设备效率对比',
    type: 'comparison',
    variables: ['效率', '功率'],
    createTime: '2025-07-25 09:15:00',
    status: '已完成',
  },
  {
    id: '3',
    name: '质量趋势监控',
    type: 'trend',
    variables: ['质量得分', '缺陷率'],
    createTime: '2025-07-24 16:45:00',
    status: '进行中',
  },
])

// ===== 计算属性 =====
const selectedVariables = computed(() => {
  // 根据分析类型返回相应的变量
  return ['温度', '压力', '流量', '功率', 'pH值', '湿度']
})

const analysisData = computed(() => {
  // 生成模拟对比分析数据
  return generateMockAnalysisData()
})

const trendData = computed(() => {
  // 生成模拟趋势数据
  return generateMockTrendData()
})

const correlationData = computed(() => {
  // 生成模拟相关性数据
  return generateMockCorrelationData()
})

const distributionData = computed(() => {
  // 生成模拟分布数据
  return generateMockDistributionData()
})

const trendSeries = computed(() => [
  { name: '设备A', dataKey: 'deviceA', color: '#409EFF' },
  { name: '设备B', dataKey: 'deviceB', color: '#67C23A' },
  { name: '设备C', dataKey: 'deviceC', color: '#E6A23C' },
])

// ===== 方法 =====

/**
 * 选择分析类型
 */
function selectAnalysisType(type: string) {
  selectedAnalysisType.value = type

  const typeName = analysisTypes.value.find(t => t.key === type)?.name
  ElMessage.success(`已切换到${typeName}分析`)
}

/**
 * 创建新分析
 */
function createNewAnalysis() {
  showNewAnalysisDialog.value = true

  // 重置表单
  newAnalysisForm.value = {
    name: '',
    type: '',
    variables: [],
    dateRange: [],
    description: '',
  }
}

/**
 * 保存新分析
 */
function saveNewAnalysis() {
  if (!newAnalysisForm.value.name) {
    ElMessage.warning('请输入分析名称')
    return
  }

  if (!newAnalysisForm.value.type) {
    ElMessage.warning('请选择分析类型')
    return
  }

  if (!newAnalysisForm.value.variables.length) {
    ElMessage.warning('请至少选择一个变量')
    return
  }

  // 添加到历史记录
  const newAnalysis = {
    id: Date.now().toString(),
    name: newAnalysisForm.value.name,
    type: newAnalysisForm.value.type,
    variables: newAnalysisForm.value.variables.map(
      key => availableVariables.value.find(v => v.key === key)?.name || key
    ),
    createTime: new Date().toLocaleString('zh-CN'),
    status: '已完成',
  }

  analysisHistory.value.unshift(newAnalysis)

  // 应用新分析
  selectedAnalysisType.value = newAnalysisForm.value.type

  showNewAnalysisDialog.value = false
  ElMessage.success('分析创建成功')
}

/**
 * 应用模板
 */
function applyTemplate(template: any) {
  selectedAnalysisType.value = template.type
  showTemplateDialog.value = false

  ElMessage.success(`已应用模板: ${template.name}`)
}

/**
 * 加载历史分析
 */
function loadAnalysis(analysis: any) {
  selectedAnalysisType.value = analysis.type
  showHistoryDialog.value = false

  ElMessage.success(`已加载分析: ${analysis.name}`)
}

/**
 * 复制分析
 */
function copyAnalysis(analysis: any) {
  const copy = {
    ...analysis,
    id: Date.now().toString(),
    name: `${analysis.name} - 副本`,
    createTime: new Date().toLocaleString('zh-CN'),
    status: '已完成',
  }

  analysisHistory.value.unshift(copy)
  ElMessage.success('分析已复制')
}

/**
 * 删除分析
 */
function deleteAnalysis(analysis: any) {
  ElMessageBox.confirm(`确定要删除分析 "${analysis.name}" 吗？`, '删除确认', {
    confirmButtonText: '删除',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(() => {
      const index = analysisHistory.value.findIndex(a => a.id === analysis.id)
      if (index !== -1) {
        analysisHistory.value.splice(index, 1)
        ElMessage.success('分析已删除')
      }
    })
    .catch(() => {
      // 取消操作
    })
}

/**
 * 获取分析类型标签
 */
function getAnalysisTypeTag(type: string): string {
  const typeMap: { [key: string]: string } = {
    comparison: 'primary',
    trend: 'success',
    correlation: 'warning',
    distribution: 'info',
  }
  return typeMap[type] || 'info'
}

/**
 * 获取分析类型名称
 */
function getAnalysisTypeName(type: string): string {
  const analysis = analysisTypes.value.find(a => a.key === type)
  return analysis?.name || type
}

/**
 * 获取状态类型
 */
function getStatusType(status: string): string {
  if (status === '已完成') return 'success'
  if (status === '进行中') return 'warning'
  if (status === '失败') return 'danger'
  return 'info'
}

/**
 * 生成模拟数据
 */
function generateMockAnalysisData() {
  const data = []
  const now = new Date()

  for (let i = 0; i < 100; i++) {
    const time = new Date(now.getTime() - i * 60 * 60 * 1000)
    data.unshift({
      timestamp: time.toISOString().slice(0, 19).replace('T', ' '),
      temperature: 25 + Math.random() * 10,
      pressure: 1.0 + Math.random() * 0.5,
      flow: 150 + Math.random() * 50,
      power: 800 + Math.random() * 200,
    })
  }

  return data
}

function generateMockTrendData() {
  const data = []
  const now = new Date()

  for (let i = 0; i < 48; i++) {
    const time = new Date(now.getTime() - i * 60 * 60 * 1000)
    data.unshift({
      timestamp: time.toISOString().slice(0, 19).replace('T', ' '),
      deviceA: 80 + Math.random() * 20 + Math.sin(i / 8) * 10,
      deviceB: 75 + Math.random() * 25 + Math.cos(i / 6) * 8,
      deviceC: 85 + Math.random() * 15 + Math.sin(i / 10) * 12,
    })
  }

  return data
}

function generateMockCorrelationData() {
  // 相关性分析的模拟数据在组件内部生成
  return []
}

function generateMockDistributionData() {
  // 分布对比的模拟数据在组件内部生成
  return []
}

// ===== 生命周期 =====
onMounted(() => {
  // 初始化默认分析类型
  selectedAnalysisType.value = 'comparison'
})
</script>

<style scoped lang="scss">
.data-comparison-page {
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
      }
    }

    .header-actions {
      display: flex;
      gap: 12px;
    }
  }

  .page-content {
    .analysis-selector {
      margin-bottom: 24px;

      .selector-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 20px;

        h3 {
          margin: 0;
          color: #303133;
          font-size: 18px;
        }

        .selector-tips {
          color: #909399;
          cursor: help;
        }
      }

      .analysis-types {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 16px;

        .analysis-type {
          padding: 20px;
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

            .type-icon {
              color: #409eff;
            }
          }

          .type-icon {
            font-size: 32px;
            color: #909399;
            margin-bottom: 12px;
          }

          .type-info {
            margin-bottom: 12px;

            .type-name {
              font-size: 16px;
              font-weight: 600;
              color: #303133;
              margin-bottom: 4px;
            }

            .type-description {
              font-size: 13px;
              color: #606266;
              line-height: 1.5;
            }
          }

          .type-features {
            display: flex;
            flex-wrap: wrap;
            gap: 6px;
          }
        }
      }
    }

    .analysis-panel {
      background: white;
      border-radius: 8px;
      border: 1px solid #ebeef5;
      padding: 24px;
    }

    .empty-actions {
      display: flex;
      gap: 12px;
      margin-top: 16px;
    }
  }
}

// 对话框样式
.template-grid {
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
      font-size: 24px;
      color: #409eff;
      margin-bottom: 12px;
    }

    .template-info {
      .template-name {
        font-size: 14px;
        font-weight: 600;
        color: #303133;
        margin-bottom: 6px;
      }

      .template-description {
        font-size: 12px;
        color: #606266;
        line-height: 1.4;
        margin-bottom: 8px;
      }

      .template-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 4px;
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .data-comparison-page {
    .analysis-selector .analysis-types {
      grid-template-columns: 1fr;
    }
  }

  .template-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .data-comparison-page {
    .page-header {
      flex-direction: column;
      gap: 16px;
      align-items: stretch;

      .header-actions {
        justify-content: center;
        flex-wrap: wrap;
      }
    }

    .analysis-panel {
      padding: 16px;
    }
  }
}
</style>
