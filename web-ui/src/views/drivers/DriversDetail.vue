<template>
  <div class="drivers-detail-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <el-button :icon="ArrowLeft" @click="handleGoBack">
          返回列表
        </el-button>

        <div class="header-title">
          <el-icon :size="32" :class="getStatusIconClass()">
            <component :is="getStatusIcon()" />
          </el-icon>
          <div class="title-info">
            <h1>{{ driverData.name }}</h1>
            <p class="driver-type">
              {{ driverData.protocol }} - {{ driverData.connectionType }}
            </p>
          </div>
        </div>
      </div>

      <div class="header-actions">
        <el-dropdown @command="handleCommand">
          <el-button :icon="Operation">
            操作
            <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                command="start"
                :disabled="driverData.status === 'running'"
              >
                <el-icon><VideoPlay /></el-icon>
                启动驱动
              </el-dropdown-item>
              <el-dropdown-item
                command="stop"
                :disabled="driverData.status !== 'running'"
              >
                <el-icon><VideoPause /></el-icon>
                停止驱动
              </el-dropdown-item>
              <el-dropdown-item
                command="restart"
                :disabled="driverData.status === 'stopped'"
              >
                <el-icon><Refresh /></el-icon>
                重启驱动
              </el-dropdown-item>
              <el-dropdown-item command="test" divided>
                <el-icon><Link /></el-icon>
                测试连接
              </el-dropdown-item>
              <el-dropdown-item command="export">
                <el-icon><Download /></el-icon>
                导出配置
              </el-dropdown-item>
              <el-dropdown-item command="scan">
                <el-icon><Search /></el-icon>
                扫描设备
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <el-button type="primary" :icon="Edit" @click="handleEditDriver">
          编辑配置
        </el-button>
      </div>
    </div>

    <!-- 状态概览 -->
    <div class="status-overview">
      <el-row :gutter="20">
        <el-col :span="6">
          <div class="status-card">
            <div class="card-header">
              <el-icon><Setting /></el-icon>
              <span>驱动状态</span>
            </div>
            <div class="card-content">
              <StatusTag :status="driverData.status" size="large" />
            </div>
          </div>
        </el-col>

        <el-col :span="6">
          <div class="status-card">
            <div class="card-header">
              <el-icon><DataBoard /></el-icon>
              <span>数据点</span>
            </div>
            <div class="card-content">
              <div class="metric-value">{{ driverData.dataPointCount }}</div>
              <div class="metric-label">已连接数据点</div>
            </div>
          </div>
        </el-col>

        <el-col :span="6">
          <div class="status-card">
            <div class="card-header">
              <el-icon><Timer /></el-icon>
              <span>扫描率</span>
            </div>
            <div class="card-content">
              <div class="metric-value">{{ driverData.scanRate }}</div>
              <div class="metric-label">点/秒</div>
            </div>
          </div>
        </el-col>

        <el-col :span="6">
          <div class="status-card">
            <div class="card-header">
              <el-icon><Warning /></el-icon>
              <span>错误统计</span>
            </div>
            <div class="card-content">
              <div
                class="metric-value"
                :class="{ error: driverData.errorCount > 0 }"
              >
                {{ driverData.errorCount }}
              </div>
              <div class="metric-label">通信错误</div>
            </div>
          </div>
        </el-col>
      </el-row>
    </div>

    <!-- 主要内容 -->
    <div class="main-content">
      <el-row :gutter="20">
        <!-- 左侧列 -->
        <el-col :span="16">
          <!-- 数据点监控 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><DataLine /></el-icon>
                数据点监控
              </h3>
              <div class="header-actions">
                <el-input
                  v-model="pointFilter"
                  placeholder="搜索数据点..."
                  style="width: 200px"
                  clearable
                >
                  <template #prefix>
                    <el-icon><Search /></el-icon>
                  </template>
                </el-input>
                <el-select
                  v-model="qualityFilter"
                  placeholder="质量等级"
                  style="width: 120px"
                >
                  <el-option label="全部" value="" />
                  <el-option label="好" value="good" />
                  <el-option label="不确定" value="uncertain" />
                  <el-option label="坏" value="bad" />
                </el-select>
                <el-button :icon="Refresh" @click="refreshDataPoints">
                  刷新
                </el-button>
              </div>
            </div>

            <div class="datapoints-table">
              <el-table
                :data="filteredDataPoints"
                stripe
                max-height="350"
                empty-text="暂无数据点"
              >
                <el-table-column prop="name" label="数据点名称" min-width="150">
                  <template #default="{ row }">
                    <div class="datapoint-name">
                      <el-icon class="point-icon">
                        <component :is="getDataPointIcon(row.type)" />
                      </el-icon>
                      <span>{{ row.name }}</span>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column prop="address" label="地址" width="120">
                  <template #default="{ row }">
                    <code class="address-code">{{ row.address }}</code>
                  </template>
                </el-table-column>
                <el-table-column prop="value" label="当前值" width="120">
                  <template #default="{ row }">
                    <span
                      class="point-value"
                      :class="getValueClass(row.quality)"
                    >
                      {{ formatPointValue(row.value, row.type) }}
                    </span>
                  </template>
                </el-table-column>
                <el-table-column prop="quality" label="质量" width="80">
                  <template #default="{ row }">
                    <el-tag :type="getQualityType(row.quality)" size="small">
                      {{ getQualityText(row.quality) }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="updateTime" label="更新时间" width="160">
                  <template #default="{ row }">
                    <span class="update-time">{{
                      formatTime(row.updateTime)
                    }}</span>
                  </template>
                </el-table-column>
                <el-table-column label="操作" width="120">
                  <template #default="{ row }">
                    <el-button-group size="small">
                      <el-button :icon="View" @click="showPointDetail(row)" />
                      <el-button
                        :icon="TrendCharts"
                        @click="showPointTrend(row)"
                      />
                    </el-button-group>
                  </template>
                </el-table-column>
              </el-table>

              <div v-if="dataPoints.length > 10" class="table-pagination">
                <el-pagination
                  v-model:current-page="currentPage"
                  :page-size="pageSize"
                  :total="dataPoints.length"
                  layout="prev, pager, next, jumper"
                />
              </div>
            </div>
          </div>

          <!-- 性能监控 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><TrendCharts /></el-icon>
                性能监控
              </h3>
              <div class="header-actions">
                <el-button-group size="small">
                  <el-button
                    v-for="period in timePeriods"
                    :key="period.value"
                    :type="selectedPeriod === period.value ? 'primary' : ''"
                    @click="selectedPeriod = period.value"
                  >
                    {{ period.label }}
                  </el-button>
                </el-button-group>
              </div>
            </div>

            <div class="charts-container">
              <el-row :gutter="16">
                <el-col :span="12">
                  <div class="chart-card">
                    <h4>扫描速率 (点/秒)</h4>
                    <ChartContainer
                      title="扫描速率"
                      chart-type="line"
                      :data="scanRateData"
                      :height="'200px'"
                    />
                  </div>
                </el-col>
                <el-col :span="12">
                  <div class="chart-card">
                    <h4>响应时间 (ms)</h4>
                    <ChartContainer
                      title="响应时间"
                      chart-type="line"
                      :data="responseTimeData"
                      :height="'200px'"
                    />
                  </div>
                </el-col>
              </el-row>

              <el-row :gutter="16" style="margin-top: 16px">
                <el-col :span="12">
                  <div class="chart-card">
                    <h4>成功率 (%)</h4>
                    <ChartContainer
                      title="成功率"
                      chart-type="bar"
                      :data="successRateData"
                      :height="'200px'"
                    />
                  </div>
                </el-col>
                <el-col :span="12">
                  <div class="chart-card">
                    <h4>数据质量分布</h4>
                    <ChartContainer
                      title="数据质量分布"
                      chart-type="pie"
                      :data="qualityDistribution"
                      :height="'200px'"
                    />
                  </div>
                </el-col>
              </el-row>
            </div>
          </div>

          <!-- 驱动日志 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><Document /></el-icon>
                驱动日志
              </h3>
              <div class="header-actions">
                <el-input
                  v-model="logFilter"
                  placeholder="搜索日志..."
                  style="width: 200px"
                  clearable
                >
                  <template #prefix>
                    <el-icon><Search /></el-icon>
                  </template>
                </el-input>
                <el-select
                  v-model="logLevel"
                  placeholder="日志级别"
                  style="width: 120px"
                >
                  <el-option label="全部" value="" />
                  <el-option label="错误" value="error" />
                  <el-option label="警告" value="warn" />
                  <el-option label="信息" value="info" />
                  <el-option label="调试" value="debug" />
                </el-select>
                <el-button :icon="Refresh" @click="refreshLogs">
                  刷新
                </el-button>
              </div>
            </div>

            <div class="logs-container">
              <el-table
                :data="filteredLogs"
                stripe
                max-height="400"
                empty-text="暂无日志数据"
              >
                <el-table-column prop="timestamp" label="时间" width="180">
                  <template #default="{ row }">
                    <span class="log-time">{{
                      formatTime(row.timestamp)
                    }}</span>
                  </template>
                </el-table-column>
                <el-table-column prop="level" label="级别" width="80">
                  <template #default="{ row }">
                    <el-tag :type="getLogLevelType(row.level)" size="small">
                      {{ row.level.toUpperCase() }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column
                  prop="message"
                  label="消息"
                  show-overflow-tooltip
                >
                  <template #default="{ row }">
                    <span class="log-message">{{ row.message }}</span>
                  </template>
                </el-table-column>
                <el-table-column prop="source" label="来源" width="120">
                  <template #default="{ row }">
                    <el-tag size="small" type="info">{{ row.source }}</el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </div>
        </el-col>

        <!-- 右侧列 -->
        <el-col :span="8">
          <!-- 基本信息 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><InfoFilled /></el-icon>
                基本信息
              </h3>
              <el-button size="small" text @click="handleEditDriver">
                <el-icon><Edit /></el-icon>
                编辑
              </el-button>
            </div>

            <div class="info-container">
              <el-descriptions :column="1" border>
                <el-descriptions-item label="驱动名称">
                  <strong>{{ driverData.name }}</strong>
                </el-descriptions-item>
                <el-descriptions-item label="通信协议">
                  <el-tag type="primary">{{ driverData.protocol }}</el-tag>
                </el-descriptions-item>
                <el-descriptions-item label="连接类型">
                  <el-tag type="success">{{
                    driverData.connectionType
                  }}</el-tag>
                </el-descriptions-item>
                <el-descriptions-item label="设备地址">
                  {{
                    driverData.config?.host ||
                    driverData.config?.serialPort ||
                    'N/A'
                  }}
                </el-descriptions-item>
                <el-descriptions-item label="启用状态">
                  <el-switch
                    v-model="driverData.enabled"
                    :disabled="loading"
                    @change="handleToggleEnabled"
                  />
                </el-descriptions-item>
                <el-descriptions-item label="自动扫描">
                  <el-switch
                    v-model="driverData.autoScan"
                    :disabled="loading"
                    @change="handleToggleAutoScan"
                  />
                </el-descriptions-item>
                <el-descriptions-item label="扫描间隔">
                  {{ driverData.scanInterval }}ms
                </el-descriptions-item>
                <el-descriptions-item label="创建时间">
                  {{ formatTime(driverData.createdAt) }}
                </el-descriptions-item>
              </el-descriptions>
            </div>
          </div>

          <!-- 驱动配置 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><Setting /></el-icon>
                驱动配置
              </h3>
              <el-button size="small" text @click="showConfigDialog = true">
                <el-icon><View /></el-icon>
                查看详情
              </el-button>
            </div>

            <div class="config-summary">
              <div
                v-for="(value, key) in driverData.config"
                :key="key"
                class="config-item"
              >
                <span class="config-label">{{ formatConfigLabel(key) }}:</span>
                <span class="config-value">{{ value }}</span>
              </div>
            </div>

            <div class="section-header" style="margin-top: 20px">
              <h4>性能设置</h4>
            </div>

            <el-descriptions :column="1" size="small">
              <el-descriptions-item label="最大并发">
                {{ driverData.performance?.maxConcurrentRequests || 10 }}
              </el-descriptions-item>
              <el-descriptions-item label="请求超时">
                {{ driverData.performance?.timeout || 5000 }}ms
              </el-descriptions-item>
              <el-descriptions-item label="重试次数">
                {{ driverData.retry?.maxRetries || 3 }}
              </el-descriptions-item>
            </el-descriptions>
          </div>

          <!-- 通信状态 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><Connection /></el-icon>
                通信状态
              </h3>
            </div>

            <div class="comm-stats">
              <div class="stat-item">
                <div class="stat-label">成功率</div>
                <el-progress
                  :percentage="driverData.stats?.successRate || 0"
                  :color="getSuccessRateColor()"
                  :show-text="true"
                  :stroke-width="12"
                />
              </div>

              <div class="stat-item">
                <div class="stat-label">平均响应时间</div>
                <div class="stat-value">
                  {{ driverData.stats?.avgResponseTime || 0 }}ms
                </div>
              </div>

              <div class="stat-item">
                <div class="stat-label">最后通信时间</div>
                <div class="stat-value">
                  {{
                    formatTime(
                      driverData.stats?.lastCommunication || new Date()
                    )
                  }}
                </div>
              </div>
            </div>
          </div>

          <!-- 最近事件 -->
          <div class="content-section">
            <div class="section-header">
              <h3>
                <el-icon><Clock /></el-icon>
                最近事件
              </h3>
            </div>

            <div class="events-timeline">
              <el-timeline>
                <el-timeline-item
                  v-for="event in recentEvents"
                  :key="event.id"
                  :timestamp="formatTime(event.timestamp)"
                  :type="getEventType(event.type)"
                >
                  <div class="event-content">
                    <div class="event-title">{{ event.title }}</div>
                    <div class="event-description">{{ event.description }}</div>
                  </div>
                </el-timeline-item>
              </el-timeline>
            </div>
          </div>
        </el-col>
      </el-row>
    </div>

    <!-- 数据点详情对话框 -->
    <el-dialog
      v-model="showPointDialog"
      :title="`数据点详情 - ${selectedPoint?.name}`"
      width="600px"
      :destroy-on-close="true"
    >
      <div v-if="selectedPoint" class="point-detail">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="名称">
            {{ selectedPoint.name }}
          </el-descriptions-item>
          <el-descriptions-item label="类型">
            <el-tag>{{ selectedPoint.type }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="地址">
            <code>{{ selectedPoint.address }}</code>
          </el-descriptions-item>
          <el-descriptions-item label="当前值">
            {{ formatPointValue(selectedPoint.value, selectedPoint.type) }}
          </el-descriptions-item>
          <el-descriptions-item label="质量">
            <el-tag :type="getQualityType(selectedPoint.quality)">
              {{ getQualityText(selectedPoint.quality) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="更新时间">
            {{ formatTime(selectedPoint.updateTime) }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
    </el-dialog>

    <!-- 配置详情对话框 -->
    <el-dialog
      v-model="showConfigDialog"
      title="驱动配置详情"
      width="800px"
      :destroy-on-close="true"
    >
      <el-tabs v-model="configTab">
        <el-tab-pane label="JSON 格式" name="json">
          <div class="config-json">
            <pre><code>{{ formatConfigJson() }}</code></pre>
          </div>
        </el-tab-pane>
        <el-tab-pane label="YAML 格式" name="yaml">
          <div class="config-yaml">
            <pre><code>{{ formatConfigYaml() }}</code></pre>
          </div>
        </el-tab-pane>
      </el-tabs>
    </el-dialog>

    <!-- 加载遮罩 -->
    <div
      v-if="loading"
      v-loading="loading"
      element-loading-text="正在加载..."
      class="loading-overlay"
    ></div>
  </div>
</template>

<script setup lang="ts">
import {
  ArrowLeft,
  Operation,
  ArrowDown,
  VideoPlay,
  VideoPause,
  Refresh,
  Link,
  Download,
  Search,
  Edit,
  Setting,
  DataBoard,
  Timer,
  Warning,
  DataLine,
  TrendCharts,
  Document,
  InfoFilled,
  View,
  Connection,
  Clock,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, reactive, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'

// 导入业务组件
import { StatusTag } from '../../components/base'
import { ChartContainer } from '../../components/business'
// import { DriverStatus, PerformanceChart } from '../../components/business' // 暂时注释掉不存在的组件

// 类型定义
interface DriverData {
  id: string
  name: string
  description: string
  protocol: string
  connectionType: string
  status: 'running' | 'stopped' | 'error' | 'starting'
  enabled: boolean
  autoScan: boolean
  scanInterval: number
  dataPointCount: number
  scanRate: number
  errorCount: number
  config: Record<string, any>
  performance?: {
    maxConcurrentRequests: number
    timeout: number
    scanInterval: number
  }
  retry?: {
    maxRetries: number
    retryInterval: number
  }
  stats?: {
    successRate: number
    avgResponseTime: number
    lastCommunication: Date
  }
  createdAt: Date
  updatedAt: Date
}

interface DataPoint {
  id: string
  name: string
  address: string
  type: 'bool' | 'int16' | 'int32' | 'float' | 'string'
  value: any
  quality: 'good' | 'uncertain' | 'bad'
  updateTime: Date
}

interface LogEntry {
  id: string
  timestamp: Date
  level: 'error' | 'warn' | 'info' | 'debug'
  message: string
  source: string
}

interface Event {
  id: string
  timestamp: Date
  type: 'info' | 'success' | 'warning' | 'error'
  title: string
  description: string
}

interface ChartData {
  name: string
  value: number
  time: string
}

// 路由
const route = useRoute()
const router = useRouter()

// 状态管理
const loading = ref(false)
const showPointDialog = ref(false)
const showConfigDialog = ref(false)
const configTab = ref('json')
const selectedPeriod = ref('1h')
const pointFilter = ref('')
const qualityFilter = ref('')
const logFilter = ref('')
const logLevel = ref('')
const currentPage = ref(1)
const pageSize = ref(10)

const selectedPoint = ref<DataPoint | null>(null)

// 时间期间选项
const timePeriods = [
  { label: '1小时', value: '1h' },
  { label: '6小时', value: '6h' },
  { label: '24小时', value: '24h' },
  { label: '7天', value: '7d' },
]

// 驱动数据
const driverData = ref<DriverData>({
  id: '',
  name: '',
  description: '',
  protocol: '',
  connectionType: '',
  status: 'stopped',
  enabled: false,
  autoScan: false,
  scanInterval: 1000,
  dataPointCount: 0,
  scanRate: 0,
  errorCount: 0,
  config: {},
  createdAt: new Date(),
  updatedAt: new Date(),
})

// 数据点数据
const dataPoints = ref<DataPoint[]>([])

// 日志数据
const logs = ref<LogEntry[]>([])

// 事件数据
const recentEvents = ref<Event[]>([])

// 性能数据
const scanRateData = ref<ChartData[]>([])
const responseTimeData = ref<ChartData[]>([])
const successRateData = ref<ChartData[]>([])
const qualityDistribution = ref<ChartData[]>([])

// 计算属性
const filteredDataPoints = computed(() => {
  const filtered = dataPoints.value.filter(point => {
    const matchesFilter =
      !pointFilter.value ||
      point.name.toLowerCase().includes(pointFilter.value.toLowerCase())
    const matchesQuality =
      !qualityFilter.value || point.quality === qualityFilter.value
    return matchesFilter && matchesQuality
  })

  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filtered.slice(start, end)
})

const filteredLogs = computed(() => {
  return logs.value.filter(log => {
    const matchesFilter =
      !logFilter.value ||
      log.message.toLowerCase().includes(logFilter.value.toLowerCase())
    const matchesLevel = !logLevel.value || log.level === logLevel.value
    return matchesFilter && matchesLevel
  })
})

// 方法
const getStatusIcon = () => {
  const iconMap = {
    running: 'VideoPlay',
    starting: 'Loading',
    stopped: 'VideoPause',
    error: 'Warning',
  }
  return iconMap[driverData.value.status] || 'Setting'
}

const getStatusIconClass = () => {
  const classMap = {
    running: 'status-icon running',
    starting: 'status-icon starting',
    stopped: 'status-icon stopped',
    error: 'status-icon error',
  }
  return classMap[driverData.value.status] || 'status-icon'
}

const getDataPointIcon = (type: string) => {
  const iconMap: Record<string, string> = {
    bool: 'Switch',
    int16: 'Odometer',
    int32: 'Odometer',
    float: 'TrendCharts',
    string: 'Document',
  }
  return iconMap[type] || 'DataBoard'
}

const getQualityType = (quality: string) => {
  const typeMap: Record<string, string> = {
    good: 'success',
    uncertain: 'warning',
    bad: 'danger',
  }
  return typeMap[quality] || 'info'
}

const getQualityText = (quality: string) => {
  const textMap: Record<string, string> = {
    good: '好',
    uncertain: '不确定',
    bad: '坏',
  }
  return textMap[quality] || quality
}

const getValueClass = (quality: string) => {
  return {
    'value-good': quality === 'good',
    'value-uncertain': quality === 'uncertain',
    'value-bad': quality === 'bad',
  }
}

const getLogLevelType = (level: string) => {
  const typeMap: Record<string, string> = {
    error: 'danger',
    warn: 'warning',
    info: 'info',
    debug: 'success',
  }
  return typeMap[level] || 'info'
}

const getEventType = (type: string) => {
  const typeMap: Record<string, string> = {
    error: 'danger',
    warning: 'warning',
    success: 'success',
    info: 'primary',
  }
  return typeMap[type] || 'primary'
}

const getSuccessRateColor = () => {
  const rate = driverData.value.stats?.successRate || 0
  if (rate >= 90) return '#67c23a'
  if (rate >= 70) return '#e6a23c'
  return '#f56c6c'
}

const formatTime = (time: Date) => {
  return time.toLocaleString('zh-CN')
}

const formatPointValue = (value: any, type: string) => {
  if (value === null || value === undefined) return 'N/A'

  switch (type) {
    case 'bool':
      return value ? '真' : '假'
    case 'float':
      return typeof value === 'number' ? value.toFixed(2) : value
    default:
      return value.toString()
  }
}

const formatConfigLabel = (key: string) => {
  const labelMap: Record<string, string> = {
    host: '主机地址',
    port: '端口号',
    timeout: '超时时间',
    serialPort: '串口号',
    baudRate: '波特率',
    unitId: '单元ID',
    endpoint: '端点URL',
    scanInterval: '扫描间隔',
  }
  return labelMap[key] || key
}

const formatConfigJson = () => {
  try {
    return JSON.stringify(driverData.value, null, 2)
  } catch (error) {
    return '配置格式错误'
  }
}

const formatConfigYaml = () => {
  try {
    const obj = driverData.value
    let yaml = ''

    const toYaml = (obj: any, indent = 0) => {
      const spaces = '  '.repeat(indent)
      for (const [key, value] of Object.entries(obj)) {
        if (
          typeof value === 'object' &&
          value !== null &&
          !(value instanceof Date)
        ) {
          yaml += `${spaces}${key}:\n`
          toYaml(value, indent + 1)
        } else {
          yaml += `${spaces}${key}: ${value}\n`
        }
      }
    }

    toYaml(obj)
    return yaml
  } catch (error) {
    return '配置格式错误'
  }
}

// 事件处理
const handleGoBack = () => {
  router.push('/drivers')
}

const handleEditDriver = () => {
  router.push(`/drivers/edit/${driverData.value.id}`)
}

const handleCommand = async (command: string) => {
  loading.value = true

  try {
    switch (command) {
      case 'start':
        await handleStartDriver()
        break
      case 'stop':
        await handleStopDriver()
        break
      case 'restart':
        await handleRestartDriver()
        break
      case 'test':
        await handleTestConnection()
        break
      case 'export':
        handleExportConfig()
        break
      case 'scan':
        await handleScanDevices()
        break
    }
  } finally {
    loading.value = false
  }
}

const handleStartDriver = async () => {
  driverData.value.status = 'starting'
  await new Promise(resolve => setTimeout(resolve, 2000))
  driverData.value.status = 'running'
  ElMessage.success('驱动已启动')
  addEvent('成功', '驱动启动', '驱动已成功启动，开始数据采集')
}

const handleStopDriver = async () => {
  await new Promise(resolve => setTimeout(resolve, 1000))
  driverData.value.status = 'stopped'
  ElMessage.success('驱动已停止')
  addEvent('信息', '驱动停止', '用户手动停止驱动')
}

const handleRestartDriver = async () => {
  driverData.value.status = 'starting'
  await new Promise(resolve => setTimeout(resolve, 2500))
  driverData.value.status = 'running'
  ElMessage.success('驱动已重启')
  addEvent('成功', '驱动重启', '驱动重启成功，恢复正常数据采集')
}

const handleTestConnection = async () => {
  ElMessage.info('正在测试连接...')
  await new Promise(resolve => setTimeout(resolve, 2000))
  const success = Math.random() > 0.2
  if (success) {
    ElMessage.success('连接测试成功')
    addEvent('成功', '连接测试', '连接测试通过，设备响应正常')
  } else {
    ElMessage.error('连接测试失败')
    addEvent('错误', '连接测试', '连接测试失败，请检查设备和配置')
  }
}

const handleScanDevices = async () => {
  ElMessage.info('正在扫描设备...')
  await new Promise(resolve => setTimeout(resolve, 3000))
  const foundCount = Math.floor(Math.random() * 20) + 5
  ElMessage.success(`扫描完成，发现 ${foundCount} 个数据点`)
  addEvent('成功', '设备扫描', `扫描完成，发现 ${foundCount} 个数据点`)
}

const handleExportConfig = () => {
  const exportData = {
    driver: driverData.value,
    dataPoints: dataPoints.value,
    exportTime: new Date().toISOString(),
    version: '1.0',
  }

  const blob = new Blob([JSON.stringify(exportData, null, 2)], {
    type: 'application/json',
  })

  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `driver_${driverData.value.name}_${new Date().toISOString().split('T')[0]}.json`
  a.click()

  URL.revokeObjectURL(url)
  ElMessage.success('配置已导出')
}

const handleToggleEnabled = async (enabled: boolean) => {
  loading.value = true

  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    ElMessage.success(`驱动已${enabled ? '启用' : '禁用'}`)
    addEvent('info', '状态更改', `驱动已${enabled ? '启用' : '禁用'}`)
  } catch (error) {
    driverData.value.enabled = !enabled
    ElMessage.error('操作失败')
  } finally {
    loading.value = false
  }
}

const handleToggleAutoScan = async (enabled: boolean) => {
  loading.value = true

  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    ElMessage.success(`自动扫描已${enabled ? '启用' : '禁用'}`)
    addEvent('info', '配置更改', `自动扫描已${enabled ? '启用' : '禁用'}`)
  } catch (error) {
    driverData.value.autoScan = !enabled
    ElMessage.error('操作失败')
  } finally {
    loading.value = false
  }
}

const showPointDetail = (point: DataPoint) => {
  selectedPoint.value = point
  showPointDialog.value = true
}

const showPointTrend = (point: DataPoint) => {
  // 跳转到数据点趋势页面
  router.push(`/datapoints/${point.id}/trend`)
}

const refreshDataPoints = () => {
  generateMockDataPoints()
  ElMessage.success('数据点已刷新')
}

const refreshLogs = () => {
  generateMockLogs()
  ElMessage.success('日志已刷新')
}

const addEvent = (type: string, title: string, description: string) => {
  const event: Event = {
    id: Date.now().toString(),
    timestamp: new Date(),
    type: type as any,
    title,
    description,
  }
  recentEvents.value.unshift(event)
  if (recentEvents.value.length > 10) {
    recentEvents.value = recentEvents.value.slice(0, 10)
  }
}

// 数据初始化
const initializeData = () => {
  const driverId = route.params.id as string

  // 模拟驱动数据
  driverData.value = {
    id: driverId,
    name: 'PLC主控制器',
    description: '生产线主PLC控制器',
    protocol: 'Modbus TCP',
    connectionType: 'Ethernet',
    status: 'running',
    enabled: true,
    autoScan: true,
    scanInterval: 1000,
    dataPointCount: 48,
    scanRate: 35,
    errorCount: 2,
    config: {
      host: '',
      port: 502,
      unitId: 1,
      timeout: 5000,
    },
    performance: {
      maxConcurrentRequests: 10,
      timeout: 5000,
      scanInterval: 1000,
    },
    retry: {
      maxRetries: 3,
      retryInterval: 1000,
    },
    stats: {
      successRate: 95.6,
      avgResponseTime: 45,
      lastCommunication: new Date(),
    },
    createdAt: new Date(Date.now() - 14 * 24 * 60 * 60 * 1000), // 14 days ago
    updatedAt: new Date(Date.now() - 30 * 60 * 1000), // 30 minutes ago
  }

  generateMockDataPoints()
  generateMockLogs()
  generateMockEvents()
  generateMockChartData()
}

const generateMockDataPoints = () => {
  const types = ['bool', 'int16', 'int32', 'float', 'string'] as const
  const qualities = ['good', 'uncertain', 'bad'] as const
  const addresses = [
    '40001',
    '40002',
    '40003',
    '40004',
    '40005',
    '30001',
    '30002',
    '30003',
    '30004',
    '30005',
    '10001',
    '10002',
    '10003',
    '10004',
    '10005',
  ]

  dataPoints.value = Array.from({ length: 48 }, (_, i) => {
    const type = types[Math.floor(Math.random() * types.length)]
    let value: any

    switch (type) {
      case 'bool':
        value = Math.random() > 0.5
        break
      case 'int16':
        value = Math.floor(Math.random() * 65536)
        break
      case 'int32':
        value = Math.floor(Math.random() * 4294967296)
        break
      case 'float':
        value = (Math.random() * 1000).toFixed(2)
        break
      case 'string':
        value = `设备状态${i + 1}`
        break
    }

    return {
      id: (i + 1).toString(),
      name: `数据点${i + 1}`,
      address: addresses[i % addresses.length],
      type,
      value,
      quality: qualities[Math.floor(Math.random() * qualities.length)],
      updateTime: new Date(Date.now() - Math.random() * 60000), // Last minute
    }
  })
}

const generateMockLogs = () => {
  const levels = ['info', 'warn', 'error', 'debug'] as const
  const sources = ['driver', 'communication', 'data', 'system']
  const messages = [
    '驱动启动成功',
    '数据采集正常',
    '设备连接成功',
    '扫描周期完成',
    '数据质量异常',
    '通信超时重试',
    '配置更新完成',
    '数据点更新',
  ]

  logs.value = Array.from({ length: 80 }, (_, i) => ({
    id: (i + 1).toString(),
    timestamp: new Date(Date.now() - i * 30000), // Every 30 seconds
    level: levels[Math.floor(Math.random() * levels.length)],
    message: messages[Math.floor(Math.random() * messages.length)],
    source: sources[Math.floor(Math.random() * sources.length)],
  }))
}

const generateMockEvents = () => {
  const events: Event[] = [
    {
      id: '1',
      timestamp: new Date(Date.now() - 5 * 60000),
      type: 'success',
      title: '数据采集正常',
      description: '驱动正在正常采集数据，所有数据点状态良好',
    },
    {
      id: '2',
      timestamp: new Date(Date.now() - 15 * 60000),
      type: 'warning',
      title: '通信延迟',
      description: '检测到与设备的通信延迟较高，平均响应时间超过100ms',
    },
    {
      id: '3',
      timestamp: new Date(Date.now() - 45 * 60000),
      type: 'info',
      title: '配置更新',
      description: '驱动配置已更新，扫描间隔调整为1000ms',
    },
    {
      id: '4',
      timestamp: new Date(Date.now() - 2 * 60 * 60000),
      type: 'error',
      title: '连接失败',
      description: '与设备的连接中断，正在尝试重连',
    },
    {
      id: '5',
      timestamp: new Date(Date.now() - 6 * 60 * 60000),
      type: 'success',
      title: '驱动重启',
      description: '驱动重启成功，恢复正常数据采集',
    },
  ]

  recentEvents.value = events
}

const generateMockChartData = () => {
  const now = Date.now()
  const points = 24
  const interval = 60 * 60 * 1000 // 1 hour

  // 扫描率数据
  scanRateData.value = Array.from({ length: points }, (_, i) => {
    const time = now - (points - 1 - i) * interval
    return {
      name: new Date(time).toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
      }),
      value: Math.floor(Math.random() * 20) + 25, // 25-45 points/sec
      time: new Date(time).toISOString(),
    }
  })

  // 响应时间数据
  responseTimeData.value = Array.from({ length: points }, (_, i) => {
    const time = now - (points - 1 - i) * interval
    return {
      name: new Date(time).toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
      }),
      value: Math.floor(Math.random() * 80) + 20, // 20-100ms
      time: new Date(time).toISOString(),
    }
  })

  // 成功率数据
  successRateData.value = Array.from({ length: points }, (_, i) => {
    const time = now - (points - 1 - i) * interval
    return {
      name: new Date(time).toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
      }),
      value: Math.random() * 10 + 90, // 90-100%
      time: new Date(time).toISOString(),
    }
  })

  // 数据质量分布
  qualityDistribution.value = [
    { name: '好', value: 85, time: '' },
    { name: '不确定', value: 12, time: '' },
    { name: '坏', value: 3, time: '' },
  ]
}

// 定时刷新
let refreshTimer: NodeJS.Timeout

const startAutoRefresh = () => {
  refreshTimer = setInterval(() => {
    // 更新实时数据
    if (driverData.value.status === 'running') {
      driverData.value.scanRate = Math.floor(Math.random() * 15) + 30 // 30-45

      // 更新数据点值
      dataPoints.value.forEach(point => {
        if (Math.random() > 0.7) {
          // 30% chance to update
          point.updateTime = new Date()
          switch (point.type) {
            case 'bool':
              point.value = Math.random() > 0.5
              break
            case 'float':
              point.value = (Math.random() * 1000).toFixed(2)
              break
            case 'int16':
              point.value = Math.floor(Math.random() * 65536)
              break
          }
        }
      })
    }

    // 更新图表数据
    if (Math.random() > 0.6) {
      generateMockChartData()
    }
  }, 5000) // Every 5 seconds
}

// 生命周期
onMounted(() => {
  initializeData()
  startAutoRefresh()
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<style scoped lang="scss">
.drivers-detail-page {
  padding: 20px;
  max-width: 1600px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding: 20px 0;
  border-bottom: 1px solid var(--el-border-color-light);

  .header-left {
    display: flex;
    align-items: center;
    gap: 20px;

    .header-title {
      display: flex;
      align-items: center;
      gap: 16px;

      .status-icon {
        &.running {
          color: var(--el-color-success);
        }
        &.starting {
          color: var(--el-color-warning);
          animation: pulse 1.5s ease-in-out infinite alternate;
        }
        &.stopped {
          color: var(--el-color-info);
        }
        &.error {
          color: var(--el-color-danger);
        }
      }

      .title-info {
        h1 {
          margin: 0 0 4px 0;
          font-size: 28px;
          color: var(--el-text-color-primary);
        }

        .driver-type {
          margin: 0;
          color: var(--el-text-color-secondary);
          font-size: 14px;
        }
      }
    }
  }

  .header-actions {
    display: flex;
    gap: 12px;
  }
}

.status-overview {
  margin-bottom: 24px;

  .status-card {
    background: white;
    border-radius: 12px;
    padding: 20px;
    border: 1px solid var(--el-border-color-light);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.02);

    .card-header {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 16px;
      color: var(--el-text-color-regular);
      font-size: 14px;
      font-weight: 500;

      .el-icon {
        font-size: 16px;
        color: var(--el-color-primary);
      }
    }

    .card-content {
      .metric-value {
        font-size: 24px;
        font-weight: 600;
        color: var(--el-text-color-primary);
        margin-bottom: 4px;

        &.error {
          color: var(--el-color-danger);
        }
      }

      .metric-label {
        font-size: 12px;
        color: var(--el-text-color-secondary);
      }
    }
  }
}

.main-content {
  .content-section {
    background: white;
    border-radius: 12px;
    padding: 24px;
    margin-bottom: 20px;
    border: 1px solid var(--el-border-color-light);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.02);

    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;

      h3,
      h4 {
        margin: 0;
        font-size: 18px;
        color: var(--el-text-color-primary);
        display: flex;
        align-items: center;
        gap: 8px;

        .el-icon {
          color: var(--el-color-primary);
        }
      }

      h4 {
        font-size: 16px;
      }

      .header-actions {
        display: flex;
        align-items: center;
        gap: 12px;
      }
    }
  }
}

.datapoints-table {
  .datapoint-name {
    display: flex;
    align-items: center;
    gap: 8px;

    .point-icon {
      color: var(--el-color-primary);
      font-size: 16px;
    }
  }

  .address-code {
    font-family: 'Courier New', monospace;
    background: var(--el-fill-color-light);
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 12px;
  }

  .point-value {
    font-weight: 500;

    &.value-good {
      color: var(--el-color-success);
    }
    &.value-uncertain {
      color: var(--el-color-warning);
    }
    &.value-bad {
      color: var(--el-color-danger);
    }
  }

  .update-time {
    font-size: 12px;
    color: var(--el-text-color-secondary);
  }

  .table-pagination {
    margin-top: 16px;
    display: flex;
    justify-content: center;
  }
}

.charts-container {
  .chart-card {
    background: var(--el-fill-color-extra-light);
    border-radius: 8px;
    padding: 16px;

    h4 {
      margin: 0 0 12px 0;
      font-size: 14px;
      color: var(--el-text-color-regular);
      font-weight: 500;
    }
  }
}

.logs-container {
  .log-time {
    font-family: 'Courier New', monospace;
    font-size: 12px;
  }

  .log-message {
    font-size: 13px;
  }
}

.info-container {
  :deep(.el-descriptions__label) {
    font-weight: 500;
  }
}

.config-summary {
  .config-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--el-border-color-lighter);

    &:last-child {
      border-bottom: none;
    }

    .config-label {
      font-weight: 500;
      color: var(--el-text-color-regular);
    }

    .config-value {
      color: var(--el-text-color-primary);
      font-family: 'Courier New', monospace;
      font-size: 13px;
    }
  }
}

.comm-stats {
  .stat-item {
    margin-bottom: 16px;

    .stat-label {
      font-size: 14px;
      color: var(--el-text-color-regular);
      margin-bottom: 8px;
    }

    .stat-value {
      font-size: 16px;
      font-weight: 500;
      color: var(--el-text-color-primary);
    }
  }
}

.events-timeline {
  .event-content {
    .event-title {
      font-weight: 500;
      color: var(--el-text-color-primary);
      margin-bottom: 4px;
    }

    .event-description {
      font-size: 13px;
      color: var(--el-text-color-secondary);
      line-height: 1.4;
    }
  }
}

.point-detail {
  :deep(.el-descriptions__label) {
    font-weight: 500;
  }
}

.config-json,
.config-yaml {
  pre {
    background: var(--el-fill-color-light);
    padding: 16px;
    border-radius: 6px;
    font-size: 12px;
    max-height: 400px;
    overflow-y: auto;
    font-family: 'Courier New', monospace;

    code {
      color: var(--el-text-color-primary);
    }
  }
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(255, 255, 255, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

@keyframes pulse {
  from {
    opacity: 1;
  }
  to {
    opacity: 0.4;
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .main-content {
    .el-row {
      flex-direction: column;

      .el-col {
        width: 100% !important;
        max-width: none !important;
      }
    }
  }

  .status-overview {
    .el-row {
      flex-direction: column;

      .el-col {
        width: 100% !important;
        margin-bottom: 16px;
      }
    }
  }
}

@media (max-width: 768px) {
  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;

    .header-actions {
      flex-wrap: wrap;
    }
  }

  .charts-container {
    .el-row {
      flex-direction: column;

      .el-col {
        width: 100% !important;
        margin-bottom: 16px;
      }
    }
  }
}
</style>
