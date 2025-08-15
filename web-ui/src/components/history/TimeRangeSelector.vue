<template>
  <div class="time-range-selector">
    <div class="selector-content">
      <!-- 快速选择按钮 -->
      <div class="quick-ranges">
        <el-button-group size="small">
          <el-button
            v-for="range in quickRanges"
            :key="range.value"
            :type="currentRange === range.value ? 'primary' : 'default'"
            @click="selectQuickRange(range.value)"
          >
            {{ range.label }}
          </el-button>
          <el-button
            :type="currentRange === 'custom' ? 'primary' : 'default'"
            @click="selectCustomRange"
          >
            自定义
          </el-button>
        </el-button-group>
      </div>

      <!-- 自定义时间选择 -->
      <div v-if="currentRange === 'custom'" class="custom-range">
        <div class="date-inputs">
          <div class="input-group">
            <label class="input-label">开始时间</label>
            <el-date-picker
              v-model="customStartTime"
              type="datetime"
              placeholder="选择开始时间"
              format="YYYY-MM-DD HH:mm:ss"
              value-format="YYYY-MM-DDTHH:mm:ss.SSSZ"
              :clearable="false"
              :shortcuts="dateShortcuts"
              @change="handleCustomTimeChange"
            />
          </div>

          <div class="input-group">
            <label class="input-label">结束时间</label>
            <el-date-picker
              v-model="customEndTime"
              type="datetime"
              placeholder="选择结束时间"
              format="YYYY-MM-DD HH:mm:ss"
              value-format="YYYY-MM-DDTHH:mm:ss.SSSZ"
              :clearable="false"
              :shortcuts="dateShortcuts"
              :disabled-date="disabledEndDate"
              @change="handleCustomTimeChange"
            />
          </div>
        </div>

        <!-- 时间范围预览 -->
        <div v-if="customStartTime && customEndTime" class="range-preview">
          <div class="preview-item">
            <span class="preview-label">时间跨度:</span>
            <span class="preview-value">{{
              formatTimeSpan(customStartTime, customEndTime)
            }}</span>
          </div>
          <div class="preview-item">
            <span class="preview-label">数据点数:</span>
            <span class="preview-value">约 {{ estimateDataPoints() }} 个</span>
          </div>
        </div>
      </div>

      <!-- 当前选择显示 -->
      <div class="current-selection">
        <div class="selection-info">
          <el-icon><Clock /></el-icon>
          <span class="selection-text">{{ getCurrentSelectionText() }}</span>
        </div>

        <div class="selection-actions">
          <el-tooltip content="刷新到当前时间">
            <el-button type="text" size="small" @click="refreshToNow">
              <el-icon><Refresh /></el-icon>
            </el-button>
          </el-tooltip>

          <el-tooltip content="时区设置">
            <el-button
              type="text"
              size="small"
              @click="showTimezoneDialog = true"
            >
              <el-icon><Setting /></el-icon>
            </el-button>
          </el-tooltip>
        </div>
      </div>
    </div>

    <!-- 时区设置对话框 -->
    <el-dialog
      v-model="showTimezoneDialog"
      title="时区设置"
      width="400px"
      :close-on-click-modal="false"
    >
      <div class="timezone-settings">
        <div class="setting-item">
          <label class="setting-label">显示时区</label>
          <el-select
            v-model="selectedTimezone"
            placeholder="选择时区"
            style="width: 100%"
            filterable
          >
            <el-option
              v-for="tz in availableTimezones"
              :key="tz.value"
              :label="tz.label"
              :value="tz.value"
            />
          </el-select>
        </div>

        <div class="setting-item">
          <label class="setting-label">时间格式</label>
          <el-radio-group v-model="timeFormat">
            <el-radio label="24h">24小时制</el-radio>
            <el-radio label="12h">12小时制</el-radio>
          </el-radio-group>
        </div>

        <div class="current-time">
          <span class="time-label">当前时间:</span>
          <span class="time-value">{{ getCurrentTimeDisplay() }}</span>
        </div>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="showTimezoneDialog = false">取消</el-button>
          <el-button type="primary" @click="applyTimezoneSettings"
            >确定</el-button
          >
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * TimeRangeSelector —— 时间范围选择器组件
 *
 * 📝 Responsibilities:
 *  1. 快速时间范围选择
 *  2. 自定义时间范围设置
 *  3. 时间跨度预览和估算
 *  4. 时区设置和转换
 *  5. 时间格式化显示
 *
 * 📦 Dependencies:
 *  - Element Plus DatePicker
 *  - 日期工具函数
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { Clock, Refresh, Setting } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, watch, onMounted } from 'vue'

import { formatDateTime, getTimeRange } from '@/utils/date'

// ===== Props & Emits =====
const props = defineProps<{
  range: string
  start: string
  end: string
}>()

const emit = defineEmits<{
  'update:range': [range: string]
  'update:start': [start: string]
  'update:end': [end: string]
  change: [data: { range: string; start: string; end: string }]
}>()

// ===== 响应式数据 =====
const currentRange = ref(props.range || '1h')
const customStartTime = ref('')
const customEndTime = ref('')
const showTimezoneDialog = ref(false)
const selectedTimezone = ref('Asia/Shanghai')
const timeFormat = ref('24h')

// 快速选择范围
const quickRanges = [
  { label: '1小时', value: '1h' },
  { label: '6小时', value: '6h' },
  { label: '24小时', value: '24h' },
  { label: '3天', value: '3d' },
  { label: '7天', value: '7d' },
  { label: '30天', value: '30d' },
]

// 日期选择器快捷选项
const dateShortcuts = [
  {
    text: '今天',
    value: () => {
      const today = new Date()
      today.setHours(0, 0, 0, 0)
      return [today, new Date()]
    },
  },
  {
    text: '昨天',
    value: () => {
      const yesterday = new Date()
      yesterday.setDate(yesterday.getDate() - 1)
      yesterday.setHours(0, 0, 0, 0)
      const end = new Date(yesterday)
      end.setHours(23, 59, 59, 999)
      return [yesterday, end]
    },
  },
  {
    text: '最近3天',
    value: () => {
      const start = new Date()
      start.setDate(start.getDate() - 3)
      return [start, new Date()]
    },
  },
  {
    text: '最近7天',
    value: () => {
      const start = new Date()
      start.setDate(start.getDate() - 7)
      return [start, new Date()]
    },
  },
]

// 可用时区
const availableTimezones = [
  { label: '中国标准时间 (UTC+8)', value: 'Asia/Shanghai' },
  { label: '协调世界时 (UTC+0)', value: 'UTC' },
  { label: '美国东部时间 (UTC-5)', value: 'America/New_York' },
  { label: '美国西部时间 (UTC-8)', value: 'America/Los_Angeles' },
  { label: '欧洲中部时间 (UTC+1)', value: 'Europe/Berlin' },
  { label: '日本标准时间 (UTC+9)', value: 'Asia/Tokyo' },
]

// ===== 计算属性 =====
const isValidTimeRange = computed(() => {
  if (currentRange.value === 'custom') {
    return (
      customStartTime.value &&
      customEndTime.value &&
      new Date(customStartTime.value) < new Date(customEndTime.value)
    )
  }
  return true
})

// ===== 方法 =====

/**
 * 选择快速范围
 */
function selectQuickRange(range: string) {
  currentRange.value = range

  // 计算对应的时间范围
  const timeRange = calculateTimeRange(range)

  emit('update:range', range)
  emit('update:start', timeRange.start)
  emit('update:end', timeRange.end)
  emit('change', {
    range,
    start: timeRange.start,
    end: timeRange.end,
  })
}

/**
 * 选择自定义范围
 */
function selectCustomRange() {
  currentRange.value = 'custom'

  // 如果没有设置自定义时间，使用默认值
  if (!customStartTime.value || !customEndTime.value) {
    const now = new Date()
    const oneHourAgo = new Date(now.getTime() - 60 * 60 * 1000)

    customStartTime.value = oneHourAgo.toISOString()
    customEndTime.value = now.toISOString()
  }

  emit('update:range', 'custom')
  emit('update:start', customStartTime.value)
  emit('update:end', customEndTime.value)
  emit('change', {
    range: 'custom',
    start: customStartTime.value,
    end: customEndTime.value,
  })
}

/**
 * 处理自定义时间变化
 */
function handleCustomTimeChange() {
  if (!isValidTimeRange.value) {
    ElMessage.warning('结束时间必须晚于开始时间')
    return
  }

  emit('update:start', customStartTime.value)
  emit('update:end', customEndTime.value)
  emit('change', {
    range: 'custom',
    start: customStartTime.value,
    end: customEndTime.value,
  })
}

/**
 * 禁用结束日期（不能早于开始日期）
 */
function disabledEndDate(date: Date): boolean {
  if (!customStartTime.value) return false
  return date < new Date(customStartTime.value)
}

/**
 * 计算时间范围
 */
function calculateTimeRange(range: string): { start: string; end: string } {
  const now = new Date()
  let start: Date

  switch (range) {
    case '1h':
      start = new Date(now.getTime() - 60 * 60 * 1000)
      break
    case '6h':
      start = new Date(now.getTime() - 6 * 60 * 60 * 1000)
      break
    case '24h':
      start = new Date(now.getTime() - 24 * 60 * 60 * 1000)
      break
    case '3d':
      start = new Date(now.getTime() - 3 * 24 * 60 * 60 * 1000)
      break
    case '7d':
      start = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
      break
    case '30d':
      start = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000)
      break
    default:
      start = new Date(now.getTime() - 60 * 60 * 1000)
  }

  return {
    start: start.toISOString(),
    end: now.toISOString(),
  }
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

/**
 * 估算数据点数
 */
function estimateDataPoints(): number {
  if (
    currentRange.value === 'custom' &&
    customStartTime.value &&
    customEndTime.value
  ) {
    const diffMs =
      new Date(customEndTime.value).getTime() -
      new Date(customStartTime.value).getTime()
    const diffMinutes = diffMs / (60 * 1000)

    // 假设每分钟1个数据点
    return Math.floor(diffMinutes)
  }

  const estimates: Record<string, number> = {
    '1h': 60,
    '6h': 360,
    '24h': 1440,
    '3d': 4320,
    '7d': 10080,
    '30d': 43200,
  }

  return estimates[currentRange.value] || 60
}

/**
 * 获取当前选择文本
 */
function getCurrentSelectionText(): string {
  if (currentRange.value === 'custom') {
    if (customStartTime.value && customEndTime.value) {
      return `${formatDateTime(customStartTime.value, 'MM-DD HH:mm')} 至 ${formatDateTime(customEndTime.value, 'MM-DD HH:mm')}`
    }
    return '自定义时间范围'
  }

  const rangeLabels: Record<string, string> = {
    '1h': '最近1小时',
    '6h': '最近6小时',
    '24h': '最近24小时',
    '3d': '最近3天',
    '7d': '最近7天',
    '30d': '最近30天',
  }

  return rangeLabels[currentRange.value] || '未知范围'
}

/**
 * 刷新到当前时间
 */
function refreshToNow() {
  if (currentRange.value === 'custom') {
    // 保持时间跨度，但更新到当前时间
    const span =
      new Date(customEndTime.value).getTime() -
      new Date(customStartTime.value).getTime()
    const now = new Date()
    const newStart = new Date(now.getTime() - span)

    customStartTime.value = newStart.toISOString()
    customEndTime.value = now.toISOString()

    handleCustomTimeChange()
  } else {
    // 重新计算快速范围
    selectQuickRange(currentRange.value)
  }

  ElMessage.success('时间范围已更新到当前时间')
}

/**
 * 获取当前时间显示
 */
function getCurrentTimeDisplay(): string {
  const now = new Date()
  const format =
    timeFormat.value === '24h' ? 'YYYY-MM-DD HH:mm:ss' : 'YYYY-MM-DD hh:mm:ss A'
  return formatDateTime(now, format)
}

/**
 * 应用时区设置
 */
function applyTimezoneSettings() {
  // 这里可以应用时区设置
  showTimezoneDialog.value = false
  ElMessage.success('时区设置已应用')
}

// ===== 生命周期 =====
onMounted(() => {
  // 初始化时间范围
  if (props.range && props.start && props.end) {
    currentRange.value = props.range
    if (props.range === 'custom') {
      customStartTime.value = props.start
      customEndTime.value = props.end
    }
  } else {
    // 默认选择1小时
    selectQuickRange('1h')
  }
})

// ===== 监听器 =====
watch(
  () => props.range,
  newRange => {
    currentRange.value = newRange
  }
)

watch(
  () => [props.start, props.end],
  ([newStart, newEnd]) => {
    if (currentRange.value === 'custom') {
      customStartTime.value = newStart
      customEndTime.value = newEnd
    }
  }
)
</script>

<style scoped lang="scss">
.time-range-selector {
  .selector-content {
    .quick-ranges {
      margin-bottom: 12px;

      .el-button-group {
        display: flex;
        flex-wrap: wrap;
        gap: 4px;

        .el-button {
          margin: 0;
          border-radius: 4px;
        }
      }
    }

    .custom-range {
      margin-bottom: 12px;
      padding: 16px;
      background: #f8f9fa;
      border-radius: 6px;

      .date-inputs {
        display: flex;
        gap: 16px;
        margin-bottom: 12px;

        .input-group {
          flex: 1;

          .input-label {
            display: block;
            font-size: 13px;
            color: #606266;
            margin-bottom: 6px;
            font-weight: 500;
          }

          :deep(.el-date-editor) {
            width: 100%;
          }
        }
      }

      .range-preview {
        display: flex;
        gap: 24px;

        .preview-item {
          display: flex;
          align-items: center;
          gap: 6px;

          .preview-label {
            font-size: 12px;
            color: #909399;
          }

          .preview-value {
            font-size: 12px;
            color: #303133;
            font-weight: 500;
          }
        }
      }
    }

    .current-selection {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 8px 12px;
      background: #f0f2f5;
      border-radius: 4px;

      .selection-info {
        display: flex;
        align-items: center;
        gap: 6px;

        .el-icon {
          color: #409eff;
          font-size: 14px;
        }

        .selection-text {
          font-size: 13px;
          color: #303133;
          font-weight: 500;
        }
      }

      .selection-actions {
        display: flex;
        gap: 4px;
      }
    }
  }
}

.timezone-settings {
  .setting-item {
    margin-bottom: 20px;

    &:last-child {
      margin-bottom: 0;
    }

    .setting-label {
      display: block;
      font-size: 14px;
      color: #606266;
      margin-bottom: 8px;
      font-weight: 500;
    }
  }

  .current-time {
    padding: 12px;
    background: #f8f9fa;
    border-radius: 4px;
    margin-top: 16px;

    .time-label {
      font-size: 13px;
      color: #909399;
      margin-right: 8px;
    }

    .time-value {
      font-size: 13px;
      color: #303133;
      font-weight: 500;
      font-family: monospace;
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .time-range-selector {
    .selector-content {
      .custom-range .date-inputs {
        flex-direction: column;
        gap: 12px;

        .input-group {
          width: 100%;
        }
      }

      .current-selection {
        flex-direction: column;
        gap: 8px;
        align-items: stretch;

        .selection-info {
          justify-content: center;
        }

        .selection-actions {
          justify-content: center;
        }
      }
    }
  }

  .timezone-settings {
    .current-time {
      text-align: center;

      .time-label,
      .time-value {
        display: block;
      }

      .time-value {
        margin-top: 4px;
      }
    }
  }
}
</style>
