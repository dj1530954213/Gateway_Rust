<template>
  <div class="alerts-list-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <h1>告警管理</h1>
        <p>实时监控系统告警信息，及时处理异常状态</p>
      </div>

      <div class="header-actions">
        <el-button :icon="Setting" @click="handleAlertRules">
          告警规则
        </el-button>
        <el-button :icon="Clock" @click="handleAlertHistory">
          历史记录
        </el-button>
        <el-button type="primary" :icon="Plus" @click="handleCreateAlert">
          手动告警
        </el-button>
      </div>
    </div>

    <!-- 告警概览 -->
    <div class="alert-overview">
      <el-row :gutter="20">
        <el-col :span="6">
          <div class="overview-card critical">
            <div class="card-icon">
              <el-icon><WarningFilled /></el-icon>
            </div>
            <div class="card-content">
              <div class="card-value">{{ alertStats.critical }}</div>
              <div class="card-label">紧急告警</div>
            </div>
          </div>
        </el-col>

        <el-col :span="6">
          <div class="overview-card warning">
            <div class="card-icon">
              <el-icon><Warning /></el-icon>
            </div>
            <div class="card-content">
              <div class="card-value">{{ alertStats.warning }}</div>
              <div class="card-label">警告</div>
            </div>
          </div>
        </el-col>

        <el-col :span="6">
          <div class="overview-card info">
            <div class="card-icon">
              <el-icon><InfoFilled /></el-icon>
            </div>
            <div class="card-content">
              <div class="card-value">{{ alertStats.info }}</div>
              <div class="card-label">信息</div>
            </div>
          </div>
        </el-col>

        <el-col :span="6">
          <div class="overview-card acknowledged">
            <div class="card-icon">
              <el-icon><CircleCheck /></el-icon>
            </div>
            <div class="card-content">
              <div class="card-value">{{ alertStats.acknowledged }}</div>
              <div class="card-label">已确认</div>
            </div>
          </div>
        </el-col>
      </el-row>
    </div>

    <!-- 筛选和搜索 -->
    <div class="search-filters">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-input
            v-model="searchQuery"
            placeholder="搜索告警消息..."
            clearable
            @input="handleSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </el-col>

        <el-col :span="4">
          <el-select v-model="severityFilter" placeholder="严重等级" clearable>
            <el-option label="全部" value="" />
            <el-option label="紧急" value="critical" />
            <el-option label="警告" value="warning" />
            <el-option label="信息" value="info" />
          </el-select>
        </el-col>

        <el-col :span="4">
          <el-select v-model="statusFilter" placeholder="状态" clearable>
            <el-option label="全部" value="" />
            <el-option label="活跃" value="active" />
            <el-option label="已确认" value="acknowledged" />
            <el-option label="已解决" value="resolved" />
          </el-select>
        </el-col>

        <el-col :span="4">
          <el-select v-model="sourceFilter" placeholder="告警来源" clearable>
            <el-option label="全部" value="" />
            <el-option label="驱动" value="driver" />
            <el-option label="连接" value="connection" />
            <el-option label="数据点" value="datapoint" />
            <el-option label="系统" value="system" />
          </el-select>
        </el-col>

        <el-col :span="6">
          <el-date-picker
            v-model="dateRange"
            type="datetimerange"
            start-placeholder="开始时间"
            end-placeholder="结束时间"
            format="YYYY-MM-DD HH:mm:ss"
            value-format="YYYY-MM-DD HH:mm:ss"
            @change="handleDateRangeChange"
          />
        </el-col>
      </el-row>

      <div class="filter-actions">
        <el-button :icon="Refresh" @click="handleRefresh"> 刷新 </el-button>
        <el-button @click="handleResetFilters"> 重置筛选 </el-button>
        <el-button
          :disabled="!selectedAlerts.length"
          @click="handleBulkAcknowledge"
        >
          批量确认
        </el-button>
        <el-button
          :disabled="!selectedAlerts.length"
          type="danger"
          @click="handleBulkResolve"
        >
          批量解决
        </el-button>
      </div>
    </div>

    <!-- 告警列表 -->
    <div class="alerts-table">
      <el-table
        v-loading="loading"
        :data="filteredAlerts"
        stripe
        @selection-change="handleSelectionChange"
      >
        <el-table-column type="selection" width="55" />

        <el-table-column prop="severity" label="等级" width="80">
          <template #default="{ row }">
            <el-icon :class="getSeverityClass(row.severity)" :size="20">
              <component :is="getSeverityIcon(row.severity)" />
            </el-icon>
          </template>
        </el-table-column>

        <el-table-column prop="title" label="告警标题" min-width="200">
          <template #default="{ row }">
            <div class="alert-title">
              <span class="title-text">{{ row.title }}</span>
              <el-tag v-if="row.isNew" type="danger" size="small">新</el-tag>
            </div>
          </template>
        </el-table-column>

        <el-table-column
          prop="message"
          label="详细消息"
          show-overflow-tooltip
          min-width="250"
        >
          <template #default="{ row }">
            <span class="alert-message">{{ row.message }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="source" label="来源" width="120">
          <template #default="{ row }">
            <el-tag size="small" :type="getSourceTagType(row.source)">
              {{ formatSource(row.source) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusTagType(row.status)" size="small">
              {{ formatStatus(row.status) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="count" label="次数" width="80">
          <template #default="{ row }">
            <el-badge v-if="row.count > 1" :value="row.count" :max="99">
              <span class="count-display">{{ row.count }}</span>
            </el-badge>
            <span v-else>{{ row.count }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="timestamp" label="发生时间" width="180">
          <template #default="{ row }">
            <div class="time-info">
              <div class="timestamp">{{ formatTime(row.timestamp) }}</div>
              <div class="duration">{{ getTimeDuration(row.timestamp) }}</div>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="acknowledgedBy" label="确认人" width="120">
          <template #default="{ row }">
            <span v-if="row.acknowledgedBy" class="acknowledged-by">
              {{ row.acknowledgedBy }}
            </span>
            <span v-else class="not-acknowledged">-</span>
          </template>
        </el-table-column>

        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <div class="action-buttons">
              <el-button
                size="small"
                :icon="View"
                @click="handleViewDetail(row)"
              >
                详情
              </el-button>

              <el-button
                v-if="row.status === 'active'"
                size="small"
                type="warning"
                :icon="Check"
                @click="handleAcknowledge(row)"
              >
                确认
              </el-button>

              <el-button
                v-if="row.status !== 'resolved'"
                size="small"
                type="success"
                :icon="CircleCheck"
                @click="handleResolve(row)"
              >
                解决
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="table-pagination">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[20, 50, 100, 200]"
          :total="totalAlerts"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </div>

    <!-- 告警详情对话框 -->
    <el-dialog
      v-model="showDetailDialog"
      :title="`告警详情 - ${selectedAlert?.title}`"
      width="800px"
      :destroy-on-close="true"
    >
      <div v-if="selectedAlert" class="alert-detail">
        <el-row :gutter="20">
          <el-col :span="12">
            <el-descriptions :column="1" border>
              <el-descriptions-item label="告警标题">
                <strong>{{ selectedAlert.title }}</strong>
              </el-descriptions-item>
              <el-descriptions-item label="严重等级">
                <el-tag :type="getSeverityTagType(selectedAlert.severity)">
                  {{ formatSeverity(selectedAlert.severity) }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="告警状态">
                <el-tag :type="getStatusTagType(selectedAlert.status)">
                  {{ formatStatus(selectedAlert.status) }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="告警来源">
                <el-tag :type="getSourceTagType(selectedAlert.source)">
                  {{ formatSource(selectedAlert.source) }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="发生次数">
                {{ selectedAlert.count }}
              </el-descriptions-item>
              <el-descriptions-item label="发生时间">
                {{ formatTime(selectedAlert.timestamp) }}
              </el-descriptions-item>
            </el-descriptions>
          </el-col>

          <el-col :span="12">
            <el-descriptions :column="1" border>
              <el-descriptions-item label="确认人">
                {{ selectedAlert.acknowledgedBy || '未确认' }}
              </el-descriptions-item>
              <el-descriptions-item label="确认时间">
                {{
                  selectedAlert.acknowledgedAt
                    ? formatTime(selectedAlert.acknowledgedAt)
                    : '-'
                }}
              </el-descriptions-item>
              <el-descriptions-item label="解决人">
                {{ selectedAlert.resolvedBy || '未解决' }}
              </el-descriptions-item>
              <el-descriptions-item label="解决时间">
                {{
                  selectedAlert.resolvedAt
                    ? formatTime(selectedAlert.resolvedAt)
                    : '-'
                }}
              </el-descriptions-item>
              <el-descriptions-item label="持续时间">
                {{ getAlertDuration(selectedAlert) }}
              </el-descriptions-item>
            </el-descriptions>
          </el-col>
        </el-row>

        <div class="alert-message-detail">
          <h4>详细消息</h4>
          <div class="message-content">
            {{ selectedAlert.message }}
          </div>
        </div>

        <div v-if="selectedAlert.context" class="alert-context">
          <h4>相关信息</h4>
          <el-descriptions :column="2" border size="small">
            <el-descriptions-item
              v-for="(value, key) in selectedAlert.context"
              :key="key"
              :label="formatContextKey(key)"
            >
              {{ value }}
            </el-descriptions-item>
          </el-descriptions>
        </div>

        <div v-if="selectedAlert.status !== 'resolved'" class="alert-actions">
          <el-divider />
          <div class="action-buttons">
            <el-button
              v-if="selectedAlert.status === 'active'"
              type="warning"
              :icon="Check"
              @click="handleAcknowledge(selectedAlert)"
            >
              确认告警
            </el-button>

            <el-button
              type="success"
              :icon="CircleCheck"
              @click="handleResolve(selectedAlert)"
            >
              解决告警
            </el-button>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 手动告警对话框 -->
    <el-dialog
      v-model="showCreateDialog"
      title="创建手动告警"
      width="600px"
      :destroy-on-close="true"
    >
      <el-form
        ref="alertFormRef"
        :model="newAlert"
        :rules="alertRules"
        label-width="120px"
      >
        <el-form-item label="告警标题" prop="title">
          <el-input v-model="newAlert.title" placeholder="请输入告警标题" />
        </el-form-item>

        <el-form-item label="严重等级" prop="severity">
          <el-select v-model="newAlert.severity" placeholder="请选择严重等级">
            <el-option label="紧急" value="critical" />
            <el-option label="警告" value="warning" />
            <el-option label="信息" value="info" />
          </el-select>
        </el-form-item>

        <el-form-item label="告警来源" prop="source">
          <el-select v-model="newAlert.source" placeholder="请选择告警来源">
            <el-option label="驱动" value="driver" />
            <el-option label="连接" value="connection" />
            <el-option label="数据点" value="datapoint" />
            <el-option label="系统" value="system" />
          </el-select>
        </el-form-item>

        <el-form-item label="详细消息" prop="message">
          <el-input
            v-model="newAlert.message"
            type="textarea"
            :rows="4"
            placeholder="请输入详细的告警消息"
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button
          type="primary"
          :loading="submitting"
          @click="handleSubmitAlert"
        >
          创建告警
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import {
  Setting,
  Clock,
  Plus,
  WarningFilled,
  Warning,
  InfoFilled,
  CircleCheck,
  Search,
  Refresh,
  View,
  Check,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, reactive, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'

import { alertsApi } from '@/api'

// 类型定义
interface Alert {
  id: string
  title: string
  message: string
  severity: 'critical' | 'warning' | 'info'
  status: 'active' | 'acknowledged' | 'resolved'
  source: 'driver' | 'connection' | 'datapoint' | 'system'
  count: number
  timestamp: Date
  acknowledgedBy?: string
  acknowledgedAt?: Date
  resolvedBy?: string
  resolvedAt?: Date
  isNew?: boolean
  context?: Record<string, any>
}

interface AlertStats {
  critical: number
  warning: number
  info: number
  acknowledged: number
  total: number
}

interface NewAlert {
  title: string
  message: string
  severity: 'critical' | 'warning' | 'info'
  source: 'driver' | 'connection' | 'datapoint' | 'system'
}

// 路由
const router = useRouter()

// 状态管理
const loading = ref(false)
const submitting = ref(false)
const showDetailDialog = ref(false)
const showCreateDialog = ref(false)
const selectedAlert = ref<Alert | null>(null)
const selectedAlerts = ref<Alert[]>([])
const alertFormRef = ref()

// 搜索和筛选
const searchQuery = ref('')
const severityFilter = ref('')
const statusFilter = ref('')
const sourceFilter = ref('')
const dateRange = ref<[string, string] | null>(null)

// 分页
const currentPage = ref(1)
const pageSize = ref(20)
const totalAlerts = ref(0)

// 告警数据
const alerts = ref<Alert[]>([])
const alertStats = ref<AlertStats>({
  critical: 0,
  warning: 0,
  info: 0,
  acknowledged: 0,
  total: 0,
})

// 新告警表单
const newAlert = reactive<NewAlert>({
  title: '',
  message: '',
  severity: 'warning',
  source: 'system',
})

// 表单验证规则
const alertRules = {
  title: [
    { required: true, message: '请输入告警标题', trigger: 'blur' },
    { min: 5, max: 100, message: '标题长度应在5-100个字符', trigger: 'blur' },
  ],
  severity: [{ required: true, message: '请选择严重等级', trigger: 'change' }],
  source: [{ required: true, message: '请选择告警来源', trigger: 'change' }],
  message: [
    { required: true, message: '请输入详细消息', trigger: 'blur' },
    { min: 10, max: 500, message: '消息长度应在10-500个字符', trigger: 'blur' },
  ],
}

// 计算属性 - 现在分页在后端处理，这里直接返回alerts
const filteredAlerts = computed(() => {
  return alerts.value
})

// 方法
const getSeverityIcon = (severity: string) => {
  const iconMap = {
    critical: 'WarningFilled',
    warning: 'Warning',
    info: 'InfoFilled',
  }
  return iconMap[severity as keyof typeof iconMap] || 'InfoFilled'
}

const getSeverityClass = (severity: string) => {
  return `severity-icon ${severity}`
}

const getSeverityTagType = (severity: string) => {
  const typeMap = {
    critical: 'danger',
    warning: 'warning',
    info: 'info',
  }
  return typeMap[severity as keyof typeof typeMap] || 'info'
}

const getStatusTagType = (status: string) => {
  const typeMap = {
    active: 'danger',
    acknowledged: 'warning',
    resolved: 'success',
  }
  return typeMap[status as keyof typeof typeMap] || 'info'
}

const getSourceTagType = (source: string) => {
  const typeMap = {
    driver: 'primary',
    connection: 'success',
    datapoint: 'warning',
    system: 'info',
  }
  return typeMap[source as keyof typeof typeMap] || 'info'
}

const formatSeverity = (severity: string) => {
  const textMap = {
    critical: '紧急',
    warning: '警告',
    info: '信息',
  }
  return textMap[severity as keyof typeof textMap] || severity
}

const formatStatus = (status: string) => {
  const textMap = {
    active: '活跃',
    acknowledged: '已确认',
    resolved: '已解决',
  }
  return textMap[status as keyof typeof textMap] || status
}

const formatSource = (source: string) => {
  const textMap = {
    driver: '驱动',
    connection: '连接',
    datapoint: '数据点',
    system: '系统',
  }
  return textMap[source as keyof typeof textMap] || source
}

const formatTime = (time: Date) => {
  return time.toLocaleString('zh-CN')
}

const getTimeDuration = (time: Date) => {
  const now = new Date()
  const diff = now.getTime() - time.getTime()
  const minutes = Math.floor(diff / (1000 * 60))
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (days > 0) {
    return `${days}天前`
  } else if (hours > 0) {
    return `${hours}小时前`
  } else if (minutes > 0) {
    return `${minutes}分钟前`
  } else {
    return '刚刚'
  }
}

const getAlertDuration = (alert: Alert) => {
  const start = alert.timestamp
  const end = alert.resolvedAt || new Date()
  const diff = end.getTime() - start.getTime()
  const minutes = Math.floor(diff / (1000 * 60))
  const hours = Math.floor(minutes / 60)

  if (hours > 0) {
    return `${hours}小时${minutes % 60}分钟`
  } else {
    return `${minutes}分钟`
  }
}

const formatContextKey = (key: string) => {
  const keyMap: Record<string, string> = {
    driverName: '驱动名称',
    connectionName: '连接名称',
    datapointName: '数据点名称',
    value: '数据值',
    threshold: '阈值',
    deviceAddress: '设备地址',
    errorCode: '错误代码',
  }
  return keyMap[key] || key
}

// 事件处理
const handleSearch = async () => {
  currentPage.value = 1
  await loadAlerts()
}

const handleDateRangeChange = async () => {
  currentPage.value = 1
  await loadAlerts()
}

const handleRefresh = async () => {
  await loadAlerts()
  ElMessage.success('刷新成功')
}

const handleResetFilters = () => {
  searchQuery.value = ''
  severityFilter.value = ''
  statusFilter.value = ''
  sourceFilter.value = ''
  dateRange.value = null
  currentPage.value = 1
}

const handleSelectionChange = (selection: Alert[]) => {
  selectedAlerts.value = selection
}

const handleSizeChange = async (size: number) => {
  pageSize.value = size
  currentPage.value = 1
  await loadAlerts()
}

const handleCurrentChange = async (page: number) => {
  currentPage.value = page
  await loadAlerts()
}

const handleViewDetail = (alert: Alert) => {
  selectedAlert.value = alert
  showDetailDialog.value = true

  // 标记为已读
  if (alert.isNew) {
    alert.isNew = false
  }
}

const handleAcknowledge = async (alert: Alert) => {
  try {
    await alertsApi.acknowledge(alert.id)
    await loadAlerts()
    ElMessage.success('告警已确认')

    if (showDetailDialog.value) {
      showDetailDialog.value = false
    }
  } catch (error) {
    ElMessage.error('确认失败')
  }
}

const handleResolve = async (alert: Alert) => {
  try {
    await ElMessageBox.confirm('确定要解决这个告警吗？', '确认解决', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })

    await alertsApi.resolve(alert.id)
    await loadAlerts()
    ElMessage.success('告警已解决')

    if (showDetailDialog.value) {
      showDetailDialog.value = false
    }
  } catch (error) {
    // 取消操作
  }
}

const handleBulkAcknowledge = async () => {
  try {
    await ElMessageBox.confirm(
      `确定要批量确认 ${selectedAlerts.value.length} 个告警吗？`,
      '批量确认',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    const ids = selectedAlerts.value.map(alert => alert.id)
    await alertsApi.batchAcknowledge(ids)
    await loadAlerts()

    ElMessage.success(`已批量确认 ${selectedAlerts.value.length} 个告警`)
    selectedAlerts.value = []
  } catch (error) {
    ElMessage.error('批量确认失败')
  }
}

const handleBulkResolve = async () => {
  try {
    await ElMessageBox.confirm(
      `确定要批量解决 ${selectedAlerts.value.length} 个告警吗？`,
      '批量解决',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    const ids = selectedAlerts.value.map(alert => alert.id)
    await alertsApi.batchResolve(ids)
    await loadAlerts()

    ElMessage.success(`已批量解决 ${selectedAlerts.value.length} 个告警`)
    selectedAlerts.value = []
  } catch (error) {
    ElMessage.error('批量解决失败')
  }
}

const handleAlertRules = () => {
  router.push('/alerts/rules')
}

const handleAlertHistory = () => {
  router.push('/alerts/history')
}

const handleCreateAlert = () => {
  // 重置表单
  Object.assign(newAlert, {
    title: '',
    message: '',
    severity: 'warning',
    source: 'system',
  })
  showCreateDialog.value = true
}

const handleSubmitAlert = async () => {
  try {
    await alertFormRef.value?.validate()

    submitting.value = true
    await alertsApi.create({
      title: newAlert.title,
      message: newAlert.message,
      severity: newAlert.severity,
      source: newAlert.source,
    })

    await loadAlerts()
    ElMessage.success('手动告警创建成功')
    showCreateDialog.value = false
  } catch (error) {
    ElMessage.error('创建告警失败')
  } finally {
    submitting.value = false
  }
}

// 数据加载函数
const loadAlerts = async () => {
  try {
    loading.value = true
    const response = await alertsApi.list({
      page: currentPage.value,
      size: pageSize.value,
      severity: severityFilter.value || undefined,
      status: statusFilter.value || undefined,
      source: sourceFilter.value || undefined,
      startTime: dateRange.value?.[0] || undefined,
      endTime: dateRange.value?.[1] || undefined,
      keyword: searchQuery.value || undefined,
    })

    if (response.success && response.data) {
      alerts.value = response.data.items || []
      totalAlerts.value = response.data.total || 0
    }
  } catch (error) {
    console.error('Failed to load alerts:', error)
    ElMessage.error('加载告警列表失败')
  } finally {
    loading.value = false
  }
}

const loadAlertStats = async () => {
  try {
    const stats = await alertsApi.getStats()
    alertStats.value = stats
  } catch (error) {
    console.error('Failed to load alert stats:', error)
  }
}

// WebSocket实时更新
let wsConnection: WebSocket | null = null

const startRealtimeUpdate = () => {
  const wsUrl = `${import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080'}/ws/alerts`

  try {
    wsConnection = new WebSocket(wsUrl)

    wsConnection.onopen = () => {
      console.log('Alerts WebSocket connected')
    }

    wsConnection.onmessage = event => {
      try {
        const data = JSON.parse(event.data)
        if (data.type === 'new_alert') {
          // 添加新告警到列表顶部
          alerts.value.unshift(data.alert)
          loadAlertStats()
        } else if (data.type === 'alert_updated') {
          // 更新现有告警
          const index = alerts.value.findIndex(
            alert => alert.id === data.alert.id
          )
          if (index > -1) {
            alerts.value[index] = data.alert
            loadAlertStats()
          }
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
      }
    }

    wsConnection.onerror = error => {
      console.error('Alerts WebSocket error:', error)
    }

    wsConnection.onclose = () => {
      console.log('Alerts WebSocket disconnected')
      // 5秒后重连
      setTimeout(() => {
        if (!wsConnection || wsConnection.readyState === WebSocket.CLOSED) {
          startRealtimeUpdate()
        }
      }, 5000)
    }
  } catch (error) {
    console.error('Failed to establish WebSocket connection:', error)
  }
}

// 监听筛选器变化
watch([severityFilter, statusFilter, sourceFilter], async () => {
  currentPage.value = 1
  await loadAlerts()
})

// 生命周期
onMounted(async () => {
  await Promise.all([loadAlerts(), loadAlertStats()])
  startRealtimeUpdate()
})

onUnmounted(() => {
  if (wsConnection) {
    wsConnection.close()
    wsConnection = null
  }
})
</script>

<style scoped lang="scss">
.alerts-list-page {
  padding: 20px;
  max-width: 1800px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;

  .header-left {
    h1 {
      margin: 0 0 8px 0;
      font-size: 28px;
      color: var(--el-text-color-primary);
    }

    p {
      margin: 0;
      color: var(--el-text-color-secondary);
      font-size: 14px;
    }
  }

  .header-actions {
    display: flex;
    gap: 12px;
  }
}

.alert-overview {
  margin-bottom: 24px;

  .overview-card {
    background: white;
    border-radius: 12px;
    padding: 24px;
    display: flex;
    align-items: center;
    gap: 16px;
    border: 1px solid var(--el-border-color-light);
    transition: all 0.3s ease;

    &:hover {
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
      transform: translateY(-2px);
    }

    .card-icon {
      width: 60px;
      height: 60px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;

      .el-icon {
        font-size: 28px;
        color: white;
      }
    }

    .card-content {
      flex: 1;

      .card-value {
        font-size: 32px;
        font-weight: 700;
        margin-bottom: 4px;
        color: var(--el-text-color-primary);
      }

      .card-label {
        font-size: 14px;
        color: var(--el-text-color-secondary);
      }
    }

    &.critical {
      .card-icon {
        background: var(--el-color-danger);
      }
      .card-value {
        color: var(--el-color-danger);
      }
    }

    &.warning {
      .card-icon {
        background: var(--el-color-warning);
      }
      .card-value {
        color: var(--el-color-warning);
      }
    }

    &.info {
      .card-icon {
        background: var(--el-color-info);
      }
      .card-value {
        color: var(--el-color-info);
      }
    }

    &.acknowledged {
      .card-icon {
        background: var(--el-color-success);
      }
      .card-value {
        color: var(--el-color-success);
      }
    }
  }
}

.search-filters {
  background: white;
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
  border: 1px solid var(--el-border-color-light);

  .filter-actions {
    margin-top: 16px;
    display: flex;
    gap: 12px;
  }
}

.alerts-table {
  background: white;
  border-radius: 12px;
  padding: 20px;
  border: 1px solid var(--el-border-color-light);

  .severity-icon {
    &.critical {
      color: var(--el-color-danger);
    }
    &.warning {
      color: var(--el-color-warning);
    }
    &.info {
      color: var(--el-color-info);
    }
  }

  .alert-title {
    display: flex;
    align-items: center;
    gap: 8px;

    .title-text {
      font-weight: 500;
    }
  }

  .alert-message {
    line-height: 1.4;
  }

  .time-info {
    .timestamp {
      font-size: 13px;
      color: var(--el-text-color-primary);
    }

    .duration {
      font-size: 12px;
      color: var(--el-text-color-secondary);
      margin-top: 2px;
    }
  }

  .acknowledged-by {
    font-size: 13px;
    color: var(--el-color-success);
  }

  .not-acknowledged {
    color: var(--el-text-color-placeholder);
  }

  .count-display {
    font-weight: 500;
  }

  .action-buttons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .table-pagination {
    margin-top: 20px;
    display: flex;
    justify-content: center;
  }
}

.alert-detail {
  .alert-message-detail {
    margin-top: 20px;

    h4 {
      margin: 0 0 12px 0;
      font-size: 16px;
      color: var(--el-text-color-primary);
    }

    .message-content {
      background: var(--el-fill-color-light);
      padding: 16px;
      border-radius: 6px;
      line-height: 1.6;
      color: var(--el-text-color-primary);
    }
  }

  .alert-context {
    margin-top: 20px;

    h4 {
      margin: 0 0 12px 0;
      font-size: 16px;
      color: var(--el-text-color-primary);
    }
  }

  .alert-actions {
    .action-buttons {
      display: flex;
      gap: 12px;
      justify-content: flex-end;
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .alert-overview {
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

  .search-filters {
    .el-row {
      flex-direction: column;

      .el-col {
        width: 100% !important;
        margin-bottom: 12px;
      }
    }
  }

  .alerts-table {
    :deep(.el-table) {
      font-size: 12px;
    }

    .action-buttons {
      flex-direction: column;

      .el-button {
        font-size: 12px;
      }
    }
  }
}
</style>
