<template>
  <div class="base-table">
    <!-- 表格操作栏 -->
    <div class="table-header" v-if="showToolbar">
      <div class="table-title">
        <slot name="title">
          <h3 v-if="title">{{ title }}</h3>
        </slot>
      </div>
      <div class="table-actions">
        <slot name="actions">
          <el-button
            v-if="showRefresh"
            type="default"
            :icon="Refresh"
            @click="handleRefresh"
            :loading="loading"
          >
            刷新
          </el-button>
        </slot>
      </div>
    </div>

    <!-- 搜索和筛选栏 -->
    <div class="table-filters" v-if="showFilters">
      <slot name="filters">
        <el-input
          v-if="searchable"
          v-model="searchValue"
          placeholder="搜索..."
          :prefix-icon="Search"
          clearable
          @input="handleSearch"
          style="width: 200px; margin-right: 12px;"
        />
        
        <slot name="custom-filters" />
        
        <el-button
          v-if="showResetFilter"
          type="default"
          @click="handleResetFilters"
        >
          重置筛选
        </el-button>
      </slot>
    </div>

    <!-- 主表格 -->
    <el-table
      ref="tableRef"
      :data="tableData"
      :loading="loading"
      :height="height"
      :max-height="maxHeight"
      :stripe="stripe"
      :border="border"
      :size="size"
      :empty-text="emptyText"
      :row-key="rowKey"
      :default-sort="defaultSort"
      :highlight-current-row="highlightCurrentRow"
      @selection-change="handleSelectionChange"
      @sort-change="handleSortChange"
      @row-click="handleRowClick"
      @row-dblclick="handleRowDoubleClick"
      v-bind="$attrs"
    >
      <!-- 多选列 -->
      <el-table-column
        v-if="selectable"
        type="selection"
        width="55"
        :selectable="selectableFunction"
      />
      
      <!-- 序号列 -->
      <el-table-column
        v-if="showIndex"
        type="index"
        label="序号"
        width="80"
        :index="indexMethod"
      />

      <!-- 动态列 -->
      <template v-for="column in columns" :key="column.prop || column.key">
        <el-table-column
          :prop="column.prop"
          :label="column.label"
          :width="column.width"
          :min-width="column.minWidth"
          :fixed="column.fixed"
          :sortable="column.sortable"
          :align="column.align || 'left'"
          :header-align="column.headerAlign || column.align || 'left'"
          :show-overflow-tooltip="column.showOverflowTooltip !== false"
          :class-name="column.className"
          :label-class-name="column.labelClassName"
        >
          <template #default="scope" v-if="column.slot">
            <slot :name="column.slot" v-bind="scope" />
          </template>
          
          <template #default="scope" v-else-if="column.formatter">
            <span v-html="column.formatter(scope.row[column.prop], scope.row, scope.$index)"></span>
          </template>
          
          <template #default="scope" v-else-if="column.type === 'tag'">
            <el-tag
              :type="getTagType(scope.row[column.prop], column.tagMap)"
              :size="column.tagSize || 'small'"
              :effect="column.tagEffect || 'light'"
            >
              {{ getTagText(scope.row[column.prop], column.tagMap) }}
            </el-tag>
          </template>
          
          <template #default="scope" v-else-if="column.type === 'switch'">
            <el-switch
              :model-value="scope.row[column.prop]"
              @change="(value) => handleSwitchChange(value, scope.row, column)"
              :disabled="column.disabled"
            />
          </template>
          
          <template #default="scope" v-else-if="column.type === 'actions'">
            <div class="table-actions-cell">
              <template v-for="action in column.actions" :key="action.key">
                <el-button
                  v-if="isActionVisible(action, scope.row)"
                  :type="action.type || 'text'"
                  :size="action.size || 'small'"
                  :icon="action.icon"
                  :loading="getActionLoading(action, scope.row)"
                  :disabled="isActionDisabled(action, scope.row)"
                  @click="handleActionClick(action, scope.row, scope.$index)"
                >
                  {{ action.label }}
                </el-button>
                
                <el-popconfirm
                  v-else-if="action.confirm && isActionVisible(action, scope.row)"
                  :title="action.confirm.title || '确定要执行此操作吗？'"
                  :confirm-button-text="action.confirm.confirmText || '确定'"
                  :cancel-button-text="action.confirm.cancelText || '取消'"
                  @confirm="handleActionClick(action, scope.row, scope.$index)"
                >
                  <template #reference>
                    <el-button
                      :type="action.type || 'text'"
                      :size="action.size || 'small'"
                      :icon="action.icon"
                      :disabled="isActionDisabled(action, scope.row)"
                    >
                      {{ action.label }}
                    </el-button>
                  </template>
                </el-popconfirm>
              </template>
            </div>
          </template>
          
          <template #header v-if="column.headerSlot">
            <slot :name="column.headerSlot" :column="column" />
          </template>
        </el-table-column>
      </template>

      <!-- 自定义列 -->
      <slot />
    </el-table>

    <!-- 分页器 -->
    <div class="table-pagination" v-if="pagination">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="pageSizes"
        :total="total"
        :small="paginationSmall"
        :background="paginationBackground"
        :layout="paginationLayout"
        @size-change="handleSizeChange"
        @current-change="handleCurrentChange"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { Search, Refresh } from '@element-plus/icons-vue'
import type { TableColumnCtx } from 'element-plus'

export interface TableColumn {
  prop?: string
  key?: string
  label: string
  width?: string | number
  minWidth?: string | number
  fixed?: boolean | string
  sortable?: boolean | string
  align?: 'left' | 'center' | 'right'
  headerAlign?: 'left' | 'center' | 'right'
  showOverflowTooltip?: boolean
  className?: string
  labelClassName?: string
  slot?: string
  headerSlot?: string
  formatter?: (value: any, row: any, index: number) => string
  type?: 'tag' | 'switch' | 'actions'
  tagMap?: Record<string, { type: string; text: string }>
  tagSize?: 'large' | 'default' | 'small'
  tagEffect?: 'dark' | 'light' | 'plain'
  disabled?: boolean | ((row: any) => boolean)
  actions?: TableAction[]
}

export interface TableAction {
  key: string
  label: string
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'text'
  size?: 'large' | 'default' | 'small'
  icon?: any
  visible?: boolean | ((row: any) => boolean)
  disabled?: boolean | ((row: any) => boolean)
  loading?: boolean | ((row: any) => boolean)
  confirm?: {
    title?: string
    confirmText?: string
    cancelText?: string
  }
}

interface Props {
  data?: any[]
  columns: TableColumn[]
  loading?: boolean
  height?: string | number
  maxHeight?: string | number
  stripe?: boolean
  border?: boolean
  size?: 'large' | 'default' | 'small'
  emptyText?: string
  rowKey?: string | ((row: any) => string)
  defaultSort?: { prop: string; order: 'ascending' | 'descending' }
  highlightCurrentRow?: boolean
  
  // 工具栏
  title?: string
  showToolbar?: boolean
  showRefresh?: boolean
  
  // 筛选
  showFilters?: boolean
  searchable?: boolean
  showResetFilter?: boolean
  
  // 选择
  selectable?: boolean
  selectableFunction?: (row: any, index: number) => boolean
  
  // 序号
  showIndex?: boolean
  indexMethod?: (index: number) => number
  
  // 分页
  pagination?: boolean
  currentPage?: number
  pageSize?: number
  total?: number
  pageSizes?: number[]
  paginationSmall?: boolean
  paginationBackground?: boolean
  paginationLayout?: string
}

const props = withDefaults(defineProps<Props>(), {
  data: () => [],
  loading: false,
  stripe: true,
  border: false,
  size: 'default',
  emptyText: '暂无数据',
  highlightCurrentRow: true,
  showToolbar: true,
  showRefresh: true,
  showFilters: true,
  searchable: true,
  showResetFilter: true,
  selectable: false,
  showIndex: false,
  pagination: true,
  currentPage: 1,
  pageSize: 20,
  total: 0,
  pageSizes: () => [10, 20, 50, 100],
  paginationSmall: false,
  paginationBackground: true,
  paginationLayout: 'total, sizes, prev, pager, next, jumper',
})

interface Emits {
  refresh: []
  search: [keyword: string]
  resetFilters: []
  selectionChange: [selection: any[]]
  sortChange: [sort: { column: TableColumnCtx<any> | null; prop: string; order: string | null }]
  rowClick: [row: any, column: TableColumnCtx<any>, event: Event]
  rowDoubleClick: [row: any, column: TableColumnCtx<any>, event: Event]
  sizeChange: [size: number]
  currentChange: [page: number]
  actionClick: [action: TableAction, row: any, index: number]
  switchChange: [value: boolean, row: any, column: TableColumn]
}

const emit = defineEmits<Emits>()

// 表格引用
const tableRef = ref()

// 搜索
const searchValue = ref('')

// 表格数据（过滤后）
const tableData = computed(() => {
  let filteredData = props.data
  
  // 搜索过滤
  if (searchValue.value && props.searchable) {
    const keyword = searchValue.value.toLowerCase()
    filteredData = filteredData.filter(row => {
      return props.columns.some(column => {
        if (!column.prop) return false
        const value = row[column.prop]
        return value && value.toString().toLowerCase().includes(keyword)
      })
    })
  }
  
  return filteredData
})

// 分页相关
const currentPage = ref(props.currentPage)
const pageSize = ref(props.pageSize)

watch(() => props.currentPage, (val) => {
  currentPage.value = val
})

watch(() => props.pageSize, (val) => {
  pageSize.value = val
})

// 事件处理
const handleRefresh = () => {
  emit('refresh')
}

const handleSearch = (keyword: string) => {
  emit('search', keyword)
}

const handleResetFilters = () => {
  searchValue.value = ''
  emit('resetFilters')
}

const handleSelectionChange = (selection: any[]) => {
  emit('selectionChange', selection)
}

const handleSortChange = (sort: { column: TableColumnCtx<any> | null; prop: string; order: string | null }) => {
  emit('sortChange', sort)
}

const handleRowClick = (row: any, column: TableColumnCtx<any>, event: Event) => {
  emit('rowClick', row, column, event)
}

const handleRowDoubleClick = (row: any, column: TableColumnCtx<any>, event: Event) => {
  emit('rowDoubleClick', row, column, event)
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  emit('sizeChange', size)
}

const handleCurrentChange = (page: number) => {
  currentPage.value = page
  emit('currentChange', page)
}

const handleActionClick = (action: TableAction, row: any, index: number) => {
  emit('actionClick', action, row, index)
}

const handleSwitchChange = (value: boolean, row: any, column: TableColumn) => {
  emit('switchChange', value, row, column)
}

// 工具函数
const getTagType = (value: any, tagMap?: Record<string, { type: string; text: string }>) => {
  return tagMap?.[value]?.type || 'info'
}

const getTagText = (value: any, tagMap?: Record<string, { type: string; text: string }>) => {
  return tagMap?.[value]?.text || value
}

const isActionVisible = (action: TableAction, row: any) => {
  if (typeof action.visible === 'function') {
    return action.visible(row)
  }
  return action.visible !== false
}

const isActionDisabled = (action: TableAction, row: any) => {
  if (typeof action.disabled === 'function') {
    return action.disabled(row)
  }
  return action.disabled === true
}

const getActionLoading = (action: TableAction, row: any) => {
  if (typeof action.loading === 'function') {
    return action.loading(row)
  }
  return action.loading === true
}

const indexMethod = (index: number) => {
  if (props.indexMethod) {
    return props.indexMethod(index)
  }
  return (currentPage.value - 1) * pageSize.value + index + 1
}

// 公开方法
const clearSelection = () => {
  tableRef.value?.clearSelection()
}

const toggleRowSelection = (row: any, selected?: boolean) => {
  tableRef.value?.toggleRowSelection(row, selected)
}

const toggleAllSelection = () => {
  tableRef.value?.toggleAllSelection()
}

const setCurrentRow = (row: any) => {
  tableRef.value?.setCurrentRow(row)
}

const clearSort = () => {
  tableRef.value?.clearSort()
}

const doLayout = () => {
  nextTick(() => {
    tableRef.value?.doLayout()
  })
}

defineExpose({
  clearSelection,
  toggleRowSelection,
  toggleAllSelection,
  setCurrentRow,
  clearSort,
  doLayout,
  tableRef,
})
</script>

<style scoped lang="scss">
.base-table {
  .table-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    
    .table-title {
      h3 {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        color: var(--el-text-color-primary);
      }
    }
    
    .table-actions {
      display: flex;
      gap: 8px;
    }
  }
  
  .table-filters {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
    padding: 16px;
    background: var(--el-bg-color-light);
    border-radius: 6px;
  }
  
  .table-actions-cell {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  
  .table-pagination {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .base-table {
    .table-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }
    
    .table-filters {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
      
      .el-input {
        width: 100% !important;
      }
    }
    
    .table-pagination {
      .el-pagination {
        flex-wrap: wrap;
        justify-content: center;
      }
    }
  }
}
</style>