<template>
  <div class="search-box" :class="containerClass">
    <el-input
      v-model="searchValue"
      :placeholder="placeholder"
      :size="size"
      :disabled="disabled"
      :clearable="clearable"
      :prefix-icon="prefixIcon"
      :suffix-icon="suffixIcon"
      :class="inputClass"
      @input="handleInput"
      @change="handleChange"
      @clear="handleClear"
      @focus="handleFocus"
      @blur="handleBlur"
      @keyup.enter="handleEnter"
    >
      <template #prepend v-if="$slots.prepend">
        <slot name="prepend" />
      </template>
      
      <template #append v-if="showSearchButton || $slots.append">
        <slot name="append">
          <el-button
            v-if="showSearchButton"
            :icon="searchButtonIcon"
            :loading="searching"
            :disabled="disabled"
            @click="handleSearch"
          >
            {{ searchButtonText }}
          </el-button>
        </slot>
      </template>
    </el-input>
    
    <!-- 搜索建议下拉 -->
    <div
      v-if="showSuggestions && filteredSuggestions.length > 0"
      class="search-suggestions"
      ref="suggestionsRef"
    >
      <div
        v-for="(suggestion, index) in filteredSuggestions"
        :key="index"
        class="suggestion-item"
        :class="{ 'suggestion-item--active': index === activeSuggestionIndex }"
        @click="handleSuggestionClick(suggestion)"
        @mouseenter="activeSuggestionIndex = index"
      >
        <el-icon v-if="suggestion.icon" class="suggestion-icon">
          <component :is="suggestion.icon" />
        </el-icon>
        
        <div class="suggestion-content">
          <div class="suggestion-text" v-html="highlightMatch(suggestion.text)"></div>
          <div v-if="suggestion.description" class="suggestion-description">
            {{ suggestion.description }}
          </div>
        </div>
        
        <div v-if="suggestion.category" class="suggestion-category">
          {{ suggestion.category }}
        </div>
      </div>
      
      <div v-if="showHistory && recentSearches.length > 0" class="search-history">
        <el-divider>最近搜索</el-divider>
        <div
          v-for="(item, index) in recentSearches"
          :key="`history-${index}`"
          class="history-item"
          @click="handleHistoryClick(item)"
        >
          <el-icon class="history-icon">
            <Clock />
          </el-icon>
          <span class="history-text">{{ item }}</span>
          <el-button
            type="text"
            size="small"
            :icon="Close"
            @click.stop="removeFromHistory(index)"
          />
        </div>
      </div>
    </div>
    
    <!-- 高级搜索面板 -->
    <div v-if="showAdvancedSearch && advancedVisible" class="advanced-search">
      <el-card shadow="never">
        <template #header>
          <div class="advanced-search-header">
            <span>高级搜索</span>
            <el-button type="text" :icon="Close" @click="advancedVisible = false" />
          </div>
        </template>
        
        <el-form :model="advancedForm" size="small" label-width="80px">
          <el-form-item
            v-for="field in advancedFields"
            :key="field.key"
            :label="field.label"
          >
            <el-input
              v-if="field.type === 'text'"
              v-model="advancedForm[field.key]"
              :placeholder="field.placeholder"
            />
            
            <el-select
              v-else-if="field.type === 'select'"
              v-model="advancedForm[field.key]"
              :placeholder="field.placeholder"
              clearable
            >
              <el-option
                v-for="option in field.options"
                :key="option.value"
                :label="option.label"
                :value="option.value"
              />
            </el-select>
            
            <el-date-picker
              v-else-if="field.type === 'daterange'"
              v-model="advancedForm[field.key]"
              type="daterange"
              range-separator="至"
              start-placeholder="开始日期"
              end-placeholder="结束日期"
              :placeholder="field.placeholder"
            />
          </el-form-item>
          
          <el-form-item>
            <el-button type="primary" size="small" @click="handleAdvancedSearch">
              搜索
            </el-button>
            <el-button size="small" @click="handleAdvancedReset">
              重置
            </el-button>
          </el-form-item>
        </el-form>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { Search, Close, Clock } from '@element-plus/icons-vue'

export interface SearchSuggestion {
  text: string
  value?: any
  description?: string
  category?: string
  icon?: any
}

export interface AdvancedField {
  key: string
  label: string
  type: 'text' | 'select' | 'daterange'
  placeholder?: string
  options?: Array<{ label: string; value: any }>
}

interface Props {
  modelValue?: string
  placeholder?: string
  size?: 'large' | 'default' | 'small'
  disabled?: boolean
  clearable?: boolean
  searching?: boolean
  
  // 图标
  prefixIcon?: any
  suffixIcon?: any
  searchButtonIcon?: any
  searchButtonText?: string
  showSearchButton?: boolean
  
  // 搜索建议
  suggestions?: SearchSuggestion[]
  showSuggestions?: boolean
  maxSuggestions?: number
  minChars?: number
  
  // 搜索历史
  showHistory?: boolean
  maxHistory?: number
  historyKey?: string
  
  // 高级搜索
  showAdvancedSearch?: boolean
  advancedFields?: AdvancedField[]
  
  // 实时搜索
  realtime?: boolean
  debounceDelay?: number
  
  // 样式
  width?: string
  containerClass?: string
  inputClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  placeholder: '请输入搜索关键词',
  size: 'default',
  disabled: false,
  clearable: true,
  searching: false,
  prefixIcon: Search,
  searchButtonIcon: Search,
  searchButtonText: '搜索',
  showSearchButton: false,
  suggestions: () => [],
  showSuggestions: true,
  maxSuggestions: 10,
  minChars: 1,
  showHistory: true,
  maxHistory: 5,
  historyKey: 'search_history',
  showAdvancedSearch: false,
  advancedFields: () => [],
  realtime: false,
  debounceDelay: 300,
})

interface Emits {
  'update:modelValue': [value: string]
  search: [keyword: string, advanced?: any]
  change: [value: string]
  clear: []
  focus: [event: FocusEvent]
  blur: [event: FocusEvent]
  suggestionSelect: [suggestion: SearchSuggestion]
  advancedSearch: [conditions: any]
}

const emit = defineEmits<Emits>()

// 状态
const searchValue = ref(props.modelValue)
const showSuggestions = ref(false)
const activeSuggestionIndex = ref(-1)
const advancedVisible = ref(false)
const suggestionsRef = ref()

// 搜索历史
const recentSearches = ref<string[]>([])

// 高级搜索表单
const advancedForm = reactive<Record<string, any>>({})

// 防抖定时器
let debounceTimer: NodeJS.Timeout | null = null

// 监听modelValue变化
watch(() => props.modelValue, (val) => {
  searchValue.value = val
})

// 监听searchValue变化
watch(searchValue, (val) => {
  emit('update:modelValue', val)
  
  if (props.realtime) {
    handleDebouncedSearch()
  }
})

// 过滤后的搜索建议
const filteredSuggestions = computed(() => {
  if (!searchValue.value || searchValue.value.length < props.minChars) {
    return []
  }
  
  const keyword = searchValue.value.toLowerCase()
  return props.suggestions
    .filter(s => s.text.toLowerCase().includes(keyword))
    .slice(0, props.maxSuggestions)
})

// 容器样式类
const containerClass = computed(() => {
  const classes = ['search-box']
  
  if (props.containerClass) {
    classes.push(props.containerClass)
  }
  
  if (props.width) {
    classes.push('search-box--custom-width')
  }
  
  return classes.join(' ')
})

// 加载搜索历史
const loadSearchHistory = () => {
  try {
    const history = localStorage.getItem(props.historyKey)
    if (history) {
      recentSearches.value = JSON.parse(history)
    }
  } catch (error) {
    console.error('Failed to load search history:', error)
  }
}

// 保存搜索历史
const saveSearchHistory = () => {
  try {
    localStorage.setItem(props.historyKey, JSON.stringify(recentSearches.value))
  } catch (error) {
    console.error('Failed to save search history:', error)
  }
}

// 添加到搜索历史
const addToHistory = (keyword: string) => {
  if (!keyword.trim()) return
  
  // 移除重复项
  const index = recentSearches.value.indexOf(keyword)
  if (index > -1) {
    recentSearches.value.splice(index, 1)
  }
  
  // 添加到开头
  recentSearches.value.unshift(keyword)
  
  // 限制历史记录数量
  if (recentSearches.value.length > props.maxHistory) {
    recentSearches.value = recentSearches.value.slice(0, props.maxHistory)
  }
  
  saveSearchHistory()
}

// 从历史记录中移除
const removeFromHistory = (index: number) => {
  recentSearches.value.splice(index, 1)
  saveSearchHistory()
}

// 高亮匹配文本
const highlightMatch = (text: string) => {
  if (!searchValue.value) return text
  
  const keyword = searchValue.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${keyword})`, 'gi')
  return text.replace(regex, '<mark>$1</mark>')
}

// 防抖搜索
const handleDebouncedSearch = () => {
  if (debounceTimer) {
    clearTimeout(debounceTimer)
  }
  
  debounceTimer = setTimeout(() => {
    if (searchValue.value.trim()) {
      emit('search', searchValue.value.trim())
    }
  }, props.debounceDelay)
}

// 事件处理
const handleInput = (value: string) => {
  showSuggestions.value = value.length >= props.minChars
  activeSuggestionIndex.value = -1
}

const handleChange = (value: string) => {
  emit('change', value)
}

const handleClear = () => {
  showSuggestions.value = false
  emit('clear')
}

const handleFocus = (event: FocusEvent) => {
  if (searchValue.value.length >= props.minChars) {
    showSuggestions.value = true
  }
  emit('focus', event)
}

const handleBlur = (event: FocusEvent) => {
  // 延迟隐藏建议，以便点击建议项
  setTimeout(() => {
    showSuggestions.value = false
  }, 200)
  emit('blur', event)
}

const handleEnter = () => {
  if (activeSuggestionIndex.value >= 0 && filteredSuggestions.value.length > 0) {
    handleSuggestionClick(filteredSuggestions.value[activeSuggestionIndex.value])
  } else {
    handleSearch()
  }
}

const handleSearch = () => {
  const keyword = searchValue.value.trim()
  if (keyword) {
    addToHistory(keyword)
    emit('search', keyword)
  }
  showSuggestions.value = false
}

const handleSuggestionClick = (suggestion: SearchSuggestion) => {
  searchValue.value = suggestion.text
  emit('suggestionSelect', suggestion)
  if (suggestion.value !== undefined) {
    emit('search', suggestion.text, suggestion.value)
  } else {
    handleSearch()
  }
  showSuggestions.value = false
}

const handleHistoryClick = (keyword: string) => {
  searchValue.value = keyword
  emit('search', keyword)
  showSuggestions.value = false
}

const handleAdvancedSearch = () => {
  emit('advancedSearch', { ...advancedForm })
  advancedVisible.value = false
}

const handleAdvancedReset = () => {
  Object.keys(advancedForm).forEach(key => {
    advancedForm[key] = ''
  })
}

// 键盘导航
const handleKeyNavigation = (event: KeyboardEvent) => {
  if (!showSuggestions.value || filteredSuggestions.value.length === 0) return
  
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      activeSuggestionIndex.value = Math.min(
        activeSuggestionIndex.value + 1,
        filteredSuggestions.value.length - 1
      )
      break
    case 'ArrowUp':
      event.preventDefault()
      activeSuggestionIndex.value = Math.max(activeSuggestionIndex.value - 1, -1)
      break
    case 'Escape':
      showSuggestions.value = false
      activeSuggestionIndex.value = -1
      break
  }
}

// 生命周期
onMounted(() => {
  loadSearchHistory()
  
  // 初始化高级搜索表单
  props.advancedFields.forEach(field => {
    advancedForm[field.key] = ''
  })
  
  // 添加键盘事件监听
  document.addEventListener('keydown', handleKeyNavigation)
})

onUnmounted(() => {
  if (debounceTimer) {
    clearTimeout(debounceTimer)
  }
  document.removeEventListener('keydown', handleKeyNavigation)
})
</script>

<style scoped lang="scss">
.search-box {
  position: relative;
  
  &.search-box--custom-width {
    width: v-bind(width);
  }
  
  .search-suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 1000;
    max-height: 300px;
    overflow-y: auto;
    background: var(--el-bg-color-overlay);
    border: 1px solid var(--el-border-color);
    border-radius: 6px;
    box-shadow: var(--el-box-shadow-light);
    margin-top: 4px;
    
    .suggestion-item {
      display: flex;
      align-items: center;
      padding: 8px 12px;
      cursor: pointer;
      transition: background-color 0.3s;
      
      &:hover,
      &.suggestion-item--active {
        background-color: var(--el-fill-color-light);
      }
      
      .suggestion-icon {
        margin-right: 8px;
        color: var(--el-text-color-secondary);
      }
      
      .suggestion-content {
        flex: 1;
        min-width: 0;
        
        .suggestion-text {
          font-size: 14px;
          color: var(--el-text-color-primary);
          
          :deep(mark) {
            background-color: var(--el-color-primary-light-8);
            color: var(--el-color-primary);
            padding: 0 2px;
            border-radius: 2px;
          }
        }
        
        .suggestion-description {
          font-size: 12px;
          color: var(--el-text-color-secondary);
          margin-top: 2px;
        }
      }
      
      .suggestion-category {
        font-size: 12px;
        color: var(--el-text-color-secondary);
        margin-left: 8px;
      }
    }
    
    .search-history {
      border-top: 1px solid var(--el-border-color-lighter);
      
      .history-item {
        display: flex;
        align-items: center;
        padding: 6px 12px;
        cursor: pointer;
        transition: background-color 0.3s;
        
        &:hover {
          background-color: var(--el-fill-color-light);
        }
        
        .history-icon {
          margin-right: 8px;
          color: var(--el-text-color-secondary);
          font-size: 12px;
        }
        
        .history-text {
          flex: 1;
          font-size: 13px;
          color: var(--el-text-color-regular);
        }
      }
    }
  }
  
  .advanced-search {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 1000;
    margin-top: 4px;
    
    .advanced-search-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      span {
        font-weight: 600;
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .search-box {
    .search-suggestions {
      position: fixed;
      left: 16px;
      right: 16px;
      top: auto;
    }
    
    .advanced-search {
      position: fixed;
      left: 16px;
      right: 16px;
      top: auto;
    }
  }
}
</style>