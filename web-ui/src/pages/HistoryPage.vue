<template>
  <div class="history-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="title-section">
          <h1 class="page-title">历史数据</h1>
          <p class="page-description">查询和分析设备的历史数据变化趋势</p>
        </div>

        <div class="header-actions">
          <el-button
            type="primary"
            :disabled="!hasData"
            @click="showExportDialog = true"
          >
            <el-icon><Download /></el-icon>
            导出数据
          </el-button>
          <el-button :loading="loading" @click="refreshData">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </div>
      </div>
    </div>

    <!-- 查询条件面板 -->
    <el-card class="query-panel" shadow="never">
      <div class="query-form">
        <!-- 设备和标签选择 -->
        <div class="query-row">
          <div class="query-item">
            <label class="query-label">设备选择</label>
            <el-select
              v-model="queryForm.deviceIds"
              multiple
              placeholder="请选择设备"
              style="width: 280px"
              @change="handleDeviceChange"
            >
              <el-option
                v-for="device in availableDevices"
                :key="device.id"
                :label="device.name"
                :value="device.id"
              >
                <div class="device-option">
                  <span class="device-name">{{ device.name }}</span>
                  <span class="device-protocol">{{ device.protocol }}</span>
                </div>
              </el-option>
            </el-select>
          </div>

          <div class="query-item">
            <label class="query-label">数据点位</label>
            <el-select
              v-model="queryForm.tagIds"
              multiple
              placeholder="请选择数据点位"
              style="width: 280px"
              :disabled="!queryForm.deviceIds.length"
            >
              <el-option
                v-for="tag in availableTags"
                :key="tag.id"
                :label="tag.name"
                :value="tag.id"
              >
                <div class="tag-option">
                  <span class="tag-name">{{ tag.name }}</span>
                  <span class="tag-address">{{ tag.address }}</span>
                  <span v-if="tag.unit" class="tag-unit">({{ tag.unit }})</span>
                </div>
              </el-option>
            </el-select>
          </div>
        </div>

        <!-- 时间范围和聚合方式 -->
        <div class="query-row">
          <div class="query-item">
            <label class="query-label">时间范围</label>
            <TimeRangeSelector
              v-model:range="queryForm.timeRange"
              v-model:start="queryForm.startTime"
              v-model:end="queryForm.endTime"
              @change="handleTimeRangeChange"
            />
          </div>

          <div class="query-item">
            <label class="query-label">数据聚合</label>
            <el-select
              v-model="queryForm.aggregation"
              placeholder="选择聚合方式"
              style="width: 150px"
            >
              <el-option label="原始数据" value="raw" />
              <el-option label="1分钟平均" value="1m" />
              <el-option label="5分钟平均" value="5m" />
              <el-option label="15分钟平均" value="15m" />
              <el-option label="1小时平均" value="1h" />
              <el-option label="1天平均" value="1d" />
            </el-select>
          </div>

          <div class="query-item">
            <label class="query-label">数据类型</label>
            <el-select
              v-model="queryForm.dataType"
              placeholder="选择数据类型"
              style="width: 120px"
            >
              <el-option label="数值" value="numeric" />
              <el-option label="布尔" value="boolean" />
              <el-option label="字符串" value="string" />
              <el-option label="全部" value="all" />
            </el-select>
          </div>
        </div>

        <!-- 查询按钮和高级选项 -->
        <div class="query-row">
          <div class="query-actions">
            <el-button
              type="primary"
              :loading="loading"
              :disabled="!canQuery"
              @click="executeQuery"
            >
              <el-icon><Search /></el-icon>
              查询数据
            </el-button>

            <el-button @click="resetQuery">
              <el-icon><RefreshLeft /></el-icon>
              重置条件
            </el-button>

            <el-button
              type="text"
              @click="showAdvancedOptions = !showAdvancedOptions"
            >
              {{ showAdvancedOptions ? '收起' : '展开' }}高级选项
              <el-icon>
                <ArrowDown v-if="!showAdvancedOptions" />
                <ArrowUp v-else />
              </el-icon>
            </el-button>
          </div>

          <div v-if="queryStats.totalRecords > 0" class="query-stats">
            <span class="stats-item">
              查询结果: {{ formatNumber(queryStats.totalRecords) }} 条记录
            </span>
            <span class="stats-item">
              时间跨度: {{ queryStats.timeSpan }}
            </span>
            <span class="stats-item">
              查询耗时: {{ queryStats.queryTime }}ms
            </span>
          </div>
        </div>

        <!-- 高级选项 -->
        <div v-if="showAdvancedOptions" class="advanced-options">
          <div class="options-row">
            <div class="option-item">
              <label class="option-label">最大记录数</label>
              <el-input-number
                v-model="queryForm.maxRecords"
                :min="100"
                :max="100000"
                :step="1000"
                style="width: 150px"
              />
            </div>

            <div class="option-item">
              <label class="option-label">数据质量</label>
              <el-select
                v-model="queryForm.quality"
                placeholder="选择数据质量"
                style="width: 120px"
              >
                <el-option label="全部" value="all" />
                <el-option label="良好" value="good" />
                <el-option label="可疑" value="uncertain" />
                <el-option label="错误" value="bad" />
              </el-select>
            </div>

            <div class="option-item">
              <el-checkbox v-model="queryForm.includeNull"
                >包含空值</el-checkbox
              >
            </div>

            <div class="option-item">
              <el-checkbox v-model="queryForm.fillGaps"
                >填充数据间隙</el-checkbox
              >
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 数据展示区域 -->
    <div class="data-display">
      <!-- 展示模式切换 -->
      <div class="display-header">
        <el-radio-group v-model="displayMode" class="display-tabs">
          <el-radio-button label="chart">图表视图</el-radio-button>
          <el-radio-button label="table">表格视图</el-radio-button>
          <el-radio-button label="both">混合视图</el-radio-button>
        </el-radio-group>

        <div class="display-actions">
          <el-tooltip content="全屏显示">
            <el-button
              v-if="displayMode !== 'table'"
              type="text"
              @click="toggleFullscreen"
            >
              <el-icon><FullScreen /></el-icon>
            </el-button>
          </el-tooltip>
        </div>
      </div>

      <!-- 图表视图 -->
      <el-card
        v-if="displayMode === 'chart' || displayMode === 'both'"
        class="chart-container"
        shadow="never"
        :class="{ fullscreen: isFullscreen }"
      >
        <HistoryDataChart
          :data="historyData"
          :loading="loading"
          :time-range="queryForm.timeRange"
          :aggregation="queryForm.aggregation"
          @point-click="handleChartPointClick"
        />
      </el-card>

      <!-- 表格视图 -->
      <el-card
        v-if="displayMode === 'table' || displayMode === 'both'"
        class="table-container"
        shadow="never"
      >
        <HistoryDataTable
          :data="historyData"
          :loading="loading"
          :pagination="tablePagination"
          @page-change="handleTablePageChange"
          @size-change="handleTableSizeChange"
          @sort-change="handleTableSortChange"
        />
      </el-card>

      <!-- 空状态 -->
      <el-card
        v-if="!loading && !hasData"
        class="empty-container"
        shadow="never"
      >
        <el-empty description="暂无历史数据" :image-size="120">
          <template #description>
            <p>请选择设备和数据点位，然后点击查询按钮获取历史数据</p>
          </template>
          <el-button type="primary" @click="showQuickQuery">
            快速查询
          </el-button>
        </el-empty>
      </el-card>
    </div>

    <!-- 数据导出对话框 -->
    <DataExportDialog
      v-model:visible="showExportDialog"
      :data="historyData"
      :query-form="queryForm"
      @export="handleDataExport"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * HistoryPage —— 历史数据查询页面
 *
 * 📝 Responsibilities:
 *  1. 历史数据查询条件设置
 *  2. 数据可视化展示（图表/表格）
 *  3. 数据导出功能
 *  4. 查询结果统计信息
 *  5. 高级查询选项
 *
 * 📦 Dependencies:
 *  - TimeRangeSelector 时间范围选择器
 *  - HistoryDataChart 历史数据图表
 *  - HistoryDataTable 历史数据表格
 *  - DataExportDialog 数据导出对话框
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Download,
  Refresh,
  Search,
  RefreshLeft,
  ArrowDown,
  ArrowUp,
  FullScreen,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { useRouter } from 'vue-router'

import DataExportDialog from '@/components/history/DataExportDialog.vue'
import HistoryDataChart from '@/components/history/HistoryDataChart.vue'
import HistoryDataTable from '@/components/history/HistoryDataTable.vue'
import TimeRangeSelector from '@/components/history/TimeRangeSelector.vue'
import { useDevicesStore, useTagsStore, useHistoryStore } from '@/stores'
import { formatDateTime, getTimeRange } from '@/utils/date'
import { formatNumber } from '@/utils/format'

// 组件导入

// ===== 路由 =====
const router = useRouter()

// ===== Stores =====
const devicesStore = useDevicesStore()
const tagsStore = useTagsStore()
const historyStore = useHistoryStore()

// ===== 响应式数据 =====
const loading = ref(false)
const showAdvancedOptions = ref(false)
const showExportDialog = ref(false)
const displayMode = ref<'chart' | 'table' | 'both'>('chart')
const isFullscreen = ref(false)

// 查询表单
const queryForm = ref({
  deviceIds: [] as string[],
  tagIds: [] as string[],
  timeRange: '1h',
  startTime: '',
  endTime: '',
  aggregation: 'raw',
  dataType: 'all',
  maxRecords: 10000,
  quality: 'all',
  includeNull: false,
  fillGaps: false,
})

// 查询统计
const queryStats = ref({
  totalRecords: 0,
  timeSpan: '',
  queryTime: 0,
})

// 表格分页
const tablePagination = ref({
  currentPage: 1,
  pageSize: 50,
  total: 0,
})

// 可用设备和标签
const availableDevices = ref([])
const availableTags = ref([])

// 历史数据
const historyData = ref([])

// ===== 计算属性 =====
const canQuery = computed(() => {
  return (
    queryForm.value.deviceIds.length > 0 &&
    queryForm.value.tagIds.length > 0 &&
    (queryForm.value.timeRange !== 'custom' ||
      (queryForm.value.startTime && queryForm.value.endTime))
  )
})

const hasData = computed(() => {
  return historyData.value && historyData.value.length > 0
})

// ===== 方法 =====

/**
 * 初始化数据
 */
async function initializeData() {
  try {
    // 加载可用设备
    await devicesStore.fetchDevices({ page: 1, size: 1000 })
    availableDevices.value = devicesStore.state.devices

    // 设置默认时间范围
    const timeRange = getTimeRange('hour', 1)
    queryForm.value.startTime = timeRange.start
    queryForm.value.endTime = timeRange.end
  } catch (error) {
    console.error('初始化历史数据页面失败:', error)
    ElMessage.error('页面初始化失败')
  }
}

/**
 * 处理设备选择变化
 */
async function handleDeviceChange() {
  if (queryForm.value.deviceIds.length === 0) {
    availableTags.value = []
    queryForm.value.tagIds = []
    return
  }

  try {
    // 加载选中设备的标签
    const tagPromises = queryForm.value.deviceIds.map(deviceId =>
      tagsStore.fetchTagsByDevice(deviceId)
    )

    const tagResults = await Promise.all(tagPromises)
    availableTags.value = tagResults.flat()

    // 清空已选择的标签（如果不在新的可用标签中）
    queryForm.value.tagIds = queryForm.value.tagIds.filter(tagId =>
      availableTags.value.some(tag => tag.id === tagId)
    )
  } catch (error) {
    console.error('加载设备标签失败:', error)
    ElMessage.error('加载设备标签失败')
  }
}

/**
 * 处理时间范围变化
 */
function handleTimeRangeChange() {
  // 时间范围选择器组件会自动更新startTime和endTime
}

/**
 * 执行查询
 */
async function executeQuery() {
  if (!canQuery.value) {
    ElMessage.warning('请选择设备、数据点位和时间范围')
    return
  }

  loading.value = true
  const startTime = Date.now()

  try {
    const queryParams = {
      tag_ids: queryForm.value.tagIds,
      start_time: queryForm.value.startTime,
      end_time: queryForm.value.endTime,
      aggregation: queryForm.value.aggregation as any,
      interval: queryForm.value.aggregation === 'raw' ? '1s' : '1m',
      page: 1,
      size: queryForm.value.maxRecords,
    }

    // 调用缓存版本的查询方法
    await historyStore.fetchTimeSeriesData(queryParams)

    // 从 store 状态获取数据
    historyData.value = historyStore.state.timeSeriesData.flatMap(series =>
      series.data.map((point, index) => ({
        id: `${series.tag_id}_${index}`,
        timestamp: point.timestamp,
        deviceId: series.device_id || '',
        deviceName: series.device_name || '未知设备',
        tagId: series.tag_id,
        tagName: series.tag_name,
        tagAddress: series.tag_address || '',
        value: point.value,
        rawValue: point.value,
        unit: series.unit || '',
        dataType: 'float',
        quality: 'good',
      }))
    )

    // 更新查询统计
    queryStats.value = {
      totalRecords: historyData.value.length,
      timeSpan: formatTimeSpan(
        queryForm.value.startTime,
        queryForm.value.endTime
      ),
      queryTime: Date.now() - startTime,
    }

    // 更新表格分页
    tablePagination.value.total = historyData.value.length
    tablePagination.value.currentPage = 1

    if (historyData.value.length === 0) {
      ElMessage.info('未查询到符合条件的历史数据')
    } else {
      ElMessage.success(
        `查询完成，共获取 ${queryStats.value.totalRecords} 条记录`
      )
    }
  } catch (error) {
    console.error('查询历史数据失败:', error)
    ElMessage.error('查询历史数据失败')
    historyData.value = []
    queryStats.value = { totalRecords: 0, timeSpan: '', queryTime: 0 }
  } finally {
    loading.value = false
  }
}

/**
 * 重置查询条件
 */
function resetQuery() {
  queryForm.value = {
    deviceIds: [],
    tagIds: [],
    timeRange: '1h',
    startTime: '',
    endTime: '',
    aggregation: 'raw',
    dataType: 'all',
    maxRecords: 10000,
    quality: 'all',
    includeNull: false,
    fillGaps: false,
  }

  // 重置时间范围
  const timeRange = getTimeRange('hour', 1)
  queryForm.value.startTime = timeRange.start
  queryForm.value.endTime = timeRange.end

  // 清空数据
  historyData.value = []
  queryStats.value = { totalRecords: 0, timeSpan: '', queryTime: 0 }
  availableTags.value = []
}

/**
 * 刷新数据
 */
async function refreshData() {
  if (canQuery.value) {
    await executeQuery()
  } else {
    await initializeData()
  }
}

/**
 * 显示快速查询
 */
function showQuickQuery() {
  if (availableDevices.value.length > 0) {
    // 选择第一个设备
    queryForm.value.deviceIds = [availableDevices.value[0].id]
    handleDeviceChange()
  }
}

/**
 * 切换全屏
 */
function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value

  nextTick(() => {
    // 触发图表重新渲染
    window.dispatchEvent(new Event('resize'))
  })
}

/**
 * 处理图表点击
 */
function handleChartPointClick(point: any) {
  console.log('图表点击:', point)
  // 可以显示点击点的详细信息
}

/**
 * 处理表格页面变化
 */
function handleTablePageChange(page: number) {
  tablePagination.value.currentPage = page
  // 这里可以实现分页加载
}

/**
 * 处理表格页面大小变化
 */
function handleTableSizeChange(size: number) {
  tablePagination.value.pageSize = size
  tablePagination.value.currentPage = 1
  // 这里可以重新加载数据
}

/**
 * 处理表格排序变化
 */
function handleTableSortChange(sort: any) {
  console.log('表格排序:', sort)
  // 这里可以实现排序功能
}

/**
 * 处理数据导出
 */
function handleDataExport(exportConfig: any) {
  console.log('导出配置:', exportConfig)
  ElMessage.success('数据导出功能开发中...')
}

/**
 * 格式化时间跨度
 */
function formatTimeSpan(startTime: string, endTime: string): string {
  if (!startTime || !endTime) return ''

  const start = new Date(startTime)
  const end = new Date(endTime)
  const diffMs = end.getTime() - start.getTime()

  const days = Math.floor(diffMs / (24 * 60 * 60 * 1000))
  const hours = Math.floor((diffMs % (24 * 60 * 60 * 1000)) / (60 * 60 * 1000))
  const minutes = Math.floor((diffMs % (60 * 60 * 1000)) / (60 * 1000))

  const parts = []
  if (days > 0) parts.push(`${days}天`)
  if (hours > 0) parts.push(`${hours}小时`)
  if (minutes > 0) parts.push(`${minutes}分钟`)

  return parts.join('') || '1分钟内'
}

// ===== 生命周期 =====
onMounted(async () => {
  await initializeData()
})

// ===== 监听器 =====
watch(
  () => queryForm.value.timeRange,
  newRange => {
    if (newRange !== 'custom') {
      const timeRange = getTimeRange(
        newRange === '1h'
          ? 'hour'
          : newRange === '6h'
            ? 'hour'
            : newRange === '24h'
              ? 'hour'
              : newRange === '7d'
                ? 'day'
                : 'day',

        newRange === '1h'
          ? 1
          : newRange === '6h'
            ? 6
            : newRange === '24h'
              ? 24
              : newRange === '7d'
                ? 7
                : 1
      )

      queryForm.value.startTime = timeRange.start
      queryForm.value.endTime = timeRange.end
    }
  }
)
</script>

<style scoped lang="scss">
.history-page {
  padding: 24px;
  background: #f5f5f5;
  min-height: 100vh;

  .page-header {
    margin-bottom: 16px;

    .header-content {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;

      .title-section {
        .page-title {
          font-size: 28px;
          font-weight: 600;
          color: #303133;
          margin: 0 0 8px 0;
        }

        .page-description {
          font-size: 14px;
          color: #606266;
          margin: 0;
        }
      }

      .header-actions {
        display: flex;
        gap: 12px;
      }
    }
  }

  .query-panel {
    margin-bottom: 16px;

    .query-form {
      .query-row {
        display: flex;
        align-items: flex-end;
        gap: 24px;
        margin-bottom: 16px;
        flex-wrap: wrap;

        &:last-child {
          margin-bottom: 0;
        }
      }

      .query-item {
        display: flex;
        flex-direction: column;

        .query-label {
          font-size: 13px;
          color: #606266;
          margin-bottom: 6px;
          font-weight: 500;
        }
      }

      .device-option,
      .tag-option {
        display: flex;
        justify-content: space-between;
        align-items: center;
        width: 100%;

        .device-name,
        .tag-name {
          font-weight: 500;
        }

        .device-protocol,
        .tag-address {
          font-size: 12px;
          color: #909399;
          font-family: monospace;
        }

        .tag-unit {
          font-size: 12px;
          color: #67c23a;
        }
      }

      .query-actions {
        display: flex;
        align-items: center;
        gap: 12px;
      }

      .query-stats {
        display: flex;
        align-items: center;
        gap: 16px;
        margin-left: auto;

        .stats-item {
          font-size: 13px;
          color: #606266;

          &:not(:last-child)::after {
            content: '|';
            margin-left: 16px;
            color: #dcdfe6;
          }
        }
      }

      .advanced-options {
        padding-top: 16px;
        border-top: 1px solid #ebeef5;

        .options-row {
          display: flex;
          align-items: center;
          gap: 24px;
          flex-wrap: wrap;
        }

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

  .data-display {
    .display-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;

      .display-tabs {
        .el-radio-button {
          margin-right: 0;
        }
      }

      .display-actions {
        display: flex;
        gap: 8px;
      }
    }

    .chart-container,
    .table-container {
      margin-bottom: 16px;

      &.fullscreen {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        z-index: 2000;
        margin: 0;
        border-radius: 0;
      }
    }

    .empty-container {
      text-align: center;
      padding: 60px 20px;
    }
  }
}

@media (max-width: 1200px) {
  .history-page {
    .page-header .header-content {
      flex-direction: column;
      gap: 16px;
      align-items: stretch;
    }

    .query-panel .query-form .query-row {
      flex-direction: column;
      align-items: stretch;
      gap: 16px;

      .query-item {
        width: 100%;

        :deep(.el-select) {
          width: 100% !important;
        }
      }

      .query-actions {
        justify-content: center;
      }

      .query-stats {
        margin-left: 0;
        justify-content: center;
        flex-wrap: wrap;
      }
    }
  }
}

@media (max-width: 768px) {
  .history-page {
    padding: 16px;

    .page-header .title-section .page-title {
      font-size: 24px;
    }

    .data-display .display-header {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;

      .display-tabs {
        width: 100%;

        :deep(.el-radio-button) {
          flex: 1;
        }
      }
    }
  }
}
</style>
