<template>
  <el-dialog
    v-model="visible"
    title="生成报表"
    width="900px"
    :close-on-click-modal="false"
    @closed="handleClose"
  >
    <div class="report-generator">
      <!-- 步骤指示器 -->
      <el-steps :active="currentStep" align-center class="steps">
        <el-step title="选择模板" />
        <el-step title="配置参数" />
        <el-step title="预览生成" />
      </el-steps>
      
      <!-- 步骤1: 模板选择 -->
      <div v-if="currentStep === 0" class="step-content template-selection">
        <h4>选择报表模板</h4>
        <div class="template-grid">
          <div
            v-for="template in reportTemplates"
            :key="template.id"
            :class="['template-card', { selected: selectedTemplate?.id === template.id }]"
            @click="selectTemplate(template)"
          >
            <div class="template-icon">
              <el-icon><component :is="template.icon" /></el-icon>
            </div>
            <div class="template-info">
              <div class="template-name">{{ template.name }}</div>
              <div class="template-desc">{{ template.description }}</div>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 步骤2: 参数配置 -->
      <div v-if="currentStep === 1" class="step-content parameter-config">
        <h4>配置报表参数</h4>
        
        <el-form :model="reportConfig" label-width="120px">
          <!-- 基础设置 -->
          <el-card class="config-section" shadow="never">
            <template #header>
              <span>基础设置</span>
            </template>
            
            <el-form-item label="报表名称">
              <el-input v-model="reportConfig.name" placeholder="请输入报表名称" />
            </el-form-item>
            
            <el-form-item label="时间范围">
              <el-date-picker
                v-model="reportConfig.dateRange"
                type="datetimerange"
                range-separator="至"
                start-placeholder="开始时间"
                end-placeholder="结束时间"
                format="YYYY-MM-DD HH:mm:ss"
                value-format="YYYY-MM-DD HH:mm:ss"
              />
            </el-form-item>
            
            <el-form-item label="数据粒度">
              <el-select v-model="reportConfig.granularity" placeholder="选择数据粒度">
                <el-option label="分钟" value="minute" />
                <el-option label="小时" value="hour" />
                <el-option label="天" value="day" />
                <el-option label="周" value="week" />
                <el-option label="月" value="month" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="导出格式">
              <el-checkbox-group v-model="reportConfig.formats">
                <el-checkbox label="pdf" border>PDF</el-checkbox>
                <el-checkbox label="excel" border>Excel</el-checkbox>
                <el-checkbox label="csv" border>CSV</el-checkbox>
                <el-checkbox label="json" border>JSON</el-checkbox>
              </el-checkbox-group>
            </el-form-item>
          </el-card>
          
          <!-- 数据源设置 -->
          <el-card class="config-section" shadow="never">
            <template #header>
              <span>数据源设置</span>
            </template>
            
            <el-form-item label="设备筛选">
              <el-select
                v-model="reportConfig.deviceIds"
                multiple
                filterable
                placeholder="选择设备"
                collapse-tags
                collapse-tags-tooltip
              >
                <el-option
                  v-for="device in devices"
                  :key="device.id"
                  :label="device.name"
                  :value="device.id"
                />
              </el-select>
            </el-form-item>
            
            <el-form-item label="标签筛选">
              <el-select
                v-model="reportConfig.tagIds"
                multiple
                filterable
                placeholder="选择标签"
                collapse-tags
                collapse-tags-tooltip
              >
                <el-option
                  v-for="tag in tags"
                  :key="tag.id"
                  :label="tag.name"
                  :value="tag.id"
                />
              </el-select>
            </el-form-item>
            
            <el-form-item label="数据类型">
              <el-checkbox-group v-model="reportConfig.dataTypes">
                <el-checkbox label="telemetry" border>遥测数据</el-checkbox>
                <el-checkbox label="events" border>事件数据</el-checkbox>
                <el-checkbox label="alarms" border>报警数据</el-checkbox>
                <el-checkbox label="status" border>状态数据</el-checkbox>
              </el-checkbox-group>
            </el-form-item>
          </el-card>
          
          <!-- 图表设置 -->
          <el-card class="config-section" shadow="never">
            <template #header>
              <span>图表设置</span>
            </template>
            
            <el-form-item label="图表类型">
              <el-checkbox-group v-model="reportConfig.chartTypes">
                <el-checkbox label="line" border>折线图</el-checkbox>
                <el-checkbox label="bar" border>柱状图</el-checkbox>
                <el-checkbox label="pie" border>饼图</el-checkbox>
                <el-checkbox label="scatter" border>散点图</el-checkbox>
                <el-checkbox label="heatmap" border>热力图</el-checkbox>
              </el-checkbox-group>
            </el-form-item>
            
            <el-form-item label="统计指标">
              <el-checkbox-group v-model="reportConfig.statistics">
                <el-checkbox label="avg" border>平均值</el-checkbox>
                <el-checkbox label="max" border>最大值</el-checkbox>
                <el-checkbox label="min" border>最小值</el-checkbox>
                <el-checkbox label="sum" border>总和</el-checkbox>
                <el-checkbox label="count" border>计数</el-checkbox>
                <el-checkbox label="std" border>标准差</el-checkbox>
              </el-checkbox-group>
            </el-form-item>
            
            <el-form-item label="分组方式">
              <el-select v-model="reportConfig.groupBy" placeholder="选择分组方式">
                <el-option label="按设备" value="device" />
                <el-option label="按标签" value="tag" />
                <el-option label="按时间" value="time" />
                <el-option label="按状态" value="status" />
              </el-select>
            </el-form-item>
          </el-card>
        </el-form>
      </div>
      
      <!-- 步骤3: 预览生成 -->
      <div v-if="currentStep === 2" class="step-content preview-generate">
        <h4>报表预览</h4>
        
        <div class="preview-tabs">
          <el-tabs v-model="activePreviewTab">
            <el-tab-pane label="报表配置" name="config">
              <div class="config-preview">
                <el-descriptions :column="2" border>
                  <el-descriptions-item label="报表名称">
                    {{ reportConfig.name }}
                  </el-descriptions-item>
                  <el-descriptions-item label="模板类型">
                    {{ selectedTemplate?.name }}
                  </el-descriptions-item>
                  <el-descriptions-item label="时间范围">
                    {{ formatDateRange(reportConfig.dateRange) }}
                  </el-descriptions-item>
                  <el-descriptions-item label="数据粒度">
                    {{ getGranularityLabel(reportConfig.granularity) }}
                  </el-descriptions-item>
                  <el-descriptions-item label="设备数量">
                    {{ reportConfig.deviceIds?.length || 0 }} 台
                  </el-descriptions-item>
                  <el-descriptions-item label="标签数量">
                    {{ reportConfig.tagIds?.length || 0 }} 个
                  </el-descriptions-item>
                  <el-descriptions-item label="导出格式">
                    {{ reportConfig.formats?.join(', ') }}
                  </el-descriptions-item>
                  <el-descriptions-item label="图表类型">
                    {{ reportConfig.chartTypes?.join(', ') }}
                  </el-descriptions-item>
                </el-descriptions>
              </div>
            </el-tab-pane>
            
            <el-tab-pane label="数据预览" name="data">
              <div class="data-preview">
                <el-table :data="previewData" style="width: 100%" max-height="300">
                  <el-table-column prop="timestamp" label="时间" width="180" />
                  <el-table-column prop="device" label="设备" width="120" />
                  <el-table-column prop="tag" label="标签" width="120" />
                  <el-table-column prop="value" label="数值" width="100" />
                  <el-table-column prop="unit" label="单位" width="80" />
                  <el-table-column prop="status" label="状态" width="100">
                    <template #default="{ row }">
                      <el-tag :type="getStatusType(row.status)" size="small">
                        {{ row.status }}
                      </el-tag>
                    </template>
                  </el-table-column>
                </el-table>
                
                <div class="preview-info">
                  <span>预览数据：前10条记录，预计总数据量：{{ estimatedDataCount }} 条</span>
                </div>
              </div>
            </el-tab-pane>
            
            <el-tab-pane label="图表预览" name="chart">
              <div class="chart-preview">
                <div ref="previewChartRef" class="preview-chart-container"></div>
              </div>
            </el-tab-pane>
          </el-tabs>
        </div>
        
        <!-- 生成进度 -->
        <div v-if="generating" class="generation-progress">
          <el-progress :percentage="generationProgress" :status="generationStatus">
            <template #default="{ percentage }">
              <span class="progress-text">{{ generationMessage }} {{ percentage }}%</span>
            </template>
          </el-progress>
        </div>
      </div>
    </div>
    
    <!-- 对话框底部按钮 -->
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleCancel">取消</el-button>
        <el-button v-if="currentStep > 0" @click="prevStep">上一步</el-button>
        <el-button 
          v-if="currentStep < 2" 
          type="primary" 
          @click="nextStep"
          :disabled="!canNext"
        >
          下一步
        </el-button>
        <el-button 
          v-if="currentStep === 2" 
          type="primary" 
          @click="generateReport"
          :loading="generating"
          :disabled="!canGenerate"
        >
          {{ generating ? '生成中...' : '生成报表' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * ReportGeneratorDialog —— 报表生成器对话框
 *
 * 📝 Responsibilities:
 *  1. 报表模板选择
 *  2. 报表参数配置
 *  3. 数据预览和生成
 *  4. 多格式导出支持
 *
 * 📦 Dependencies:
 *  - Element Plus UI组件
 *  - ECharts 图表预览
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import * as echarts from 'echarts'
import {
  Document,
  DataAnalysis,
  TrendCharts,
  PieChart,
  Grid,
  Monitor
} from '@element-plus/icons-vue'

// ===== Props =====
const props = defineProps<{
  modelValue: boolean
}>()`

// ===== Emits =====
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'report-generated': [report: any]
}>()

// ===== 响应式数据 =====
const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const currentStep = ref(0)
const activePreviewTab = ref('config')
const generating = ref(false)
const generationProgress = ref(0)
const generationStatus = ref('')
const generationMessage = ref('')

// 预览图表
const previewChartRef = ref<HTMLDivElement>()
const previewChart = ref<echarts.ECharts>()

// 报表模板
const reportTemplates = ref([
  {
    id: 'device_summary',
    name: '设备概览报表',
    description: '设备状态、性能指标的综合分析',
    icon: 'Monitor'
  },
  {
    id: 'data_analysis',
    name: '数据分析报表',
    description: '数据趋势、统计分析和质量评估',
    icon: 'DataAnalysis'
  },
  {
    id: 'trend_report',
    name: '趋势分析报表',
    description: '长期趋势变化和预测分析',
    icon: 'TrendCharts'
  },
  {
    id: 'status_report',
    name: '状态分布报表',
    description: '设备状态分布和异常统计',
    icon: 'PieChart'
  },
  {
    id: 'performance_report',
    name: '性能评估报表',
    description: '系统性能指标和优化建议',
    icon: 'Grid'
  },
  {
    id: 'custom_report',
    name: '自定义报表',
    description: '灵活配置的个性化报表',
    icon: 'Document'
  }
])

const selectedTemplate = ref(null)

// 报表配置
const reportConfig = ref({
  name: '',
  dateRange: [],
  granularity: 'hour',
  formats: ['pdf'],
  deviceIds: [],
  tagIds: [],
  dataTypes: ['telemetry'],
  chartTypes: ['line'],
  statistics: ['avg'],
  groupBy: 'device'
})

// 模拟设备和标签数据
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

// 预览数据
const previewData = ref([
  {
    timestamp: '2025-07-27 10:00:00',
    device: 'PLC-01',
    tag: '温度',
    value: 25.6,
    unit: '°C',
    status: '正常'
  },
  {
    timestamp: '2025-07-27 10:15:00',
    device: 'PLC-01',
    tag: '压力',
    value: 1.2,
    unit: 'MPa',
    status: '正常'
  },
  {
    timestamp: '2025-07-27 10:30:00',
    device: 'PLC-02',
    tag: '流量',
    value: 150.8,
    unit: 'L/min',
    status: '警告'
  }
])

const estimatedDataCount = ref(15420)

// ===== 计算属性 =====
const canNext = computed(() => {
  if (currentStep.value === 0) {
    return selectedTemplate.value !== null
  }
  if (currentStep.value === 1) {
    return reportConfig.value.name && 
           reportConfig.value.dateRange?.length === 2 &&
           reportConfig.value.formats?.length > 0
  }
  return true
})

const canGenerate = computed(() => {
  return !generating.value && 
         reportConfig.value.deviceIds?.length > 0 &&
         reportConfig.value.tagIds?.length > 0
})

// ===== 方法 =====

/**
 * 选择模板
 */
function selectTemplate(template: any) {
  selectedTemplate.value = template
  
  // 根据模板类型预设配置
  if (template.id === 'device_summary') {
    reportConfig.value.chartTypes = ['pie', 'bar']
    reportConfig.value.statistics = ['count', 'avg']
    reportConfig.value.dataTypes = ['telemetry', 'status']
  } else if (template.id === 'data_analysis') {
    reportConfig.value.chartTypes = ['line', 'scatter']
    reportConfig.value.statistics = ['avg', 'max', 'min', 'std']
    reportConfig.value.dataTypes = ['telemetry']
  } else if (template.id === 'trend_report') {
    reportConfig.value.chartTypes = ['line']
    reportConfig.value.statistics = ['avg']
    reportConfig.value.granularity = 'day'
  }
}

/**
 * 步骤控制
 */
function nextStep() {
  if (canNext.value && currentStep.value < 2) {
    currentStep.value++
    
    if (currentStep.value === 2) {
      // 进入预览步骤时初始化预览图表
      nextTick(() => {
        initPreviewChart()
      })
    }
  }
}

function prevStep() {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

/**
 * 初始化预览图表
 */
function initPreviewChart() {
  if (!previewChartRef.value) return
  
  previewChart.value = echarts.init(previewChartRef.value)
  
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
    xAxis: {
      type: 'time',
      boundaryGap: false
    },
    yAxis: {
      type: 'value'
    },
    series: [
      {
        name: '数据预览',
        type: 'line',
        smooth: true,
        data: generatePreviewChartData()
      }
    ]
  }
  
  previewChart.value.setOption(option)
}

/**
 * 从真实API获取预览图表数据
 */
async function generatePreviewChartData() {
  try {
    const response = await fetch('/api/v1/reports/preview-data', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json'
      }
    })
    
    if (response.ok) {
      return await response.json()
    } else {
      throw new Error('获取预览数据失败')
    }
  } catch (error) {
    console.error('获取预览数据失败:', error)
    return []
  }
}

/**
 * 生成报表
 */
async function generateReport() {
  generating.value = true
  generationProgress.value = 0
  generationStatus.value = 'active'
  generationMessage.value = '准备生成报表'
  
  try {
    // 调用真实API生成报表
    const response = await fetch('/api/v1/reports/generate', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        name: reportConfig.value.name,
        template: selectedTemplate.value,
        config: reportConfig.value
      })
    })
    
    if (response.ok) {
      const reportInfo = await response.json()
      
      generationStatus.value = 'success'
      generationProgress.value = 100
      generationMessage.value = '完成'
      
      emit('report-generated', reportInfo)
      ElMessage.success('报表生成成功！')
      
      // 延迟关闭对话框
      setTimeout(() => {
        handleClose()
      }, 1000)
    } else {
      const errorData = await response.json()
      throw new Error(errorData.message || '生成失败')
    }
    
  } catch (error) {
    console.error('生成报表失败:', error)
    generationStatus.value = 'exception'
    generationMessage.value = '生成失败'
    ElMessage.error('生成报表失败')
  } finally {
    generating.value = false
  }
}

/**
 * 工具函数
 */
function formatDateRange(dateRange: string[]): string {
  if (!dateRange || dateRange.length !== 2) return '--'
  return `${dateRange[0]} 至 ${dateRange[1]}`
}

function getGranularityLabel(granularity: string): string {
  const labels: { [key: string]: string } = {
    minute: '分钟',
    hour: '小时',
    day: '天',
    week: '周',
    month: '月'
  }
  return labels[granularity] || granularity
}

function getStatusType(status: string): string {
  if (status === '正常') return 'success'
  if (status === '警告') return 'warning'
  if (status === '异常') return 'danger'
  return 'info'
}

/**
 * 事件处理
 */
function handleCancel() {
  handleClose()
}

function handleClose() {
  visible.value = false
  
  // 重置状态
  currentStep.value = 0
  selectedTemplate.value = null
  generating.value = false
  generationProgress.value = 0
  activePreviewTab.value = 'config'
  
  // 重置配置
  reportConfig.value = {
    name: '',
    dateRange: [],
    granularity: 'hour',
    formats: ['pdf'],
    deviceIds: [],
    tagIds: [],
    dataTypes: ['telemetry'],
    chartTypes: ['line'],
    statistics: ['avg'],
    groupBy: 'device'
  }
  
  // 销毁预览图表
  if (previewChart.value) {
    previewChart.value.dispose()
    previewChart.value = undefined
  }
}

// ===== 生命周期 =====
onMounted(() => {
  // 设置默认日期范围为最近7天
  const now = new Date()
  const sevenDaysAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
  
  reportConfig.value.dateRange = [
    sevenDaysAgo.toISOString().slice(0, 19).replace('T', ' '),
    now.toISOString().slice(0, 19).replace('T', ' ')
  ]
  
  // 设置默认选择
  reportConfig.value.deviceIds = ['1', '2']
  reportConfig.value.tagIds = ['1', '2']
})

// ===== 监听器 =====
watch(() => activePreviewTab.value, (tab) => {
  if (tab === 'chart') {
    nextTick(() => {
      if (previewChartRef.value && !previewChart.value) {
        initPreviewChart()
      }
    })
  }
})
</script>

<style scoped lang="scss">
.report-generator {
  .steps {
    margin-bottom: 32px;
  }
  
  .step-content {
    min-height: 400px;
    
    h4 {
      margin: 0 0 20px 0;
      color: #303133;
      font-size: 16px;
    }
  }
  
  // 模板选择
  .template-selection {
    .template-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 16px;
      
      .template-card {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 20px;
        border: 2px solid #ebeef5;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.3s;
        
        &:hover {
          border-color: #c6e2ff;
          background: #f0f9ff;
        }
        
        &.selected {
          border-color: #409eff;
          background: #f0f9ff;
          
          .template-icon {
            color: #409eff;
          }
        }
        
        .template-icon {
          font-size: 32px;
          color: #909399;
          flex-shrink: 0;
        }
        
        .template-info {
          flex: 1;
          
          .template-name {
            font-size: 16px;
            font-weight: 600;
            color: #303133;
            margin-bottom: 4px;
          }
          
          .template-desc {
            font-size: 14px;
            color: #606266;
            line-height: 1.4;
          }
        }
      }
    }
  }
  
  // 参数配置
  .parameter-config {
    .config-section {
      margin-bottom: 20px;
      
      :deep(.el-card__header) {
        padding: 12px 20px;
        background: #fafafa;
        
        span {
          font-weight: 600;
          color: #303133;
        }
      }
      
      :deep(.el-card__body) {
        padding: 20px;
      }
    }
  }
  
  // 预览生成
  .preview-generate {
    .preview-tabs {
      margin-bottom: 20px;
      
      .config-preview {
        :deep(.el-descriptions__label) {
          font-weight: 600;
        }
      }
      
      .data-preview {
        .preview-info {
          margin-top: 12px;
          font-size: 12px;
          color: #909399;
          text-align: center;
        }
      }
      
      .chart-preview {
        .preview-chart-container {
          height: 300px;
          border: 1px solid #ebeef5;
          border-radius: 6px;
        }
      }
    }
    
    .generation-progress {
      margin-top: 20px;
      
      .progress-text {
        font-size: 14px;
        color: #606266;
      }
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

// 响应式设计
@media (max-width: 768px) {
  .report-generator {
    .template-selection .template-grid {
      grid-template-columns: 1fr;
    }
    
    .parameter-config {
      :deep(.el-form-item__label) {
        width: 100px !important;
      }
    }
  }
}
</style>