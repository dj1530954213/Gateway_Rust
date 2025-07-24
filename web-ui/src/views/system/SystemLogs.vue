<template>
  <div class="system-logs">
    <!-- 页面标题和工具栏 -->
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">
          <el-icon><Document /></el-icon>
          系统日志查看器
        </h1>
        <span class="page-description">实时查看和分析系统日志信息</span>
      </div>
      <div class="header-actions">
        <el-button type="primary" @click="refreshLogs">
          <el-icon><Refresh /></el-icon>
          刷新日志
        </el-button>
        <el-button @click="clearAllLogs">
          <el-icon><Delete /></el-icon>
          清空日志
        </el-button>
        <el-button @click="downloadLogs">
          <el-icon><Download /></el-icon>
          下载日志
        </el-button>
        <el-button @click="showSettings = true">
          <el-icon><Setting /></el-icon>
          日志设置
        </el-button>
      </div>
    </div>

    <!-- 日志统计概览 -->
    <div class="logs-overview">
      <el-row :gutter="20">
        <el-col :span="6">
          <StatCard
            title="今日日志"
            :value="logStats.todayCount"
            :trend="logStats.todayTrend"
            trend-label="较昨日"
            color="#409EFF"
            icon="Document"
          />
        </el-col>
        <el-col :span="6">
          <StatCard
            title="错误日志"
            :value="logStats.errorCount"
            :trend="logStats.errorTrend"
            trend-label="较昨日"
            color="#F56C6C"
            icon="Warning"
          />
        </el-col>
        <el-col :span="6">
          <StatCard
            title="警告日志"
            :value="logStats.warningCount"
            :trend="logStats.warningTrend"
            trend-label="较昨日"
            color="#E6A23C"
            icon="Warning"
          />
        </el-col>
        <el-col :span="6">
          <StatCard
            title="存储占用"
            :value="logStats.storageSize"
            suffix="MB"
            :trend="logStats.storageTrend"
            trend-label="较上周"
            color="#67C23A"
            icon="Folder"
          />
        </el-col>
      </el-row>
    </div>

    <!-- 日志筛选器 -->
    <div class="logs-filters">
      <el-card shadow="never">
        <div class="filter-row">
          <el-row :gutter="20">
            <el-col :span="6">
              <el-input
                v-model="filters.search"
                placeholder="搜索日志内容..."
                clearable
                @input="handleSearch"
              >
                <template #prefix>
                  <el-icon><Search /></el-icon>
                </template>
              </el-input>
            </el-col>
            <el-col :span="3">
              <el-select
                v-model="filters.level"
                placeholder="日志级别"
                clearable
                @change="handleFilter"
              >
                <el-option label="全部" value="" />
                <el-option label="错误" value="error">
                  <span class="level-option error">ERROR</span>
                </el-option>
                <el-option label="警告" value="warn">
                  <span class="level-option warn">WARN</span>
                </el-option>
                <el-option label="信息" value="info">
                  <span class="level-option info">INFO</span>
                </el-option>
                <el-option label="调试" value="debug">
                  <span class="level-option debug">DEBUG</span>
                </el-option>
                <el-option label="跟踪" value="trace">
                  <span class="level-option trace">TRACE</span>
                </el-option>
              </el-select>
            </el-col>
            <el-col :span="4">
              <el-select
                v-model="filters.source"
                placeholder="日志来源"
                clearable
                @change="handleFilter"
              >
                <el-option label="全部来源" value="" />
                <el-option
                  v-for="source in logSources"
                  :key="source.value"
                  :label="source.label"
                  :value="source.value"
                />
              </el-select>
            </el-col>
            <el-col :span="6">
              <el-date-picker
                v-model="filters.timeRange"
                type="datetimerange"
                range-separator="至"
                start-placeholder="开始时间"
                end-placeholder="结束时间"
                format="YYYY-MM-DD HH:mm:ss"
                value-format="YYYY-MM-DD HH:mm:ss"
                @change="handleFilter"
              />
            </el-col>
            <el-col :span="5">
              <el-button-group>
                <el-button
                  :type="realTimeMode ? 'primary' : ''"
                  @click="toggleRealTime"
                >
                  <el-icon><VideoPlay /></el-icon>
                  {{ realTimeMode ? '停止实时' : '实时模式' }}
                </el-button>
                <el-button @click="resetFilters">重置</el-button>
              </el-button-group>
            </el-col>
          </el-row>
        </div>
      </el-card>
    </div>

    <!-- 日志视图切换 -->
    <div class="logs-view-controls">
      <el-card shadow="never">
        <div class="view-controls">
          <div class="view-modes">
            <el-button-group>
              <el-button
                :type="viewMode === 'table' ? 'primary' : ''"
                @click="setViewMode('table')"
              >
                <el-icon><List /></el-icon>
                表格视图
              </el-button>
              <el-button
                :type="viewMode === 'console' ? 'primary' : ''"
                @click="setViewMode('console')"
              >
                <el-icon><Monitor /></el-icon>
                控制台视图
              </el-button>
              <el-button
                :type="viewMode === 'timeline' ? 'primary' : ''"
                @click="setViewMode('timeline')"
              >
                <el-icon><Clock /></el-icon>
                时间线视图
              </el-button>
            </el-button-group>
          </div>
          
          <div class="display-options">
            <el-checkbox v-model="displayOptions.showLineNumbers">显示行号</el-checkbox>
            <el-checkbox v-model="displayOptions.wordWrap">自动换行</el-checkbox>
            <el-checkbox v-model="displayOptions.highlightErrors">高亮错误</el-checkbox>
            <el-select v-model="displayOptions.fontSize" size="small" style="width: 100px;">
              <el-option label="小" value="12px" />
              <el-option label="中" value="14px" />
              <el-option label="大" value="16px" />
            </el-select>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 日志内容显示 -->
    <div class="logs-content">
      <el-card shadow="never" class="logs-container">
        <!-- 表格视图 -->
        <div v-if="viewMode === 'table'" class="table-view">
          <el-table
            :data="filteredLogs"
            style="width: 100%"
            v-loading="loading"
            @sort-change="handleSortChange"
            :row-class-name="getRowClassName"
          >
            <el-table-column
              v-if="displayOptions.showLineNumbers"
              type="index"
              width="80"
              label="行号"
            />
            <el-table-column
              prop="timestamp"
              label="时间"
              width="180"
              sortable="custom"
            >
              <template #default="{ row }">
                <span class="timestamp">{{ formatTimestamp(row.timestamp) }}</span>
              </template>
            </el-table-column>
            <el-table-column
              prop="level"
              label="级别"
              width="80"
              sortable="custom"
            >
              <template #default="{ row }">
                <span :class="getLevelClass(row.level)">
                  {{ row.level.toUpperCase() }}
                </span>
              </template>
            </el-table-column>
            <el-table-column
              prop="source"
              label="来源"
              width="150"
              sortable="custom"
            >
              <template #default="{ row }">
                <el-tag size="small" :type="getSourceTagType(row.source)">
                  {{ row.source }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column
              prop="thread"
              label="线程"
              width="100"
              sortable="custom"
            />
            <el-table-column
              prop="message"
              label="消息内容"
              min-width="400"
              :show-overflow-tooltip="!displayOptions.wordWrap"
            >
              <template #default="{ row }">
                <div
                  :class="{
                    'log-message': true,
                    'word-wrap': displayOptions.wordWrap,
                    'highlight-error': displayOptions.highlightErrors && row.level === 'error'
                  }"
                  :style="{ fontSize: displayOptions.fontSize }"
                >
                  {{ row.message }}
                </div>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="120" fixed="right">
              <template #default="{ row }">
                <el-button size="small" @click="showLogDetail(row)">
                  详情
                </el-button>
                <el-button size="small" @click="copyLog(row)">
                  复制
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <!-- 控制台视图 -->
        <div v-else-if="viewMode === 'console'" class="console-view">
          <div
            ref="consoleContainer"
            class="console-container"
            :style="{ fontSize: displayOptions.fontSize }"
          >
            <div
              v-for="(log, index) in filteredLogs"
              :key="log.id"
              :class="['console-line', getLevelClass(log.level)]"
              @click="showLogDetail(log)"
            >
              <span v-if="displayOptions.showLineNumbers" class="line-number">
                {{ index + 1 }}
              </span>
              <span class="timestamp">{{ formatTimestamp(log.timestamp) }}</span>
              <span class="level">{{ log.level.toUpperCase() }}</span>
              <span class="source">[{{ log.source }}]</span>
              <span class="thread">({{ log.thread }})</span>
              <span
                :class="{
                  'message': true,
                  'word-wrap': displayOptions.wordWrap,
                  'highlight-error': displayOptions.highlightErrors && log.level === 'error'
                }"
              >
                {{ log.message }}
              </span>
            </div>
          </div>
        </div>

        <!-- 时间线视图 -->
        <div v-else-if="viewMode === 'timeline'" class="timeline-view">
          <el-timeline>
            <el-timeline-item
              v-for="log in filteredLogs"
              :key="log.id"
              :timestamp="formatTimestamp(log.timestamp)"
              :type="getTimelineType(log.level)"
              placement="top"
            >
              <el-card class="timeline-log" :class="getLevelClass(log.level)">
                <div class="timeline-header">
                  <div class="log-info">
                    <span :class="getLevelClass(log.level)">
                      {{ log.level.toUpperCase() }}
                    </span>
                    <el-tag size="small" :type="getSourceTagType(log.source)">
                      {{ log.source }}
                    </el-tag>
                    <span class="thread-info">Thread: {{ log.thread }}</span>
                  </div>
                  <div class="log-actions">
                    <el-button size="small" @click="showLogDetail(log)">
                      详情
                    </el-button>
                  </div>
                </div>
                <div
                  :class="{
                    'timeline-message': true,
                    'word-wrap': displayOptions.wordWrap,
                    'highlight-error': displayOptions.highlightErrors && log.level === 'error'
                  }"
                  :style="{ fontSize: displayOptions.fontSize }"
                >
                  {{ log.message }}
                </div>
              </el-card>
            </el-timeline-item>
          </el-timeline>
        </div>

        <!-- 分页 -->
        <div class="pagination-container">
          <el-pagination
            v-model:current-page="pagination.page"
            v-model:page-size="pagination.size"
            :page-sizes="[50, 100, 200, 500]"
            :total="pagination.total"
            layout="total, sizes, prev, pager, next, jumper"
            @size-change="handlePageSizeChange"
            @current-change="handlePageChange"
          />
        </div>
      </el-card>
    </div>

    <!-- 日志详情对话框 -->
    <el-dialog
      v-model="logDetailVisible"
      title="日志详情"
      width="80%"
      :close-on-click-modal="false"
    >
      <LogViewer v-if="selectedLog" :logs="[selectedLog]" />
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="logDetailVisible = false">关闭</el-button>
          <el-button type="primary" @click="copyLog(selectedLog)">
            复制日志
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 日志设置对话框 -->
    <el-dialog
      v-model="showSettings"
      title="日志设置"
      width="600px"
    >
      <div class="settings-form">
        <el-form :model="logSettings" label-width="140px">
          <el-form-item label="日志级别">
            <el-checkbox-group v-model="logSettings.enabledLevels">
              <el-checkbox label="error">错误</el-checkbox>
              <el-checkbox label="warn">警告</el-checkbox>
              <el-checkbox label="info">信息</el-checkbox>
              <el-checkbox label="debug">调试</el-checkbox>
              <el-checkbox label="trace">跟踪</el-checkbox>
            </el-checkbox-group>
          </el-form-item>
          <el-form-item label="自动刷新间隔">
            <el-select v-model="logSettings.refreshInterval">
              <el-option label="不自动刷新" :value="0" />
              <el-option label="1秒" :value="1000" />
              <el-option label="5秒" :value="5000" />
              <el-option label="10秒" :value="10000" />
              <el-option label="30秒" :value="30000" />
            </el-select>
          </el-form-item>
          <el-form-item label="最大日志行数">
            <el-input-number
              v-model="logSettings.maxLogLines"
              :min="100"
              :max="10000"
              :step="100"
            />
          </el-form-item>
          <el-form-item label="日志保留时间">
            <el-select v-model="logSettings.retentionPeriod">
              <el-option label="1天" :value="86400000" />
              <el-option label="7天" :value="604800000" />
              <el-option label="30天" :value="2592000000" />
              <el-option label="90天" :value="7776000000" />
            </el-select>
          </el-form-item>
          <el-form-item label="日志文件分割">
            <el-select v-model="logSettings.rotationSize">
              <el-option label="10MB" value="10MB" />
              <el-option label="50MB" value="50MB" />
              <el-option label="100MB" value="100MB" />
              <el-option label="500MB" value="500MB" />
            </el-select>
          </el-form-item>
          <el-form-item label="启用日志归档">
            <el-switch v-model="logSettings.enableArchiving" />
          </el-form-item>
        </el-form>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showSettings = false">取消</el-button>
          <el-button type="primary" @click="saveSettings">保存设置</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 快速操作面板 -->
    <div class="quick-actions" v-if="realTimeMode">
      <el-card shadow="never">
        <div class="actions-panel">
          <span class="status-indicator">
            <el-icon class="pulse"><VideoPlay /></el-icon>
            实时模式已启用
          </span>
          <div class="actions">
            <el-button size="small" @click="pauseRealTime">
              暂停
            </el-button>
            <el-button size="small" @click="jumpToBottom">
              跳到底部
            </el-button>
            <el-button size="small" @click="scrollToTop">
              回到顶部
            </el-button>
          </div>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Document, Refresh, Delete, Download, Setting, Search, Warning,
  Folder, VideoPlay, List, Monitor, Clock
} from '@element-plus/icons-vue'
// import StatCard from '../../components/common/StatCard.vue' // 暂时注释掉不存在的组件
import { LogViewer } from '../../components/business'

// 响应式数据
const loading = ref(false)
const realTimeMode = ref(false)
const viewMode = ref('table')
const showSettings = ref(false)
const logDetailVisible = ref(false)
const selectedLog = ref<any>(null)
const refreshTimer = ref<NodeJS.Timeout | null>(null)

// DOM 引用
const consoleContainer = ref<HTMLElement>()

// 日志统计
const logStats = ref({
  todayCount: 25847,
  todayTrend: 8.5,
  errorCount: 156,
  errorTrend: -12.3,
  warningCount: 892,
  warningTrend: 5.7,
  storageSize: 245.6,
  storageTrend: 15.8
})

// 筛选条件
const filters = ref({
  search: '',
  level: '',
  source: '',
  timeRange: null as any
})

// 显示选项
const displayOptions = ref({
  showLineNumbers: true,
  wordWrap: false,
  highlightErrors: true,
  fontSize: '14px'
})

// 分页信息
const pagination = ref({
  page: 1,
  size: 100,
  total: 0
})

// 日志来源
const logSources = ref([
  { label: '网关核心', value: 'gateway-core' },
  { label: 'Modbus 驱动', value: 'modbus-driver' },
  { label: 'OPC UA 驱动', value: 'opcua-driver' },
  { label: 'MQTT 代理', value: 'mqtt-broker' },
  { label: '数据处理器', value: 'data-processor' },
  { label: 'Web API', value: 'web-api' },
  { label: '告警引擎', value: 'alert-engine' },
  { label: '系统监控', value: 'system-monitor' }
])

// 日志数据
const logs = ref([
  {
    id: 1,
    timestamp: '2024-01-20 15:30:25.123',
    level: 'info',
    source: 'gateway-core',
    thread: 'main',
    message: '系统启动完成，所有服务正常运行',
    details: {
      file: 'main.rs',
      line: 145,
      function: 'startup_complete',
      context: {
        version: '2.1.0',
        build: 'release',
        uptime: '00:00:05'
      }
    }
  },
  {
    id: 2,
    timestamp: '2024-01-20 15:30:24.856',
    level: 'warn',
    source: 'opcua-driver',
    thread: 'driver-pool-2',
    message: '连接到 OPC UA 服务器响应超时，正在重试... (尝试次数: 3/5)',
    details: {
      file: 'opcua_client.rs',
      line: 278,
      function: 'connect_with_retry',
      context: {
        endpoint: 'opc.tcp://192.168.1.100:4840',
        timeout: 5000,
        retryAttempt: 3,
        maxRetries: 5
      }
    }
  },
  {
    id: 3,
    timestamp: '2024-01-20 15:30:23.445',
    level: 'error',
    source: 'alert-engine',
    thread: 'alert-worker-1',
    message: '告警引擎服务连接失败: Connection refused (os error 111)',
    details: {
      file: 'alert_engine.rs',
      line: 89,
      function: 'start_engine',
      context: {
        error: 'Connection refused (os error 111)',
        service: 'alert-engine',
        port: 8081,
        lastAttempt: '2024-01-20 15:30:23'
      }
    }
  },
  {
    id: 4,
    timestamp: '2024-01-20 15:30:22.789',
    level: 'info',
    source: 'data-processor',
    thread: 'processor-1',
    message: '已处理 1000 条数据记录，当前队列长度: 25',
    details: {
      file: 'data_processor.rs',
      line: 156,
      function: 'process_batch',
      context: {
        processedCount: 1000,
        queueLength: 25,
        processingTime: '245ms',
        throughput: '4081 records/sec'
      }
    }
  },
  {
    id: 5,
    timestamp: '2024-01-20 15:30:21.234',
    level: 'debug',
    source: 'mqtt-broker',
    thread: 'mqtt-handler-3',
    message: '新客户端连接建立: client_id=sensor_001, ip=192.168.1.50',
    details: {
      file: 'mqtt_handler.rs',
      line: 67,
      function: 'handle_connect',
      context: {
        clientId: 'sensor_001',
        clientIp: '192.168.1.50',
        keepAlive: 60,
        cleanSession: true
      }
    }
  },
  {
    id: 6,
    timestamp: '2024-01-20 15:30:20.678',
    level: 'info',
    source: 'modbus-driver',
    thread: 'modbus-scanner',
    message: '设备扫描完成，发现 5 个活跃设备',
    details: {
      file: 'modbus_scanner.rs',
      line: 123,
      function: 'scan_devices',
      context: {
        activeDevices: 5,
        totalDevices: 8,
        scanDuration: '1.5s',
        lastScan: '2024-01-20 15:30:20'
      }
    }
  }
])

// 日志设置
const logSettings = ref({
  enabledLevels: ['error', 'warn', 'info', 'debug'],
  refreshInterval: 5000,
  maxLogLines: 1000,
  retentionPeriod: 604800000, // 7天
  rotationSize: '100MB',
  enableArchiving: true
})

// 计算属性
const filteredLogs = computed(() => {
  let result = [...logs.value]

  // 应用筛选条件
  if (filters.value.search) {
    const search = filters.value.search.toLowerCase()
    result = result.filter(log =>
      log.message.toLowerCase().includes(search) ||
      log.source.toLowerCase().includes(search) ||
      log.thread.toLowerCase().includes(search)
    )
  }

  if (filters.value.level) {
    result = result.filter(log => log.level === filters.value.level)
  }

  if (filters.value.source) {
    result = result.filter(log => log.source === filters.value.source)
  }

  if (filters.value.timeRange) {
    const [startTime, endTime] = filters.value.timeRange
    result = result.filter(log => {
      const logTime = new Date(log.timestamp).getTime()
      return logTime >= new Date(startTime).getTime() && logTime <= new Date(endTime).getTime()
    })
  }

  // 应用日志级别过滤
  result = result.filter(log => logSettings.value.enabledLevels.includes(log.level))

  // 限制日志行数
  if (result.length > logSettings.value.maxLogLines) {
    result = result.slice(0, logSettings.value.maxLogLines)
  }

  // 更新分页总数
  pagination.value.total = result.length

  // 应用分页
  const start = (pagination.value.page - 1) * pagination.value.size
  const end = start + pagination.value.size
  return result.slice(start, end)
})

// 方法
const refreshLogs = async () => {
  loading.value = true
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // 模拟新增日志
    const newLogs = Array.from({ length: 10 }, (_, index) => ({
      id: logs.value.length + index + 1,
      timestamp: new Date().toISOString().slice(0, 23).replace('T', ' '),
      level: ['info', 'warn', 'error', 'debug'][Math.floor(Math.random() * 4)],
      source: logSources.value[Math.floor(Math.random() * logSources.value.length)].value,
      thread: `thread-${Math.floor(Math.random() * 10)}`,
      message: `新生成的日志消息 ${Date.now()}`,
      details: {
        file: 'system.rs',
        line: Math.floor(Math.random() * 1000),
        function: 'log_entry',
        context: {}
      }
    }))
    
    // 添加到日志列表开头
    logs.value = [...newLogs, ...logs.value]
    
    // 更新统计信息
    logStats.value.todayCount += newLogs.length
    
    ElMessage.success('日志刷新成功')
  } catch (error) {
    ElMessage.error('日志刷新失败')
  } finally {
    loading.value = false
  }
}

const clearAllLogs = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要清空所有日志吗？此操作不可撤销。',
      '清空日志',
      {
        confirmButtonText: '确定清空',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    logs.value = []
    logStats.value.todayCount = 0
    logStats.value.errorCount = 0
    logStats.value.warningCount = 0
    
    ElMessage.success('日志已清空')
  } catch (error) {
    // 用户取消操作
  }
}

const downloadLogs = async () => {
  try {
    ElMessage.info('正在准备日志下载...')
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // 模拟下载
    const blob = new Blob([JSON.stringify(logs.value, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `system-logs-${new Date().toISOString().slice(0, 10)}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    
    ElMessage.success('日志下载完成')
  } catch (error) {
    ElMessage.error('日志下载失败')
  }
}

const toggleRealTime = () => {
  realTimeMode.value = !realTimeMode.value
  if (realTimeMode.value) {
    startRealTimeMode()
    ElMessage.success('实时模式已启用')
  } else {
    stopRealTimeMode()
    ElMessage.info('实时模式已停用')
  }
}

const startRealTimeMode = () => {
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
  }
  
  refreshTimer.value = setInterval(() => {
    // 模拟实时日志
    const newLog = {
      id: logs.value.length + 1,
      timestamp: new Date().toISOString().slice(0, 23).replace('T', ' '),
      level: ['info', 'warn', 'error', 'debug'][Math.floor(Math.random() * 4)],
      source: logSources.value[Math.floor(Math.random() * logSources.value.length)].value,
      thread: `thread-${Math.floor(Math.random() * 10)}`,
      message: `实时日志消息 ${Date.now()}`,
      details: {
        file: 'realtime.rs',
        line: Math.floor(Math.random() * 1000),
        function: 'realtime_log',
        context: {}
      }
    }
    
    logs.value.unshift(newLog)
    
    // 限制日志数量
    if (logs.value.length > logSettings.value.maxLogLines) {
      logs.value = logs.value.slice(0, logSettings.value.maxLogLines)
    }
    
    // 自动滚动到顶部（最新日志）
    nextTick(() => {
      if (viewMode.value === 'console' && consoleContainer.value) {
        consoleContainer.value.scrollTop = 0
      }
    })
  }, 2000)
}

const stopRealTimeMode = () => {
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
    refreshTimer.value = null
  }
}

const pauseRealTime = () => {
  stopRealTimeMode()
  realTimeMode.value = false
  ElMessage.info('实时模式已暂停')
}

const setViewMode = (mode: string) => {
  viewMode.value = mode
}

const handleSearch = () => {
  pagination.value.page = 1
}

const handleFilter = () => {
  pagination.value.page = 1
}

const resetFilters = () => {
  filters.value = {
    search: '',
    level: '',
    source: '',
    timeRange: null
  }
  pagination.value.page = 1
}

const handleSortChange = ({ prop, order }: any) => {
  // 实现排序逻辑
  if (!prop || !order) return
  
  logs.value.sort((a: any, b: any) => {
    let aVal = a[prop]
    let bVal = b[prop]
    
    if (prop === 'timestamp') {
      aVal = new Date(aVal).getTime()
      bVal = new Date(bVal).getTime()
    }
    
    if (order === 'ascending') {
      return aVal > bVal ? 1 : -1
    } else {
      return aVal < bVal ? 1 : -1
    }
  })
}

const handlePageChange = (page: number) => {
  pagination.value.page = page
}

const handlePageSizeChange = (size: number) => {
  pagination.value.size = size
  pagination.value.page = 1
}

const showLogDetail = (log: any) => {
  selectedLog.value = log
  logDetailVisible.value = true
}

const copyLog = (log: any) => {
  const logText = `[${log.timestamp}] ${log.level.toUpperCase()} [${log.source}] (${log.thread}) ${log.message}`
  
  if (navigator.clipboard) {
    navigator.clipboard.writeText(logText).then(() => {
      ElMessage.success('日志已复制到剪贴板')
    })
  } else {
    // 降级方案
    const textArea = document.createElement('textarea')
    textArea.value = logText
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
    ElMessage.success('日志已复制到剪贴板')
  }
}

const jumpToBottom = () => {
  if (viewMode.value === 'console' && consoleContainer.value) {
    consoleContainer.value.scrollTop = consoleContainer.value.scrollHeight
  }
}

const scrollToTop = () => {
  if (viewMode.value === 'console' && consoleContainer.value) {
    consoleContainer.value.scrollTop = 0
  }
}

const saveSettings = () => {
  // 应用设置
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
  }
  
  if (realTimeMode.value && logSettings.value.refreshInterval > 0) {
    startRealTimeMode()
  }
  
  showSettings.value = false
  ElMessage.success('设置已保存')
}

// 工具函数
const formatTimestamp = (timestamp: string) => {
  return timestamp
}

const getLevelClass = (level: string) => {
  return `level-${level}`
}

const getSourceTagType = (source: string) => {
  const tagTypeMap: { [key: string]: string } = {
    'gateway-core': 'primary',
    'modbus-driver': 'success',
    'opcua-driver': 'success',
    'mqtt-broker': 'warning',
    'data-processor': 'info',
    'web-api': 'primary',
    'alert-engine': 'danger',
    'system-monitor': 'info'
  }
  return tagTypeMap[source] || 'info'
}

const getTimelineType = (level: string) => {
  const typeMap: { [key: string]: string } = {
    error: 'danger',
    warn: 'warning',
    info: 'primary',
    debug: 'info',
    trace: 'info'
  }
  return typeMap[level] || 'info'
}

const getRowClassName = ({ row }: any) => {
  return `row-${row.level}`
}

// 生命周期
onMounted(() => {
  refreshLogs()
})

onUnmounted(() => {
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
  }
})
</script>

<style scoped lang="scss">
.system-logs {
  padding: 24px;
  background: #f5f7fa;
  min-height: 100vh;

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
    background: white;
    padding: 20px 24px;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);

    .header-left {
      .page-title {
        margin: 0;
        font-size: 24px;
        color: #303133;
        display: flex;
        align-items: center;
        gap: 8px;
      }

      .page-description {
        color: #909399;
        margin-left: 32px;
      }
    }

    .header-actions {
      display: flex;
      gap: 12px;
    }
  }

  .logs-overview {
    margin-bottom: 24px;
  }

  .logs-filters {
    margin-bottom: 16px;

    .filter-row {
      .level-option {
        padding: 2px 6px;
        border-radius: 3px;
        font-weight: bold;
        font-size: 12px;

        &.error { background: #fef0f0; color: #f56c6c; }
        &.warn { background: #fdf6ec; color: #e6a23c; }
        &.info { background: #f4f4f5; color: #909399; }
        &.debug { background: #f0f9ff; color: #409eff; }
        &.trace { background: #f5f7fa; color: #909399; }
      }
    }
  }

  .logs-view-controls {
    margin-bottom: 16px;

    .view-controls {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .view-modes {
        display: flex;
        gap: 16px;
      }

      .display-options {
        display: flex;
        gap: 16px;
        align-items: center;
      }
    }
  }

  .logs-content {
    .logs-container {
      min-height: 600px;

      .table-view {
        // 表格行样式
        :deep(.row-error) {
          background-color: #fef0f0 !important;
        }

        :deep(.row-warn) {
          background-color: #fdf6ec !important;
        }

        :deep(.row-debug) {
          background-color: #f4f4f5 !important;
        }

        .timestamp {
          font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
          font-size: 12px;
          color: #909399;
        }

        .log-message {
          &.word-wrap {
            word-wrap: break-word;
            white-space: pre-wrap;
          }

          &.highlight-error {
            background: #fef0f0;
            color: #f56c6c;
            padding: 2px 4px;
            border-radius: 3px;
          }
        }
      }

      .console-view {
        .console-container {
          height: 600px;
          overflow-y: auto;
          background: #1e1e1e;
          border-radius: 4px;
          padding: 16px;
          font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
          line-height: 1.4;

          .console-line {
            display: flex;
            align-items: baseline;
            gap: 8px;
            margin-bottom: 4px;
            cursor: pointer;
            padding: 2px 4px;
            border-radius: 2px;

            &:hover {
              background: rgba(255, 255, 255, 0.1);
            }

            .line-number {
              color: #666;
              min-width: 40px;
              text-align: right;
              font-size: 11px;
            }

            .timestamp {
              color: #888;
              min-width: 140px;
              font-size: 11px;
            }

            .level {
              min-width: 50px;
              font-weight: bold;
              font-size: 11px;
            }

            .source {
              color: #5dade2;
              min-width: 120px;
              font-size: 11px;
            }

            .thread {
              color: #f39c12;
              min-width: 80px;
              font-size: 11px;
            }

            .message {
              flex: 1;
              color: #e8e8e8;

              &.word-wrap {
                word-wrap: break-word;
                white-space: pre-wrap;
              }

              &.highlight-error {
                background: rgba(245, 108, 108, 0.2);
                color: #f56c6c;
                padding: 2px 4px;
                border-radius: 3px;
              }
            }

            &.level-error {
              .level { color: #f56c6c; }
            }

            &.level-warn {
              .level { color: #e6a23c; }
            }

            &.level-info {
              .level { color: #67c23a; }
            }

            &.level-debug {
              .level { color: #909399; }
            }

            &.level-trace {
              .level { color: #606266; }
            }
          }
        }
      }

      .timeline-view {
        .timeline-log {
          margin: 16px 0;

          &.level-error {
            border-left: 4px solid #f56c6c;
          }

          &.level-warn {
            border-left: 4px solid #e6a23c;
          }

          &.level-info {
            border-left: 4px solid #409eff;
          }

          &.level-debug {
            border-left: 4px solid #909399;
          }

          .timeline-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 12px;

            .log-info {
              display: flex;
              align-items: center;
              gap: 12px;

              .thread-info {
                font-size: 12px;
                color: #909399;
              }
            }
          }

          .timeline-message {
            color: #606266;
            line-height: 1.5;

            &.word-wrap {
              word-wrap: break-word;
              white-space: pre-wrap;
            }

            &.highlight-error {
              background: #fef0f0;
              color: #f56c6c;
              padding: 4px 8px;
              border-radius: 4px;
            }
          }
        }
      }

      .pagination-container {
        display: flex;
        justify-content: center;
        margin-top: 24px;
        padding-top: 20px;
        border-top: 1px solid #ebeef5;
      }
    }
  }

  .quick-actions {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 1000;

    .actions-panel {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 20px;
      background: #409eff;
      color: white;
      border-radius: 8px;
      min-width: 300px;

      .status-indicator {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 14px;

        .pulse {
          animation: pulse 1.5s ease-in-out infinite alternate;
        }

        @keyframes pulse {
          from { opacity: 1; }
          to { opacity: 0.5; }
        }
      }

      .actions {
        display: flex;
        gap: 8px;

        .el-button {
          border-color: rgba(255, 255, 255, 0.3);
          color: white;

          &:hover {
            background: rgba(255, 255, 255, 0.1);
          }
        }
      }
    }
  }

  .settings-form {
    .el-form-item {
      margin-bottom: 24px;
    }

    .el-checkbox-group {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
    }
  }

  // 日志级别颜色
  .level-error {
    color: #f56c6c !important;
  }

  .level-warn {
    color: #e6a23c !important;
  }

  .level-info {
    color: #909399 !important;
  }

  .level-debug {
    color: #409eff !important;
  }

  .level-trace {
    color: #606266 !important;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .system-logs {
    padding: 12px;

    .page-header {
      flex-direction: column;
      gap: 16px;
      text-align: center;

      .header-actions {
        width: 100%;
        justify-content: center;
        flex-wrap: wrap;
      }
    }

    .logs-view-controls {
      .view-controls {
        flex-direction: column;
        gap: 16px;
        align-items: flex-start;

        .display-options {
          flex-wrap: wrap;
        }
      }
    }

    .quick-actions {
      position: relative;
      right: auto;
      bottom: auto;
      margin-top: 20px;

      .actions-panel {
        flex-direction: column;
        gap: 12px;
        text-align: center;
        min-width: auto;

        .actions {
          justify-content: center;
        }
      }
    }
  }
}
</style>