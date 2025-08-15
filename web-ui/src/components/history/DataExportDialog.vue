<template>
  <el-dialog
    v-model="dialogVisible"
    title="数据导出"
    width="600px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @close="handleClose"
  >
    <div class="export-dialog">
      <!-- 导出配置 -->
      <div class="export-config">
        <div class="config-section">
          <h4 class="section-title">导出格式</h4>
          <el-radio-group v-model="exportConfig.format" class="format-options">
            <el-radio label="csv">
              <div class="format-option">
                <el-icon><Document /></el-icon>
                <div class="format-info">
                  <span class="format-name">CSV 文件</span>
                  <span class="format-desc">适用于 Excel 和数据分析工具</span>
                </div>
              </div>
            </el-radio>

            <el-radio label="excel">
              <div class="format-option">
                <el-icon><DocumentChecked /></el-icon>
                <div class="format-info">
                  <span class="format-name">Excel 文件</span>
                  <span class="format-desc">包含格式化和图表的 Excel 文件</span>
                </div>
              </div>
            </el-radio>

            <el-radio label="json">
              <div class="format-option">
                <el-icon><Collection /></el-icon>
                <div class="format-info">
                  <span class="format-name">JSON 文件</span>
                  <span class="format-desc">程序数据交换格式</span>
                </div>
              </div>
            </el-radio>
          </el-radio-group>
        </div>

        <div class="config-section">
          <h4 class="section-title">导出范围</h4>
          <el-radio-group v-model="exportConfig.scope">
            <el-radio label="current"
              >当前页数据 ({{ currentPageCount }} 条)</el-radio
            >
            <el-radio label="selected"
              >选中数据 ({{ selectedCount }} 条)</el-radio
            >
            <el-radio label="all">全部数据 ({{ totalCount }} 条)</el-radio>
          </el-radio-group>
        </div>

        <div class="config-section">
          <h4 class="section-title">导出选项</h4>
          <div class="export-options">
            <div class="option-group">
              <h5 class="group-title">数据列选择</h5>
              <el-checkbox-group
                v-model="exportConfig.columns"
                class="column-checkboxes"
              >
                <el-checkbox
                  v-for="column in availableColumns"
                  :key="column.key"
                  :label="column.key"
                >
                  {{ column.label }}
                </el-checkbox>
              </el-checkbox-group>
            </div>

            <div class="option-group">
              <h5 class="group-title">数据处理</h5>
              <div class="processing-options">
                <el-checkbox v-model="exportConfig.includeHeaders"
                  >包含列标题</el-checkbox
                >
                <el-checkbox v-model="exportConfig.formatNumbers"
                  >格式化数值</el-checkbox
                >
                <el-checkbox v-model="exportConfig.formatDates"
                  >格式化日期</el-checkbox
                >
                <el-checkbox v-model="exportConfig.includeMetadata"
                  >包含元数据</el-checkbox
                >
              </div>
            </div>

            <div v-if="exportConfig.format === 'csv'" class="option-group">
              <h5 class="group-title">CSV 设置</h5>
              <div class="csv-options">
                <div class="option-item">
                  <label class="option-label">分隔符</label>
                  <el-select
                    v-model="exportConfig.delimiter"
                    style="width: 120px"
                  >
                    <el-option label="逗号 (,)" value="," />
                    <el-option label="分号 (;)" value=";" />
                    <el-option label="制表符" value="\t" />
                  </el-select>
                </div>

                <div class="option-item">
                  <label class="option-label">编码</label>
                  <el-select
                    v-model="exportConfig.encoding"
                    style="width: 120px"
                  >
                    <el-option label="UTF-8" value="utf-8" />
                    <el-option label="GBK" value="gbk" />
                    <el-option label="UTF-8 BOM" value="utf-8-bom" />
                  </el-select>
                </div>
              </div>
            </div>

            <div v-if="exportConfig.format === 'excel'" class="option-group">
              <h5 class="group-title">Excel 设置</h5>
              <div class="excel-options">
                <el-checkbox v-model="exportConfig.createChart"
                  >生成图表</el-checkbox
                >
                <el-checkbox v-model="exportConfig.applyStyles"
                  >应用样式</el-checkbox
                >
                <el-checkbox v-model="exportConfig.multipleSheets"
                  >分表导出</el-checkbox
                >
              </div>
            </div>
          </div>
        </div>

        <!-- 文件名设置 -->
        <div class="config-section">
          <h4 class="section-title">文件设置</h4>
          <div class="filename-config">
            <div class="filename-input">
              <label class="option-label">文件名</label>
              <el-input
                v-model="exportConfig.filename"
                placeholder="请输入文件名"
                style="width: 300px"
              >
                <template #suffix>
                  <span class="file-extension">.{{ getFileExtension() }}</span>
                </template>
              </el-input>
            </div>

            <div class="filename-tips">
              <el-icon><InfoFilled /></el-icon>
              <span>文件名不能包含以下字符: \ / : * ? " < > |</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 导出预览 -->
      <div v-if="previewData.length > 0" class="export-preview">
        <h4 class="section-title">数据预览 (前10行)</h4>
        <div class="preview-table">
          <el-table
            :data="previewData"
            size="small"
            border
            stripe
            max-height="200"
          >
            <el-table-column
              v-for="column in previewColumns"
              :key="column.key"
              :prop="column.key"
              :label="column.label"
              :width="column.width"
              show-overflow-tooltip
            />
          </el-table>
        </div>
      </div>

      <!-- 导出进度 -->
      <div v-if="exporting" class="export-progress">
        <div class="progress-info">
          <el-icon class="is-loading"><Loading /></el-icon>
          <span>正在导出数据...</span>
        </div>
        <el-progress
          :percentage="exportProgress"
          :show-text="true"
          :format="formatProgress"
        />
        <div class="progress-stats">
          <span
            >已处理: {{ processedCount }} / {{ totalExportCount }} 条记录</span
          >
          <span>预计剩余时间: {{ estimatedTime }}</span>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <div class="footer-info">
          <span class="export-size">预计文件大小: {{ estimatedSize }}</span>
        </div>

        <div class="footer-actions">
          <el-button :disabled="exporting" @click="handleClose">取消</el-button>
          <el-button :disabled="exporting" @click="generatePreview"
            >预览</el-button
          >
          <el-button
            type="primary"
            :loading="exporting"
            :disabled="!canExport"
            @click="startExport"
          >
            {{ exporting ? '导出中...' : '开始导出' }}
          </el-button>
        </div>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * DataExportDialog —— 数据导出对话框组件
 *
 * 📝 Responsibilities:
 *  1. 多格式数据导出 (CSV, Excel, JSON)
 *  2. 导出配置和选项设置
 *  3. 数据预览和进度显示
 *  4. 文件下载处理
 *  5. 导出状态管理
 *
 * 📦 Dependencies:
 *  - Element Plus Dialog/Form
 *  - XLSX 或 ExcelJS (Excel导出)
 *  - File-saver (文件下载)
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Document,
  DocumentChecked,
  Collection,
  InfoFilled,
  Loading,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, watch, nextTick } from 'vue'

import { formatDateTime, formatFilename } from '@/utils/date'
import { formatNumber, formatBytes } from '@/utils/format'

// ===== Props & Emits =====
const props = defineProps<{
  visible: boolean
  data: any[]
  queryForm?: any
  selectedData?: any[]
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  export: [config: any]
}>()

// ===== 响应式数据 =====
const dialogVisible = ref(props.visible)
const exporting = ref(false)
const exportProgress = ref(0)
const processedCount = ref(0)
const totalExportCount = ref(0)

// 导出配置
const exportConfig = ref({
  format: 'csv',
  scope: 'current',
  columns: [] as string[],
  includeHeaders: true,
  formatNumbers: true,
  formatDates: true,
  includeMetadata: false,
  filename: '',
  delimiter: ',',
  encoding: 'utf-8',
  createChart: false,
  applyStyles: true,
  multipleSheets: false,
})

// 预览数据
const previewData = ref([])
const previewColumns = ref([])

// 可用列配置
const availableColumns = [
  { key: 'timestamp', label: '时间戳', width: 160 },
  { key: 'deviceName', label: '设备名称', width: 120 },
  { key: 'tagName', label: '数据点位', width: 150 },
  { key: 'tagAddress', label: '点位地址', width: 120 },
  { key: 'value', label: '数值', width: 100 },
  { key: 'unit', label: '单位', width: 80 },
  { key: 'dataType', label: '数据类型', width: 100 },
  { key: 'quality', label: '数据质量', width: 100 },
  { key: 'rawValue', label: '原始值', width: 100 },
]

// ===== 计算属性 =====
const currentPageCount = computed(() => {
  return props.data ? props.data.length : 0
})

const selectedCount = computed(() => {
  return props.selectedData ? props.selectedData.length : 0
})

const totalCount = computed(() => {
  // 这里应该从查询结果中获取总记录数
  return currentPageCount.value
})

const canExport = computed(() => {
  return (
    exportConfig.value.columns.length > 0 &&
    exportConfig.value.filename.trim() !== '' &&
    getExportDataCount() > 0
  )
})

const estimatedSize = computed(() => {
  const recordCount = getExportDataCount()
  const columnCount = exportConfig.value.columns.length
  const avgDataSize = 50 // 平均每个字段50字节

  const sizeBytes = recordCount * columnCount * avgDataSize
  return formatBytes(sizeBytes)
})

const estimatedTime = computed(() => {
  if (!exporting.value || exportProgress.value === 0) return '--'

  const elapsed = Date.now() - exportStartTime.value
  const remaining =
    (elapsed / exportProgress.value) * (100 - exportProgress.value)

  return `${Math.ceil(remaining / 1000)}秒`
})

// ===== 内部数据 =====
const exportStartTime = ref(0)

// ===== 方法 =====

/**
 * 获取文件扩展名
 */
function getFileExtension(): string {
  const extensions: Record<string, string> = {
    csv: 'csv',
    excel: 'xlsx',
    json: 'json',
  }
  return extensions[exportConfig.value.format] || 'txt'
}

/**
 * 获取导出数据数量
 */
function getExportDataCount(): number {
  switch (exportConfig.value.scope) {
    case 'current':
      return currentPageCount.value
    case 'selected':
      return selectedCount.value
    case 'all':
      return totalCount.value
    default:
      return 0
  }
}

/**
 * 格式化进度文本
 */
function formatProgress(percentage: number): string {
  return `${percentage}%`
}

/**
 * 生成预览数据
 */
function generatePreview() {
  if (!props.data || props.data.length === 0) {
    ElMessage.warning('没有可预览的数据')
    return
  }

  // 获取要导出的数据
  let dataToPreview = getExportData()

  // 取前10行作为预览
  dataToPreview = dataToPreview.slice(0, 10)

  // 根据选中的列筛选数据
  const filteredData = dataToPreview.map(row => {
    const filteredRow: any = {}
    exportConfig.value.columns.forEach(columnKey => {
      if (row.hasOwnProperty(columnKey)) {
        filteredRow[columnKey] = formatCellValue(row[columnKey], columnKey)
      }
    })
    return filteredRow
  })

  // 设置预览列
  previewColumns.value = availableColumns.filter(col =>
    exportConfig.value.columns.includes(col.key)
  )

  previewData.value = filteredData
  ElMessage.success('预览数据已生成')
}

/**
 * 获取要导出的数据
 */
function getExportData(): any[] {
  switch (exportConfig.value.scope) {
    case 'current':
      return props.data || []
    case 'selected':
      return props.selectedData || []
    case 'all':
      // 这里应该重新查询全部数据
      return props.data || []
    default:
      return []
  }
}

/**
 * 格式化单元格值
 */
function formatCellValue(value: any, columnKey: string): string {
  if (value === null || value === undefined) return ''

  switch (columnKey) {
    case 'timestamp':
      return exportConfig.value.formatDates
        ? formatDateTime(value, 'YYYY-MM-DD HH:mm:ss')
        : String(value)
    case 'value':
    case 'rawValue':
      return exportConfig.value.formatNumbers
        ? formatNumber(parseFloat(value), 2)
        : String(value)
    case 'quality':
      const qualityMap: Record<string, string> = {
        good: '良好',
        uncertain: '可疑',
        bad: '错误',
      }
      return qualityMap[value] || value
    case 'dataType':
      const typeMap: Record<string, string> = {
        boolean: '布尔',
        string: '字符串',
        integer: '整数',
        float: '浮点',
      }
      return typeMap[value] || value
    default:
      return String(value)
  }
}

/**
 * 开始导出
 */
async function startExport() {
  if (!canExport.value) {
    ElMessage.warning('请完善导出配置')
    return
  }

  exporting.value = true
  exportProgress.value = 0
  processedCount.value = 0
  exportStartTime.value = Date.now()

  try {
    const exportData = getExportData()
    totalExportCount.value = exportData.length

    // 模拟导出进度
    await simulateExportProgress()

    // 根据格式进行导出
    switch (exportConfig.value.format) {
      case 'csv':
        await exportToCSV(exportData)
        break
      case 'excel':
        await exportToExcel(exportData)
        break
      case 'json':
        await exportToJSON(exportData)
        break
    }

    ElMessage.success('数据导出成功！')
    emit('export', { ...exportConfig.value, success: true })
    handleClose()
  } catch (error) {
    console.error('数据导出失败:', error)
    ElMessage.error('数据导出失败')
  } finally {
    exporting.value = false
    exportProgress.value = 0
    processedCount.value = 0
  }
}

/**
 * 模拟导出进度
 */
async function simulateExportProgress() {
  const steps = 20
  const stepSize = totalExportCount.value / steps

  for (let i = 0; i < steps; i++) {
    await new Promise(resolve => setTimeout(resolve, 100))
    exportProgress.value = Math.round((i + 1) * 5)
    processedCount.value = Math.min(
      Math.round((i + 1) * stepSize),
      totalExportCount.value
    )
  }
}

/**
 * 导出为 CSV
 */
async function exportToCSV(data: any[]) {
  let csvContent = ''

  // 添加列标题
  if (exportConfig.value.includeHeaders) {
    const headers = exportConfig.value.columns.map(columnKey => {
      const column = availableColumns.find(col => col.key === columnKey)
      return column ? column.label : columnKey
    })
    csvContent += `${headers.join(exportConfig.value.delimiter)}\n`
  }

  // 添加数据行
  data.forEach(row => {
    const values = exportConfig.value.columns.map(columnKey => {
      const value = formatCellValue(row[columnKey], columnKey)
      // CSV 值包含分隔符时需要加引号
      return value.includes(exportConfig.value.delimiter) ? `"${value}"` : value
    })
    csvContent += `${values.join(exportConfig.value.delimiter)}\n`
  })

  // 下载文件
  downloadFile(csvContent, `${exportConfig.value.filename}.csv`, 'text/csv')
}

/**
 * 导出为 Excel
 */
async function exportToExcel(data: any[]) {
  // 这里需要集成 XLSX 或 ExcelJS 库
  // 为了简化，暂时使用 CSV 格式
  ElMessage.info('Excel 导出功能开发中，当前使用 CSV 格式')
  await exportToCSV(data)
}

/**
 * 导出为 JSON
 */
async function exportToJSON(data: any[]) {
  const exportData = {
    metadata: {
      exportTime: new Date().toISOString(),
      recordCount: data.length,
      columns: exportConfig.value.columns,
      query: props.queryForm || {},
    },
    data: data.map(row => {
      const filteredRow: any = {}
      exportConfig.value.columns.forEach(columnKey => {
        if (row.hasOwnProperty(columnKey)) {
          filteredRow[columnKey] = row[columnKey]
        }
      })
      return filteredRow
    }),
  }

  const jsonContent = JSON.stringify(exportData, null, 2)
  downloadFile(
    jsonContent,
    `${exportConfig.value.filename}.json`,
    'application/json'
  )
}

/**
 * 下载文件
 */
function downloadFile(content: string, filename: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType })
  const url = window.URL.createObjectURL(blob)

  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()

  document.body.removeChild(link)
  window.URL.revokeObjectURL(url)
}

/**
 * 处理对话框关闭
 */
function handleClose() {
  if (exporting.value) {
    ElMessageBox.confirm('导出正在进行中，确定要取消吗？', '确认取消', {
      type: 'warning',
    })
      .then(() => {
        exporting.value = false
        dialogVisible.value = false
      })
      .catch(() => {
        // 用户取消关闭
      })
  } else {
    dialogVisible.value = false
  }
}

// ===== 监听器 =====
watch(
  () => props.visible,
  visible => {
    dialogVisible.value = visible
  }
)

watch(dialogVisible, visible => {
  emit('update:visible', visible)

  if (visible) {
    // 对话框打开时初始化配置
    exportConfig.value.columns = availableColumns.map(col => col.key)
    exportConfig.value.filename = formatFilename(
      `历史数据_${formatDateTime(new Date(), 'YYYYMMDD')}`
    )
  } else {
    // 对话框关闭时重置状态
    previewData.value = []
    previewColumns.value = []
    exporting.value = false
    exportProgress.value = 0
  }
})

watch(
  () => exportConfig.value.format,
  () => {
    // 格式变化时更新文件名扩展名
    const baseName = exportConfig.value.filename.replace(/\.[^/.]+$/, '')
    exportConfig.value.filename = baseName
  }
)
</script>

<style scoped lang="scss">
.export-dialog {
  .export-config {
    .config-section {
      margin-bottom: 24px;

      &:last-child {
        margin-bottom: 0;
      }

      .section-title {
        font-size: 16px;
        font-weight: 600;
        color: #303133;
        margin: 0 0 12px 0;
      }
    }

    .format-options {
      display: flex;
      flex-direction: column;
      gap: 12px;

      .el-radio {
        margin-right: 0;

        .format-option {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px;
          border-radius: 4px;
          transition: background-color 0.2s;

          &:hover {
            background: #f5f7fa;
          }

          .el-icon {
            font-size: 20px;
            color: #409eff;
          }

          .format-info {
            display: flex;
            flex-direction: column;

            .format-name {
              font-weight: 500;
              color: #303133;
            }

            .format-desc {
              font-size: 12px;
              color: #909399;
              margin-top: 2px;
            }
          }
        }
      }
    }

    .export-options {
      .option-group {
        margin-bottom: 16px;

        &:last-child {
          margin-bottom: 0;
        }

        .group-title {
          font-size: 14px;
          font-weight: 500;
          color: #606266;
          margin: 0 0 8px 0;
        }

        .column-checkboxes {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
          gap: 8px;
        }

        .processing-options {
          display: flex;
          flex-wrap: wrap;
          gap: 16px;
        }

        .csv-options,
        .excel-options {
          display: flex;
          gap: 16px;
          align-items: center;
          flex-wrap: wrap;

          .option-item {
            display: flex;
            align-items: center;
            gap: 8px;

            .option-label {
              font-size: 13px;
              color: #606266;
              white-space: nowrap;
            }
          }
        }
      }
    }

    .filename-config {
      .filename-input {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 8px;

        .option-label {
          font-size: 13px;
          color: #606266;
          white-space: nowrap;
        }

        .file-extension {
          color: #909399;
          font-size: 12px;
        }
      }

      .filename-tips {
        display: flex;
        align-items: center;
        gap: 4px;
        font-size: 12px;
        color: #e6a23c;

        .el-icon {
          font-size: 14px;
        }
      }
    }
  }

  .export-preview {
    margin-top: 24px;
    padding-top: 24px;
    border-top: 1px solid #ebeef5;

    .section-title {
      font-size: 16px;
      font-weight: 600;
      color: #303133;
      margin: 0 0 12px 0;
    }

    .preview-table {
      border-radius: 6px;
      overflow: hidden;
    }
  }

  .export-progress {
    margin-top: 24px;
    padding: 20px;
    background: #f8f9fa;
    border-radius: 6px;

    .progress-info {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 12px;

      .el-icon {
        font-size: 16px;
        color: #409eff;
      }

      span {
        font-weight: 500;
        color: #303133;
      }
    }

    .progress-stats {
      display: flex;
      justify-content: space-between;
      margin-top: 8px;
      font-size: 12px;
      color: #606266;
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;

  .footer-info {
    .export-size {
      font-size: 13px;
      color: #606266;
    }
  }

  .footer-actions {
    display: flex;
    gap: 12px;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .export-dialog {
    .export-config {
      .format-options {
        .el-radio .format-option {
          padding: 12px 8px;

          .format-info {
            .format-name {
              font-size: 14px;
            }

            .format-desc {
              font-size: 11px;
            }
          }
        }
      }

      .export-options .option-group {
        .column-checkboxes {
          grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
        }

        .processing-options,
        .csv-options,
        .excel-options {
          flex-direction: column;
          align-items: stretch;
          gap: 12px;
        }
      }

      .filename-config .filename-input {
        flex-direction: column;
        align-items: stretch;
        gap: 8px;

        :deep(.el-input) {
          width: 100% !important;
        }
      }
    }
  }

  .dialog-footer {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;

    .footer-info,
    .footer-actions {
      text-align: center;
    }
  }
}
</style>
