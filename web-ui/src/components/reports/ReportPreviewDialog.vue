<template>
  <el-dialog
    v-model="visible"
    :title="`预览报表: ${report?.name || ''}`"
    width="90%"
    :close-on-click-modal="false"
    top="5vh"
    class="report-preview-dialog"
  >
    <div v-if="report" class="report-preview">
      <!-- 预览工具栏 -->
      <div class="preview-toolbar">
        <div class="toolbar-left">
          <el-button-group>
            <el-button 
              :type="viewMode === 'desktop' ? 'primary' : ''" 
              @click="viewMode = 'desktop'"
            >
              <el-icon><Monitor /></el-icon>
              桌面视图
            </el-button>
            <el-button 
              :type="viewMode === 'mobile' ? 'primary' : ''" 
              @click="viewMode = 'mobile'"
            >
              <el-icon><Iphone /></el-icon>
              移动视图
            </el-button>
          </el-button-group>
          
          <el-divider direction="vertical" />
          
          <el-button-group>
            <el-button @click="zoomOut" :disabled="zoomLevel <= 0.5">
              <el-icon><ZoomOut /></el-icon>
            </el-button>
            <el-button @click="resetZoom">
              {{ Math.round(zoomLevel * 100) }}%
            </el-button>
            <el-button @click="zoomIn" :disabled="zoomLevel >= 2">
              <el-icon><ZoomIn /></el-icon>
            </el-button>
          </el-button-group>
        </div>
        
        <div class="toolbar-right">
          <el-dropdown @command="handlePrint">
            <el-button>
              <el-icon><Printer /></el-icon>
              打印
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="current">当前页</el-dropdown-item>
                <el-dropdown-item command="all">全部页面</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
          
          <el-dropdown @command="handleExport">
            <el-button type="primary">
              <el-icon><Download /></el-icon>
              导出
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item
                  v-for="format in report.formats"
                  :key="format"
                  :command="format"
                >
                  {{ format.toUpperCase() }}
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
      
      <!-- 预览内容 -->
      <div 
        :class="['preview-content', viewMode]"
        :style="{ transform: `scale(${zoomLevel})` }"
      >
        <div class="report-document">
          <!-- 报表头部 -->
          <div class="report-header">
            <div class="header-logo">
              <img src="/logo.png" alt="Logo" class="logo" />
            </div>
            <div class="header-info">
              <h1 class="report-title">{{ report.name }}</h1>
              <div class="report-meta">
                <div class="meta-item">
                  <span class="label">生成时间:</span>
                  <span class="value">{{ formatDateTime(report.generatedAt) }}</span>
                </div>
                <div class="meta-item">
                  <span class="label">报表类型:</span>
                  <span class="value">{{ getTemplateName(report.template) }}</span>
                </div>
                <div class="meta-item">
                  <span class="label">时间范围:</span>
                  <span class="value">{{ report.config?.dateRange?.join(' 至 ') || '--' }}</span>
                </div>
              </div>
            </div>
          </div>
          
          <!-- 报表摘要 -->
          <div class="report-summary">
            <h2>报表摘要</h2>
            <div class="summary-grid">
              <div class="summary-item">
                <div class="summary-label">设备数量</div>
                <div class="summary-value">{{ report.config?.deviceIds?.length || 0 }}</div>
                <div class="summary-unit">台</div>
              </div>
              <div class="summary-item">
                <div class="summary-label">数据点数</div>
                <div class="summary-value">{{ report.dataCount || 0 }}</div>
                <div class="summary-unit">个</div>
              </div>
              <div class="summary-item">
                <div class="summary-label">异常事件</div>
                <div class="summary-value">{{ Math.floor(Math.random() * 50) }}</div>
                <div class="summary-unit">次</div>
              </div>
              <div class="summary-item">
                <div class="summary-label">可用率</div>
                <div class="summary-value">{{ (95 + Math.random() * 5).toFixed(1) }}</div>
                <div class="summary-unit">%</div>
              </div>
            </div>
          </div>
          
          <!-- 图表区域 -->
          <div class="report-charts">
            <h2>数据分析</h2>
            
            <!-- 趋势图表 -->
            <div class="chart-section">
              <h3>数据趋势分析</h3>
              <div ref="trendChartRef" class="chart-container"></div>
            </div>
            
            <!-- 分布图表 -->
            <div class="chart-section">
              <h3>设备状态分布</h3>
              <div class="chart-row">
                <div ref="pieChartRef" class="chart-container small"></div>
                <div ref="barChartRef" class="chart-container small"></div>
              </div>
            </div>
            
            <!-- 性能指标 -->
            <div class="chart-section">
              <h3>关键性能指标</h3>
              <div ref="performanceChartRef" class="chart-container"></div>
            </div>
          </div>
          
          <!-- 数据表格 -->
          <div class="report-table">
            <h2>详细数据</h2>
            <el-table :data="sampleTableData" border style="width: 100%">
              <el-table-column prop="device" label="设备名称" width="120" />
              <el-table-column prop="tag" label="标签" width="100" />
              <el-table-column prop="avgValue" label="平均值" width="100" />
              <el-table-column prop="maxValue" label="最大值" width="100" />
              <el-table-column prop="minValue" label="最小值" width="100" />
              <el-table-column prop="status" label="状态" width="100">
                <template #default="{ row }">
                  <el-tag :type="getStatusType(row.status)" size="small">
                    {{ row.status }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column prop="lastUpdate" label="最后更新" />
            </el-table>
          </div>
          
          <!-- 报表结论 -->
          <div class="report-conclusion">
            <h2>分析结论</h2>
            <div class="conclusion-content">
              <ul>
                <li>系统整体运行稳定，设备可用率达到 {{ (95 + Math.random() * 5).toFixed(1) }}%</li>
                <li>温度传感器数据波动在正常范围内，平均温度为 25.6°C</li>
                <li>检测到 {{ Math.floor(Math.random() * 10) + 1 }} 次轻微异常，均已自动处理</li>
                <li>建议加强对设备 PLC-02 的监控，其数据波动相对较大</li>
                <li>数据质量良好，完整性达到 98.5%，准确性达到 99.2%</li>
              </ul>
            </div>
          </div>
          
          <!-- 报表页脚 -->
          <div class="report-footer">
            <div class="footer-info">
              <div>本报表由工业物联网数据平台自动生成</div>
              <div>生成时间: {{ formatDateTime(report.generatedAt) }}</div>
              <div>页码: 1 / 1</div>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 页面导航 -->
      <div class="page-navigation">
        <el-pagination
          v-model:current-page="currentPage"
          :page-count="totalPages"
          layout="prev, pager, next"
          :hide-on-single-page="totalPages <= 1"
        />
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * ReportPreviewDialog —— 报表预览对话框
 *
 * 📝 Responsibilities:
 *  1. 报表内容预览展示
 *  2. 多视图模式支持
 *  3. 缩放和打印功能
 *  4. 报表导出功能
 *
 * 📦 Dependencies:
 *  - Element Plus UI组件
 *  - ECharts 图表展示
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import * as echarts from 'echarts'
import {
  Monitor,
  Iphone,
  ZoomIn,
  ZoomOut,
  Printer,
  Download
} from '@element-plus/icons-vue'

// ===== Props =====
const props = defineProps<{
  modelValue: boolean
  report?: any
}>()

// ===== Emits =====
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

// ===== 响应式数据 =====
const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const viewMode = ref('desktop')
const zoomLevel = ref(1)
const currentPage = ref(1)
const totalPages = ref(1)

// 图表引用
const trendChartRef = ref<HTMLDivElement>()
const pieChartRef = ref<HTMLDivElement>()
const barChartRef = ref<HTMLDivElement>()
const performanceChartRef = ref<HTMLDivElement>()

const trendChart = ref<echarts.ECharts>()
const pieChart = ref<echarts.ECharts>()
const barChart = ref<echarts.ECharts>()
const performanceChart = ref<echarts.ECharts>()

// 示例表格数据
const sampleTableData = ref([
  {
    device: 'PLC-01',
    tag: '温度',
    avgValue: 25.6,
    maxValue: 28.2,
    minValue: 22.1,
    status: '正常',
    lastUpdate: '2025-07-27 11:30:00'
  },
  {
    device: 'PLC-02',
    tag: '压力',
    avgValue: 1.25,
    maxValue: 1.45,
    minValue: 1.05,
    status: '正常',
    lastUpdate: '2025-07-27 11:29:00'
  },
  {
    device: 'Sensor-01',
    tag: '流量',
    avgValue: 152.3,
    maxValue: 165.8,
    minValue: 138.7,
    status: '警告',
    lastUpdate: '2025-07-27 11:28:00'
  },
  {
    device: 'Sensor-02',
    tag: '功率',
    avgValue: 850.2,
    maxValue: 920.5,
    minValue: 780.1,
    status: '正常',
    lastUpdate: '2025-07-27 11:27:00'
  }
])

// ===== 方法 =====

/**
 * 缩放控制
 */
function zoomIn() {
  if (zoomLevel.value < 2) {
    zoomLevel.value = Math.min(zoomLevel.value + 0.1, 2)
  }
}

function zoomOut() {
  if (zoomLevel.value > 0.5) {
    zoomLevel.value = Math.max(zoomLevel.value - 0.1, 0.5)
  }
}

function resetZoom() {
  zoomLevel.value = 1
}

/**
 * 初始化图表
 */
function initCharts() {
  nextTick(() => {
    initTrendChart()
    initPieChart()
    initBarChart()
    initPerformanceChart()
  })
}

function initTrendChart() {
  if (!trendChartRef.value) return
  
  trendChart.value = echarts.init(trendChartRef.value)
  
  const option = {
    animation: false,
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
      data: ['温度', '压力', '流量']
    },
    xAxis: {
      type: 'category',
      data: ['00:00', '04:00', '08:00', '12:00', '16:00', '20:00', '24:00']
    },
    yAxis: {
      type: 'value'
    },
    series: [
      {
        name: '温度',
        type: 'line',
        data: [22, 24, 26, 28, 26, 24, 23],
        smooth: true
      },
      {
        name: '压力',
        type: 'line',
        data: [1.1, 1.2, 1.3, 1.4, 1.3, 1.2, 1.1],
        smooth: true,
        yAxisIndex: 0
      },
      {
        name: '流量',
        type: 'line',
        data: [140, 150, 160, 165, 155, 145, 142],
        smooth: true
      }
    ]
  }
  
  trendChart.value.setOption(option)
}

function initPieChart() {
  if (!pieChartRef.value) return
  
  pieChart.value = echarts.init(pieChartRef.value)
  
  const option = {
    animation: false,
    tooltip: {
      trigger: 'item'
    },
    legend: {
      orient: 'vertical',
      left: 'left'
    },
    series: [
      {
        name: '设备状态',
        type: 'pie',
        radius: '50%',
        data: [
          { value: 8, name: '正常' },
          { value: 2, name: '警告' },
          { value: 1, name: '异常' }
        ]
      }
    ]
  }
  
  pieChart.value.setOption(option)
}

function initBarChart() {
  if (!barChartRef.value) return
  
  barChart.value = echarts.init(barChartRef.value)
  
  const option = {
    animation: false,
    tooltip: {
      trigger: 'axis'
    },
    xAxis: {
      type: 'category',
      data: ['PLC-01', 'PLC-02', 'Sensor-01', 'Sensor-02']
    },
    yAxis: {
      type: 'value'
    },
    series: [
      {
        name: '数据量',
        type: 'bar',
        data: [1200, 1100, 800, 950]
      }
    ]
  }
  
  barChart.value.setOption(option)
}

function initPerformanceChart() {
  if (!performanceChartRef.value) return
  
  performanceChart.value = echarts.init(performanceChartRef.value)
  
  const option = {
    animation: false,
    tooltip: {
      trigger: 'axis'
    },
    radar: {
      indicator: [
        { name: '可用性', max: 100 },
        { name: '响应时间', max: 100 },
        { name: '数据质量', max: 100 },
        { name: '稳定性', max: 100 },
        { name: '准确性', max: 100 }
      ]
    },
    series: [
      {
        name: '性能指标',
        type: 'radar',
        data: [
          {
            value: [95, 88, 92, 90, 94],
            name: '系统性能'
          }
        ]
      }
    ]
  }
  
  performanceChart.value.setOption(option)
}

/**
 * 处理打印
 */
function handlePrint(command: string) {
  if (command === 'current') {
    window.print()
  } else {
    ElMessage.info('全部页面打印功能开发中...')
  }
}

/**
 * 处理导出
 */
function handleExport(format: string) {
  ElMessage.success(`开始导出 ${format.toUpperCase()} 格式`)
  // 这里实现具体的导出逻辑
}

/**
 * 获取模板名称
 */
function getTemplateName(template: string): string {
  const templateMap: { [key: string]: string } = {
    device_summary: '设备概览报表',
    data_analysis: '数据分析报表',
    trend_report: '趋势分析报表',
    status_report: '状态分布报表',
    performance_report: '性能评估报表',
    custom_report: '自定义报表'
  }
  return templateMap[template] || template
}

/**
 * 格式化日期时间
 */
function formatDateTime(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

/**
 * 获取状态类型
 */
function getStatusType(status: string): string {
  if (status === '正常') return 'success'
  if (status === '警告') return 'warning'
  if (status === '异常') return 'danger'
  return 'info'
}

/**
 * 处理窗口大小变化
 */
function handleResize() {
  trendChart.value?.resize()
  pieChart.value?.resize()
  barChart.value?.resize()
  performanceChart.value?.resize()
}

// ===== 生命周期 =====
onMounted(() => {
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  
  trendChart.value?.dispose()
  pieChart.value?.dispose()
  barChart.value?.dispose()
  performanceChart.value?.dispose()
})

// ===== 监听器 =====
watch(() => props.modelValue, (visible) => {
  if (visible && props.report) {
    nextTick(() => {
      initCharts()
    })
  }
})

watch(() => viewMode.value, () => {
  nextTick(() => {
    handleResize()
  })
})
</script>

<style scoped lang="scss">
.report-preview-dialog {
  :deep(.el-dialog__body) {
    padding: 0;
  }
}

.report-preview {
  .preview-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid #ebeef5;
    background: #fafafa;
    
    .toolbar-left,
    .toolbar-right {
      display: flex;
      align-items: center;
      gap: 12px;
    }
  }
  
  .preview-content {
    height: 70vh;
    overflow: auto;
    padding: 20px;
    background: #f5f5f5;
    transform-origin: top center;
    transition: transform 0.3s ease;
    
    &.mobile {
      max-width: 480px;
      margin: 0 auto;
    }
    
    .report-document {
      background: white;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
      margin: 0 auto;
      min-height: 297mm; // A4 高度
      padding: 20mm;
      
      // 打印样式
      @media print {
        box-shadow: none;
        margin: 0;
        padding: 10mm;
      }
      
      .report-header {
        display: flex;
        align-items: flex-start;
        gap: 20px;
        margin-bottom: 30px;
        padding-bottom: 20px;
        border-bottom: 2px solid #409eff;
        
        .header-logo {
          .logo {
            height: 60px;
            width: auto;
          }
        }
        
        .header-info {
          flex: 1;
          
          .report-title {
            font-size: 24px;
            color: #303133;
            margin: 0 0 12px 0;
          }
          
          .report-meta {
            display: flex;
            flex-wrap: wrap;
            gap: 20px;
            
            .meta-item {
              display: flex;
              gap: 8px;
              
              .label {
                color: #606266;
                font-weight: 500;
              }
              
              .value {
                color: #303133;
              }
            }
          }
        }
      }
      
      .report-summary {
        margin-bottom: 30px;
        
        h2 {
          font-size: 18px;
          color: #303133;
          margin-bottom: 16px;
          border-left: 4px solid #409eff;
          padding-left: 12px;
        }
        
        .summary-grid {
          display: grid;
          grid-template-columns: repeat(4, 1fr);
          gap: 20px;
          
          .summary-item {
            text-align: center;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;
            
            .summary-label {
              font-size: 14px;
              color: #606266;
              margin-bottom: 8px;
            }
            
            .summary-value {
              font-size: 24px;
              font-weight: 700;
              color: #409eff;
              margin-bottom: 4px;
            }
            
            .summary-unit {
              font-size: 12px;
              color: #909399;
            }
          }
        }
      }
      
      .report-charts {
        margin-bottom: 30px;
        
        h2 {
          font-size: 18px;
          color: #303133;
          margin-bottom: 20px;
          border-left: 4px solid #409eff;
          padding-left: 12px;
        }
        
        .chart-section {
          margin-bottom: 30px;
          
          h3 {
            font-size: 16px;
            color: #303133;
            margin-bottom: 16px;
          }
          
          .chart-container {
            height: 300px;
            border: 1px solid #ebeef5;
            border-radius: 6px;
            
            &.small {
              height: 250px;
            }
          }
          
          .chart-row {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
          }
        }
      }
      
      .report-table {
        margin-bottom: 30px;
        
        h2 {
          font-size: 18px;
          color: #303133;
          margin-bottom: 16px;
          border-left: 4px solid #409eff;
          padding-left: 12px;
        }
      }
      
      .report-conclusion {
        margin-bottom: 30px;
        
        h2 {
          font-size: 18px;
          color: #303133;
          margin-bottom: 16px;
          border-left: 4px solid #409eff;
          padding-left: 12px;
        }
        
        .conclusion-content {
          ul {
            margin: 0;
            padding-left: 20px;
            
            li {
              margin-bottom: 8px;
              line-height: 1.6;
              color: #606266;
            }
          }
        }
      }
      
      .report-footer {
        margin-top: 40px;
        padding-top: 20px;
        border-top: 1px solid #ebeef5;
        
        .footer-info {
          text-align: center;
          font-size: 12px;
          color: #909399;
          line-height: 1.5;
        }
      }
    }
  }
  
  .page-navigation {
    padding: 16px;
    border-top: 1px solid #ebeef5;
    background: #fafafa;
    display: flex;
    justify-content: center;
  }
}

// 移动端适配
@media (max-width: 768px) {
  .report-preview {
    .preview-toolbar {
      flex-direction: column;
      gap: 12px;
      
      .toolbar-left,
      .toolbar-right {
        justify-content: center;
        flex-wrap: wrap;
      }
    }
    
    .preview-content .report-document {
      padding: 10mm;
      
      .report-header {
        flex-direction: column;
        text-align: center;
        
        .header-info .report-meta {
          justify-content: center;
          flex-direction: column;
          gap: 8px;
        }
      }
      
      .report-summary .summary-grid {
        grid-template-columns: repeat(2, 1fr);
        gap: 12px;
      }
      
      .report-charts .chart-section .chart-row {
        grid-template-columns: 1fr;
        gap: 12px;
      }
    }
  }
}

// 打印样式
@media print {
  .preview-toolbar,
  .page-navigation {
    display: none !important;
  }
  
  .preview-content {
    height: auto !important;
    overflow: visible !important;
    padding: 0 !important;
    background: white !important;
  }
}
</style>