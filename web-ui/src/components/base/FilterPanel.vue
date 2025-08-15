<template>
  <div class="filter-panel" :class="panelClass">
    <!-- 筛选头部 -->
    <div v-if="showHeader" class="filter-header">
      <div class="filter-title">
        <el-icon v-if="titleIcon">
          <component :is="titleIcon" />
        </el-icon>
        <span>{{ title }}</span>
      </div>

      <div class="filter-actions">
        <el-button
          v-if="showReset"
          type="link"
          size="small"
          :disabled="!hasActiveFilters"
          @click="handleReset"
        >
          重置筛选 ({{ activeFilterCount }})
        </el-button>

        <el-button
          v-if="collapsible"
          type="link"
          size="small"
          :icon="isCollapsed ? ArrowDown : ArrowUp"
          @click="toggleCollapse"
        />
      </div>
    </div>

    <!-- 筛选内容 -->
    <el-collapse-transition>
      <div v-show="!isCollapsed" class="filter-content">
        <el-form
          :model="filterValues"
          :label-width="labelWidth"
          :size="size"
          :inline="inline"
          class="filter-form"
        >
          <template v-for="filter in processedFilters" :key="filter.key">
            <!-- 分组标题 -->
            <div v-if="filter.type === 'group'" class="filter-group">
              <el-divider content-position="left">
                {{ filter.label }}
              </el-divider>
            </div>

            <!-- 筛选项 -->
            <el-form-item
              v-else
              :label="filter.label"
              :label-width="filter.labelWidth"
              class="filter-item"
              :class="filter.className"
            >
              <!-- 文本输入 -->
              <el-input
                v-if="filter.type === 'text'"
                v-model="filterValues[filter.key]"
                :placeholder="filter.placeholder"
                :clearable="filter.clearable !== false"
                @input="handleFilterChange(filter.key, $event)"
                @clear="handleFilterClear(filter.key)"
              />

              <!-- 选择器 -->
              <el-select
                v-else-if="
                  filter.type === 'select' || filter.type === 'multiSelect'
                "
                v-model="filterValues[filter.key]"
                :placeholder="filter.placeholder"
                :clearable="filter.clearable !== false"
                :multiple="filter.type === 'multiSelect'"
                :collapse-tags="filter.collapseTags"
                :collapse-tags-tooltip="filter.collapseTagsTooltip"
                :filterable="filter.filterable"
                @change="handleFilterChange(filter.key, $event)"
                @clear="handleFilterClear(filter.key)"
              >
                <el-option
                  v-for="option in getFilterOptions(filter)"
                  :key="option.value"
                  :label="option.label"
                  :value="option.value"
                  :disabled="option.disabled"
                />
              </el-select>

              <!-- 日期范围选择器 -->
              <el-date-picker
                v-else-if="filter.type === 'daterange'"
                v-model="filterValues[filter.key]"
                type="daterange"
                :placeholder="filter.placeholder"
                :range-separator="filter.rangeSeparator || '至'"
                :start-placeholder="filter.startPlaceholder || '开始日期'"
                :end-placeholder="filter.endPlaceholder || '结束日期'"
                :value-format="filter.valueFormat || 'YYYY-MM-DD'"
                :clearable="filter.clearable !== false"
                @change="handleFilterChange(filter.key, $event)"
              />

              <!-- 日期时间范围选择器 -->
              <el-date-picker
                v-else-if="filter.type === 'datetimerange'"
                v-model="filterValues[filter.key]"
                type="datetimerange"
                :placeholder="filter.placeholder"
                :range-separator="filter.rangeSeparator || '至'"
                :start-placeholder="filter.startPlaceholder || '开始时间'"
                :end-placeholder="filter.endPlaceholder || '结束时间'"
                :value-format="filter.valueFormat || 'YYYY-MM-DD HH:mm:ss'"
                :clearable="filter.clearable !== false"
                @change="handleFilterChange(filter.key, $event)"
              />

              <!-- 数字范围 -->
              <div
                v-else-if="filter.type === 'numberRange'"
                class="number-range"
              >
                <el-input-number
                  v-model="filterValues[filter.key + '_min']"
                  :placeholder="filter.minPlaceholder || '最小值'"
                  :min="filter.min"
                  :max="filter.max"
                  :step="filter.step"
                  :precision="filter.precision"
                  size="small"
                  @change="handleRangeChange(filter.key, 'min', $event)"
                />
                <span class="range-separator">-</span>
                <el-input-number
                  v-model="filterValues[filter.key + '_max']"
                  :placeholder="filter.maxPlaceholder || '最大值'"
                  :min="filter.min"
                  :max="filter.max"
                  :step="filter.step"
                  :precision="filter.precision"
                  size="small"
                  @change="handleRangeChange(filter.key, 'max', $event)"
                />
              </div>

              <!-- 开关 -->
              <el-switch
                v-else-if="filter.type === 'switch'"
                v-model="filterValues[filter.key]"
                :active-text="filter.activeText"
                :inactive-text="filter.inactiveText"
                :active-value="filter.activeValue"
                :inactive-value="filter.inactiveValue"
                @change="handleFilterChange(filter.key, $event)"
              />

              <!-- 单选框组 -->
              <el-radio-group
                v-else-if="filter.type === 'radio'"
                v-model="filterValues[filter.key]"
                @change="handleFilterChange(filter.key, $event)"
              >
                <el-radio
                  v-for="option in getFilterOptions(filter)"
                  :key="option.value"
                  :label="option.value"
                  :disabled="option.disabled"
                >
                  {{ option.label }}
                </el-radio>
              </el-radio-group>

              <!-- 复选框组 -->
              <el-checkbox-group
                v-else-if="filter.type === 'checkbox'"
                v-model="filterValues[filter.key]"
                @change="handleFilterChange(filter.key, $event)"
              >
                <el-checkbox
                  v-for="option in getFilterOptions(filter)"
                  :key="option.value"
                  :label="option.value"
                  :disabled="option.disabled"
                >
                  {{ option.label }}
                </el-checkbox>
              </el-checkbox-group>

              <!-- 滑块 -->
              <el-slider
                v-else-if="filter.type === 'slider'"
                v-model="filterValues[filter.key]"
                :min="filter.min"
                :max="filter.max"
                :step="filter.step"
                :range="filter.range"
                :show-stops="filter.showStops"
                :show-tooltip="filter.showTooltip"
                :marks="filter.marks"
                @change="handleFilterChange(filter.key, $event)"
              />

              <!-- 自定义插槽 -->
              <slot
                v-else-if="filter.type === 'slot'"
                :name="filter.slotName || filter.key"
                :filter="filter"
                :value="filterValues[filter.key]"
                :on-change="value => handleFilterChange(filter.key, value)"
              />
            </el-form-item>
          </template>
        </el-form>

        <!-- 快速筛选标签 -->
        <div
          v-if="showQuickFilters && quickFilters.length > 0"
          class="quick-filters"
        >
          <el-divider content-position="left">快速筛选</el-divider>
          <div class="quick-filter-tags">
            <el-tag
              v-for="quick in quickFilters"
              :key="quick.key"
              :type="isQuickFilterActive(quick) ? 'primary' : 'info'"
              :effect="isQuickFilterActive(quick) ? 'dark' : 'plain'"
              clickable
              @click="handleQuickFilter(quick)"
            >
              {{ quick.label }}
            </el-tag>
          </div>
        </div>

        <!-- 应用按钮 -->
        <div v-if="showApplyButton" class="filter-apply">
          <el-button
            type="primary"
            size="small"
            :loading="applying"
            @click="handleApply"
          >
            应用筛选
          </el-button>
          <el-button size="small" @click="handleReset"> 重置 </el-button>
        </div>
      </div>
    </el-collapse-transition>

    <!-- 已选筛选条件显示 -->
    <div
      v-if="showActiveFilters && activeFilters.length > 0"
      class="active-filters"
    >
      <span class="active-filters-label">当前筛选：</span>
      <el-tag
        v-for="(filter, index) in activeFilters"
        :key="index"
        closable
        size="small"
        @close="handleRemoveFilter(filter.key)"
      >
        {{ filter.label }}: {{ filter.displayValue }}
      </el-tag>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowDown, ArrowUp, Filter } from '@element-plus/icons-vue'
import { ref, reactive, computed, watch, onMounted } from 'vue'

export interface FilterItem {
  key: string
  label: string
  type:
    | 'text'
    | 'select'
    | 'multiSelect'
    | 'daterange'
    | 'datetimerange'
    | 'numberRange'
    | 'switch'
    | 'radio'
    | 'checkbox'
    | 'slider'
    | 'slot'
    | 'group'
  placeholder?: string
  clearable?: boolean
  className?: string
  labelWidth?: string

  // 选择器选项
  options?: Array<{ label: string; value: any; disabled?: boolean }>
  optionsLoader?: () => Promise<
    Array<{ label: string; value: any; disabled?: boolean }>
  >

  // 选择器特有属性
  collapseTags?: boolean
  collapseTagsTooltip?: boolean
  filterable?: boolean

  // 日期选择器特有属性
  rangeSeparator?: string
  startPlaceholder?: string
  endPlaceholder?: string
  valueFormat?: string

  // 数字范围特有属性
  min?: number
  max?: number
  step?: number
  precision?: number
  minPlaceholder?: string
  maxPlaceholder?: string

  // 开关特有属性
  activeText?: string
  inactiveText?: string
  activeValue?: any
  inactiveValue?: any

  // 滑块特有属性
  range?: boolean
  showStops?: boolean
  showTooltip?: boolean
  marks?: Record<number, string>

  // 自定义插槽
  slotName?: string
}

export interface QuickFilter {
  key: string
  label: string
  filters: Record<string, any>
}

interface Props {
  filters: FilterItem[]
  modelValue?: Record<string, any>

  // 外观
  title?: string
  titleIcon?: any
  showHeader?: boolean
  showReset?: boolean
  collapsible?: boolean
  collapsed?: boolean

  // 表单布局
  inline?: boolean
  labelWidth?: string
  size?: 'large' | 'default' | 'small'

  // 功能选项
  showQuickFilters?: boolean
  quickFilters?: QuickFilter[]
  showApplyButton?: boolean
  showActiveFilters?: boolean
  applying?: boolean

  // 实时筛选
  realtime?: boolean
  debounceDelay?: number

  // 样式
  panelClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: () => ({}),
  title: '筛选条件',
  titleIcon: Filter,
  showHeader: true,
  showReset: true,
  collapsible: false,
  collapsed: false,
  inline: true,
  labelWidth: '80px',
  size: 'default',
  showQuickFilters: false,
  quickFilters: () => [],
  showApplyButton: false,
  showActiveFilters: true,
  applying: false,
  realtime: true,
  debounceDelay: 300,
})

interface Emits {
  'update:modelValue': [value: Record<string, any>]
  filter: [filters: Record<string, any>]
  reset: []
  apply: [filters: Record<string, any>]
}

const emit = defineEmits<Emits>()

// 状态
const isCollapsed = ref(props.collapsed)
const filterValues = reactive<Record<string, any>>({ ...props.modelValue })

// 防抖定时器
let debounceTimer: NodeJS.Timeout | null = null

// 监听modelValue变化
watch(
  () => props.modelValue,
  newValue => {
    Object.assign(filterValues, newValue)
  },
  { deep: true }
)

// 监听filterValues变化
watch(
  filterValues,
  newValue => {
    emit('update:modelValue', { ...newValue })

    if (props.realtime && !props.showApplyButton) {
      handleDebouncedFilter()
    }
  },
  { deep: true }
)

// 处理后的筛选项列表
const processedFilters = computed(() => {
  return props.filters.map(filter => {
    // 为选择器字段加载选项
    if (
      (filter.type === 'select' || filter.type === 'multiSelect') &&
      filter.optionsLoader &&
      !filter.options
    ) {
      filter.optionsLoader().then(options => {
        filter.options = options
      })
    }

    return filter
  })
})

// 活跃筛选条件数量
const activeFilterCount = computed(() => {
  return Object.values(filterValues).filter(value => {
    if (Array.isArray(value)) return value.length > 0
    return value !== null && value !== undefined && value !== ''
  }).length
})

// 是否有活跃筛选条件
const hasActiveFilters = computed(() => activeFilterCount.value > 0)

// 已选筛选条件显示
const activeFilters = computed(() => {
  const result = []

  for (const [key, value] of Object.entries(filterValues)) {
    if (
      value === null ||
      value === undefined ||
      value === '' ||
      (Array.isArray(value) && value.length === 0)
    ) {
      continue
    }

    const filter = props.filters.find(
      f => f.key === key || key.startsWith(f.key)
    )
    if (!filter) continue

    let displayValue = ''

    if (filter.type === 'select' || filter.type === 'multiSelect') {
      const options = getFilterOptions(filter)
      if (Array.isArray(value)) {
        displayValue = value
          .map(v => {
            const option = options.find(o => o.value === v)
            return option ? option.label : v
          })
          .join(', ')
      } else {
        const option = options.find(o => o.value === value)
        displayValue = option ? option.label : value
      }
    } else if (filter.type === 'daterange' || filter.type === 'datetimerange') {
      if (Array.isArray(value) && value.length === 2) {
        displayValue = `${value[0]} ~ ${value[1]}`
      }
    } else if (filter.type === 'numberRange') {
      const minKey = `${key}_min`
      const maxKey = `${key}_max`
      const minValue = filterValues[minKey]
      const maxValue = filterValues[maxKey]
      if (minValue !== null && maxValue !== null) {
        displayValue = `${minValue} ~ ${maxValue}`
      } else if (minValue !== null) {
        displayValue = `≥ ${minValue}`
      } else if (maxValue !== null) {
        displayValue = `≤ ${maxValue}`
      }
    } else {
      displayValue = String(value)
    }

    if (displayValue) {
      result.push({
        key,
        label: filter.label,
        displayValue,
      })
    }
  }

  return result
})

// 获取筛选项选项
const getFilterOptions = (filter: FilterItem) => {
  return filter.options || []
}

// 判断快速筛选是否激活
const isQuickFilterActive = (quick: QuickFilter) => {
  if (!quick || !quick.filters || typeof quick.filters !== 'object') {
    return false
  }
  return Object.entries(quick.filters).every(([key, value]) => {
    return filterValues[key] === value
  })
}

// 防抖筛选
const handleDebouncedFilter = () => {
  if (debounceTimer) {
    clearTimeout(debounceTimer)
  }

  debounceTimer = setTimeout(() => {
    emit('filter', { ...filterValues })
  }, props.debounceDelay)
}

// 事件处理
const toggleCollapse = () => {
  isCollapsed.value = !isCollapsed.value
}

const handleFilterChange = (key: string, value: any) => {
  filterValues[key] = value
}

const handleFilterClear = (key: string) => {
  filterValues[key] = null
}

const handleRangeChange = (key: string, type: 'min' | 'max', value: any) => {
  const rangeKey = `${key}_${type}`
  filterValues[rangeKey] = value

  // 组合范围值
  const minValue = filterValues[`${key}_min`]
  const maxValue = filterValues[`${key}_max`]
  if (minValue !== null || maxValue !== null) {
    filterValues[key] = { min: minValue, max: maxValue }
  } else {
    delete filterValues[key]
  }
}

const handleQuickFilter = (quick: QuickFilter) => {
  if (!quick || !quick.filters || typeof quick.filters !== 'object') {
    return
  }

  if (isQuickFilterActive(quick)) {
    // 取消快速筛选
    Object.keys(quick.filters).forEach(key => {
      filterValues[key] = null
    })
  } else {
    // 应用快速筛选
    Object.assign(filterValues, quick.filters)
  }
}

const handleApply = () => {
  emit('apply', { ...filterValues })
}

const handleReset = () => {
  Object.keys(filterValues).forEach(key => {
    filterValues[key] = null
  })
  emit('reset')
}

const handleRemoveFilter = (key: string) => {
  filterValues[key] = null
}

// 初始化
onMounted(() => {
  // 初始化数字范围筛选的子字段
  props.filters.forEach(filter => {
    if (filter.type === 'numberRange') {
      if (filterValues[filter.key]) {
        const range = filterValues[filter.key]
        filterValues[`${filter.key}_min`] = range.min
        filterValues[`${filter.key}_max`] = range.max
      } else {
        filterValues[`${filter.key}_min`] = null
        filterValues[`${filter.key}_max`] = null
      }
    }
  })
})
</script>

<style scoped lang="scss">
.filter-panel {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 6px;

  .filter-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--el-border-color-lighter);
    background: var(--el-bg-color-light);

    .filter-title {
      display: flex;
      align-items: center;
      gap: 6px;
      font-weight: 600;
      color: var(--el-text-color-primary);
    }

    .filter-actions {
      display: flex;
      align-items: center;
      gap: 8px;
    }
  }

  .filter-content {
    padding: 16px;

    .filter-form {
      .filter-group {
        margin: 16px 0 8px 0;

        &:first-child {
          margin-top: 0;
        }
      }

      .filter-item {
        margin-bottom: 12px;

        .number-range {
          display: flex;
          align-items: center;
          gap: 8px;

          .range-separator {
            color: var(--el-text-color-secondary);
          }
        }
      }
    }

    .quick-filters {
      margin-top: 16px;

      .quick-filter-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-top: 8px;
      }
    }

    .filter-apply {
      margin-top: 16px;
      padding-top: 16px;
      border-top: 1px solid var(--el-border-color-lighter);

      .el-button + .el-button {
        margin-left: 8px;
      }
    }
  }

  .active-filters {
    padding: 8px 16px;
    background: var(--el-bg-color-light);
    border-top: 1px solid var(--el-border-color-lighter);
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;

    .active-filters-label {
      font-size: 12px;
      color: var(--el-text-color-secondary);
      white-space: nowrap;
    }
  }
}

// 内联模式样式调整
.filter-panel {
  .filter-form.el-form--inline {
    .filter-item {
      margin-right: 16px;
      margin-bottom: 8px;
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .filter-panel {
    .filter-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .filter-content {
      .filter-form.el-form--inline {
        .filter-item {
          margin-right: 0;
          margin-bottom: 12px;
          display: block;
        }
      }

      .quick-filter-tags {
        justify-content: flex-start;
      }
    }

    .active-filters {
      flex-direction: column;
      align-items: flex-start;
    }
  }
}
</style>
