<template>
  <div class="analytics-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-title">
        <el-icon :size="24">
          <DataAnalysis />
        </el-icon>
        <h1>数据分析</h1>
      </div>
      
      <div class="header-actions">
        <TimeRangePicker
          v-model="analysisTimeRange"
          :show-quick-ranges="true"
          size="small"
          @change="handleTimeRangeChange"
        />
        
        <el-button-group size="small">
          <el-button
            :type="viewMode === 'dashboard' ? 'primary' : 'default'"
            :icon="DataBoard"
            @click="viewMode = 'dashboard'"
          >
            仪表板
          </el-button>
          <el-button
            :type="viewMode === 'reports' ? 'primary' : 'default'"
            :icon="Document"
            @click="viewMode = 'reports'"
          >
            报表
          </el-button>
          <el-button
            :type="viewMode === 'custom' ? 'primary' : 'default'"
            :icon="Setting"
            @click="viewMode = 'custom'"
          >
            自定义
          </el-button>
        </el-button-group>
        
        <el-dropdown trigger="click" @command="handleAction">
          <el-button :icon="More" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="export" :icon="Download">
                导出数据
              </el-dropdown-item>
              <el-dropdown-item command="schedule" :icon="Timer">
                定时报表
              </el-dropdown-item>
              <el-dropdown-item command="share" :icon="Share">
                分享报告
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
    
    <!-- 仪表板视图 -->
    <div v-if="viewMode === 'dashboard'" class="dashboard-view">
      <!-- KPI概览 -->
      <div class="kpi-overview">
        <el-row :gutter="16">
          <el-col :span="6" v-for="kpi in kpiMetrics" :key="kpi.id">
            <el-card shadow="never" class="kpi-card">
              <div class="kpi-content">
                <div class="kpi-header">
                  <span class="kpi-title">{{ kpi.title }}</span>
                  <el-icon class="kpi-icon" :class="kpi.iconClass">
                    <component :is="kpi.icon" />
                  </el-icon>
                </div>
                
                <div class="kpi-value">
                  <span class="current-value">{{ formatKpiValue(kpi.value, kpi.unit) }}</span>
                  <div class="trend-info" :class="getTrendClass(kpi.trend)">
                    <el-icon>
                      <component :is="getTrendIcon(kpi.trend)" />
                    </el-icon>
                    <span>{{ kpi.trendText }}</span>
                  </div>
                </div>
                
                <div class="kpi-footer">
                  <span class="comparison">较上期{{ kpi.comparison }}</span>
                </div>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </div>
      
      <!-- 分析图表 -->
      <div class="analysis-charts">
        <el-row :gutter="16">
          <!-- 数据趋势分析 -->
          <el-col :span="12">
            <el-card shadow="never" class="chart-card">
              <template #header>
                <div class="card-header">
                  <span>数据趋势分析</span>
                  <div class="chart-controls">
                    <el-select
                      v-model="selectedTrendDatapoints"
                      multiple
                      size="small"
                      placeholder="选择数据点"
                      style="width: 200px"
                      @change="handleTrendDatapointsChange"
                    >
                      <el-option
                        v-for="dp in availableDatapoints"
                        :key="dp.id"
                        :label="dp.name"
                        :value="dp.id"
                      />
                    </el-select>
                  </div>
                </div>
              </template>
              
              <ChartContainer
                title="数据趋势"
                chart-type="line"
                :data="getTrendChartData()"
                :height="'300px'"
                :support-zoom="true"
              />
            </el-card>
          </el-col>
          
          <!-- 数据分布分析 -->
          <el-col :span="12">
            <el-card shadow="never" class="chart-card">
              <template #header>
                <div class="card-header">
                  <span>数据分布分析</span>
                  <div class="chart-controls">
                    <el-select
                      v-model="selectedDistributionDatapoint"
                      size="small"
                      placeholder="选择数据点"
                      style="width: 200px"
                      @change="handleDistributionDatapointChange"
                    >
                      <el-option
                        v-for="dp in availableDatapoints"
                        :key="dp.id"
                        :label="dp.name"
                        :value="dp.id"
                      />
                    </el-select>
                  </div>
                </div>
              </template>
              
              <ChartContainer
                title="数据分布"
                chart-type="histogram"
                :data="getDistributionChartData()"
                :height="'300px'"
              />
            </el-card>
          </el-col>
        </el-row>
        
        <el-row :gutter="16" style="margin-top: 16px">
          <!-- 设备性能分析 -->
          <el-col :span="8">
            <el-card shadow="never" class="chart-card">
              <template #header>
                <div class="card-header">
                  <span>设备性能分析</span>
                </div>
              </template>
              
              <ChartContainer
                title="设备效率"
                chart-type="radar"
                :data="getDevicePerformanceData()"
                :height="'280px'"
              />
            </el-card>
          </el-col>
          
          <!-- 报警统计 -->
          <el-col :span="8">
            <el-card shadow="never" class="chart-card">
              <template #header>
                <div class="card-header">
                  <span>报警统计</span>
                </div>
              </template>
              
              <ChartContainer
                title="报警分布"
                chart-type="doughnut"
                :data="getAlarmStatsData()"
                :height="'280px'"
              />
            </el-card>
          </el-col>
          
          <!-- 数据质量监控 -->
          <el-col :span="8">
            <el-card shadow="never" class="chart-card">
              <template #header>
                <div class="card-header">
                  <span>数据质量监控</span>
                </div>
              </template>
              
              <div class="data-quality-metrics">
                <div 
                  v-for="quality in dataQualityMetrics"
                  :key="quality.id"
                  class="quality-item"
                >
                  <div class="quality-header">
                    <span class="quality-name">{{ quality.name }}</span>
                    <StatusTag :status="quality.status" size="small" />
                  </div>
                  
                  <div class="quality-progress">
                    <el-progress
                      :percentage="quality.percentage"
                      :color="getQualityColor(quality.percentage)"
                      :show-text="false"
                    />
                    <span class="quality-value">{{ quality.percentage }}%</span>
                  </div>
                </div>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </div>
      
      <!-- 预测分析 -->
      <div class="prediction-analysis">
        <el-card shadow="never">
          <template #header>
            <div class="card-header">
              <span>预测分析</span>
              <div class="chart-controls">
                <el-select
                  v-model="selectedPredictionDatapoint"
                  size="small"
                  placeholder="选择数据点"
                  style="width: 200px"
                  @change="handlePredictionDatapointChange"
                >
                  <el-option
                    v-for="dp in numericDatapoints"
                    :key="dp.id"
                    :label="dp.name"
                    :value="dp.id"
                  />
                </el-select>
                
                <el-button size="small" type="primary" @click="handleRunPrediction">
                  运行预测
                </el-button>
              </div>
            </div>
          </template>
          
          <div class="prediction-content">
            <el-row :gutter="16">
              <el-col :span="18">
                <ChartContainer
                  title="趋势预测"
                  chart-type="line"
                  :data="getPredictionChartData()"
                  :height="'350px'"
                  :support-zoom="true"
                />
              </el-col>
              
              <el-col :span="6">
                <div class="prediction-summary">
                  <h4>预测摘要</h4>
                  
                  <div class="prediction-metrics">
                    <div class="prediction-item">
                      <span class="metric-label">预测准确度</span>
                      <el-progress
                        :percentage="predictionAccuracy"
                        :color="getAccuracyColor(predictionAccuracy)"
                      />
                    </div>
                    
                    <div class="prediction-item">
                      <span class="metric-label">置信区间</span>
                      <div class="confidence-range">
                        {{ predictionConfidenceRange.min.toFixed(2) }} ~ {{ predictionConfidenceRange.max.toFixed(2) }}
                      </div>
                    </div>
                    
                    <div class="prediction-item">
                      <span class="metric-label">预测模型</span>
                      <el-tag size="small">{{ predictionModel }}</el-tag>
                    </div>
                    
                    <div class="prediction-item">
                      <span class="metric-label">异常风险</span>
                      <el-tag 
                        :type="getRiskType(anomalyRisk)"
                        size="small"
                      >
                        {{ getRiskText(anomalyRisk) }}
                      </el-tag>
                    </div>
                  </div>
                </div>
              </el-col>
            </el-row>
          </div>
        </el-card>
      </div>
    </div>
    
    <!-- 报表视图 -->
    <div v-else-if="viewMode === 'reports'" class="reports-view">
      <div class="reports-toolbar">
        <div class="toolbar-left">
          <el-button type="primary" :icon="Plus" @click="handleCreateReport">
            新建报表
          </el-button>
          
          <el-button :icon="Refresh" @click="handleRefreshReports">
            刷新
          </el-button>
        </div>
        
        <div class="toolbar-right">
          <SearchBox
            v-model="reportSearchText"
            placeholder="搜索报表..."
            size="small"
            style="width: 300px"
            @search="handleSearchReports"
          />
        </div>
      </div>
      
      <div class="reports-grid">
        <el-row :gutter="16">
          <el-col :span="8" v-for="report in filteredReports" :key="report.id">
            <el-card shadow="hover" class="report-card" @click="handleViewReport(report)">
              <div class="report-content">
                <div class="report-header">
                  <el-icon class="report-icon" :class="getReportTypeClass(report.type)">
                    <component :is="getReportTypeIcon(report.type)" />
                  </el-icon>
                  
                  <div class="report-info">
                    <h4 class="report-title">{{ report.title }}</h4>
                    <p class="report-description">{{ report.description }}</p>
                  </div>
                  
                  <el-dropdown trigger="click" @command="(cmd) => handleReportAction(cmd, report)">
                    <el-button :icon="More" size="small" text />
                    <template #dropdown>
                      <el-dropdown-menu>
                        <el-dropdown-item command="edit" :icon="Edit">
                          编辑
                        </el-dropdown-item>
                        <el-dropdown-item command="duplicate" :icon="DocumentCopy">
                          复制
                        </el-dropdown-item>
                        <el-dropdown-item command="export" :icon="Download">
                          导出
                        </el-dropdown-item>
                        <el-dropdown-item command="delete" :icon="Delete" divided>
                          删除
                        </el-dropdown-item>
                      </el-dropdown-menu>
                    </template>
                  </el-dropdown>
                </div>
                
                <div class="report-preview">
                  <div class="preview-chart">
                    <ChartContainer
                      :title="report.title"
                      :chart-type="report.chartType"
                      :data="getReportPreviewData(report)"
                      :height="'120px'"
                      :show-legend="false"
                      :show-toolbar="false"
                    />
                  </div>
                </div>
                
                <div class="report-footer">
                  <div class="report-meta">
                    <StatusTag :status="report.status" size="small" />
                    <span class="last-run">{{ formatTime(report.lastRunTime) }}</span>
                  </div>
                  
                  <div class="report-actions">
                    <el-button size="small" text @click.stop="handleRunReport(report)">
                      运行
                    </el-button>
                  </div>
                </div>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </div>
    </div>
    
    <!-- 自定义报表设计器 -->
    <div v-else-if="viewMode === 'custom'" class="custom-view">
      <div class="designer-layout">
        <div class="designer-sidebar">
          <div class="sidebar-section">
            <h4>数据源</h4>
            <div class="data-source-list">
              <div 
                v-for="source in dataSources"
                :key="source.id"
                class="data-source-item"
                draggable="true"
                @dragstart="handleDragStart($event, source)"
              >
                <el-icon><Monitor /></el-icon>
                <span>{{ source.name }}</span>
              </div>
            </div>
          </div>
          
          <div class="sidebar-section">
            <h4>图表类型</h4>
            <div class="chart-types-list">
              <div 
                v-for="chartType in chartTypes"
                :key="chartType.type"
                class="chart-type-item"
                @click="handleSelectChartType(chartType)"
              >
                <el-icon><component :is="chartType.icon" /></el-icon>
                <span>{{ chartType.name }}</span>
              </div>
            </div>
          </div>
          
          <div class="sidebar-section">
            <h4>组件库</h4>
            <div class="components-list">
              <div 
                v-for="component in reportComponents"
                :key="component.type"
                class="component-item"
                draggable="true"
                @dragstart="handleDragStart($event, component)"
              >
                <el-icon><component :is="component.icon" /></el-icon>
                <span>{{ component.name }}</span>
              </div>
            </div>
          </div>
        </div>
        
        <div class="designer-canvas">
          <div class="canvas-toolbar">
            <div class="toolbar-left">
              <el-button size="small" :icon="FolderOpened" @click="handleLoadTemplate">
                模板
              </el-button>
              
              <el-button size="small" :icon="DocumentAdd" @click="handleSaveReport">
                保存
              </el-button>
              
              <el-button size="small" :icon="View" @click="handlePreviewReport">
                预览
              </el-button>
            </div>
            
            <div class="toolbar-right">
              <el-button-group size="small">
                <el-button 
                  :type="canvasMode === 'design' ? 'primary' : 'default'"
                  @click="canvasMode = 'design'"
                >
                  设计
                </el-button>
                <el-button 
                  :type="canvasMode === 'preview' ? 'primary' : 'default'"
                  @click="canvasMode = 'preview'"
                >
                  预览
                </el-button>
              </el-button-group>
            </div>
          </div>
          
          <div 
            class="canvas-area"
            @drop="handleCanvasDrop"
            @dragover.prevent
          >
            <div v-if="reportElements.length === 0" class="canvas-placeholder">
              <el-empty description="拖拽组件到此处开始设计报表" />
            </div>
            
            <div 
              v-for="element in reportElements"
              :key="element.id"
              class="report-element"
              :class="{ active: selectedElementId === element.id }"
              :style="getElementStyle(element)"
              @click="handleSelectElement(element)"
            >
              <component
                :is="getElementComponent(element.type)"
                :element="element"
                :preview-mode="canvasMode === 'preview'"
              />
              
              <div v-if="canvasMode === 'design'" class="element-controls">
                <el-button 
                  size="small" 
                  :icon="Setting" 
                  @click="handleConfigElement(element)"
                />
                <el-button 
                  size="small" 
                  :icon="Delete" 
                  @click="handleDeleteElement(element)"
                />
              </div>
            </div>
          </div>
        </div>
        
        <div class="designer-properties">
          <div class="properties-panel">
            <h4>属性设置</h4>
            
            <div v-if="selectedElement" class="element-properties">
              <div class="property-group">
                <label>标题</label>
                <el-input 
                  v-model="selectedElement.title"
                  size="small"
                  placeholder="输入标题"
                />
              </div>
              
              <div class="property-group">
                <label>位置</label>
                <el-row :gutter="8">
                  <el-col :span="12">
                    <el-input-number 
                      v-model="selectedElement.position.x"
                      size="small"
                      placeholder="X"
                    />
                  </el-col>
                  <el-col :span="12">
                    <el-input-number 
                      v-model="selectedElement.position.y"
                      size="small"
                      placeholder="Y"
                    />
                  </el-col>
                </el-row>
              </div>
              
              <div class="property-group">
                <label>尺寸</label>
                <el-row :gutter="8">
                  <el-col :span="12">
                    <el-input-number 
                      v-model="selectedElement.size.width"
                      size="small"
                      placeholder="宽度"
                    />
                  </el-col>
                  <el-col :span="12">
                    <el-input-number 
                      v-model="selectedElement.size.height"
                      size="small"
                      placeholder="高度"
                    />
                  </el-col>
                </el-row>
              </div>
              
              <div v-if="selectedElement.type === 'chart'" class="property-group">
                <label>图表类型</label>
                <el-select 
                  v-model="selectedElement.chartType"
                  size="small"
                  style="width: 100%"
                >
                  <el-option
                    v-for="type in chartTypes"
                    :key="type.type"
                    :label="type.name"
                    :value="type.type"
                  />
                </el-select>
              </div>
              
              <div v-if="selectedElement.type === 'chart'" class="property-group">
                <label>数据源</label>
                <el-select 
                  v-model="selectedElement.dataSource"
                  size="small"
                  style="width: 100%"
                >
                  <el-option
                    v-for="source in dataSources"
                    :key="source.id"
                    :label="source.name"
                    :value="source.id"
                  />
                </el-select>
              </div>
            </div>
            
            <div v-else class="no-selection">
              <el-empty description="请选择一个元素" />
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 报表预览对话框 -->
    <el-dialog
      v-model="reportPreviewVisible"
      :title="`预览报表 - ${previewingReport?.title}`"
      width="80%"
      :before-close="handleClosePreview"
    >
      <div class="report-preview-content">
        <div v-if="previewingReport" class="preview-header">
          <h2>{{ previewingReport.title }}</h2>
          <p>{{ previewingReport.description }}</p>
          <div class="preview-meta">
            <span>生成时间: {{ formatTime(new Date()) }}</span>
          </div>
        </div>
        
        <div class="preview-body">
          <ChartContainer
            v-if="previewingReport"
            :title="previewingReport.title"
            :chart-type="previewingReport.chartType"
            :data="getReportPreviewData(previewingReport)"
            :height="'400px'"
          />
        </div>
      </div>
      
      <template #footer>
        <el-button @click="reportPreviewVisible = false">关闭</el-button>
        <el-button type="primary" @click="handleExportReport">
          导出PDF
        </el-button>
      </template>
    </el-dialog>
    
    <!-- 报表编辑对话框 -->
    <el-dialog
      v-model="reportEditVisible"
      :title="editingReport?.id ? '编辑报表' : '新建报表'"
      width="600px"
      :before-close="handleCloseEdit"
    >
      <BaseForm
        v-if="editingReport"
        v-model="editingReport"
        :fields="reportFormFields"
        :rules="reportFormRules"
        label-width="100px"
      />
      
      <template #footer>
        <el-button @click="reportEditVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSaveReport">
          {{ editingReport?.id ? '更新' : '创建' }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  DataAnalysis,
  DataBoard,
  Document,
  Setting,
  More,
  Download,
  Timer,
  Share,
  TrendCharts,
  Top,
  Bottom,
  Plus,
  Refresh,
  Edit,
  DocumentCopy,
  Delete,
  FolderOpened,
  DocumentAdd,
  View
} from '@element-plus/icons-vue'

// 导入组件
import { BaseForm, SearchBox, StatusTag } from '../../components/base'
import { 
  TimeRangePicker, 
  ChartContainer
} from '../../components/business'

// 导入API
import { analyticsApi } from '@/api/analytics'

// 类型定义
interface KpiMetric {
  id: string
  title: string
  value: number
  unit: string
  trend: 'up' | 'down' | 'stable'
  trendText: string
  comparison: string
  icon: any
  iconClass: string
}

interface DataPoint {
  id: string
  name: string
  dataType: string
  currentValue: any
}

interface Report {
  id: string
  title: string
  description: string
  type: 'trend' | 'distribution' | 'performance' | 'custom'
  chartType: string
  status: 'active' | 'inactive' | 'error'
  lastRunTime: Date
  dataSource?: string
}

interface ReportElement {
  id: string
  type: 'chart' | 'table' | 'text' | 'kpi'
  title: string
  position: { x: number; y: number }
  size: { width: number; height: number }
  chartType?: string
  dataSource?: string
  config?: any
}

interface DataSource {
  id: string
  name: string
  type: 'datapoints' | 'drivers' | 'alarms'
}

// 状态管理
const viewMode = ref<'dashboard' | 'reports' | 'custom'>('dashboard')
const canvasMode = ref<'design' | 'preview'>('design')
const analysisTimeRange = ref({
  startTime: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
  endTime: new Date()
})

// 数据点相关
const selectedTrendDatapoints = ref<string[]>([])
const selectedDistributionDatapoint = ref<string>('')
const selectedPredictionDatapoint = ref<string>('')

// 对话框状态
const reportPreviewVisible = ref(false)
const reportEditVisible = ref(false)
const previewingReport = ref<Report | null>(null)
const editingReport = ref<Report | null>(null)

// 报表相关
const reportSearchText = ref('')
const selectedElementId = ref<string>('')
const reportElements = ref<ReportElement[]>([])

// 预测分析
const predictionAccuracy = ref(85)
const predictionModel = ref('ARIMA')
const predictionConfidenceRange = ref({ min: 65.2, max: 89.7 })
const anomalyRisk = ref<'low' | 'medium' | 'high'>('low')

// KPI数据从API获取
const kpiMetrics = ref<KpiMetric[]>([])

// 数据点从API获取
const availableDatapoints = ref<DataPoint[]>([])

// 数据质量指标从API获取
const dataQualityMetrics = ref<any[]>([])

// 报表列表从API获取
const reports = ref<Report[]>([])

// 数据源从API获取
const dataSources = ref<DataSource[]>([])

const chartTypes = ref([
  { type: 'line', name: '折线图', icon: TrendCharts },
  { type: 'bar', name: '柱状图', icon: DataBoard },
  { type: 'pie', name: '饼图', icon: DataBoard },
  { type: 'scatter', name: '散点图', icon: DataBoard },
  { type: 'radar', name: '雷达图', icon: DataBoard },
  { type: 'gauge', name: '仪表盘', icon: DataBoard }
])

const reportComponents = ref([
  { type: 'chart', name: '图表组件', icon: TrendCharts },
  { type: 'table', name: '表格组件', icon: Document },
  { type: 'text', name: '文本组件', icon: Edit },
  { type: 'kpi', name: 'KPI组件', icon: DataBoard }
])

// 计算属性
const numericDatapoints = computed(() => {
  return availableDatapoints.value.filter(dp => 
    dp.dataType === 'float' || dp.dataType === 'integer'
  )
})

const filteredReports = computed(() => {
  if (!reportSearchText.value) {
    return reports.value
  }
  
  return reports.value.filter(report =>
    report.title.toLowerCase().includes(reportSearchText.value.toLowerCase()) ||
    report.description.toLowerCase().includes(reportSearchText.value.toLowerCase())
  )
})

const selectedElement = computed(() => {
  return reportElements.value.find(el => el.id === selectedElementId.value)
})

// 报表表单字段
const reportFormFields = computed(() => [
  {
    name: 'title',
    label: '报表标题',
    type: 'input',
    required: true,
    placeholder: '请输入报表标题'
  },
  {
    name: 'description', 
    label: '报表描述',
    type: 'textarea',
    placeholder: '请输入报表描述'
  },
  {
    name: 'type',
    label: '报表类型',
    type: 'select',
    required: true,
    options: [
      { label: '趋势分析', value: 'trend' },
      { label: '分布分析', value: 'distribution' },
      { label: '性能分析', value: 'performance' },
      { label: '自定义', value: 'custom' }
    ]
  },
  {
    name: 'chartType',
    label: '图表类型',
    type: 'select',
    required: true,
    options: chartTypes.value.map(ct => ({ label: ct.name, value: ct.type }))
  }
])

const reportFormRules = {
  title: [{ required: true, message: '请输入报表标题', trigger: 'blur' }],
  type: [{ required: true, message: '请选择报表类型', trigger: 'change' }],
  chartType: [{ required: true, message: '请选择图表类型', trigger: 'change' }]
}

// API调用方法
const loadKpiMetrics = async () => {
  try {
    const response = await analyticsApi.getKpiMetrics()
    kpiMetrics.value = response.data || []
  } catch (error) {
    console.error('获取KPI指标失败:', error)
    kpiMetrics.value = []
  }
}

const loadDatapoints = async () => {
  try {
    const response = await analyticsApi.getDatapoints()
    availableDatapoints.value = response.data || []
  } catch (error) {
    console.error('获取数据点失败:', error)
    availableDatapoints.value = []
  }
}

const loadDataQualityMetrics = async () => {
  try {
    const response = await analyticsApi.getDataQualityMetrics()
    dataQualityMetrics.value = response.data || []
  } catch (error) {
    console.error('获取数据质量指标失败:', error)
    dataQualityMetrics.value = []
  }
}

const loadReports = async () => {
  try {
    const response = await analyticsApi.getReports()
    reports.value = response.data || []
  } catch (error) {
    console.error('获取报表列表失败:', error)
    reports.value = []
  }
}

const loadDataSources = async () => {
  try {
    const response = await analyticsApi.getDataSources()
    dataSources.value = response.data || []
  } catch (error) {
    console.error('获取数据源失败:', error)
    dataSources.value = []
  }
}

// 方法
const formatKpiValue = (value: number, unit: string) => {
  return `${value}${unit}`
}

const formatTime = (time: Date) => {
  return time.toLocaleString('zh-CN')
}

const getTrendClass = (trend: string) => {
  return {
    'trend-up': trend === 'up',
    'trend-down': trend === 'down',
    'trend-stable': trend === 'stable'
  }
}

const getTrendIcon = (trend: string) => {
  switch (trend) {
    case 'up': return Top
    case 'down': return Bottom
    case 'stable':
    default: return TrendCharts
  }
}

const getQualityColor = (percentage: number) => {
  if (percentage >= 90) return '#67c23a'
  if (percentage >= 80) return '#e6a23c'
  return '#f56c6c'
}

const getAccuracyColor = (accuracy: number) => {
  if (accuracy >= 90) return '#67c23a'
  if (accuracy >= 80) return '#409eff'
  return '#e6a23c'
}

const getRiskType = (risk: string) => {
  switch (risk) {
    case 'low': return 'success'
    case 'medium': return 'warning'
    case 'high': return 'danger'
    default: return 'info'
  }
}

const getRiskText = (risk: string) => {
  switch (risk) {
    case 'low': return '低风险'
    case 'medium': return '中等风险'
    case 'high': return '高风险'
    default: return '未知风险'
  }
}

const getReportTypeClass = (type: string) => {
  return `report-icon--${type}`
}

const getReportTypeIcon = (type: string) => {
  switch (type) {
    case 'trend': return TrendCharts
    case 'distribution': return DataBoard
    case 'performance': return Monitor
    case 'custom': return Setting
    default: return Document
  }
}

// 图表数据从API获取，不再生成模拟数据
const getTrendChartData = () => {
  // 直接返回空数据，实际数据应从API获取
  return { series: [] }
}

const getDistributionChartData = () => {
  // 直接返回空数据，实际数据应从API获取
  return { series: [] }
}

const getDevicePerformanceData = () => {
  // 从API获取真实设备性能数据
  return { series: [] }
}

const getAlarmStatsData = () => {
  // 从API获取真实报警统计数据
  return { series: [] }
}

const getPredictionChartData = () => {
  // 从API获取真实的历史数据和预测结果
  return { series: [] }
}

const getReportPreviewData = (report: Report) => {
  // 根据报表类型生成预览数据
  switch (report.type) {
    case 'trend':
      return getTrendChartData()
    case 'distribution':
      return getDistributionChartData()
    case 'performance':
      return getDevicePerformanceData()
    default:
      return { series: [] }
  }
}

const getElementStyle = (element: ReportElement) => {
  return {
    position: 'absolute',
    left: `${element.position.x}px`,
    top: `${element.position.y}px`,
    width: `${element.size.width}px`,
    height: `${element.size.height}px`
  }
}

const getElementComponent = (type: string) => {
  // 根据元素类型返回对应的组件
  switch (type) {
    case 'chart': return ChartContainer
    case 'table': return 'div' // 表格组件
    case 'text': return 'div'  // 文本组件
    case 'kpi': return 'div'   // KPI组件
    default: return 'div'
  }
}

// 事件处理
const handleTimeRangeChange = (range: any) => {
  analysisTimeRange.value = range
  ElMessage.info('时间范围已更新')
}

const handleAction = (command: string) => {
  switch (command) {
    case 'export':
      handleExportData()
      break
    case 'schedule':
      ElMessage.info('定时报表功能开发中')
      break
    case 'share':
      ElMessage.info('分享功能开发中')
      break
  }
}

const handleExportData = () => {
  const exportData = {
    timeRange: analysisTimeRange.value,
    kpiMetrics: kpiMetrics.value,
    exportTime: new Date().toISOString()
  }
  
  const blob = new Blob([JSON.stringify(exportData, null, 2)], {
    type: 'application/json'
  })
  
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `analytics_data_${new Date().toISOString().split('T')[0]}.json`
  a.click()
  
  URL.revokeObjectURL(url)
  ElMessage.success('数据导出成功')
}

const handleTrendDatapointsChange = () => {
  ElMessage.info(`已选择 ${selectedTrendDatapoints.value.length} 个数据点`)
}

const handleDistributionDatapointChange = () => {
  const datapoint = availableDatapoints.value.find(dp => dp.id === selectedDistributionDatapoint.value)
  ElMessage.info(`切换到数据点: ${datapoint?.name}`)
}

const handlePredictionDatapointChange = () => {
  const datapoint = availableDatapoints.value.find(dp => dp.id === selectedPredictionDatapoint.value)
  ElMessage.info(`选择预测数据点: ${datapoint?.name}`)
}

const handleRunPrediction = async () => {
  if (!selectedPredictionDatapoint.value) {
    ElMessage.warning('请先选择数据点')
    return
  }
  
  ElMessage.info('正在运行预测分析...')
  
  try {
    // 调用真实的预测分析API
    const result = await analyticsApi.runPrediction({
      datapointId: selectedPredictionDatapoint.value,
      timeRange: analysisTimeRange.value
    })
    
    predictionAccuracy.value = result.accuracy
    predictionConfidenceRange.value = result.confidenceRange
    anomalyRisk.value = result.anomalyRisk
    
    ElMessage.success('预测分析完成')
  } catch (error) {
    console.error('预测分析失败:', error)
    ElMessage.error('预测分析失败')
  }
}

// 报表相关事件
const handleCreateReport = () => {
  editingReport.value = {
    id: '',
    title: '',
    description: '',
    type: 'trend',
    chartType: 'line',
    status: 'active',
    lastRunTime: new Date()
  }
  reportEditVisible.value = true
}

const handleRefreshReports = async () => {
  try {
    await loadReports()
    ElMessage.success('报表列表已刷新')
  } catch (error) {
    console.error('刷新报表列表失败:', error)
    ElMessage.error('刷新失败')
  }
}

const handleSearchReports = () => {
  ElMessage.info(`搜索: ${reportSearchText.value}`)
}

const handleViewReport = (report: Report) => {
  previewingReport.value = report
  reportPreviewVisible.value = true
}

const handleReportAction = async (command: string, report: Report) => {
  try {
    switch (command) {
      case 'edit':
        editingReport.value = { ...report }
        reportEditVisible.value = true
        break
      case 'duplicate':
        await analyticsApi.duplicateReport(report.id)
        ElMessage.success('报表已复制')
        await loadReports()
        break
      case 'export':
        await analyticsApi.exportReport(report.id)
        ElMessage.success(`报表 ${report.title} 已导出`)
        break
      case 'delete':
        await ElMessageBox.confirm(
          `确定要删除报表 "${report.title}" 吗？`,
          '确认删除',
          {
            confirmButtonText: '删除',
            cancelButtonText: '取消',
            type: 'warning'
          }
        )
        
        await analyticsApi.deleteReport(report.id)
        ElMessage.success('报表已删除')
        await loadReports()
        break
    }
  } catch (error) {
    if (error !== 'cancel') {
      console.error('报表操作失败:', error)
      ElMessage.error('操作失败')
    }
  }
}

const handleRunReport = async (report: Report) => {
  try {
    ElMessage.info(`正在运行报表: ${report.title}`)
    await analyticsApi.runReport(report.id)
    report.lastRunTime = new Date()
    ElMessage.success(`报表 ${report.title} 运行完成`)
  } catch (error) {
    console.error('运行报表失败:', error)
    ElMessage.error('运行报表失败')
  }
}

const handleSaveReport = async () => {
  if (!editingReport.value) return
  
  try {
    if (!editingReport.value.id) {
      // 新建报表
      await analyticsApi.createReport(editingReport.value)
      ElMessage.success('报表创建成功')
    } else {
      // 更新报表
      await analyticsApi.updateReport(editingReport.value.id, editingReport.value)
      ElMessage.success('报表更新成功')
    }
    
    reportEditVisible.value = false
    await loadReports()
  } catch (error) {
    console.error('保存报表失败:', error)
    ElMessage.error('保存报表失败')
  }
}

const handleCloseEdit = () => {
  reportEditVisible.value = false
  editingReport.value = null
}

const handleClosePreview = () => {
  reportPreviewVisible.value = false
  previewingReport.value = null
}

const handleExportReport = () => {
  if (!previewingReport.value) return
  
  ElMessage.info('导出PDF功能开发中')
}

// 自定义报表设计器事件
const handleLoadTemplate = () => {
  ElMessage.info('模板功能开发中')
}

const handlePreviewReport = () => {
  canvasMode.value = canvasMode.value === 'preview' ? 'design' : 'preview'
}

const handleSelectChartType = (chartType: any) => {
  ElMessage.info(`选择图表类型: ${chartType.name}`)
}

const handleDragStart = (event: DragEvent, item: any) => {
  if (event.dataTransfer) {
    event.dataTransfer.setData('text/plain', JSON.stringify(item))
  }
}

const handleCanvasDrop = (event: DragEvent) => {
  event.preventDefault()
  
  if (!event.dataTransfer) return
  
  const data = JSON.parse(event.dataTransfer.getData('text/plain'))
  const rect = (event.target as HTMLElement).getBoundingClientRect()
  
  const newElement: ReportElement = {
    id: Date.now().toString(),
    type: data.type || 'chart',
    title: data.name || '新组件',
    position: {
      x: event.clientX - rect.left,
      y: event.clientY - rect.top
    },
    size: {
      width: 300,
      height: 200
    },
    chartType: 'line',
    dataSource: data.id
  }
  
  reportElements.value.push(newElement)
  selectedElementId.value = newElement.id
  
  ElMessage.success('组件已添加')
}

const handleSelectElement = (element: ReportElement) => {
  selectedElementId.value = element.id
}

const handleConfigElement = (element: ReportElement) => {
  ElMessage.info(`配置元素: ${element.title}`)
}

const handleDeleteElement = (element: ReportElement) => {
  const index = reportElements.value.findIndex(el => el.id === element.id)
  if (index > -1) {
    reportElements.value.splice(index, 1)
    if (selectedElementId.value === element.id) {
      selectedElementId.value = ''
    }
    ElMessage.success('元素已删除')
  }
}

// 生命周期
onMounted(async () => {
  // 加载所有数据
  await Promise.all([
    loadKpiMetrics(),
    loadDatapoints(),
    loadDataQualityMetrics(),
    loadReports(),
    loadDataSources()
  ])
  
  // 初始化默认选择
  if (availableDatapoints.value.length > 0) {
    selectedTrendDatapoints.value = [availableDatapoints.value[0].id]
    selectedDistributionDatapoint.value = availableDatapoints.value[0].id
    selectedPredictionDatapoint.value = numericDatapoints.value[0]?.id || ''
  }
})
</script>

<style scoped lang="scss">
.analytics-page {
  padding: 16px;
  background: var(--el-bg-color-page);
  min-height: 100vh;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  
  .header-title {
    display: flex;
    align-items: center;
    gap: 12px;
    
    h1 {
      margin: 0;
      font-size: 24px;
      font-weight: 600;
      color: var(--el-text-color-primary);
    }
  }
  
  .header-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }
}

// 仪表板视图
.dashboard-view {
  .kpi-overview {
    margin-bottom: 20px;
    
    .kpi-card {
      position: relative;
      cursor: pointer;
      transition: all 0.3s;
      border: 1px solid var(--el-border-color-light);
      
      &:hover {
        transform: translateY(-2px);
        box-shadow: var(--el-box-shadow);
      }
      
      .kpi-content {
        display: flex;
        justify-content: space-between;
        align-items: center;
        
        .kpi-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          width: 100%;
          
          .kpi-title {
            font-size: 14px;
            font-weight: 500;
            color: var(--el-text-color-regular);
          }
          
          .kpi-icon {
            font-size: 28px;
            opacity: 0.8;
            
            &.kpi-icon--success {
              color: var(--el-color-success);
            }
            
            &.kpi-icon--primary {
              color: var(--el-color-primary);
            }
            
            &.kpi-icon--warning {
              color: var(--el-color-warning);
            }
            
            &.kpi-icon--info {
              color: var(--el-color-info);
            }
          }
        }
        
        .kpi-value {
          display: flex;
          justify-content: space-between;
          align-items: center;
          width: 100%;
          margin: 12px 0 8px;
          
          .current-value {
            font-size: 24px;
            font-weight: bold;
            color: var(--el-text-color-primary);
          }
          
          .trend-info {
            display: flex;
            align-items: center;
            gap: 4px;
            font-size: 12px;
            
            &.trend-up {
              color: var(--el-color-success);
            }
            
            &.trend-down {
              color: var(--el-color-danger);
            }
            
            &.trend-stable {
              color: var(--el-text-color-secondary);
            }
          }
        }
        
        .kpi-footer {
          width: 100%;
          
          .comparison {
            font-size: 12px;
            color: var(--el-text-color-secondary);
          }
        }
      }
    }
  }
  
  .analysis-charts {
    margin-bottom: 20px;
    
    .chart-card {
      .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        
        .chart-controls {
          display: flex;
          gap: 8px;
          align-items: center;
        }
      }
      
      .data-quality-metrics {
        padding: 16px 0;
        
        .quality-item {
          margin-bottom: 16px;
          
          &:last-child {
            margin-bottom: 0;
          }
          
          .quality-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 8px;
            
            .quality-name {
              font-size: 13px;
              color: var(--el-text-color-primary);
            }
          }
          
          .quality-progress {
            display: flex;
            align-items: center;
            gap: 8px;
            
            :deep(.el-progress) {
              flex: 1;
            }
            
            .quality-value {
              font-size: 12px;
              font-weight: 500;
              color: var(--el-text-color-primary);
              min-width: 40px;
            }
          }
        }
      }
    }
  }
  
  .prediction-analysis {
    .prediction-content {
      .prediction-summary {
        padding: 16px;
        background: var(--el-fill-color-light);
        border-radius: 8px;
        
        h4 {
          margin: 0 0 16px 0;
          font-size: 16px;
          color: var(--el-text-color-primary);
        }
        
        .prediction-metrics {
          .prediction-item {
            margin-bottom: 16px;
            
            &:last-child {
              margin-bottom: 0;
            }
            
            .metric-label {
              display: block;
              font-size: 12px;
              color: var(--el-text-color-regular);
              margin-bottom: 8px;
            }
            
            .confidence-range {
              font-size: 14px;
              font-weight: 500;
              color: var(--el-text-color-primary);
              padding: 4px 8px;
              background: var(--el-color-primary-light-9);
              border-radius: 4px;
              text-align: center;
            }
          }
        }
      }
    }
  }
}

// 报表视图
.reports-view {
  .reports-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    
    .toolbar-left {
      display: flex;
      gap: 12px;
    }
  }
  
  .reports-grid {
    .report-card {
      cursor: pointer;
      transition: all 0.3s;
      height: 380px;
      
      &:hover {
        transform: translateY(-2px);
        box-shadow: var(--el-box-shadow-light);
      }
      
      .report-content {
        height: 100%;
        display: flex;
        flex-direction: column;
        
        .report-header {
          display: flex;
          align-items: flex-start;
          gap: 12px;
          margin-bottom: 16px;
          
          .report-icon {
            font-size: 24px;
            margin-top: 4px;
            
            &.report-icon--trend {
              color: var(--el-color-primary);
            }
            
            &.report-icon--distribution {
              color: var(--el-color-success);
            }
            
            &.report-icon--performance {
              color: var(--el-color-warning);
            }
            
            &.report-icon--custom {
              color: var(--el-color-info);
            }
          }
          
          .report-info {
            flex: 1;
            
            .report-title {
              margin: 0 0 4px 0;
              font-size: 16px;
              font-weight: 600;
              color: var(--el-text-color-primary);
            }
            
            .report-description {
              margin: 0;
              font-size: 13px;
              color: var(--el-text-color-regular);
              line-height: 1.4;
            }
          }
        }
        
        .report-preview {
          flex: 1;
          margin-bottom: 16px;
          
          .preview-chart {
            height: 120px;
            border: 1px solid var(--el-border-color-lighter);
            border-radius: 4px;
            overflow: hidden;
          }
        }
        
        .report-footer {
          display: flex;
          justify-content: space-between;
          align-items: center;
          
          .report-meta {
            display: flex;
            align-items: center;
            gap: 8px;
            
            .last-run {
              font-size: 11px;
              color: var(--el-text-color-secondary);
            }
          }
        }
      }
    }
  }
}

// 自定义报表设计器
.custom-view {
  height: calc(100vh - 120px);
  
  .designer-layout {
    display: flex;
    height: 100%;
    border: 1px solid var(--el-border-color-light);
    border-radius: 8px;
    background: var(--el-bg-color);
    overflow: hidden;
    
    .designer-sidebar {
      width: 250px;
      background: var(--el-fill-color-light);
      border-right: 1px solid var(--el-border-color-light);
      overflow-y: auto;
      
      .sidebar-section {
        padding: 16px;
        border-bottom: 1px solid var(--el-border-color-lighter);
        
        &:last-child {
          border-bottom: none;
        }
        
        h4 {
          margin: 0 0 12px 0;
          font-size: 14px;
          font-weight: 600;
          color: var(--el-text-color-primary);
        }
        
        .data-source-list,
        .chart-types-list,
        .components-list {
          .data-source-item,
          .chart-type-item,
          .component-item {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 8px;
            margin-bottom: 4px;
            border-radius: 4px;
            cursor: pointer;
            transition: all 0.2s;
            font-size: 13px;
            
            &:hover {
              background: var(--el-color-primary-light-9);
              color: var(--el-color-primary);
            }
            
            &:last-child {
              margin-bottom: 0;
            }
          }
          
          .data-source-item {
            cursor: grab;
            
            &:active {
              cursor: grabbing;
            }
          }
          
          .component-item {
            cursor: grab;
            
            &:active {
              cursor: grabbing;
            }
          }
        }
      }
    }
    
    .designer-canvas {
      flex: 1;
      display: flex;
      flex-direction: column;
      
      .canvas-toolbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 16px;
        border-bottom: 1px solid var(--el-border-color-light);
        background: var(--el-fill-color-lighter);
        
        .toolbar-left,
        .toolbar-right {
          display: flex;
          gap: 8px;
        }
      }
      
      .canvas-area {
        flex: 1;
        position: relative;
        background: white;
        background-image: 
          linear-gradient(rgba(0, 0, 0, 0.1) 1px, transparent 1px),
          linear-gradient(90deg, rgba(0, 0, 0, 0.1) 1px, transparent 1px);
        background-size: 20px 20px;
        overflow: auto;
        
        .canvas-placeholder {
          position: absolute;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
        }
        
        .report-element {
          border: 2px dashed transparent;
          transition: all 0.2s;
          
          &:hover {
            border-color: var(--el-color-primary-light-5);
          }
          
          &.active {
            border-color: var(--el-color-primary);
            box-shadow: 0 0 0 2px var(--el-color-primary-light-8);
          }
          
          .element-controls {
            position: absolute;
            top: -32px;
            right: 0;
            display: flex;
            gap: 4px;
            background: var(--el-color-primary);
            border-radius: 4px;
            padding: 2px;
            
            .el-button {
              width: 24px;
              height: 24px;
              padding: 0;
              border: none;
              
              &:hover {
                background: var(--el-color-primary-light-3);
              }
            }
          }
        }
      }
    }
    
    .designer-properties {
      width: 280px;
      background: var(--el-fill-color-light);
      border-left: 1px solid var(--el-border-color-light);
      
      .properties-panel {
        padding: 16px;
        
        h4 {
          margin: 0 0 16px 0;
          font-size: 14px;
          font-weight: 600;
          color: var(--el-text-color-primary);
        }
        
        .element-properties {
          .property-group {
            margin-bottom: 16px;
            
            &:last-child {
              margin-bottom: 0;
            }
            
            label {
              display: block;
              font-size: 12px;
              color: var(--el-text-color-regular);
              margin-bottom: 4px;
            }
          }
        }
        
        .no-selection {
          text-align: center;
          padding: 40px 0;
        }
      }
    }
  }
}

// 对话框样式
.report-preview-content {
  .preview-header {
    margin-bottom: 24px;
    text-align: center;
    
    h2 {
      margin: 0 0 8px 0;
      color: var(--el-text-color-primary);
    }
    
    p {
      margin: 0 0 12px 0;
      color: var(--el-text-color-regular);
    }
    
    .preview-meta {
      font-size: 12px;
      color: var(--el-text-color-secondary);
    }
  }
  
  .preview-body {
    border: 1px solid var(--el-border-color-light);
    border-radius: 8px;
    padding: 16px;
  }
}

// 响应式设计
@media (max-width: 1400px) {
  .custom-view {
    .designer-layout {
      .designer-sidebar {
        width: 220px;
      }
      
      .designer-properties {
        width: 260px;
      }
    }
  }
}

@media (max-width: 1200px) {
  .dashboard-view {
    .analysis-charts {
      .el-row .el-col {
        margin-bottom: 16px;
      }
    }
    
    .prediction-analysis {
      .prediction-content {
        .el-row {
          flex-direction: column;
        }
        
        .prediction-summary {
          margin-top: 16px;
        }
      }
    }
  }
  
  .custom-view {
    .designer-layout {
      flex-direction: column;
      
      .designer-sidebar {
        width: 100%;
        height: 200px;
        border-right: none;
        border-bottom: 1px solid var(--el-border-color-light);
        
        .sidebar-section {
          display: inline-block;
          width: 33.33%;
          vertical-align: top;
        }
      }
      
      .designer-properties {
        width: 100%;
        height: 200px;
        border-left: none;
        border-top: 1px solid var(--el-border-color-light);
      }
    }
  }
}

@media (max-width: 768px) {
  .analytics-page {
    padding: 8px;
  }
  
  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }
  
  .dashboard-view {
    .kpi-overview {
      .el-col {
        margin-bottom: 12px;
      }
    }
    
    .analysis-charts {
      .el-row {
        flex-direction: column;
      }
    }
  }
  
  .reports-view {
    .reports-toolbar {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }
    
    .reports-grid {
      .el-row {
        flex-direction: column;
      }
      
      .report-card {
        height: auto;
        margin-bottom: 16px;
      }
    }
  }
  
  .custom-view {
    height: auto;
    min-height: 600px;
  }
}
</style>