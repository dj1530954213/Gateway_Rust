<template>
  <div class="analytics-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="title-section">
          <h1 class="page-title">数据分析</h1>
          <p class="page-description">深度分析设备数据，洞察业务趋势和性能指标</p>
        </div>
        
        <div class="header-actions">
          <el-button 
            v-permission="'reports:generate'"
            type="primary" 
            @click="showReportDialog = true"
          >
            <el-icon><Document /></el-icon>
            生成报表
          </el-button>
          <el-button @click="refreshData" :loading="loading">
            <el-icon><Refresh /></el-icon>
            刷新数据
          </el-button>
          <el-dropdown @command="handleExportAction">
            <el-button>
              导出数据
              <el-icon><ArrowDown /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="excel">导出Excel</el-dropdown-item>
                <el-dropdown-item command="pdf">导出PDF</el-dropdown-item>
                <el-dropdown-item command="csv">导出CSV</el-dropdown-item>
                <el-dropdown-item command="json">导出JSON</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
    </div>

    <!-- 分析控制面板 -->
    <el-card class="control-panel" shadow="never">
      <div class="control-form">
        <div class="control-row">
          <div class="control-item">
            <label class="control-label">时间范围</label>
            <el-date-picker
              v-model="analysisForm.dateRange"
              type="datetimerange"
              range-separator="至"
              start-placeholder="开始时间"
              end-placeholder="结束时间"
              format="YYYY-MM-DD HH:mm:ss"
              value-format="YYYY-MM-DD HH:mm:ss"
              style="width: 350px"
              @change="handleTimeRangeChange"
            />
          </div>
          
          <div class="control-item">
            <label class="control-label">设备分组</label>
            <el-select
              v-model="analysisForm.deviceGroup"
              placeholder="选择设备分组"
              style="width: 180px"
              @change="handleDeviceGroupChange"
            >
              <el-option label="全部设备" value="" />
              <el-option label="生产线A" value="line_a" />
              <el-option label="生产线B" value="line_b" />
              <el-option label="生产线C" value="line_c" />
              <el-option label="环境监测" value="environment" />
              <el-option label="能耗监测" value="energy" />
            </el-select>
          </div>
          
          <div class="control-item">
            <label class="control-label">数据类型</label>
            <el-select
              v-model="analysisForm.dataType"
              placeholder="选择数据类型"
              style="width: 150px"
              @change="handleDataTypeChange"
            >
              <el-option label="全部数据" value="" />
              <el-option label="温度" value="temperature" />
              <el-option label="压力" value="pressure" />
              <el-option label="流量" value="flow" />
              <el-option label="电压" value="voltage" />
              <el-option label="电流" value="current" />
              <el-option label="功率" value="power" />
            </el-select>
          </div>
          
          <div class="control-item">
            <label class="control-label">聚合方式</label>
            <el-select
              v-model="analysisForm.aggregation"
              placeholder="选择聚合方式"
              style="width: 120px"
              @change="handleAggregationChange"
            >
              <el-option label="平均值" value="avg" />
              <el-option label="最大值" value="max" />
              <el-option label="最小值" value="min" />
              <el-option label="总和" value="sum" />
              <el-option label="计数" value="count" />
            </el-select>
          </div>
          
          <div class="control-item">
            <label class="control-label">采样间隔</label>
            <el-select
              v-model="analysisForm.interval"
              placeholder="选择间隔"
              style="width: 100px"
              @change="handleIntervalChange"
            >
              <el-option label="1分钟" value="1m" />
              <el-option label="5分钟" value="5m" />
              <el-option label="15分钟" value="15m" />
              <el-option label="1小时" value="1h" />
              <el-option label="1天" value="1d" />
            </el-select>
          </div>
          
          <div class="control-actions">
            <el-button type="primary" @click="executeAnalysis" :loading="analyzing">
              <el-icon><Search /></el-icon>
              分析
            </el-button>
            <el-button @click="resetFilters">
              <el-icon><RefreshRight /></el-icon>
              重置
            </el-button>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 数据概览卡片 -->
    <div class="overview-cards">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card class="overview-card" shadow="hover">
            <div class="card-content">
              <div class="card-icon temperature">
                <el-icon><TrendCharts /></el-icon>
              </div>
              <div class="card-info">
                <div class="card-title">数据点总数</div>
                <div class="card-value">{{ formatNumber(overviewData.totalPoints) }}</div>
                <div class="card-trend positive">
                  <el-icon><CaretTop /></el-icon>
                  <span>+12.5%</span>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card class="overview-card" shadow="hover">
            <div class="card-content">
              <div class="card-icon pressure">
                <el-icon><DataAnalysis /></el-icon>
              </div>
              <div class="card-info">
                <div class="card-title">活跃设备数</div>
                <div class="card-value">{{ overviewData.activeDevices }}</div>
                <div class="card-trend positive">
                  <el-icon><CaretTop /></el-icon>
                  <span>+3.2%</span>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card class="overview-card" shadow="hover">
            <div class="card-content">
              <div class="card-icon flow">
                <el-icon><PieChart /></el-icon>
              </div>
              <div class="card-info">
                <div class="card-title">数据质量</div>
                <div class="card-value">{{ overviewData.dataQuality }}%</div>
                <div class="card-trend negative">
                  <el-icon><CaretBottom /></el-icon>
                  <span>-0.8%</span>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card class="overview-card" shadow="hover">
            <div class="card-content">
              <div class="card-icon energy">
                <el-icon><Odometer /></el-icon>
              </div>
              <div class="card-info">
                <div class="card-title">处理速率</div>
                <div class="card-value">{{ formatNumber(overviewData.processRate) }}/s</div>
                <div class="card-trend positive">
                  <el-icon><CaretTop /></el-icon>
                  <span>+8.1%</span>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 主要分析图表区域 -->
    <div class="charts-container">
      <el-row :gutter="20">
        <!-- 实时数据趋势图 -->
        <el-col :span="16">
          <el-card class="chart-card" shadow="never" v-loading="chartLoading">
            <template #header>
              <div class="card-header">
                <span class="chart-title">实时数据趋势</span>
                <div class="chart-controls">
                  <el-radio-group v-model="chartTimeRange" size="small" @change="updateChartData">
                    <el-radio-button label="1h">1小时</el-radio-button>
                    <el-radio-button label="6h">6小时</el-radio-button>
                    <el-radio-button label="24h">24小时</el-radio-button>
                    <el-radio-button label="7d">7天</el-radio-button>
                  </el-radio-group>
                </div>
              </div>
            </template>
            <div class="chart-container" style="height: 400px">
              <RealTimeChart 
                :data="realtimeChartData" 
                :loading="chartLoading"
                :options="chartOptions"
                @point-click="handleChartPointClick"
              />
            </div>
          </el-card>
        </el-col>
        
        <!-- 设备状态分布 -->
        <el-col :span="8">
          <el-card class="chart-card" shadow="never">
            <template #header>
              <span class="chart-title">设备状态分布</span>
            </template>
            <div class="chart-container" style="height: 400px">
              <DeviceStatusChart 
                :data="deviceStatusData"
                @status-click="handleStatusClick"
              />
            </div>
          </el-card>
        </el-col>
      </el-row>
      
      <el-row :gutter="20" style="margin-top: 20px">
        <!-- 数据质量分析 -->
        <el-col :span="12">
          <el-card class="chart-card" shadow="never">
            <template #header>
              <div class="card-header">
                <span class="chart-title">数据质量分析</span>
                <el-button size="small" type="link" @click="showQualityDetails = true">
                  查看详情
                </el-button>
              </div>
            </template>
            <div class="chart-container" style="height: 300px">
              <DataQualityChart 
                :data="dataQualityData"
                :metrics="qualityMetrics"
              />
            </div>
          </el-card>
        </el-col>
        
        <!-- 性能指标 -->
        <el-col :span="12">
          <el-card class="chart-card" shadow="never">
            <template #header>
              <span class="chart-title">性能指标</span>
            </template>
            <div class="chart-container" style="height: 300px">
              <PerformanceMetricsChart 
                :data="performanceData"
                :thresholds="performanceThresholds"
              />
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 详细数据表格 -->
    <el-card class="data-table-card" shadow="never">
      <template #header>
        <div class="card-header">
          <span class="chart-title">详细数据</span>
          <div class="table-controls">
            <el-input
              v-model="tableSearch"
              placeholder="搜索数据..."
              style="width: 200px"
              @input="handleTableSearch"
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
            <el-button @click="showColumnSettings = true">
              <el-icon><Setting /></el-icon>
              列设置
            </el-button>
          </div>
        </div>
      </template>
      
      <el-table
        ref="dataTableRef"
        :data="tableData"
        v-loading="tableLoading"
        height="400"
        @sort-change="handleTableSort"
        @selection-change="handleTableSelection"
      >
        <el-table-column type="selection" width="55" />
        <el-table-column 
          prop="timestamp" 
          label="时间戳" 
          width="180" 
          sortable="custom"
          :formatter="formatTimestamp"
        />
        <el-table-column prop="deviceName" label="设备名称" width="150" />
        <el-table-column prop="tagName" label="标签名称" width="150" />
        <el-table-column 
          prop="value" 
          label="数值" 
          width="120" 
          sortable="custom"
          :formatter="formatValue"
        />
        <el-table-column prop="unit" label="单位" width="80" />
        <el-table-column 
          prop="quality" 
          label="质量" 
          width="100"
          :formatter="formatQuality"
        >
          <template #default="{ row }">
            <el-tag 
              :type="getQualityType(row.quality)" 
              size="small"
            >
              {{ formatQuality(row) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="source" label="数据源" width="120" />
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button type="link" size="small" @click="viewDataDetails(row)">
              详情
            </el-button>
            <el-button type="link" size="small" @click="analyzeDataPoint(row)">
              分析
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      
      <!-- 分页 -->
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="tablePagination.page"
          v-model:page-size="tablePagination.size"
          :total="tablePagination.total"
          :page-sizes="[20, 50, 100, 200]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="loadTableData"
          @current-change="loadTableData"
        />
      </div>
    </el-card>

    <!-- 分析工具面板 -->
    <el-card v-if="selectedDataPoints.length > 0" class="analysis-tools" shadow="never">
      <template #header>
        <div class="card-header">
          <span class="chart-title">分析工具 ({{ selectedDataPoints.length }} 个数据点)</span>
          <div class="tool-actions">
            <el-button @click="performStatisticalAnalysis">统计分析</el-button>
            <el-button @click="performTrendAnalysis">趋势分析</el-button>
            <el-button @click="performCorrelationAnalysis">相关性分析</el-button>
            <el-button @click="clearSelection">清除选择</el-button>
          </div>
        </div>
      </template>
      
      <div class="analysis-results" v-if="analysisResults">
        <el-tabs v-model="activeAnalysisTab">
          <el-tab-pane label="统计摘要" name="statistics">
            <StatisticalSummary :data="analysisResults.statistics" />
          </el-tab-pane>
          <el-tab-pane label="趋势分析" name="trends">
            <TrendAnalysis :data="analysisResults.trends" />
          </el-tab-pane>
          <el-tab-pane label="相关性分析" name="correlation">
            <CorrelationAnalysis :data="analysisResults.correlation" />
          </el-tab-pane>
        </el-tabs>
      </div>
    </el-card>

    <!-- 对话框组件 -->
    <!-- 报表生成对话框 -->
    <ReportGeneratorDialog
      v-model:visible="showReportDialog"
      :analysis-data="currentAnalysisData"
      @generate="handleReportGenerate"
    />

    <!-- 数据质量详情对话框 -->
    <DataQualityDialog
      v-model:visible="showQualityDetails"
      :quality-data="dataQualityData"
    />

    <!-- 列设置对话框 -->
    <ColumnSettingsDialog
      v-model:visible="showColumnSettings"
      :columns="tableColumns"
      @update="handleColumnUpdate"
    />

    <!-- 数据详情对话框 -->
    <DataDetailsDialog
      v-model:visible="showDataDetails"
      :data-point="selectedDataPoint"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * AnalyticsPage —— 数据分析页面
 *
 * 📝 Responsibilities:
 *  1. 数据分析仪表板展示
 *  2. 实时和历史数据可视化
 *  3. 多维度数据筛选和聚合
 *  4. 统计分析和趋势预测
 *  5. 数据质量监控和报告
 *  6. 报表生成和数据导出
 *
 * 📦 Dependencies:
 *  - Element Plus UI组件库
 *  - ECharts图表库
 *  - 数据分析相关API
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { usePermission } from '@/composables/usePermission'
import {
  Document,
  Refresh,
  ArrowDown,
  Search,
  RefreshRight,
  TrendCharts,
  DataAnalysis,
  PieChart,
  Odometer,
  CaretTop,
  CaretBottom,
  Setting
} from '@element-plus/icons-vue'

// 组件导入 (将在后续步骤中创建)
import RealTimeChart from '@/components/analytics/RealTimeChart.vue'
import DeviceStatusChart from '@/components/analytics/DeviceStatusChart.vue'
import DataQualityChart from '@/components/analytics/DataQualityChart.vue'
import PerformanceMetricsChart from '@/components/analytics/PerformanceMetricsChart.vue'
import StatisticalSummary from '@/components/analytics/StatisticalSummary.vue'
import TrendAnalysis from '@/components/analytics/TrendAnalysis.vue'
import CorrelationAnalysis from '@/components/analytics/CorrelationAnalysis.vue'
import ReportGeneratorDialog from '@/components/analytics/ReportGeneratorDialog.vue'
import DataQualityDialog from '@/components/analytics/DataQualityDialog.vue'
import ColumnSettingsDialog from '@/components/analytics/ColumnSettingsDialog.vue'
import DataDetailsDialog from '@/components/analytics/DataDetailsDialog.vue'

// ===== 权限管理 =====
const { hasPermission, logUserActivity } = usePermission()

// ===== 响应式数据 =====
const loading = ref(false)
const analyzing = ref(false)
const chartLoading = ref(false)
const tableLoading = ref(false)

// 分析表单
const analysisForm = ref({
  dateRange: [] as string[],
  deviceGroup: '',
  dataType: '',
  aggregation: 'avg',
  interval: '5m'
})

// 对话框状态
const showReportDialog = ref(false)
const showQualityDetails = ref(false)
const showColumnSettings = ref(false)
const showDataDetails = ref(false)

// 图表相关
const chartTimeRange = ref('6h')
const chartOptions = ref({})
const realtimeChartData = ref([])
const deviceStatusData = ref([])
const dataQualityData = ref([])
const performanceData = ref([])

// 概览数据
const overviewData = ref({
  totalPoints: 2534728,
  activeDevices: 148,
  dataQuality: 96.7,
  processRate: 15642
})

// 表格相关
const dataTableRef = ref()
const tableData = ref([])
const tableSearch = ref('')
const tablePagination = ref({
  page: 1,
  size: 50,
  total: 0
})

// 数据选择和分析
const selectedDataPoints = ref([])
const selectedDataPoint = ref(null)
const analysisResults = ref(null)
const activeAnalysisTab = ref('statistics')

// 质量指标和性能阈值
const qualityMetrics = ref({
  completeness: 96.7,
  accuracy: 98.2,
  consistency: 94.8,
  timeliness: 95.3
})

const performanceThresholds = ref({
  response_time: { warning: 100, critical: 200 },
  throughput: { warning: 1000, critical: 500 },
  error_rate: { warning: 0.05, critical: 0.1 },
  availability: { warning: 0.95, critical: 0.90 }
})

// 表格列设置
const tableColumns = ref([
  { prop: 'timestamp', label: '时间戳', visible: true, sortable: true },
  { prop: 'deviceName', label: '设备名称', visible: true, sortable: false },
  { prop: 'tagName', label: '标签名称', visible: true, sortable: false },
  { prop: 'value', label: '数值', visible: true, sortable: true },
  { prop: 'unit', label: '单位', visible: true, sortable: false },
  { prop: 'quality', label: '质量', visible: true, sortable: true },
  { prop: 'source', label: '数据源', visible: true, sortable: false }
])

// 当前分析数据（用于报表生成）
const currentAnalysisData = computed(() => ({
  form: analysisForm.value,
  overview: overviewData.value,
  charts: {
    realtime: realtimeChartData.value,
    deviceStatus: deviceStatusData.value,
    dataQuality: dataQualityData.value,
    performance: performanceData.value
  },
  tableData: tableData.value,
  analysisResults: analysisResults.value
}))

// ===== 方法 =====

/**
 * 初始化页面数据
 */
async function initializeData() {
  try {
    loading.value = true
    
    // 设置默认时间范围（最近6小时）
    const now = new Date()
    const sixHoursAgo = new Date(now.getTime() - 6 * 60 * 60 * 1000)
    analysisForm.value.dateRange = [
      sixHoursAgo.toISOString().slice(0, 19).replace('T', ' '),
      now.toISOString().slice(0, 19).replace('T', ' ')
    ]
    
    // 并行加载各种数据
    await Promise.all([
      loadOverviewData(),
      loadChartData(),
      loadTableData()
    ])
    
    // 记录页面访问
    logUserActivity({
      type: 'analytics',
      action: 'page_visit',
      target: 'analytics_dashboard'
    })
    
  } catch (error) {
    console.error('初始化数据分析页面失败:', error)
    ElMessage.error('加载数据失败')
  } finally {
    loading.value = false
  }
}

/**
 * 加载概览数据
 */
async function loadOverviewData() {
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 300))
    
    // 生成模拟概览数据
    overviewData.value = {
      totalPoints: Math.floor(Math.random() * 1000000) + 2000000,
      activeDevices: Math.floor(Math.random() * 50) + 120,
      dataQuality: Math.floor(Math.random() * 10) + 90 + Math.random(),
      processRate: Math.floor(Math.random() * 5000) + 12000
    }
    
  } catch (error) {
    console.error('加载概览数据失败:', error)
  }
}

/**
 * 加载图表数据
 */
async function loadChartData() {
  try {
    chartLoading.value = true
    
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 500))
    
    // 生成实时图表数据
    realtimeChartData.value = generateRealtimeData()
    
    // 生成设备状态数据
    deviceStatusData.value = generateDeviceStatusData()
    
    // 生成数据质量数据
    dataQualityData.value = generateDataQualityData()
    
    // 生成性能数据
    performanceData.value = generatePerformanceData()
    
  } catch (error) {
    console.error('加载图表数据失败:', error)
  } finally {
    chartLoading.value = false
  }
}

/**
 * 加载表格数据
 */
async function loadTableData() {
  try {
    tableLoading.value = true
    
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 400))
    
    // 生成表格数据
    tableData.value = generateTableData()
    tablePagination.value.total = 5000 // 模拟总数
    
  } catch (error) {
    console.error('加载表格数据失败:', error)
  } finally {
    tableLoading.value = false
  }
}

/**
 * 生成实时数据
 */
function generateRealtimeData() {
  const data = []
  const now = new Date()
  const interval = getIntervalMinutes(chartTimeRange.value)
  const points = getTimeRangePoints(chartTimeRange.value)
  
  for (let i = 0; i < points; i++) {
    const time = new Date(now.getTime() - (points - i - 1) * interval * 60 * 1000)
    data.push({
      timestamp: time.toISOString(),
      temperature: 20 + Math.random() * 10 + Math.sin(i * 0.1) * 5,
      pressure: 100 + Math.random() * 20 + Math.cos(i * 0.15) * 10,
      flow: 50 + Math.random() * 30 + Math.sin(i * 0.08) * 15,
      power: 1000 + Math.random() * 200 + Math.cos(i * 0.12) * 100
    })
  }
  
  return data
}

/**
 * 生成设备状态数据
 */
function generateDeviceStatusData() {
  return [
    { name: '正常运行', value: 125, color: '#67C23A' },
    { name: '维护中', value: 15, color: '#E6A23C' },
    { name: '故障', value: 5, color: '#F56C6C' },
    { name: '离线', value: 8, color: '#909399' }
  ]
}

/**
 * 生成数据质量数据
 */
function generateDataQualityData() {
  const data = []
  const now = new Date()
  
  for (let i = 0; i < 24; i++) {
    const time = new Date(now.getTime() - (24 - i) * 60 * 60 * 1000)
    data.push({
      timestamp: time.toISOString(),
      completeness: 95 + Math.random() * 5,
      accuracy: 96 + Math.random() * 4,
      consistency: 93 + Math.random() * 7,
      timeliness: 94 + Math.random() * 6
    })
  }
  
  return data
}

/**
 * 生成性能数据
 */
function generatePerformanceData() {
  const data = []
  const now = new Date()
  
  for (let i = 0; i < 100; i++) {
    const time = new Date(now.getTime() - (100 - i) * 5 * 60 * 1000)
    data.push({
      timestamp: time.toISOString(),
      response_time: 20 + Math.random() * 80,
      throughput: 800 + Math.random() * 400,
      error_rate: Math.random() * 0.1,
      availability: 0.95 + Math.random() * 0.05
    })
  }
  
  return data
}

/**
 * 生成表格数据
 */
function generateTableData() {
  const data = []
  const devices = ['PLC-001', 'PLC-002', 'Sensor-A1', 'Sensor-B2', 'Motor-001']
  const tags = ['温度', '压力', '流量', '电压', '电流', '功率']
  const qualities = ['Good', 'Bad', 'Uncertain']
  const sources = ['ModbusTCP', 'OPC-UA', 'Ethernet/IP', 'MQTT']
  
  for (let i = 0; i < tablePagination.value.size; i++) {
    const device = devices[Math.floor(Math.random() * devices.length)]
    const tag = tags[Math.floor(Math.random() * tags.length)]
    const quality = qualities[Math.floor(Math.random() * qualities.length)]
    const source = sources[Math.floor(Math.random() * sources.length)]
    
    data.push({
      id: `data_${i + 1}`,
      timestamp: new Date(Date.now() - Math.random() * 24 * 60 * 60 * 1000).toISOString(),
      deviceName: device,
      tagName: tag,
      value: Math.random() * 100,
      unit: getUnitByTag(tag),
      quality,
      source
    })
  }
  
  return data.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
}

/**
 * 根据标签获取单位
 */
function getUnitByTag(tag: string): string {
  const unitMap: { [key: string]: string } = {
    '温度': '°C',
    '压力': 'Pa',
    '流量': 'L/min',
    '电压': 'V',
    '电流': 'A',
    '功率': 'W'
  }
  return unitMap[tag] || ''
}

/**
 * 获取时间范围对应的数据点数
 */
function getTimeRangePoints(range: string): number {
  switch (range) {
    case '1h': return 60
    case '6h': return 72
    case '24h': return 96
    case '7d': return 168
    default: return 72
  }
}

/**
 * 获取间隔分钟数
 */
function getIntervalMinutes(range: string): number {
  switch (range) {
    case '1h': return 1
    case '6h': return 5
    case '24h': return 15
    case '7d': return 60
    default: return 5
  }
}

/**
 * 格式化数字
 */
function formatNumber(num: number): string {
  if (num >= 1000000) {
    return `${(num / 1000000).toFixed(1)  }M`
  } else if (num >= 1000) {
    return `${(num / 1000).toFixed(1)  }K`
  }
  return num.toString()
}

/**
 * 格式化时间戳
 */
function formatTimestamp(row: any): string {
  return new Date(row.timestamp).toLocaleString('zh-CN')
}

/**
 * 格式化数值
 */
function formatValue(row: any): string {
  return parseFloat(row.value).toFixed(2)
}

/**
 * 格式化质量
 */
function formatQuality(row: any): string {
  const qualityMap: { [key: string]: string } = {
    'Good': '良好',
    'Bad': '错误',
    'Uncertain': '不确定'
  }
  return qualityMap[row.quality] || row.quality
}

/**
 * 获取质量类型
 */
function getQualityType(quality: string): string {
  switch (quality) {
    case 'Good': return 'success'
    case 'Bad': return 'danger'
    case 'Uncertain': return 'warning'
    default: return 'info'
  }
}

/**
 * 刷新数据
 */
async function refreshData() {
  await initializeData()
  ElMessage.success('数据已刷新')
}

/**
 * 执行分析
 */
async function executeAnalysis() {
  try {
    analyzing.value = true
    
    // 记录分析操作
    logUserActivity({
      type: 'analytics',
      action: 'execute_analysis',
      target: 'data_analysis',
      details: analysisForm.value
    })
    
    // 模拟分析处理
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // 重新加载数据
    await Promise.all([
      loadChartData(),
      loadTableData()
    ])
    
    ElMessage.success('分析完成')
    
  } catch (error) {
    console.error('执行分析失败:', error)
    ElMessage.error('分析失败')
  } finally {
    analyzing.value = false
  }
}

/**
 * 重置筛选条件
 */
function resetFilters() {
  analysisForm.value = {
    dateRange: [],
    deviceGroup: '',
    dataType: '',
    aggregation: 'avg',
    interval: '5m'
  }
  
  // 重新设置默认时间范围
  const now = new Date()
  const sixHoursAgo = new Date(now.getTime() - 6 * 60 * 60 * 1000)
  analysisForm.value.dateRange = [
    sixHoursAgo.toISOString().slice(0, 19).replace('T', ' '),
    now.toISOString().slice(0, 19).replace('T', ' ')
  ]
}

/**
 * 事件处理函数
 */
function handleTimeRangeChange() {
  // 时间范围变更处理
}

function handleDeviceGroupChange() {
  // 设备分组变更处理
}

function handleDataTypeChange() {
  // 数据类型变更处理
}

function handleAggregationChange() {
  // 聚合方式变更处理
}

function handleIntervalChange() {
  // 采样间隔变更处理
}

function updateChartData() {
  loadChartData()
}

function handleChartPointClick(point: any) {
  console.log('图表点击:', point)
}

function handleStatusClick(status: any) {
  console.log('状态点击:', status)
}

function handleTableSearch() {
  // 实现表格搜索逻辑
  loadTableData()
}

function handleTableSort(sort: any) {
  console.log('表格排序:', sort)
  loadTableData()
}

function handleTableSelection(selection: any[]) {
  selectedDataPoints.value = selection
}

function handleColumnUpdate(columns: any[]) {
  tableColumns.value = columns
}

function handleExportAction(command: string) {
  console.log('导出操作:', command)
  ElMessage.success(`开始导出${command.toUpperCase()}格式数据`)
}

function handleReportGenerate(reportConfig: any) {
  console.log('生成报表:', reportConfig)
  ElMessage.success('报表生成中，请稍候...')
}

/**
 * 数据分析函数
 */
function viewDataDetails(row: any) {
  selectedDataPoint.value = row
  showDataDetails.value = true
}

function analyzeDataPoint(row: any) {
  console.log('分析数据点:', row)
  ElMessage.info('开始分析数据点...')
}

function performStatisticalAnalysis() {
  console.log('执行统计分析:', selectedDataPoints.value)
  // 模拟统计分析结果
  analysisResults.value = {
    statistics: generateStatisticalResults(),
    trends: null,
    correlation: null
  }
  activeAnalysisTab.value = 'statistics'
}

function performTrendAnalysis() {
  console.log('执行趋势分析:', selectedDataPoints.value)
  // 模拟趋势分析结果
  analysisResults.value = {
    statistics: analysisResults.value?.statistics || null,
    trends: generateTrendResults(),
    correlation: analysisResults.value?.correlation || null
  }
  activeAnalysisTab.value = 'trends'
}

function performCorrelationAnalysis() {
  console.log('执行相关性分析:', selectedDataPoints.value)
  // 模拟相关性分析结果
  analysisResults.value = {
    statistics: analysisResults.value?.statistics || null,
    trends: analysisResults.value?.trends || null,
    correlation: generateCorrelationResults()
  }
  activeAnalysisTab.value = 'correlation'
}

function clearSelection() {
  selectedDataPoints.value = []
  analysisResults.value = null
  dataTableRef.value?.clearSelection()
}

/**
 * 生成分析结果
 */
function generateStatisticalResults() {
  return {
    count: selectedDataPoints.value.length,
    mean: 45.67,
    median: 44.23,
    std: 12.45,
    min: 15.2,
    max: 89.7,
    q1: 35.8,
    q3: 56.2
  }
}

function generateTrendResults() {
  return {
    trend: 'increasing',
    slope: 0.23,
    r_squared: 0.85,
    forecast: [
      { timestamp: '2025-07-28 10:00:00', value: 47.2, confidence: 0.95 },
      { timestamp: '2025-07-28 11:00:00', value: 48.1, confidence: 0.92 },
      { timestamp: '2025-07-28 12:00:00', value: 49.0, confidence: 0.89 }
    ]
  }
}

function generateCorrelationResults() {
  return {
    matrix: [
      ['temperature', 'pressure', 'flow'],
      [1.0, 0.75, -0.23],
      [0.75, 1.0, -0.18],
      [-0.23, -0.18, 1.0]
    ],
    significant_pairs: [
      { var1: 'temperature', var2: 'pressure', correlation: 0.75, p_value: 0.001 }
    ]
  }
}

// ===== 生命周期 =====
onMounted(() => {
  initializeData()
})

// ===== 监听器 =====
watch(() => analysisForm.value.dateRange, () => {
  if (analysisForm.value.dateRange && analysisForm.value.dateRange.length === 2) {
    loadChartData()
  }
})
</script>

<style scoped lang="scss">
.analytics-page {
  .page-header {
    background: #fff;
    border-radius: 8px;
    margin-bottom: 20px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.05);
    
    .header-content {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 24px;
      
      .title-section {
        .page-title {
          margin: 0 0 8px 0;
          font-size: 24px;
          font-weight: 600;
          color: #303133;
        }
        
        .page-description {
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
  }
  
  .control-panel {
    margin-bottom: 20px;
    
    .control-form {
      .control-row {
        display: flex;
        align-items: center;
        gap: 20px;
        flex-wrap: wrap;
        
        .control-item {
          display: flex;
          align-items: center;
          gap: 8px;
          
          .control-label {
            font-size: 14px;
            color: #606266;
            white-space: nowrap;
          }
        }
        
        .control-actions {
          margin-left: auto;
          display: flex;
          gap: 8px;
        }
      }
    }
  }
  
  .overview-cards {
    margin-bottom: 20px;
    
    .overview-card {
      height: 120px;
      
      .card-content {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 12px;
        
        .card-icon {
          width: 60px;
          height: 60px;
          border-radius: 12px;
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 24px;
          color: white;
          
          &.temperature {
            background: linear-gradient(135deg, #ff7675, #d63031);
          }
          
          &.pressure {
            background: linear-gradient(135deg, #74b9ff, #0984e3);
          }
          
          &.flow {
            background: linear-gradient(135deg, #00cec9, #00b894);
          }
          
          &.energy {
            background: linear-gradient(135deg, #fdcb6e, #e17055);
          }
        }
        
        .card-info {
          flex: 1;
          
          .card-title {
            font-size: 13px;
            color: #909399;
            margin-bottom: 4px;
          }
          
          .card-value {
            font-size: 24px;
            font-weight: 600;
            color: #303133;
            margin-bottom: 4px;
          }
          
          .card-trend {
            display: flex;
            align-items: center;
            gap: 4px;
            font-size: 12px;
            
            &.positive {
              color: #67c23a;
            }
            
            &.negative {
              color: #f56c6c;
            }
          }
        }
      }
    }
  }
  
  .charts-container {
    margin-bottom: 20px;
    
    .chart-card {
      .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        
        .chart-title {
          font-size: 16px;
          font-weight: 600;
          color: #303133;
        }
        
        .chart-controls {
          display: flex;
          gap: 8px;
        }
      }
      
      .chart-container {
        padding: 16px 0;
      }
    }
  }
  
  .data-table-card {
    margin-bottom: 20px;
    
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      .chart-title {
        font-size: 16px;
        font-weight: 600;
        color: #303133;
      }
      
      .table-controls {
        display: flex;
        gap: 8px;
        align-items: center;
      }
    }
    
    .pagination-container {
      margin-top: 16px;
      text-align: center;
    }
  }
  
  .analysis-tools {
    margin-bottom: 20px;
    
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      .chart-title {
        font-size: 16px;
        font-weight: 600;
        color: #303133;
      }
      
      .tool-actions {
        display: flex;
        gap: 8px;
      }
    }
    
    .analysis-results {
      margin-top: 16px;
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .analytics-page {
    .page-header .header-content {
      flex-direction: column;
      align-items: flex-start;
      gap: 16px;
    }
    
    .control-panel .control-form .control-row {
      flex-direction: column;
      align-items: stretch;
      gap: 12px;
      
      .control-item {
        justify-content: space-between;
      }
      
      .control-actions {
        margin-left: 0;
        justify-content: center;
      }
    }
    
    .overview-cards .overview-card .card-content {
      flex-direction: column;
      text-align: center;
      gap: 8px;
    }
  }
}
</style>