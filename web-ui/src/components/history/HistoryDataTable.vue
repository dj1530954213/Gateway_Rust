<template>
  <div class="history-data-table">
    <!-- 表格工具栏 -->
    <div class="table-toolbar">
      <div class="toolbar-left">
        <div class="table-info">
          <span class="info-text">
            共 {{ pagination.total }} 条记录
            <template v-if="selectedRows.length > 0">
              ，已选择 {{ selectedRows.length }} 条
            </template>
          </span>
        </div>
      </div>

      <div class="toolbar-right">
        <div class="table-actions">
          <el-tooltip content="列设置">
            <el-button
              type="text"
              size="small"
              @click="showColumnSettings = true"
            >
              <el-icon><Setting /></el-icon>
            </el-button>
          </el-tooltip>

          <el-tooltip content="导出选中">
            <el-button
              type="text"
              size="small"
              :disabled="selectedRows.length === 0"
              @click="exportSelected"
            >
              <el-icon><Download /></el-icon>
            </el-button>
          </el-tooltip>

          <el-tooltip content="刷新数据">
            <el-button type="text" size="small" @click="handleRefresh">
              <el-icon><Refresh /></el-icon>
            </el-button>
          </el-tooltip>
        </div>

        <div class="page-size-selector">
          <span class="size-label">每页</span>
          <el-select
            v-model="currentPageSize"
            size="small"
            style="width: 80px"
            @change="handlePageSizeChange"
          >
            <el-option label="20" :value="20" />
            <el-option label="50" :value="50" />
            <el-option label="100" :value="100" />
            <el-option label="200" :value="200" />
          </el-select>
          <span class="size-label">条</span>
        </div>
      </div>
    </div>

    <!-- 数据表格 -->
    <div class="table-container">
      <ElTable
        ref="tableRef"
        :data="tableData"
        :loading="loading"
        height="500"
        stripe
        border
        @selection-change="handleSelectionChange"
        @sort-change="handleSortChange"
        @row-click="handleRowClick"
      >
        <!-- 选择列 -->
        <el-table-column
          type="selection"
          width="50"
          :selectable="isRowSelectable"
          fixed="left"
        />

        <!-- 序号列 -->
        <el-table-column
          type="index"
          label="序号"
          width="60"
          :index="getRowIndex"
          fixed="left"
        />

        <!-- 时间戳列 -->
        <el-table-column
          prop="timestamp"
          label="时间戳"
          width="180"
          sortable="custom"
          fixed="left"
        >
          <template #default="{ row }">
            <div class="timestamp-cell">
              <span class="timestamp-main">{{
                formatDateTime(row.timestamp, 'MM-DD HH:mm:ss')
              }}</span>
              <span class="timestamp-sub">{{
                formatDateTime(row.timestamp, 'YYYY')
              }}</span>
            </div>
          </template>
        </el-table-column>

        <!-- 设备名称列 -->
        <el-table-column
          v-if="visibleColumns.includes('deviceName')"
          prop="deviceName"
          label="设备名称"
          width="150"
          sortable="custom"
        >
          <template #default="{ row }">
            <div class="device-cell">
              <el-tag size="small" type="info">{{ row.deviceName }}</el-tag>
            </div>
          </template>
        </el-table-column>

        <!-- 标签名称列 -->
        <el-table-column
          prop="tagName"
          label="数据点位"
          width="180"
          sortable="custom"
        >
          <template #default="{ row }">
            <div class="tag-cell">
              <div class="tag-name">{{ row.tagName }}</div>
              <div class="tag-address">{{ row.tagAddress }}</div>
            </div>
          </template>
        </el-table-column>

        <!-- 数值列 -->
        <el-table-column
          prop="value"
          label="数值"
          width="120"
          sortable="custom"
        >
          <template #default="{ row }">
            <div class="value-cell">
              <span
                class="value-number"
                :class="getValueClass(row.value, row.dataType)"
              >
                {{ formatValue(row.value, row.dataType, row.unit) }}
              </span>
            </div>
          </template>
        </el-table-column>

        <!-- 单位列 -->
        <el-table-column
          v-if="visibleColumns.includes('unit')"
          prop="unit"
          label="单位"
          width="80"
        >
          <template #default="{ row }">
            <span class="unit-text">{{ row.unit || '-' }}</span>
          </template>
        </el-table-column>

        <!-- 数据类型列 -->
        <el-table-column
          v-if="visibleColumns.includes('dataType')"
          prop="dataType"
          label="类型"
          width="80"
        >
          <template #default="{ row }">
            <el-tag size="small" :type="getDataTypeTagType(row.dataType)">
              {{ getDataTypeText(row.dataType) }}
            </el-tag>
          </template>
        </el-table-column>

        <!-- 数据质量列 -->
        <el-table-column
          v-if="visibleColumns.includes('quality')"
          prop="quality"
          label="质量"
          width="80"
        >
          <template #default="{ row }">
            <div class="quality-cell">
              <div
                class="quality-indicator"
                :class="`quality-${row.quality}`"
                :title="getQualityText(row.quality)"
              ></div>
              <span class="quality-text">{{
                getQualityText(row.quality)
              }}</span>
            </div>
          </template>
        </el-table-column>

        <!-- 原始值列 -->
        <el-table-column
          v-if="visibleColumns.includes('rawValue')"
          prop="rawValue"
          label="原始值"
          width="120"
        >
          <template #default="{ row }">
            <span class="raw-value">{{ row.rawValue || '-' }}</span>
          </template>
        </el-table-column>

        <!-- 操作列 -->
        <el-table-column label="操作" width="100" fixed="right">
          <template #default="{ row }">
            <div class="action-buttons">
              <el-tooltip content="查看详情">
                <el-button
                  type="text"
                  size="small"
                  @click="viewRowDetails(row)"
                >
                  <el-icon><View /></el-icon>
                </el-button>
              </el-tooltip>

              <el-tooltip content="复制数值">
                <el-button
                  type="text"
                  size="small"
                  @click="copyValue(row.value)"
                >
                  <el-icon><CopyDocument /></el-icon>
                </el-button>
              </el-tooltip>
            </div>
          </template>
        </el-table-column>
      </ElTable>
    </div>

    <!-- 分页器 -->
    <div class="table-pagination">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="currentPageSize"
        :total="pagination.total"
        :page-sizes="[20, 50, 100, 200]"
        background
        layout="total, sizes, prev, pager, next, jumper"
        @size-change="handlePageSizeChange"
        @current-change="handlePageChange"
      />
    </div>

    <!-- 列设置对话框 -->
    <el-dialog
      v-model="showColumnSettings"
      title="列显示设置"
      width="400px"
      :close-on-click-modal="false"
    >
      <div class="column-settings">
        <div class="settings-header">
          <span>选择要显示的列</span>
          <div class="settings-actions">
            <el-button type="text" size="small" @click="selectAllColumns"
              >全选</el-button
            >
            <el-button type="text" size="small" @click="resetColumns"
              >重置</el-button
            >
          </div>
        </div>

        <div class="column-list">
          <el-checkbox-group v-model="visibleColumns">
            <div
              v-for="column in availableColumns"
              :key="column.prop"
              class="column-item"
            >
              <el-checkbox :label="column.prop">
                {{ column.label }}
              </el-checkbox>
            </div>
          </el-checkbox-group>
        </div>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="showColumnSettings = false">取消</el-button>
          <el-button type="primary" @click="applyColumnSettings"
            >确定</el-button
          >
        </div>
      </template>
    </el-dialog>

    <!-- 行详情对话框 -->
    <el-dialog
      v-model="showRowDetails"
      title="数据详情"
      width="600px"
      :close-on-click-modal="false"
    >
      <div v-if="currentRowData" class="row-details">
        <div class="details-grid">
          <div class="detail-item">
            <label class="detail-label">时间戳</label>
            <span class="detail-value">{{
              formatDateTime(currentRowData.timestamp)
            }}</span>
          </div>

          <div class="detail-item">
            <label class="detail-label">设备名称</label>
            <span class="detail-value">{{ currentRowData.deviceName }}</span>
          </div>

          <div class="detail-item">
            <label class="detail-label">数据点位</label>
            <span class="detail-value">{{ currentRowData.tagName }}</span>
          </div>

          <div class="detail-item">
            <label class="detail-label">点位地址</label>
            <span class="detail-value">{{ currentRowData.tagAddress }}</span>
          </div>

          <div class="detail-item">
            <label class="detail-label">数值</label>
            <span class="detail-value">{{
              formatValue(
                currentRowData.value,
                currentRowData.dataType,
                currentRowData.unit
              )
            }}</span>
          </div>

          <div class="detail-item">
            <label class="detail-label">原始值</label>
            <span class="detail-value">{{
              currentRowData.rawValue || '-'
            }}</span>
          </div>

          <div class="detail-item">
            <label class="detail-label">数据类型</label>
            <span class="detail-value">{{
              getDataTypeText(currentRowData.dataType)
            }}</span>
          </div>

          <div class="detail-item">
            <label class="detail-label">数据质量</label>
            <span class="detail-value">{{
              getQualityText(currentRowData.quality)
            }}</span>
          </div>

          <div class="detail-item">
            <label class="detail-label">单位</label>
            <span class="detail-value">{{ currentRowData.unit || '-' }}</span>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button type="primary" @click="showRowDetails = false"
            >关闭</el-button
          >
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * HistoryDataTable —— 历史数据表格组件
 *
 * 📝 Responsibilities:
 *  1. 历史数据表格展示
 *  2. 数据排序和筛选
 *  3. 分页处理
 *  4. 列显示设置
 *  5. 数据导出和复制
 *
 * 📦 Dependencies:
 *  - Element Plus Table
 *  - 格式化工具函数
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Setting,
  Download,
  Refresh,
  View,
  CopyDocument,
} from '@element-plus/icons-vue'
import { ElMessage, ElTable } from 'element-plus'
import { ref, computed, watch, nextTick } from 'vue'

import { formatDateTime, formatNumber } from '@/utils/format'

// ===== Props & Emits =====
const props = defineProps<{
  data: any[]
  loading?: boolean
  pagination: {
    currentPage: number
    pageSize: number
    total: number
  }
}>()

const emit = defineEmits<{
  'page-change': [page: number]
  'size-change': [size: number]
  'sort-change': [sort: { prop: string; order: string }]
  refresh: []
}>()

// ===== 响应式数据 =====
const tableRef = ref<InstanceType<typeof ElTable>>()
const selectedRows = ref([])
const showColumnSettings = ref(false)
const showRowDetails = ref(false)
const currentRowData = ref(null)

// 分页状态
const currentPage = ref(props.pagination.currentPage)
const currentPageSize = ref(props.pagination.pageSize)

// 可见列设置
const visibleColumns = ref(['deviceName', 'unit', 'dataType', 'quality'])

// 可用列配置
const availableColumns = [
  { prop: 'deviceName', label: '设备名称' },
  { prop: 'unit', label: '单位' },
  { prop: 'dataType', label: '数据类型' },
  { prop: 'quality', label: '数据质量' },
  { prop: 'rawValue', label: '原始值' },
]

// ===== 计算属性 =====
const tableData = computed(() => {
  return props.data || []
})

// ===== 方法 =====

/**
 * 获取行索引
 */
function getRowIndex(index: number): number {
  return (currentPage.value - 1) * currentPageSize.value + index + 1
}

/**
 * 检查行是否可选择
 */
function isRowSelectable(row: any): boolean {
  return row && row.timestamp
}

/**
 * 格式化数值
 */
function formatValue(value: any, dataType: string, unit?: string): string {
  if (value === null || value === undefined) return '-'

  switch (dataType) {
    case 'boolean':
      return value ? '真' : '假'
    case 'string':
      return String(value)
    case 'integer':
      return formatNumber(parseInt(value), 0)
    case 'float':
      return formatNumber(parseFloat(value), 2)
    default:
      return String(value)
  }
}

/**
 * 获取数值样式类
 */
function getValueClass(value: any, dataType: string): string {
  if (value === null || value === undefined) return 'value-null'

  switch (dataType) {
    case 'boolean':
      return value ? 'value-true' : 'value-false'
    case 'string':
      return 'value-string'
    case 'integer':
    case 'float':
      const num = parseFloat(value)
      if (num > 0) return 'value-positive'
      if (num < 0) return 'value-negative'
      return 'value-zero'
    default:
      return 'value-default'
  }
}

/**
 * 获取数据类型标签类型
 */
function getDataTypeTagType(dataType: string): string {
  const typeMap: Record<string, string> = {
    boolean: 'success',
    string: 'info',
    integer: 'warning',
    float: 'danger',
  }
  return typeMap[dataType] || 'info'
}

/**
 * 获取数据类型文本
 */
function getDataTypeText(dataType: string): string {
  const textMap: Record<string, string> = {
    boolean: '布尔',
    string: '字符串',
    integer: '整数',
    float: '浮点',
  }
  return textMap[dataType] || dataType
}

/**
 * 获取质量文本
 */
function getQualityText(quality: string): string {
  const textMap: Record<string, string> = {
    good: '良好',
    uncertain: '可疑',
    bad: '错误',
  }
  return textMap[quality] || quality
}

/**
 * 处理选择变化
 */
function handleSelectionChange(selection: any[]) {
  selectedRows.value = selection
}

/**
 * 处理排序变化
 */
function handleSortChange(sort: any) {
  emit('sort-change', {
    prop: sort.prop,
    order: sort.order,
  })
}

/**
 * 处理行点击
 */
function handleRowClick(row: any) {
  viewRowDetails(row)
}

/**
 * 处理页面变化
 */
function handlePageChange(page: number) {
  currentPage.value = page
  emit('page-change', page)
}

/**
 * 处理页面大小变化
 */
function handlePageSizeChange(size: number) {
  currentPageSize.value = size
  currentPage.value = 1
  emit('size-change', size)
}

/**
 * 处理刷新
 */
function handleRefresh() {
  emit('refresh')
}

/**
 * 查看行详情
 */
function viewRowDetails(row: any) {
  currentRowData.value = row
  showRowDetails.value = true
}

/**
 * 复制数值
 */
async function copyValue(value: any) {
  try {
    await navigator.clipboard.writeText(String(value))
    ElMessage.success('数值已复制到剪贴板')
  } catch (error) {
    ElMessage.error('复制失败')
  }
}

/**
 * 导出选中数据
 */
function exportSelected() {
  if (selectedRows.value.length === 0) {
    ElMessage.warning('请先选择要导出的数据')
    return
  }

  ElMessage.info('导出功能开发中...')
}

/**
 * 全选列
 */
function selectAllColumns() {
  visibleColumns.value = availableColumns.map(col => col.prop)
}

/**
 * 重置列设置
 */
function resetColumns() {
  visibleColumns.value = ['deviceName', 'unit', 'dataType', 'quality']
}

/**
 * 应用列设置
 */
function applyColumnSettings() {
  showColumnSettings.value = false
  ElMessage.success('列设置已应用')

  // 触发表格重新渲染
  nextTick(() => {
    tableRef.value?.doLayout()
  })
}

// ===== 监听器 =====
watch(
  () => props.pagination,
  newPagination => {
    currentPage.value = newPagination.currentPage
    currentPageSize.value = newPagination.pageSize
  },
  { deep: true }
)
</script>

<style scoped lang="scss">
.history-data-table {
  .table-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    padding: 0 4px;

    .toolbar-left {
      .table-info {
        .info-text {
          font-size: 14px;
          color: #606266;
        }
      }
    }

    .toolbar-right {
      display: flex;
      align-items: center;
      gap: 16px;

      .table-actions {
        display: flex;
        gap: 4px;
      }

      .page-size-selector {
        display: flex;
        align-items: center;
        gap: 8px;

        .size-label {
          font-size: 13px;
          color: #606266;
        }
      }
    }
  }

  .table-container {
    border-radius: 6px;
    overflow: hidden;

    :deep(.el-table) {
      .timestamp-cell {
        .timestamp-main {
          display: block;
          font-weight: 500;
          color: #303133;
        }

        .timestamp-sub {
          display: block;
          font-size: 11px;
          color: #909399;
          margin-top: 2px;
        }
      }

      .device-cell {
        .el-tag {
          font-size: 12px;
        }
      }

      .tag-cell {
        .tag-name {
          font-weight: 500;
          color: #303133;
          margin-bottom: 2px;
        }

        .tag-address {
          font-size: 11px;
          color: #909399;
          font-family: monospace;
        }
      }

      .value-cell {
        .value-number {
          font-weight: 600;
          font-family: monospace;

          &.value-null {
            color: #909399;
          }
          &.value-true {
            color: #67c23a;
          }
          &.value-false {
            color: #f56c6c;
          }
          &.value-string {
            color: #409eff;
          }
          &.value-positive {
            color: #67c23a;
          }
          &.value-negative {
            color: #f56c6c;
          }
          &.value-zero {
            color: #606266;
          }
          &.value-default {
            color: #303133;
          }
        }
      }

      .unit-text {
        font-size: 12px;
        color: #67c23a;
      }

      .quality-cell {
        display: flex;
        align-items: center;
        gap: 6px;

        .quality-indicator {
          width: 8px;
          height: 8px;
          border-radius: 50%;

          &.quality-good {
            background: #67c23a;
          }
          &.quality-uncertain {
            background: #e6a23c;
          }
          &.quality-bad {
            background: #f56c6c;
          }
        }

        .quality-text {
          font-size: 12px;
        }
      }

      .raw-value {
        font-family: monospace;
        font-size: 12px;
        color: #606266;
      }

      .action-buttons {
        display: flex;
        gap: 4px;
      }
    }
  }

  .table-pagination {
    margin-top: 16px;
    display: flex;
    justify-content: center;
  }
}

.column-settings {
  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;

    .settings-actions {
      display: flex;
      gap: 8px;
    }
  }

  .column-list {
    .column-item {
      margin-bottom: 12px;
    }
  }
}

.row-details {
  .details-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;

    .detail-item {
      display: flex;
      flex-direction: column;
      gap: 4px;

      .detail-label {
        font-size: 13px;
        color: #909399;
        font-weight: 500;
      }

      .detail-value {
        font-size: 14px;
        color: #303133;
        word-break: break-all;
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .history-data-table {
    .table-toolbar {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;

      .toolbar-left,
      .toolbar-right {
        justify-content: center;
      }
    }
  }
}

@media (max-width: 768px) {
  .history-data-table {
    .table-container {
      :deep(.el-table) {
        font-size: 12px;

        .el-table__cell {
          padding: 8px 4px;
        }
      }
    }
  }

  .row-details {
    .details-grid {
      grid-template-columns: 1fr;
      gap: 12px;
    }
  }
}
</style>
