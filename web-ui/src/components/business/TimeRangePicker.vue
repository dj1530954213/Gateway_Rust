<template>
  <div class="time-range-picker" :class="containerClass">
    <!-- 时间选择器主体 -->
    <el-popover
      v-model:visible="pickerVisible"
      :width="popoverWidth"
      trigger="click"
      placement="bottom-start"
      :popper-class="popoverClass"
    >
      <template #reference>
        <div class="time-range-trigger" :class="triggerClass" @click="handleTriggerClick">
          <el-icon class="trigger-icon">
            <Clock />
          </el-icon>
          
          <div class="trigger-content">
            <div class="time-display">{{ displayText }}</div>
            <div v-if="showDuration" class="duration-display">
              {{ durationText }}
            </div>
          </div>
          
          <el-icon class="trigger-arrow" :class="{ 'is-reverse': pickerVisible }">
            <ArrowDown />
          </el-icon>
        </div>
      </template>
      
      <!-- 时间选择面板 -->
      <div class="time-picker-panel">
        <!-- 标签页切换 -->
        <el-tabs v-model="activeTab" type="card" @tab-change="handleTabChange">
          <!-- 快速选择 -->
          <el-tab-pane label="快速选择" name="quick">
            <div class="quick-ranges">
              <div class="range-categories">
                <div
                  v-for="category in quickRangeCategories"
                  :key="category.key"
                  class="category-section"
                >
                  <div class="category-title">{{ category.title }}</div>
                  <div class="range-buttons">
                    <el-button
                      v-for="range in category.ranges"
                      :key="range.key"
                      :type="isRangeSelected(range) ? 'primary' : 'default'"
                      size="small"
                      @click="handleQuickRange(range)"
                    >
                      {{ range.label }}
                    </el-button>
                  </div>
                </div>
              </div>
            </div>
          </el-tab-pane>
          
          <!-- 自定义选择 -->
          <el-tab-pane label="自定义" name="custom">
            <div class="custom-range">
              <!-- 时间模式选择 -->
              <div class="time-mode">
                <el-radio-group v-model="timeMode" @change="handleTimeModeChange">
                  <el-radio label="absolute">绝对时间</el-radio>
                  <el-radio label="relative">相对时间</el-radio>
                </el-radio-group>
              </div>
              
              <!-- 绝对时间选择 -->
              <div v-if="timeMode === 'absolute'" class="absolute-time">
                <el-form label-width="80px" size="small">
                  <el-form-item label="开始时间">
                    <el-date-picker
                      v-model="customRange.start"
                      type="datetime"
                      placeholder="选择开始时间"
                      format="YYYY-MM-DD HH:mm:ss"
                      value-format="YYYY-MM-DD HH:mm:ss"
                      @change="handleCustomRangeChange"
                    />
                  </el-form-item>
                  <el-form-item label="结束时间">
                    <el-date-picker
                      v-model="customRange.end"
                      type="datetime"
                      placeholder="选择结束时间"
                      format="YYYY-MM-DD HH:mm:ss"
                      value-format="YYYY-MM-DD HH:mm:ss"
                      @change="handleCustomRangeChange"
                    />
                  </el-form-item>
                </el-form>
              </div>
              
              <!-- 相对时间选择 -->
              <div v-else class="relative-time">
                <el-form label-width="80px" size="small">
                  <el-form-item label="时间长度">
                    <el-input-number
                      v-model="relativeTime.value"
                      :min="1"
                      :max="getMaxValue(relativeTime.unit)"
                      @change="handleRelativeTimeChange"
                    />
                    <el-select
                      v-model="relativeTime.unit"
                      style="width: 80px; margin-left: 8px"
                      @change="handleRelativeTimeChange"
                    >
                      <el-option label="分钟" value="minutes" />
                      <el-option label="小时" value="hours" />
                      <el-option label="天" value="days" />
                      <el-option label="周" value="weeks" />
                      <el-option label="月" value="months" />
                    </el-select>
                  </el-form-item>
                  
                  <el-form-item label="结束于">
                    <el-radio-group v-model="relativeTime.endAt" @change="handleRelativeTimeChange">
                      <el-radio label="now">现在</el-radio>
                      <el-radio label="custom">指定时间</el-radio>
                    </el-radio-group>
                  </el-form-item>
                  
                  <el-form-item v-if="relativeTime.endAt === 'custom'" label="结束时间">
                    <el-date-picker
                      v-model="relativeTime.customEnd"
                      type="datetime"
                      placeholder="选择结束时间"
                      format="YYYY-MM-DD HH:mm:ss"
                      value-format="YYYY-MM-DD HH:mm:ss"
                      @change="handleRelativeTimeChange"
                    />
                  </el-form-item>
                </el-form>
              </div>
            </div>
          </el-tab-pane>
          
          <!-- 预设模板 -->
          <el-tab-pane v-if="showTemplates" label="预设模板" name="templates">
            <div class="time-templates">
              <div class="template-actions">
                <el-button
                  type="primary"
                  size="small"
                  @click="handleSaveTemplate"
                  :disabled="!hasValidRange"
                >
                  保存当前为模板
                </el-button>
              </div>
              
              <el-divider />
              
              <div class="template-list">
                <div
                  v-for="template in timeTemplates"
                  :key="template.id"
                  class="template-item"
                  @click="handleLoadTemplate(template)"
                >
                  <div class="template-info">
                    <div class="template-name">{{ template.name }}</div>
                    <div class="template-range">{{ template.description }}</div>
                  </div>
                  
                  <el-button
                    type="text"
                    size="small"
                    :icon="Delete"
                    @click.stop="handleDeleteTemplate(template)"
                  />
                </div>
              </div>
            </div>
          </el-tab-pane>
          
          <!-- 历史记录 -->
          <el-tab-pane v-if="showHistory" label="历史记录" name="history">
            <div class="time-history">
              <div class="history-actions">
                <el-button
                  type="text"
                  size="small"
                  @click="clearHistory"
                >
                  清空历史
                </el-button>
              </div>
              
              <el-divider />
              
              <div class="history-list">
                <div
                  v-for="(record, index) in timeHistory"
                  :key="index"
                  class="history-item"
                  @click="handleLoadHistory(record)"
                >
                  <div class="history-info">
                    <div class="history-range">{{ record.description }}</div>
                    <div class="history-time">{{ formatTime(record.timestamp) }}</div>
                  </div>
                </div>
              </div>
            </div>
          </el-tab-pane>
        </el-tabs>
        
        <!-- 时间预览 -->
        <div class="time-preview">
          <el-divider content-position="left">时间预览</el-divider>
          <div class="preview-content">
            <div class="preview-item">
              <label>开始时间:</label>
              <span>{{ formatPreviewTime(currentRange.start) }}</span>
            </div>
            <div class="preview-item">
              <label>结束时间:</label>
              <span>{{ formatPreviewTime(currentRange.end) }}</span>
            </div>
            <div class="preview-item">
              <label>时间跨度:</label>
              <span>{{ previewDuration }}</span>
            </div>
          </div>
        </div>
        
        <!-- 操作按钮 -->
        <div class="picker-actions">
          <el-button size="small" @click="handleCancel">取消</el-button>
          <el-button
            type="primary"
            size="small"
            @click="handleConfirm"
            :disabled="!hasValidRange"
          >
            确定
          </el-button>
        </div>
      </div>
    </el-popover>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { Clock, ArrowDown, Delete } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'

export interface TimeRange {
  start: string | Date
  end: string | Date
}

export interface QuickRange {
  key: string
  label: string
  value: number
  unit: 'minutes' | 'hours' | 'days' | 'weeks' | 'months'
  type: 'relative' | 'absolute'
}

export interface TimeTemplate {
  id: string
  name: string
  description: string
  range: TimeRange
  type: 'absolute' | 'relative'
  createTime: Date
}

export interface TimeHistoryRecord {
  range: TimeRange
  description: string
  timestamp: Date
}

interface Props {
  modelValue?: TimeRange
  
  // 显示控制
  showTemplates?: boolean
  showHistory?: boolean
  showDuration?: boolean
  
  // 外观配置
  size?: 'small' | 'default' | 'large'
  placeholder?: string
  disabled?: boolean
  
  // 时间限制
  minDate?: Date
  maxDate?: Date
  disableFuture?: boolean
  
  // 快速选择配置
  enableQuickRanges?: string[]
  
  // 自定义样式
  popoverWidth?: number
  customClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  showTemplates: true,
  showHistory: true,
  showDuration: true,
  size: 'default',
  placeholder: '选择时间范围',
  disabled: false,
  disableFuture: false,
  popoverWidth: 500,
  enableQuickRanges: () => ['recent', 'today', 'custom']
})

interface Emits {
  'update:modelValue': [range: TimeRange]
  change: [range: TimeRange]
  confirm: [range: TimeRange]
  cancel: []
}

const emit = defineEmits<Emits>()

// 状态
const pickerVisible = ref(false)
const activeTab = ref('quick')
const timeMode = ref<'absolute' | 'relative'>('absolute')

// 当前时间范围
const currentRange = ref<TimeRange>({
  start: '',
  end: ''
})

// 自定义时间范围
const customRange = ref<TimeRange>({
  start: '',
  end: ''
})

// 相对时间配置
const relativeTime = ref({
  value: 1,
  unit: 'hours' as 'minutes' | 'hours' | 'days' | 'weeks' | 'months',
  endAt: 'now' as 'now' | 'custom',
  customEnd: ''
})

// 时间模板
const timeTemplates = ref<TimeTemplate[]>([
  {
    id: '1',
    name: '工作时间',
    description: '08:00-18:00',
    range: { start: '08:00:00', end: '18:00:00' },
    type: 'absolute',
    createTime: new Date()
  }
])

// 历史记录
const timeHistory = ref<TimeHistoryRecord[]>([])

// 快速时间范围配置
const quickRangeCategories = [
  {
    key: 'recent',
    title: '最近时间',
    ranges: [
      { key: 'last_15min', label: '最近15分钟', value: 15, unit: 'minutes', type: 'relative' },
      { key: 'last_1hour', label: '最近1小时', value: 1, unit: 'hours', type: 'relative' },
      { key: 'last_6hours', label: '最近6小时', value: 6, unit: 'hours', type: 'relative' },
      { key: 'last_12hours', label: '最近12小时', value: 12, unit: 'hours', type: 'relative' },
      { key: 'last_24hours', label: '最近24小时', value: 24, unit: 'hours', type: 'relative' }
    ]
  },
  {
    key: 'today',
    title: '今日时间',
    ranges: [
      { key: 'today', label: '今天', value: 0, unit: 'days', type: 'absolute' },
      { key: 'yesterday', label: '昨天', value: 1, unit: 'days', type: 'absolute' },
      { key: 'this_week', label: '本周', value: 0, unit: 'weeks', type: 'absolute' },
      { key: 'last_week', label: '上周', value: 1, unit: 'weeks', type: 'absolute' }
    ]
  },
  {
    key: 'period',
    title: '时间周期',
    ranges: [
      { key: 'last_7days', label: '最近7天', value: 7, unit: 'days', type: 'relative' },
      { key: 'last_30days', label: '最近30天', value: 30, unit: 'days', type: 'relative' },
      { key: 'last_3months', label: '最近3个月', value: 3, unit: 'months', type: 'relative' }
    ]
  }
]

// 计算属性
const containerClass = computed(() => {
  const classes = []
  
  classes.push(`time-range-picker--${props.size}`)
  
  if (props.disabled) {
    classes.push('is-disabled')
  }
  
  if (props.customClass) {
    classes.push(props.customClass)
  }
  
  return classes.join(' ')
})

const triggerClass = computed(() => {
  const classes = ['time-range-trigger']
  
  if (pickerVisible.value) {
    classes.push('is-active')
  }
  
  if (props.disabled) {
    classes.push('is-disabled')
  }
  
  return classes.join(' ')
})

const popoverClass = computed(() => {
  return `time-range-popover time-range-popover--${props.size}`
})

const displayText = computed(() => {
  if (!currentRange.value.start || !currentRange.value.end) {
    return props.placeholder
  }
  
  const start = formatDisplayTime(currentRange.value.start)
  const end = formatDisplayTime(currentRange.value.end)
  
  return `${start} ~ ${end}`
})

const durationText = computed(() => {
  if (!currentRange.value.start || !currentRange.value.end) {
    return ''
  }
  
  return calculateDuration(currentRange.value.start, currentRange.value.end)
})

const previewDuration = computed(() => {
  if (!currentRange.value.start || !currentRange.value.end) {
    return '无效范围'
  }
  
  return calculateDuration(currentRange.value.start, currentRange.value.end)
})

const hasValidRange = computed(() => {
  return currentRange.value.start && currentRange.value.end && 
         new Date(currentRange.value.start) < new Date(currentRange.value.end)
})

// 方法
const formatTime = (date: Date) => {
  return date.toLocaleString('zh-CN')
}

const formatDisplayTime = (time: string | Date) => {
  const date = new Date(time)
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

const formatPreviewTime = (time: string | Date) => {
  if (!time) return '未设置'
  const date = new Date(time)
  return date.toLocaleString('zh-CN')
}

const calculateDuration = (start: string | Date, end: string | Date) => {
  const startTime = new Date(start)
  const endTime = new Date(end)
  const diff = endTime.getTime() - startTime.getTime()
  
  const days = Math.floor(diff / (24 * 60 * 60 * 1000))
  const hours = Math.floor((diff % (24 * 60 * 60 * 1000)) / (60 * 60 * 1000))
  const minutes = Math.floor((diff % (60 * 60 * 1000)) / (60 * 1000))
  
  if (days > 0) {
    return `${days}天${hours}小时`
  } else if (hours > 0) {
    return `${hours}小时${minutes}分钟`
  } else {
    return `${minutes}分钟`
  }
}

const getMaxValue = (unit: string) => {
  const maxValues = {
    minutes: 1440, // 24小时
    hours: 168,    // 7天
    days: 365,     // 1年
    weeks: 52,     // 1年
    months: 12     // 1年
  }
  return maxValues[unit] || 100
}

const isRangeSelected = (range: QuickRange) => {
  // 简单比较逻辑，实际应该根据range计算时间范围并比较
  return false
}

// 事件处理
const handleTriggerClick = () => {
  if (!props.disabled) {
    pickerVisible.value = !pickerVisible.value
  }
}

const handleTabChange = (tabName: string) => {
  activeTab.value = tabName
}

const handleTimeModeChange = () => {
  // 时间模式变化处理
  updateCurrentRange()
}

const handleQuickRange = (range: QuickRange) => {
  const now = new Date()
  let start: Date
  let end: Date = now
  
  if (range.type === 'relative') {
    switch (range.unit) {
      case 'minutes':
        start = new Date(now.getTime() - range.value * 60 * 1000)
        break
      case 'hours':
        start = new Date(now.getTime() - range.value * 60 * 60 * 1000)
        break
      case 'days':
        start = new Date(now.getTime() - range.value * 24 * 60 * 60 * 1000)
        break
      case 'weeks':
        start = new Date(now.getTime() - range.value * 7 * 24 * 60 * 60 * 1000)
        break
      case 'months':
        start = new Date()
        start.setMonth(start.getMonth() - range.value)
        break
      default:
        start = new Date(now.getTime() - 60 * 60 * 1000) // 默认1小时
    }
  } else {
    // 绝对时间处理（今天、昨天等）
    start = new Date()
    end = new Date()
    
    switch (range.key) {
      case 'today':
        start.setHours(0, 0, 0, 0)
        end.setHours(23, 59, 59, 999)
        break
      case 'yesterday':
        start.setDate(start.getDate() - 1)
        start.setHours(0, 0, 0, 0)
        end.setDate(end.getDate() - 1)
        end.setHours(23, 59, 59, 999)
        break
      // 更多绝对时间处理...
    }
  }
  
  currentRange.value = {
    start: start.toISOString(),
    end: end.toISOString()
  }
  
  addToHistory()
}

const handleCustomRangeChange = () => {
  if (customRange.value.start && customRange.value.end) {
    currentRange.value = { ...customRange.value }
  }
}

const handleRelativeTimeChange = () => {
  updateCurrentRange()
}

const updateCurrentRange = () => {
  if (timeMode.value === 'relative') {
    const now = relativeTime.value.endAt === 'now' ? new Date() : new Date(relativeTime.value.customEnd)
    let start: Date
    
    switch (relativeTime.value.unit) {
      case 'minutes':
        start = new Date(now.getTime() - relativeTime.value.value * 60 * 1000)
        break
      case 'hours':
        start = new Date(now.getTime() - relativeTime.value.value * 60 * 60 * 1000)
        break
      case 'days':
        start = new Date(now.getTime() - relativeTime.value.value * 24 * 60 * 60 * 1000)
        break
      case 'weeks':
        start = new Date(now.getTime() - relativeTime.value.value * 7 * 24 * 60 * 60 * 1000)
        break
      case 'months':
        start = new Date(now)
        start.setMonth(start.getMonth() - relativeTime.value.value)
        break
      default:
        start = new Date(now.getTime() - 60 * 60 * 1000)
    }
    
    currentRange.value = {
      start: start.toISOString(),
      end: now.toISOString()
    }
  }
}

const handleSaveTemplate = () => {
  const templateName = prompt('请输入模板名称:')
  if (templateName && hasValidRange.value) {
    const template: TimeTemplate = {
      id: Date.now().toString(),
      name: templateName,
      description: displayText.value,
      range: { ...currentRange.value },
      type: timeMode.value,
      createTime: new Date()
    }
    
    timeTemplates.value.push(template)
    ElMessage.success('模板保存成功')
  }
}

const handleLoadTemplate = (template: TimeTemplate) => {
  currentRange.value = { ...template.range }
  ElMessage.success('模板加载成功')
}

const handleDeleteTemplate = (template: TimeTemplate) => {
  const index = timeTemplates.value.findIndex(t => t.id === template.id)
  if (index > -1) {
    timeTemplates.value.splice(index, 1)
    ElMessage.success('模板删除成功')
  }
}

const handleLoadHistory = (record: TimeHistoryRecord) => {
  currentRange.value = { ...record.range }
}

const clearHistory = () => {
  timeHistory.value = []
  ElMessage.success('历史记录已清空')
}

const addToHistory = () => {
  if (!hasValidRange.value) return
  
  const record: TimeHistoryRecord = {
    range: { ...currentRange.value },
    description: displayText.value,
    timestamp: new Date()
  }
  
  // 去重并添加到开头
  timeHistory.value = timeHistory.value.filter(h => h.description !== record.description)
  timeHistory.value.unshift(record)
  
  // 限制历史记录数量
  if (timeHistory.value.length > 10) {
    timeHistory.value = timeHistory.value.slice(0, 10)
  }
}

const handleConfirm = () => {
  if (hasValidRange.value) {
    emit('update:modelValue', currentRange.value)
    emit('change', currentRange.value)
    emit('confirm', currentRange.value)
    addToHistory()
    pickerVisible.value = false
  }
}

const handleCancel = () => {
  pickerVisible.value = false
  emit('cancel')
}

// 监听
watch(() => props.modelValue, (newValue) => {
  if (newValue) {
    currentRange.value = { ...newValue }
  }
}, { immediate: true })

// 生命周期
onMounted(() => {
  if (props.modelValue) {
    currentRange.value = { ...props.modelValue }
  }
})
</script>

<style scoped lang="scss">
.time-range-picker {
  display: inline-block;
  
  &.time-range-picker--small {
    .time-range-trigger {
      padding: 4px 8px;
      font-size: 12px;
    }
  }
  
  &.time-range-picker--large {
    .time-range-trigger {
      padding: 8px 12px;
      font-size: 16px;
    }
  }
  
  &.is-disabled {
    .time-range-trigger {
      background-color: var(--el-disabled-bg-color);
      border-color: var(--el-disabled-border-color);
      color: var(--el-disabled-text-color);
      cursor: not-allowed;
    }
  }
}

.time-range-trigger {
  display: flex;
  align-items: center;
  padding: 6px 10px;
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  background-color: var(--el-bg-color);
  cursor: pointer;
  transition: all 0.3s;
  min-width: 200px;
  
  &:hover {
    border-color: var(--el-color-primary);
  }
  
  &.is-active {
    border-color: var(--el-color-primary);
    box-shadow: 0 0 0 2px var(--el-color-primary-light-8);
  }
  
  .trigger-icon {
    margin-right: 8px;
    color: var(--el-text-color-secondary);
  }
  
  .trigger-content {
    flex: 1;
    min-width: 0;
    
    .time-display {
      font-size: 14px;
      color: var(--el-text-color-primary);
    }
    
    .duration-display {
      font-size: 12px;
      color: var(--el-text-color-secondary);
      margin-top: 2px;
    }
  }
  
  .trigger-arrow {
    margin-left: 8px;
    color: var(--el-text-color-secondary);
    transition: transform 0.3s;
    
    &.is-reverse {
      transform: rotate(180deg);
    }
  }
}

:deep(.time-range-popover) {
  padding: 0;
  
  .time-picker-panel {
    .el-tabs {
      .el-tabs__header {
        margin: 0;
        
        .el-tabs__nav-wrap {
          padding: 0 16px;
        }
      }
      
      .el-tabs__content {
        padding: 16px;
        min-height: 300px;
      }
    }
  }
}

.quick-ranges {
  .range-categories {
    .category-section {
      margin-bottom: 16px;
      
      .category-title {
        font-size: 13px;
        font-weight: 600;
        color: var(--el-text-color-primary);
        margin-bottom: 8px;
      }
      
      .range-buttons {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }
    }
  }
}

.custom-range {
  .time-mode {
    margin-bottom: 16px;
  }
  
  .absolute-time,
  .relative-time {
    .el-form-item {
      margin-bottom: 12px;
    }
  }
}

.time-templates,
.time-history {
  .template-actions,
  .history-actions {
    text-align: right;
    margin-bottom: 8px;
  }
  
  .template-list,
  .history-list {
    max-height: 200px;
    overflow-y: auto;
    
    .template-item,
    .history-item {
      display: flex;
      align-items: center;
      padding: 8px;
      border-radius: 4px;
      cursor: pointer;
      transition: background-color 0.3s;
      
      &:hover {
        background-color: var(--el-fill-color-light);
      }
      
      .template-info,
      .history-info {
        flex: 1;
        
        .template-name,
        .history-range {
          font-size: 14px;
          color: var(--el-text-color-primary);
          margin-bottom: 2px;
        }
        
        .template-range,
        .history-time {
          font-size: 12px;
          color: var(--el-text-color-secondary);
        }
      }
    }
  }
}

.time-preview {
  border-top: 1px solid var(--el-border-color-lighter);
  padding-top: 12px;
  
  .preview-content {
    .preview-item {
      display: flex;
      justify-content: space-between;
      margin-bottom: 4px;
      
      label {
        font-size: 12px;
        color: var(--el-text-color-secondary);
      }
      
      span {
        font-size: 12px;
        color: var(--el-text-color-primary);
      }
    }
  }
}

.picker-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--el-border-color-lighter);
}

// 响应式设计
@media (max-width: 768px) {
  .time-range-trigger {
    min-width: 150px;
    
    .trigger-content {
      .time-display {
        font-size: 12px;
      }
      
      .duration-display {
        font-size: 11px;
      }
    }
  }
  
  :deep(.time-range-popover) {
    max-width: 90vw;
  }
}
</style>