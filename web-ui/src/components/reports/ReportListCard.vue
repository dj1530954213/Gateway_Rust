<template>
  <div class="report-list-card">
    <div class="card-header">
      <h4>报表列表</h4>
      <div class="header-actions">
        <el-button type="primary" @click="showGenerator = true">
          <el-icon><Plus /></el-icon>
          生成报表
        </el-button>
        <el-button @click="refreshReports">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- 筛选器 -->
    <div class="filters">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-select
            v-model="filters.template"
            placeholder="模板类型"
            clearable
          >
            <el-option
              v-for="template in templateOptions"
              :key="template.value"
              :label="template.label"
              :value="template.value"
            />
          </el-select>
        </el-col>
        <el-col :span="6">
          <el-select v-model="filters.status" placeholder="状态" clearable>
            <el-option label="已完成" value="completed" />
            <el-option label="生成中" value="generating" />
            <el-option label="失败" value="failed" />
            <el-option label="已过期" value="expired" />
          </el-select>
        </el-col>
        <el-col :span="6">
          <el-date-picker
            v-model="filters.dateRange"
            type="daterange"
            range-separator="至"
            start-placeholder="开始日期"
            end-placeholder="结束日期"
            format="YYYY-MM-DD"
            value-format="YYYY-MM-DD"
          />
        </el-col>
        <el-col :span="6">
          <el-input
            v-model="filters.keyword"
            placeholder="搜索报表名称"
            clearable
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </el-col>
      </el-row>
    </div>

    <!-- 报表列表 -->
    <div class="report-list">
      <div
        v-for="report in filteredReports"
        :key="report.id"
        class="report-item"
        @click="handleReportClick(report)"
      >
        <div class="report-icon">
          <el-icon>
            <component :is="getReportIcon(report.template)" />
          </el-icon>
        </div>

        <div class="report-info">
          <div class="report-name">{{ report.name }}</div>
          <div class="report-meta">
            <span class="template">{{ getTemplateName(report.template) }}</span>
            <span class="separator">·</span>
            <span class="date">{{ formatDate(report.generatedAt) }}</span>
            <span class="separator">·</span>
            <span class="size">{{ report.fileSize }}</span>
          </div>
          <div class="report-desc">{{ report.description }}</div>
        </div>

        <div class="report-status">
          <el-tag :type="getStatusType(report.status)" size="small">
            {{ getStatusText(report.status) }}
          </el-tag>
        </div>

        <div class="report-actions" @click.stop>
          <!-- 下载按钮 -->
          <el-dropdown
            v-if="report.status === 'completed'"
            trigger="click"
            @command="command => handleDownload(report, command)"
          >
            <el-button type="primary" size="small">
              <el-icon><Download /></el-icon>
              下载
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

          <!-- 重新生成按钮 -->
          <el-button
            v-if="report.status === 'failed'"
            type="warning"
            size="small"
            @click="handleRegenerate(report)"
          >
            <el-icon><RefreshRight /></el-icon>
            重新生成
          </el-button>

          <!-- 进度条 -->
          <div
            v-if="report.status === 'generating'"
            class="generation-progress"
          >
            <el-progress
              :percentage="report.progress || 0"
              :stroke-width="4"
              size="small"
            />
          </div>

          <!-- 更多操作 -->
          <el-dropdown
            trigger="click"
            @command="command => handleAction(report, command)"
          >
            <el-button size="small" text>
              <el-icon><MoreFilled /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="preview">预览</el-dropdown-item>
                <el-dropdown-item command="share">分享</el-dropdown-item>
                <el-dropdown-item command="schedule">定时生成</el-dropdown-item>
                <el-dropdown-item command="copy">复制配置</el-dropdown-item>
                <el-dropdown-item command="delete" divided
                  >删除</el-dropdown-item
                >
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>

      <!-- 空状态 -->
      <el-empty v-if="filteredReports.length === 0" description="暂无报表数据">
        <el-button type="primary" @click="showGenerator = true">
          生成第一个报表
        </el-button>
      </el-empty>
    </div>

    <!-- 分页 -->
    <div v-if="filteredReports.length > 0" class="pagination">
      <el-pagination
        v-model:current-page="pagination.current"
        v-model:page-size="pagination.size"
        :page-sizes="[10, 20, 50, 100]"
        :total="pagination.total"
        layout="total, sizes, prev, pager, next, jumper"
        @size-change="handleSizeChange"
        @current-change="handleCurrentChange"
      />
    </div>

    <!-- 报表生成器 -->
    <ReportGeneratorDialog
      v-model="showGenerator"
      @report-generated="handleReportGenerated"
    />

    <!-- 报表预览对话框 -->
    <ReportPreviewDialog v-model="showPreview" :report="selectedReport" />

    <!-- 分享对话框 -->
    <ReportShareDialog v-model="showShare" :report="selectedReport" />

    <!-- 定时任务对话框 -->
    <ReportScheduleDialog v-model="showSchedule" :report="selectedReport" />
  </div>
</template>

<script setup lang="ts">
/**
 * ReportListCard —— 报表列表卡片组件
 *
 * 📝 Responsibilities:
 *  1. 报表列表展示和管理
 *  2. 报表筛选和搜索
 *  3. 报表下载和分享
 *  4. 报表生成和调度
 *
 * 📦 Dependencies:
 *  - Element Plus UI组件
 *  - ReportGeneratorDialog 报表生成器
 *  - ReportPreviewDialog 报表预览
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Plus,
  Refresh,
  Search,
  Download,
  RefreshRight,
  MoreFilled,
  Document,
  DataAnalysis,
  TrendCharts,
  PieChart,
  Grid,
  Monitor,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, onMounted } from 'vue'

import ReportGeneratorDialog from './ReportGeneratorDialog.vue'
import ReportPreviewDialog from './ReportPreviewDialog.vue'
import ReportScheduleDialog from './ReportScheduleDialog.vue'
import ReportShareDialog from './ReportShareDialog.vue'

// ===== 响应式数据 =====
const showGenerator = ref(false)
const showPreview = ref(false)
const showShare = ref(false)
const showSchedule = ref(false)
const selectedReport = ref(null)

// 筛选器
const filters = ref({
  template: '',
  status: '',
  dateRange: [],
  keyword: '',
})

// 分页
const pagination = ref({
  current: 1,
  size: 20,
  total: 0,
})

// 模板选项
const templateOptions = ref([
  { label: '设备概览报表', value: 'device_summary' },
  { label: '数据分析报表', value: 'data_analysis' },
  { label: '趋势分析报表', value: 'trend_report' },
  { label: '状态分布报表', value: 'status_report' },
  { label: '性能评估报表', value: 'performance_report' },
  { label: '自定义报表', value: 'custom_report' },
])

// 报表数据
const reports = ref([
  {
    id: '1',
    name: '设备状态月度报表',
    template: 'device_summary',
    description: '2025年7月设备运行状态综合分析',
    status: 'completed',
    generatedAt: '2025-07-27T10:30:00Z',
    fileSize: '2.3 MB',
    formats: ['pdf', 'excel'],
    progress: 100,
    downloads: 15,
  },
  {
    id: '2',
    name: '数据质量分析报表',
    template: 'data_analysis',
    description: '数据完整性和准确性评估',
    status: 'generating',
    generatedAt: '2025-07-27T11:00:00Z',
    fileSize: '',
    formats: ['pdf'],
    progress: 65,
    downloads: 0,
  },
  {
    id: '3',
    name: '温度趋势分析',
    template: 'trend_report',
    description: '过去30天温度数据趋势分析',
    status: 'completed',
    generatedAt: '2025-07-26T15:20:00Z',
    fileSize: '1.8 MB',
    formats: ['pdf', 'csv'],
    progress: 100,
    downloads: 8,
  },
  {
    id: '4',
    name: '异常事件统计',
    template: 'status_report',
    description: '设备异常事件分布统计',
    status: 'failed',
    generatedAt: '2025-07-26T09:15:00Z',
    fileSize: '',
    formats: ['excel'],
    progress: 0,
    downloads: 0,
  },
  {
    id: '5',
    name: '系统性能评估',
    template: 'performance_report',
    description: '系统整体性能指标评估',
    status: 'expired',
    generatedAt: '2025-07-20T14:00:00Z',
    fileSize: '3.1 MB',
    formats: ['pdf'],
    progress: 100,
    downloads: 22,
  },
])

// ===== 计算属性 =====
const filteredReports = computed(() => {
  let result = [...reports.value]

  // 模板筛选
  if (filters.value.template) {
    result = result.filter(report => report.template === filters.value.template)
  }

  // 状态筛选
  if (filters.value.status) {
    result = result.filter(report => report.status === filters.value.status)
  }

  // 日期范围筛选
  if (filters.value.dateRange && filters.value.dateRange.length === 2) {
    const [start, end] = filters.value.dateRange
    result = result.filter(report => {
      const reportDate = new Date(report.generatedAt).toISOString().slice(0, 10)
      return reportDate >= start && reportDate <= end
    })
  }

  // 关键词搜索
  if (filters.value.keyword) {
    const keyword = filters.value.keyword.toLowerCase()
    result = result.filter(
      report =>
        report.name.toLowerCase().includes(keyword) ||
        report.description.toLowerCase().includes(keyword)
    )
  }

  // 更新分页总数
  pagination.value.total = result.length

  // 分页处理
  const start = (pagination.value.current - 1) * pagination.value.size
  const end = start + pagination.value.size
  return result.slice(start, end)
})

// ===== 方法 =====

/**
 * 获取报表图标
 */
function getReportIcon(template: string): string {
  const iconMap: { [key: string]: string } = {
    device_summary: 'Monitor',
    data_analysis: 'DataAnalysis',
    trend_report: 'TrendCharts',
    status_report: 'PieChart',
    performance_report: 'Grid',
    custom_report: 'Document',
  }
  return iconMap[template] || 'Document'
}

/**
 * 获取模板名称
 */
function getTemplateName(template: string): string {
  const option = templateOptions.value.find(opt => opt.value === template)
  return option?.label || template
}

/**
 * 获取状态类型
 */
function getStatusType(status: string): string {
  const typeMap: { [key: string]: string } = {
    completed: 'success',
    generating: 'warning',
    failed: 'danger',
    expired: 'info',
  }
  return typeMap[status] || 'info'
}

/**
 * 获取状态文本
 */
function getStatusText(status: string): string {
  const textMap: { [key: string]: string } = {
    completed: '已完成',
    generating: '生成中',
    failed: '失败',
    expired: '已过期',
  }
  return textMap[status] || status
}

/**
 * 格式化日期
 */
function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/**
 * 事件处理
 */
function handleReportClick(report: any) {
  selectedReport.value = report
  if (report.status === 'completed') {
    showPreview.value = true
  }
}

function handleDownload(report: any, format: string) {
  // 模拟下载
  ElMessage.success(`开始下载 ${report.name} (${format.toUpperCase()})`)

  // 增加下载次数
  const index = reports.value.findIndex(r => r.id === report.id)
  if (index !== -1) {
    reports.value[index].downloads++
  }
}

function handleRegenerate(report: any) {
  ElMessageBox.confirm(
    '确定要重新生成此报表吗？这将覆盖之前的配置。',
    '重新生成报表',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    }
  )
    .then(() => {
      // 重新生成逻辑
      const index = reports.value.findIndex(r => r.id === report.id)
      if (index !== -1) {
        reports.value[index].status = 'generating'
        reports.value[index].progress = 0

        // 模拟生成过程
        simulateGeneration(reports.value[index])
      }

      ElMessage.success('开始重新生成报表')
    })
    .catch(() => {
      // 取消操作
    })
}

function handleAction(report: any, command: string) {
  selectedReport.value = report

  switch (command) {
    case 'preview':
      if (report.status === 'completed') {
        showPreview.value = true
      } else {
        ElMessage.warning('报表尚未生成完成')
      }
      break

    case 'share':
      if (report.status === 'completed') {
        showShare.value = true
      } else {
        ElMessage.warning('只能分享已完成的报表')
      }
      break

    case 'schedule':
      showSchedule.value = true
      break

    case 'copy':
      // 复制配置逻辑
      ElMessage.success('报表配置已复制，可在生成新报表时使用')
      break

    case 'delete':
      handleDeleteReport(report)
      break
  }
}

function handleDeleteReport(report: any) {
  ElMessageBox.confirm(
    `确定要删除报表 "${report.name}" 吗？此操作不可恢复。`,
    '删除报表',
    {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    }
  )
    .then(() => {
      const index = reports.value.findIndex(r => r.id === report.id)
      if (index !== -1) {
        reports.value.splice(index, 1)
        ElMessage.success('报表已删除')
      }
    })
    .catch(() => {
      // 取消操作
    })
}

function handleReportGenerated(report: any) {
  // 添加新生成的报表到列表
  const newReport = {
    ...report,
    status: 'generating',
    progress: 0,
    downloads: 0,
    fileSize: '',
  }

  reports.value.unshift(newReport)

  // 模拟生成过程
  simulateGeneration(newReport)
}

function simulateGeneration(report: any) {
  const interval = setInterval(() => {
    if (report.progress >= 100) {
      clearInterval(interval)
      report.status = 'completed'
      report.fileSize = `${(Math.random() * 3 + 1).toFixed(1)} MB`
      return
    }

    report.progress += Math.random() * 15
    if (report.progress > 100) {
      report.progress = 100
    }
  }, 500)
}

function refreshReports() {
  ElMessage.success('报表列表已刷新')
  // 这里可以重新获取数据
}

function handleSizeChange(size: number) {
  pagination.value.size = size
  pagination.value.current = 1
}

function handleCurrentChange(current: number) {
  pagination.value.current = current
}

// ===== 生命周期 =====
onMounted(() => {
  pagination.value.total = reports.value.length
})
</script>

<style scoped lang="scss">
.report-list-card {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;

    h4 {
      margin: 0;
      color: #303133;
      font-size: 18px;
    }

    .header-actions {
      display: flex;
      gap: 12px;
    }
  }

  .filters {
    margin-bottom: 20px;
    padding: 16px;
    background: #fafafa;
    border-radius: 6px;
  }

  .report-list {
    .report-item {
      display: flex;
      align-items: center;
      gap: 16px;
      padding: 16px;
      border: 1px solid #ebeef5;
      border-radius: 8px;
      margin-bottom: 12px;
      cursor: pointer;
      transition: all 0.3s;

      &:hover {
        border-color: #c6e2ff;
        background: #f0f9ff;
      }

      .report-icon {
        font-size: 32px;
        color: #409eff;
        flex-shrink: 0;
      }

      .report-info {
        flex: 1;
        min-width: 0;

        .report-name {
          font-size: 16px;
          font-weight: 600;
          color: #303133;
          margin-bottom: 4px;

          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .report-meta {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 12px;
          color: #909399;
          margin-bottom: 4px;

          .separator {
            color: #dcdfe6;
          }
        }

        .report-desc {
          font-size: 14px;
          color: #606266;
          line-height: 1.4;

          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
      }

      .report-status {
        flex-shrink: 0;
      }

      .report-actions {
        display: flex;
        align-items: center;
        gap: 8px;
        flex-shrink: 0;

        .generation-progress {
          width: 120px;
        }
      }
    }
  }

  .pagination {
    margin-top: 20px;
    display: flex;
    justify-content: center;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .report-list-card {
    .card-header {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;

      .header-actions {
        justify-content: center;
      }
    }

    .filters {
      :deep(.el-row) {
        flex-direction: column;
        gap: 8px;
      }
    }

    .report-list .report-item {
      flex-direction: column;
      align-items: stretch;
      gap: 12px;

      .report-icon {
        align-self: center;
      }

      .report-info {
        text-align: center;

        .report-meta {
          justify-content: center;
        }
      }

      .report-status {
        align-self: center;
      }

      .report-actions {
        justify-content: center;
      }
    }
  }
}
</style>
