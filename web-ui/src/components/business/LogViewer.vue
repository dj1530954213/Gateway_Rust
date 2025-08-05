<template>
  <div class="log-viewer" :class="containerClass">
    <!-- 日志查看器头部 -->
    <div class="log-header">
      <div class="header-left">
        <div class="log-title">
          <el-icon :size="18">
            <Document />
          </el-icon>
          <span>{{ title }}</span>
          <el-tag v-if="showStatus" :type="connectionStatusType" size="small">
            {{ connectionStatusText }}
          </el-tag>
        </div>
      </div>
      
      <div class="header-right">
        <!-- 实时模式切换 -->
        <el-switch
          v-if="supportRealtime"
          v-model="realtimeMode"
          active-text="实时"
          size="small"
          @change="handleRealtimeToggle"
        />
        
        <!-- 刷新按钮 -->
        <el-button
          type="text"
          size="small"
          :icon="Refresh"
          :loading="loading"
          @click="handleRefresh"
        />
        
        <!-- 清空日志 -->
        <el-button
          type="text"
          size="small"
          :icon="Delete"
          @click="handleClear"
        />
        
        <!-- 更多操作 -->
        <el-dropdown trigger="click" @command="handleAction">
          <el-button type="text" size="small" :icon="MoreFilled" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="export" :icon="Download">
                导出日志
              </el-dropdown-item>
              <el-dropdown-item command="settings" :icon="Setting">
                查看器设置
              </el-dropdown-item>
              <el-dropdown-item command="stats" :icon="DataAnalysis">
                日志统计
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
    
    <!-- 过滤器栏 -->
    <div class="log-filters">
      <!-- 日志级别过滤 -->
      <div class="filter-group">
        <label>级别:</label>
        <el-checkbox-group v-model="selectedLevels" size="small" @change="handleLevelFilter">
          <el-checkbox
            v-for="level in logLevels"
            :key="level.value"
            :label="level.value"
            :style="{ color: level.color }"
          >
            {{ level.label }}
          </el-checkbox>
        </el-checkbox-group>
      </div>
      
      <!-- 时间范围 -->
      <div class="filter-group">
        <label>时间:</label>
        <TimeRangePicker
          v-model="timeRange"
          size="small"
          :show-templates="false"
          :show-history="false"
          style="width: 280px"
          @change="handleTimeRangeChange"
        />
      </div>
      
      <!-- 搜索框 -->
      <div class="filter-group">
        <SearchBox
          v-model="searchKeyword"
          placeholder="搜索日志内容"
          size="small"
          style="width: 250px"
          @search="handleSearch"
          @clear="handleSearchClear"
        />
      </div>
      
      <!-- 高级过滤 -->
      <el-popover
        v-model:visible="advancedFilterVisible"
        width="300"
        trigger="click"
        placement="bottom-end"
      >
        <template #reference>
          <el-button type="text" size="small" :icon="Filter">
            高级过滤
          </el-button>
        </template>
        
        <div class="advanced-filters">
          <el-form size="small" label-width="80px">
            <el-form-item label="来源模块">
              <el-select v-model="sourceFilter" clearable placeholder="选择模块">
                <el-option
                  v-for="source in logSources"
                  :key="source"
                  :label="source"
                  :value="source"
                />
              </el-select>
            </el-form-item>
            
            <el-form-item label="线程ID">
              <el-input v-model="threadFilter" placeholder="输入线程ID" />
            </el-form-item>
            
            <el-form-item label="正则表达式">
              <el-input v-model="regexFilter" placeholder="输入正则表达式" />
            </el-form-item>
            
            <el-form-item>
              <el-button type="primary" size="small" @click="applyAdvancedFilter">
                应用筛选
              </el-button>
              <el-button size="small" @click="clearAdvancedFilter">
                清空
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </el-popover>
    </div>
    
    <!-- 日志统计 -->
    <div v-if="showStats" class="log-stats">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-statistic title="总计" :value="totalLogs" />
        </el-col>
        <el-col :span="6">
          <el-statistic title="错误" :value="errorCount" value-style="color: var(--el-color-danger)" />
        </el-col>
        <el-col :span="6">
          <el-statistic title="警告" :value="warningCount" value-style="color: var(--el-color-warning)" />
        </el-col>
        <el-col :span="6">
          <el-statistic title="信息" :value="infoCount" value-style="color: var(--el-color-info)" />
        </el-col>
      </el-row>
    </div>
    
    <!-- 日志内容区域 -->
    <div class="log-content" ref="logContentRef">
      <!-- 加载状态 -->
      <LoadingCard
        v-if="loading && logs.length === 0"
        status="loading"
        loading-text="正在加载日志..."
        :min-height="logHeight"
      />
      
      <!-- 空状态 -->
      <div v-else-if="filteredLogs.length === 0" class="log-empty">
        <el-empty
          :description="logs.length === 0 ? '暂无日志数据' : '没有匹配的日志'"
          :image-size="100"
        />
      </div>
      
      <!-- 日志列表 -->
      <div v-else class="log-list" :style="{ height: logHeight }">
        <el-scrollbar ref="scrollbarRef" :height="logHeight">
          <div
            v-for="(log, index) in paginatedLogs"
            :key="log.id"
            class="log-item"
            :class="getLogItemClass(log)"
            @click="handleLogClick(log)"
          >
            <div class="log-timestamp">
              {{ formatTimestamp(log.timestamp) }}
            </div>
            
            <div class="log-level">
              <el-tag
                :type="getLevelTagType(log.level)"
                size="small"
                effect="dark"
              >
                {{ log.level.toUpperCase() }}
              </el-tag>
            </div>
            
            <div class="log-source">
              {{ log.source || 'Unknown' }}
            </div>
            
            <div class="log-message" v-html="highlightKeyword(log.message)"></div>
            
            <div v-if="log.thread" class="log-thread">
              [{{ log.thread }}]
            </div>
            
            <div class="log-actions">
              <el-button
                type="text"
                size="small"
                :icon="View"
                @click.stop="handleViewDetail(log)"
              />
            </div>
          </div>
          
          <!-- 加载更多指示器 -->
          <div v-if="hasMoreLogs && !realtimeMode" class="load-more">
            <el-button
              type="text"
              size="small"
              :loading="loadingMore"
              @click="handleLoadMore"
            >
              加载更多日志
            </el-button>
          </div>
        </el-scrollbar>
      </div>
    </div>
    
    <!-- 日志控制栏 -->
    <div class="log-controls">
      <div class="controls-left">
        <span class="log-count">
          显示 {{ filteredLogs.length }} / {{ totalLogs }} 条日志
        </span>
        
        <el-checkbox
          v-model="autoScroll"
          size="small"
          @change="handleAutoScrollToggle"
        >
          自动滚动
        </el-checkbox>
        
        <el-checkbox
          v-model="showTimestamp"
          size="small"
          @change="handleTimestampToggle"
        >
          显示时间戳
        </el-checkbox>
      </div>
      
      <div class="controls-right">
        <el-pagination
          v-if="!realtimeMode && enablePagination"
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="filteredLogs.length"
          :page-sizes="[50, 100, 200, 500]"
          layout="sizes, prev, pager, next"
          size="small"
          @size-change="handlePageSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </div>
    
    <!-- 日志详情对话框 -->
    <el-dialog
      v-model="detailDialogVisible"
      title="日志详情"
      width="700px"
      :before-close="handleDetailDialogClose"
    >
      <div v-if="selectedLog" class="log-detail">
        <el-descriptions border column="2">
          <el-descriptions-item label="时间戳" span="2">
            {{ formatDetailTimestamp(selectedLog.timestamp) }}
          </el-descriptions-item>
          <el-descriptions-item label="级别">
            <StatusTag :status="selectedLog.level" />
          </el-descriptions-item>
          <el-descriptions-item label="来源">
            {{ selectedLog.source || 'Unknown' }}
          </el-descriptions-item>
          <el-descriptions-item v-if="selectedLog.thread" label="线程">
            {{ selectedLog.thread }}
          </el-descriptions-item>
          <el-descriptions-item v-if="selectedLog.category" label="分类">
            {{ selectedLog.category }}
          </el-descriptions-item>
          <el-descriptions-item label="消息" span="2">
            <pre class="log-message-detail">{{ selectedLog.message }}</pre>
          </el-descriptions-item>
          <el-descriptions-item v-if="selectedLog.stackTrace" label="堆栈跟踪" span="2">
            <pre class="log-stacktrace">{{ selectedLog.stackTrace }}</pre>
          </el-descriptions-item>
        </el-descriptions>
      </div>
      
      <template #footer>
        <el-button @click="detailDialogVisible = false">关闭</el-button>
        <el-button type="primary" @click="handleCopyLog">
          复制日志
        </el-button>
      </template>
    </el-dialog>
    
    <!-- 设置对话框 -->
    <el-dialog
      v-model="settingsDialogVisible"
      title="查看器设置"
      width="500px"
      :before-close="handleSettingsDialogClose"
    >
      <BaseForm
        v-model="viewerSettings"
        :fields="settingsFields"
        label-width="120px"
      />
      
      <template #footer>
        <el-button @click="settingsDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSaveSettings">
          保存设置
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, watch, nextTick, onMounted, onUnmounted } from 'vue'
import {
  Document,
  Refresh,
  Delete,
  MoreFilled,
  Download,
  Setting,
  DataAnalysis,
  Filter,
  View
} from '@element-plus/icons-vue'
import { LoadingCard, SearchBox, BaseForm, StatusTag } from '../base'
import { TimeRangePicker } from './'
import { ElMessage } from 'element-plus'

export type LogLevel = 'debug' | 'info' | 'warn' | 'error'

export interface LogEntry {
  id: string
  timestamp: Date
  level: LogLevel
  source?: string
  thread?: string
  category?: string
  message: string
  stackTrace?: string
  data?: any
}

interface Props {
  title?: string
  logs?: LogEntry[]
  
  // 功能控制
  supportRealtime?: boolean
  enablePagination?: boolean
  showStats?: boolean
  showStatus?: boolean
  
  // 外观配置
  height?: string
  maxLogs?: number
  
  // 自定义样式
  customClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: '日志查看器',
  logs: () => [],
  supportRealtime: true,
  enablePagination: true,
  showStats: true,
  showStatus: true,
  height: '500px',
  maxLogs: 10000,
})

interface Emits {
  refresh: []
  clear: []
  export: [logs: LogEntry[]]
  'realtime-toggle': [enabled: boolean]
}

const emit = defineEmits<Emits>()

// 状态
const loading = ref(false)
const loadingMore = ref(false)
const realtimeMode = ref(false)
const autoScroll = ref(true)
const showTimestamp = ref(true)
const currentPage = ref(1)
const pageSize = ref(100)

// 过滤器状态
const selectedLevels = ref<LogLevel[]>(['debug', 'info', 'warn', 'error'])
const timeRange = ref({
  start: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
  end: new Date().toISOString()
})
const searchKeyword = ref('')
const advancedFilterVisible = ref(false)
const sourceFilter = ref('')
const threadFilter = ref('')
const regexFilter = ref('')

// 对话框状态
const detailDialogVisible = ref(false)
const settingsDialogVisible = ref(false)
const selectedLog = ref<LogEntry | null>(null)

// 内部日志数据
const logs = ref<LogEntry[]>([])

// 查看器设置
const viewerSettings = reactive({
  fontSize: 14,
  lineHeight: 1.4,
  theme: 'light',
  maxDisplayLogs: 1000,
  autoRefreshInterval: 5000
})

// 引用
const logContentRef = ref()
const scrollbarRef = ref()

// 实时更新定时器
let realtimeTimer: NodeJS.Timeout | null = null

// 日志级别配置
const logLevels = [
  { value: 'debug', label: 'DEBUG', color: '#909399' },
  { value: 'info', label: 'INFO', color: '#409eff' },
  { value: 'warn', label: 'WARN', color: '#e6a23c' },
  { value: 'error', label: 'ERROR', color: '#f56c6c' }
]

// 日志来源列表
const logSources = computed(() => {
  const sources = new Set<string>()
  logs.value.forEach(log => {
    if (log.source) sources.add(log.source)
  })
  return Array.from(sources).sort()
})

// 设置表单字段
const settingsFields = [
  {
    key: 'fontSize',
    label: '字体大小',
    type: 'number',
    min: 10,
    max: 20,
    unit: 'px'
  },
  {
    key: 'lineHeight',
    label: '行高',
    type: 'number',
    min: 1,
    max: 2,
    step: 0.1
  },
  {
    key: 'theme',
    label: '主题',
    type: 'select',
    options: [
      { label: '浅色', value: 'light' },
      { label: '深色', value: 'dark' }
    ]
  },
  {
    key: 'maxDisplayLogs',
    label: '最大显示条数',
    type: 'number',
    min: 100,
    max: 5000
  },
  {
    key: 'autoRefreshInterval',
    label: '自动刷新间隔',
    type: 'number',
    unit: '毫秒',
    min: 1000,
    max: 60000
  }
]

// 计算属性
const containerClass = computed(() => {
  const classes = ['log-viewer']
  
  if (props.customClass) {
    classes.push(props.customClass)
  }
  
  if (viewerSettings.theme === 'dark') {
    classes.push('log-viewer--dark')
  }
  
  return classes.join(' ')
})

const logHeight = computed(() => props.height)

const connectionStatusType = computed(() => {
  return realtimeMode.value ? 'success' : 'info'
})

const connectionStatusText = computed(() => {
  return realtimeMode.value ? '实时连接' : '历史查看'
})

const totalLogs = computed(() => logs.value.length)

const errorCount = computed(() => 
  logs.value.filter(log => log.level === 'error').length
)

const warningCount = computed(() => 
  logs.value.filter(log => log.level === 'warn').length
)

const infoCount = computed(() => 
  logs.value.filter(log => log.level === 'info').length
)

const filteredLogs = computed(() => {
  let filtered = logs.value

  // 级别过滤
  filtered = filtered.filter(log => selectedLevels.value.includes(log.level))

  // 时间范围过滤
  if (timeRange.value.start && timeRange.value.end) {
    const startTime = new Date(timeRange.value.start).getTime()
    const endTime = new Date(timeRange.value.end).getTime()
    filtered = filtered.filter(log => {
      const logTime = log.timestamp.getTime()
      return logTime >= startTime && logTime <= endTime
    })
  }

  // 关键词搜索
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase()
    filtered = filtered.filter(log =>
      log.message.toLowerCase().includes(keyword) ||
      (log.source && log.source.toLowerCase().includes(keyword))
    )
  }

  // 来源过滤
  if (sourceFilter.value) {
    filtered = filtered.filter(log => log.source === sourceFilter.value)
  }

  // 线程过滤
  if (threadFilter.value) {
    filtered = filtered.filter(log => log.thread === threadFilter.value)
  }

  // 正则表达式过滤
  if (regexFilter.value) {
    try {
      const regex = new RegExp(regexFilter.value, 'i')
      filtered = filtered.filter(log => regex.test(log.message))
    } catch (error) {
      // 正则表达式无效时忽略此过滤器
    }
  }

  return filtered.slice(0, viewerSettings.maxDisplayLogs)
})

const paginatedLogs = computed(() => {
  if (!props.enablePagination || realtimeMode.value) {
    return filteredLogs.value
  }
  
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredLogs.value.slice(start, end)
})

const hasMoreLogs = computed(() => {
  return filteredLogs.value.length < totalLogs.value
})

// 方法
const formatTimestamp = (timestamp: Date) => {
  if (!showTimestamp.value) return ''
  return timestamp.toLocaleString('zh-CN', {
    hour12: false,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3
  })
}

const formatDetailTimestamp = (timestamp: Date) => {
  return timestamp.toLocaleString('zh-CN', {
    hour12: false,
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3
  })
}

const getLevelTagType = (level: LogLevel) => {
  const typeMap = {
    debug: '',
    info: 'info',
    warn: 'warning',
    error: 'danger'
  }
  return typeMap[level] || ''
}

const getLogItemClass = (log: LogEntry) => {
  return `log-item--${log.level}`
}

const highlightKeyword = (text: string) => {
  if (!searchKeyword.value) return text
  
  const keyword = searchKeyword.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${keyword})`, 'gi')
  return text.replace(regex, '<mark>$1</mark>')
}

// 事件处理
const handleRealtimeToggle = (enabled: boolean) => {
  realtimeMode.value = enabled
  
  if (enabled) {
    startRealtimeUpdate()
  } else {
    stopRealtimeUpdate()
  }
  
  emit('realtime-toggle', enabled)
}

const startRealtimeUpdate = () => {
  if (realtimeTimer) {
    clearInterval(realtimeTimer)
  }
  
  realtimeTimer = setInterval(() => {
    // 从WebSocket或API拉取新日志
    fetchLatestLogs()
    if (autoScroll.value) {
      scrollToBottom()
    }
  }, viewerSettings.autoRefreshInterval)
}

const stopRealtimeUpdate = () => {
  if (realtimeTimer) {
    clearInterval(realtimeTimer)
    realtimeTimer = null
  }
}

/**
 * 从API获取最新日志
 */
const fetchLatestLogs = async () => {
  try {
    // 获取最新的日志条目
    const response = await fetch('/api/v1/logs/latest', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json'
      }
    })
    
    if (response.ok) {
      const newLogs = await response.json()
      if (newLogs.length > 0) {
        logs.value.push(...newLogs)
        
        // 限制日志数量
        if (logs.value.length > props.maxLogs) {
          logs.value = logs.value.slice(-props.maxLogs)
        }
      }
    }
  } catch (error) {
    console.error('获取最新日志失败:', error)
  }
}

const scrollToBottom = () => {
  nextTick(() => {
    if (scrollbarRef.value) {
      scrollbarRef.value.setScrollTop(scrollbarRef.value.wrapRef.scrollHeight)
    }
  })
}

const handleRefresh = () => {
  loading.value = true
  emit('refresh')
  
  setTimeout(() => {
    loading.value = false
  }, 1000)
}

const handleClear = () => {
  logs.value = []
  emit('clear')
  ElMessage.success('日志已清空')
}

const handleAction = (command: string) => {
  switch (command) {
    case 'export':
      handleExport()
      break
    case 'settings':
      settingsDialogVisible.value = true
      break
    case 'stats':
      // 显示统计信息
      ElMessage.info('统计功能开发中')
      break
  }
}

const handleExport = () => {
  emit('export', filteredLogs.value)
  ElMessage.success('日志导出成功')
}

const handleLevelFilter = () => {
  currentPage.value = 1
}

const handleTimeRangeChange = () => {
  currentPage.value = 1
}

const handleSearch = () => {
  currentPage.value = 1
}

const handleSearchClear = () => {
  searchKeyword.value = ''
  currentPage.value = 1
}

const applyAdvancedFilter = () => {
  advancedFilterVisible.value = false
  currentPage.value = 1
}

const clearAdvancedFilter = () => {
  sourceFilter.value = ''
  threadFilter.value = ''
  regexFilter.value = ''
  currentPage.value = 1
}

const handleLogClick = (log: LogEntry) => {
  // 可以添加日志点击处理逻辑
}

const handleViewDetail = (log: LogEntry) => {
  selectedLog.value = log
  detailDialogVisible.value = true
}

const handleDetailDialogClose = () => {
  detailDialogVisible.value = false
  selectedLog.value = null
}

const handleCopyLog = () => {
  if (selectedLog.value) {
    const logText = `[${formatDetailTimestamp(selectedLog.value.timestamp)}] [${selectedLog.value.level.toUpperCase()}] [${selectedLog.value.source}] ${selectedLog.value.message}`
    navigator.clipboard.writeText(logText)
    ElMessage.success('日志已复制到剪贴板')
  }
}

const handleAutoScrollToggle = () => {
  if (autoScroll.value && realtimeMode.value) {
    scrollToBottom()
  }
}

const handleTimestampToggle = () => {
  // 时间戳显示切换处理
}

const handleLoadMore = async () => {
  loadingMore.value = true
  
  try {
    // 获取更多历史日志
    const oldestLog = logs.value[0]
    const before = oldestLog ? oldestLog.timestamp : new Date().toISOString()
    
    const response = await fetch(`/api/v1/logs?before=${before}&limit=50`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json'
      }
    })
    
    if (response.ok) {
      const olderLogs = await response.json()
      if (olderLogs.length > 0) {
        logs.value.unshift(...olderLogs)
      } else {
        hasMoreLogs.value = false
      }
    }
  } catch (error) {
    console.error('加载更多日志失败:', error)
  } finally {
    loadingMore.value = false
  }
}

const handlePageSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
}

const handlePageChange = (page: number) => {
  currentPage.value = page
}

const handleSettingsDialogClose = () => {
  settingsDialogVisible.value = false
}

const handleSaveSettings = () => {
  settingsDialogVisible.value = false
  ElMessage.success('设置保存成功')
  
  // 如果实时模式开启，重新启动定时器
  if (realtimeMode.value) {
    stopRealtimeUpdate()
    startRealtimeUpdate()
  }
}

// 监听
watch(() => props.logs, (newLogs) => {
  logs.value = [...newLogs]
}, { immediate: true })

// 生命周期
onMounted(() => {
  // 初始化加载日志
  if (props.logs.length === 0) {
    fetchLatestLogs()
  }
})

onUnmounted(() => {
  stopRealtimeUpdate()
})
</script>

<style scoped lang="scss">
.log-viewer {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  
  &.log-viewer--dark {
    background: #1e1e1e;
    border-color: #333;
    color: #d4d4d4;
    
    .log-item {
      &:hover {
        background: #2d2d2d;
      }
    }
  }
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);
  
  .header-left {
    .log-title {
      display: flex;
      align-items: center;
      gap: 8px;
      font-weight: 600;
      color: var(--el-text-color-primary);
    }
  }
  
  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
}

.log-filters {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);
  flex-wrap: wrap;
  
  .filter-group {
    display: flex;
    align-items: center;
    gap: 8px;
    
    label {
      font-size: 12px;
      color: var(--el-text-color-secondary);
      white-space: nowrap;
    }
  }
}

.advanced-filters {
  .el-form-item {
    margin-bottom: 12px;
  }
}

.log-stats {
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.log-content {
  flex: 1;
  min-height: 0;
  
  .log-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 300px;
  }
  
  .log-list {
    .log-item {
      display: grid;
      grid-template-columns: 180px 80px 120px 1fr 100px 60px;
      gap: 8px;
      align-items: center;
      padding: 4px 8px;
      border-bottom: 1px solid var(--el-border-color-lighter);
      transition: background-color 0.3s;
      font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
      font-size: v-bind('viewerSettings.fontSize + "px"');
      line-height: v-bind('viewerSettings.lineHeight');
      
      &:hover {
        background-color: var(--el-fill-color-light);
        cursor: pointer;
      }
      
      &.log-item--debug {
        color: #909399;
      }
      
      &.log-item--info {
        color: #409eff;
      }
      
      &.log-item--warn {
        color: #e6a23c;
      }
      
      &.log-item--error {
        color: #f56c6c;
      }
      
      .log-timestamp {
        font-size: 11px;
        color: var(--el-text-color-secondary);
        white-space: nowrap;
      }
      
      .log-level {
        text-align: center;
      }
      
      .log-source {
        font-size: 12px;
        color: var(--el-text-color-secondary);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }
      
      .log-message {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        
        :deep(mark) {
          background-color: var(--el-color-warning-light-8);
          color: var(--el-color-warning-dark-2);
          padding: 0 2px;
          border-radius: 2px;
        }
      }
      
      .log-thread {
        font-size: 11px;
        color: var(--el-text-color-placeholder);
        text-align: center;
      }
      
      .log-actions {
        text-align: center;
        opacity: 0;
        transition: opacity 0.3s;
      }
      
      &:hover .log-actions {
        opacity: 1;
      }
    }
    
    .load-more {
      text-align: center;
      padding: 16px;
    }
  }
}

.log-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);
  
  .controls-left {
    display: flex;
    align-items: center;
    gap: 16px;
    
    .log-count {
      font-size: 12px;
      color: var(--el-text-color-secondary);
    }
  }
}

.log-detail {
  .log-message-detail,
  .log-stacktrace {
    background: var(--el-fill-color-light);
    padding: 8px;
    border-radius: 4px;
    white-space: pre-wrap;
    word-break: break-all;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.4;
    max-height: 200px;
    overflow-y: auto;
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .log-list .log-item {
    grid-template-columns: 150px 70px 100px 1fr 80px 50px;
    gap: 6px;
  }
}

@media (max-width: 768px) {
  .log-filters {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  
  .log-controls {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  
  .log-list .log-item {
    grid-template-columns: 1fr;
    gap: 4px;
    
    .log-timestamp,
    .log-level,
    .log-source,
    .log-thread {
      display: inline-block;
      margin-right: 8px;
    }
    
    .log-message {
      grid-column: 1;
      margin-top: 4px;
    }
    
    .log-actions {
      grid-column: 1;
      text-align: left;
      opacity: 1;
    }
  }
}
</style>