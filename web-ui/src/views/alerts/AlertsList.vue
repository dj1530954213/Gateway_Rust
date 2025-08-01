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
        <el-button :icon="Refresh" @click="handleRefresh">
          刷新
        </el-button>
        <el-button @click="handleResetFilters">
          重置筛选
        </el-button>
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
        
        <el-table-column prop="message" label="详细消息" show-overflow-tooltip min-width="250">
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
            <el-badge :value="row.count" :max="99" v-if="row.count > 1">
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
                {{ selectedAlert.acknowledgedAt ? formatTime(selectedAlert.acknowledgedAt) : '-' }}
              </el-descriptions-item>
              <el-descriptions-item label="解决人">
                {{ selectedAlert.resolvedBy || '未解决' }}
              </el-descriptions-item>
              <el-descriptions-item label="解决时间">
                {{ selectedAlert.resolvedAt ? formatTime(selectedAlert.resolvedAt) : '-' }}
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
        
        <div class="alert-context" v-if="selectedAlert.context">
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
        
        <div class="alert-actions" v-if="selectedAlert.status !== 'resolved'">
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
      <el-form :model="newAlert" :rules="alertRules" ref="alertFormRef" label-width="120px">
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
        <el-button type="primary" @click="handleSubmitAlert" :loading="submitting">
          创建告警
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
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
  Check
} from '@element-plus/icons-vue'

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
  total: 0
})

// 新告警表单
const newAlert = reactive<NewAlert>({
  title: '',
  message: '',
  severity: 'warning',
  source: 'system'
})

// 表单验证规则
const alertRules = {
  title: [
    { required: true, message: '请输入告警标题', trigger: 'blur' },
    { min: 5, max: 100, message: '标题长度应在5-100个字符', trigger: 'blur' }
  ],
  severity: [
    { required: true, message: '请选择严重等级', trigger: 'change' }
  ],
  source: [
    { required: true, message: '请选择告警来源', trigger: 'change' }
  ],
  message: [
    { required: true, message: '请输入详细消息', trigger: 'blur' },
    { min: 10, max: 500, message: '消息长度应在10-500个字符', trigger: 'blur' }
  ]
}

// 计算属性
const filteredAlerts = computed(() => {
  let filtered = alerts.value
  
  // 搜索过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(alert => 
      alert.title.toLowerCase().includes(query) ||
      alert.message.toLowerCase().includes(query)
    )
  }
  
  // 严重等级过滤
  if (severityFilter.value) {
    filtered = filtered.filter(alert => alert.severity === severityFilter.value)
  }
  
  // 状态过滤
  if (statusFilter.value) {
    filtered = filtered.filter(alert => alert.status === statusFilter.value)
  }
  
  // 来源过滤
  if (sourceFilter.value) {
    filtered = filtered.filter(alert => alert.source === sourceFilter.value)
  }
  
  // 时间过滤
  if (dateRange.value && dateRange.value[0] && dateRange.value[1]) {
    const start = new Date(dateRange.value[0])
    const end = new Date(dateRange.value[1])
    filtered = filtered.filter(alert => 
      alert.timestamp >= start && alert.timestamp <= end
    )
  }
  
  totalAlerts.value = filtered.length
  
  // 分页
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filtered.slice(start, end)
})

// 方法
const getSeverityIcon = (severity: string) => {
  const iconMap = {
    critical: 'WarningFilled',
    warning: 'Warning',
    info: 'InfoFilled'
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
    info: 'info'
  }
  return typeMap[severity as keyof typeof typeMap] || 'info'
}

const getStatusTagType = (status: string) => {
  const typeMap = {
    active: 'danger',
    acknowledged: 'warning',
    resolved: 'success'
  }
  return typeMap[status as keyof typeof typeMap] || 'info'
}

const getSourceTagType = (source: string) => {
  const typeMap = {
    driver: 'primary',
    connection: 'success',
    datapoint: 'warning',
    system: 'info'
  }
  return typeMap[source as keyof typeof typeMap] || 'info'
}

const formatSeverity = (severity: string) => {
  const textMap = {
    critical: '紧急',
    warning: '警告',
    info: '信息'
  }
  return textMap[severity as keyof typeof textMap] || severity
}

const formatStatus = (status: string) => {
  const textMap = {
    active: '活跃',
    acknowledged: '已确认',
    resolved: '已解决'
  }
  return textMap[status as keyof typeof textMap] || status
}

const formatSource = (source: string) => {
  const textMap = {
    driver: '驱动',
    connection: '连接',
    datapoint: '数据点',
    system: '系统'
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
    errorCode: '错误代码'
  }
  return keyMap[key] || key
}

// 事件处理
const handleSearch = () => {
  currentPage.value = 1
}

const handleDateRangeChange = () => {
  currentPage.value = 1
}

const handleRefresh = async () => {
  loading.value = true
  try {
    await generateMockAlerts()
    updateAlertStats()
    ElMessage.success('刷新成功')
  } finally {
    loading.value = false
  }
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

const handleSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
}

const handleCurrentChange = (page: number) => {
  currentPage.value = page
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
    await new Promise(resolve => setTimeout(resolve, 500))
    alert.status = 'acknowledged'
    alert.acknowledgedBy = '当前用户'
    alert.acknowledgedAt = new Date()
    
    updateAlertStats()
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
    await ElMessageBox.confirm(
      '确定要解决这个告警吗？',
      '确认解决',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    await new Promise(resolve => setTimeout(resolve, 500))
    alert.status = 'resolved'
    alert.resolvedBy = '当前用户'
    alert.resolvedAt = new Date()
    
    updateAlertStats()
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
        type: 'warning'
      }
    )
    
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    selectedAlerts.value.forEach(alert => {
      if (alert.status === 'active') {
        alert.status = 'acknowledged'
        alert.acknowledgedBy = '当前用户'
        alert.acknowledgedAt = new Date()
      }
    })
    
    updateAlertStats()
    ElMessage.success(`已批量确认 ${selectedAlerts.value.length} 个告警`)
    selectedAlerts.value = []
  } catch (error) {
    // 取消操作
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
        type: 'warning'
      }
    )
    
    await new Promise(resolve => setTimeout(resolve, 1200))
    
    selectedAlerts.value.forEach(alert => {
      if (alert.status !== 'resolved') {
        alert.status = 'resolved'
        alert.resolvedBy = '当前用户'
        alert.resolvedAt = new Date()
      }
    })
    
    updateAlertStats()
    ElMessage.success(`已批量解决 ${selectedAlerts.value.length} 个告警`)
    selectedAlerts.value = []
  } catch (error) {
    // 取消操作
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
    source: 'system'
  })
  showCreateDialog.value = true
}

const handleSubmitAlert = async () => {
  try {
    await alertFormRef.value?.validate()
    
    submitting.value = true
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // 创建新告警
    const alert: Alert = {
      id: Date.now().toString(),
      title: newAlert.title,
      message: newAlert.message,
      severity: newAlert.severity,
      status: 'active',
      source: newAlert.source,
      count: 1,
      timestamp: new Date(),
      isNew: true
    }
    
    alerts.value.unshift(alert)
    updateAlertStats()
    
    ElMessage.success('手动告警创建成功')
    showCreateDialog.value = false
  } catch (error) {
    ElMessage.error('表单填写不完整')
  } finally {
    submitting.value = false
  }
}

const updateAlertStats = () => {
  const stats = {
    critical: 0,
    warning: 0,
    info: 0,
    acknowledged: 0,
    total: alerts.value.length
  }
  
  alerts.value.forEach(alert => {
    stats[alert.severity]++
    if (alert.status === 'acknowledged') {
      stats.acknowledged++
    }
  })
  
  alertStats.value = stats
}

// 数据初始化
const generateMockAlerts = async () => {
  const severities = ['critical', 'warning', 'info'] as const
  const sources = ['driver', 'connection', 'datapoint', 'system'] as const
  const statuses = ['active', 'acknowledged', 'resolved'] as const
  
  const alertTitles = {
    critical: [
      '驱动通信中断',
      '设备连接失败',
      '系统内存不足',
      '硬盘空间告急',
      '数据库连接异常'
    ],
    warning: [
      '数据采集延迟',
      '网络延迟较高',
      'CPU使用率过高',
      '数据点质量异常',
      '驱动重连频繁'
    ],
    info: [
      '系统启动完成',
      '驱动配置更新',
      '定时任务执行',
      '数据备份完成',
      '用户登录成功'
    ]
  }
  
  const alertMessages = {
    critical: [
      '驱动与设备的通信已中断，无法获取数据。请检查网络连接和设备状态。',
      '设备连接失败，无法建立通信链路。可能原因：网络故障、设备断电或配置错误。',
      '系统内存使用率已超过98%，可能影响系统正常运行。建议立即释放内存或重启系统。',
      '硬盘空间使用率已超过95%，系统即将无法写入新数据。请立即清理不必要的文件。',
      '数据库连接中断，数据存储功能受影响。请检查数据库服务器状态和网络连接。'
    ],
    warning: [
      '数据采集延迟超过预设阈值，当前延迟为2.5秒。可能影响实时性。',
      '网络延迟较高，平均响应时间为150ms。建议检查网络环境。',
      'CPU使用率持续处于85%以上，可能影响系统性能。建议优化进程或增加资源。',
      '数据点值质量状态为“不确定”，可能影响数据可靠性。',
      '驱动在过10分钟内发生了3次重连，建议检查连接稳定性。'
    ],
    info: [
      '系统已成功启动，所有模块初始化完成。当前版本：v2.1.0',
      '驱动配置已更新，扫描间隔调整为1000ms。配置立即生效。',
      '定时任务已执行：数据备份任务在凌晨2:00执行完成。',
      '数据备份操作已成功完成，共备份125MB数据。备份文件存储于/backup/目录。',
      '用户“admin”于09:30成功登录系统，IP地址：192.168.1.100。'
    ]
  }
  
  alerts.value = Array.from({ length: 150 }, (_, i) => {
    const severity = severities[Math.floor(Math.random() * severities.length)]
    const source = sources[Math.floor(Math.random() * sources.length)]
    const status = statuses[Math.floor(Math.random() * statuses.length)]
    
    const titles = alertTitles[severity]
    const messages = alertMessages[severity]
    
    const title = titles[Math.floor(Math.random() * titles.length)]
    const message = messages[Math.floor(Math.random() * messages.length)]
    
    const timestamp = new Date(Date.now() - Math.random() * 7 * 24 * 60 * 60 * 1000) // Last 7 days
    
    const alert: Alert = {
      id: (i + 1).toString(),
      title,
      message,
      severity,
      status,
      source,
      count: Math.floor(Math.random() * 10) + 1,
      timestamp,
      isNew: Math.random() > 0.8 && status === 'active'
    }
    
    // 添加确认信息
    if (status === 'acknowledged' || status === 'resolved') {
      alert.acknowledgedBy = '管理员'
      alert.acknowledgedAt = new Date(timestamp.getTime() + Math.random() * 60 * 60 * 1000)
    }
    
    // 添加解决信息
    if (status === 'resolved') {
      alert.resolvedBy = '管理员'
      alert.resolvedAt = new Date(timestamp.getTime() + Math.random() * 2 * 60 * 60 * 1000)
    }
    
    // 添加上下文信息
    if (source === 'driver') {
      alert.context = {
        driverName: '主控制器驱动',
        deviceAddress: '192.168.1.100:502'
      }
    } else if (source === 'datapoint') {
      alert.context = {
        datapointName: `数据点${Math.floor(Math.random() * 100)}`,
        value: (Math.random() * 1000).toFixed(2),
        threshold: '500.00'
      }
    }
    
    return alert
  })
  
  // 按时间排序，最新的在前
  alerts.value.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
}

// 定时刷新
let refreshTimer: NodeJS.Timeout

const startAutoRefresh = () => {
  refreshTimer = setInterval(() => {
    // 模拟新告警
    if (Math.random() > 0.7) {
      const severities = ['critical', 'warning', 'info'] as const
      const sources = ['driver', 'connection', 'datapoint', 'system'] as const
      
      const severity = severities[Math.floor(Math.random() * severities.length)]
      const source = sources[Math.floor(Math.random() * sources.length)]
      
      const newAlert: Alert = {
        id: Date.now().toString(),
        title: '实时告警测试',
        message: '这是一个模拟的实时告警消息。',
        severity,
        status: 'active',
        source,
        count: 1,
        timestamp: new Date(),
        isNew: true
      }
      
      alerts.value.unshift(newAlert)
      updateAlertStats()
      
      // 只保留最近的200条记录
      if (alerts.value.length > 200) {
        alerts.value = alerts.value.slice(0, 200)
      }
    }
    
    // 更新统计数据
    updateAlertStats()
  }, 30000) // Every 30 seconds
}

// 生命周期
onMounted(async () => {
  loading.value = true
  try {
    await generateMockAlerts()
    updateAlertStats()
    startAutoRefresh()
  } finally {
    loading.value = false
  }
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
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