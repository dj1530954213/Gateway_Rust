<template>
  <div class="alert-history">
    <!-- 页面标题和工具栏 -->
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">
          <el-icon><Clock /></el-icon>
          告警历史
        </h1>
        <span class="page-description">查看和分析历史告警记录</span>
      </div>
      <div class="header-actions">
        <el-button type="primary" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          刷新数据
        </el-button>
        <el-button @click="exportHistory">
          <el-icon><Download /></el-icon>
          导出记录
        </el-button>
        <el-button @click="showSettings = true">
          <el-icon><Setting /></el-icon>
          显示设置
        </el-button>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-cards">
      <el-row :gutter="20">
        <el-col :span="6">
          <StatCard
            title="今日告警"
            :value="stats.todayAlerts"
            :trend="stats.todayTrend"
            trend-label="较昨日"
            color="#E74C3C"
            icon="Warning"
          />
        </el-col>
        <el-col :span="6">
          <StatCard
            title="本周告警"
            :value="stats.weekAlerts"
            :trend="stats.weekTrend"
            trend-label="较上周"
            color="#F39C12"
            icon="Bell"
          />
        </el-col>
        <el-col :span="6">
          <StatCard
            title="平均响应时间"
            :value="stats.avgResponseTime"
            suffix="秒"
            :trend="stats.responseTrend"
            trend-label="较上周"
            color="#3498DB"
            icon="Timer"
          />
        </el-col>
        <el-col :span="6">
          <StatCard
            title="处理率"
            :value="stats.handleRate"
            suffix="%"
            :trend="stats.handleTrend"
            trend-label="较上周"
            color="#27AE60"
            icon="Check"
          />
        </el-col>
      </el-row>
    </div>

    <!-- 筛选器 -->
    <div class="filters-container">
      <el-card shadow="never">
        <div class="filters">
          <el-row :gutter="20">
            <el-col :span="6">
              <el-input
                v-model="filters.search"
                placeholder="搜索告警内容..."
                clearable
                @input="handleSearch"
              >
                <template #prefix>
                  <el-icon><Search /></el-icon>
                </template>
              </el-input>
            </el-col>
            <el-col :span="4">
              <el-select
                v-model="filters.severity"
                placeholder="严重级别"
                clearable
                @change="handleFilter"
              >
                <el-option label="全部" value="" />
                <el-option label="紧急" value="critical" />
                <el-option label="严重" value="major" />
                <el-option label="警告" value="warning" />
                <el-option label="信息" value="info" />
              </el-select>
            </el-col>
            <el-col :span="4">
              <el-select
                v-model="filters.status"
                placeholder="处理状态"
                clearable
                @change="handleFilter"
              >
                <el-option label="全部" value="" />
                <el-option label="未处理" value="unhandled" />
                <el-option label="处理中" value="handling" />
                <el-option label="已处理" value="handled" />
                <el-option label="已忽略" value="ignored" />
              </el-select>
            </el-col>
            <el-col :span="4">
              <el-select
                v-model="filters.type"
                placeholder="告警类型"
                clearable
                @change="handleFilter"
              >
                <el-option label="全部" value="" />
                <el-option label="数据点" value="datapoint" />
                <el-option label="驱动" value="driver" />
                <el-option label="连接" value="connection" />
                <el-option label="系统" value="system" />
              </el-select>
            </el-col>
            <el-col :span="6">
              <el-date-picker
                v-model="filters.dateRange"
                type="datetimerange"
                range-separator="至"
                start-placeholder="开始时间"
                end-placeholder="结束时间"
                format="YYYY-MM-DD HH:mm:ss"
                value-format="YYYY-MM-DD HH:mm:ss"
                @change="handleFilter"
              />
            </el-col>
          </el-row>
          <el-row :gutter="20" style="margin-top: 15px;">
            <el-col :span="6">
              <el-select
                v-model="filters.source"
                placeholder="告警源"
                clearable
                @change="handleFilter"
              >
                <el-option label="全部" value="" />
                <el-option
                  v-for="source in alertSources"
                  :key="source.id"
                  :label="source.name"
                  :value="source.id"
                />
              </el-select>
            </el-col>
            <el-col :span="4">
              <el-select
                v-model="filters.acknowledged"
                placeholder="确认状态"
                clearable
                @change="handleFilter"
              >
                <el-option label="全部" value="" />
                <el-option label="已确认" value="true" />
                <el-option label="未确认" value="false" />
              </el-select>
            </el-col>
            <el-col :span="4">
              <el-select
                v-model="filters.priority"
                placeholder="优先级"
                clearable
                @change="handleFilter"
              >
                <el-option label="全部" value="" />
                <el-option label="高优先级" value="high" />
                <el-option label="中优先级" value="medium" />
                <el-option label="低优先级" value="low" />
              </el-select>
            </el-col>
            <el-col :span="6">
              <el-input
                v-model="filters.operator"
                placeholder="处理人员"
                clearable
                @input="handleSearch"
              >
                <template #prefix>
                  <el-icon><User /></el-icon>
                </template>
              </el-input>
            </el-col>
            <el-col :span="4">
              <el-button @click="resetFilters">重置筛选</el-button>
            </el-col>
          </el-row>
        </div>
      </el-card>
    </div>

    <!-- 告警历史表格 -->
    <div class="table-container">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <span>告警历史记录 ({{ pagination.total }}条)</span>
            <div class="header-actions">
              <el-button-group>
                <el-button
                  :type="viewMode === 'table' ? 'primary' : ''"
                  @click="viewMode = 'table'"
                >
                  <el-icon><List /></el-icon>
                  表格视图
                </el-button>
                <el-button
                  :type="viewMode === 'timeline' ? 'primary' : ''"
                  @click="viewMode = 'timeline'"
                >
                  <el-icon><Clock /></el-icon>
                  时间线视图
                </el-button>
              </el-button-group>
            </div>
          </div>
        </template>

        <!-- 表格视图 -->
        <div v-if="viewMode === 'table'">
          <el-table
            v-loading="loading"
            :data="alertHistory"
            style="width: 100%"
            @selection-change="handleSelectionChange"
            @sort-change="handleSortChange"
          >
            <el-table-column type="selection" width="55" />
            <el-table-column
              prop="id"
              label="告警ID"
              width="100"
              sortable="custom"
            />
            <el-table-column
              prop="severity"
              label="严重级别"
              width="100"
              sortable="custom"
            >
              <template #default="{ row }">
                <AlertSeverity :severity="row.severity" />
              </template>
            </el-table-column>
            <el-table-column
              prop="type"
              label="类型"
              width="100"
            >
              <template #default="{ row }">
                <el-tag :type="getTypeTagType(row.type)" size="small">
                  {{ getTypeText(row.type) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column
              prop="title"
              label="告警标题"
              min-width="200"
              show-overflow-tooltip
            />
            <el-table-column
              prop="source"
              label="告警源"
              width="150"
              show-overflow-tooltip
            />
            <el-table-column
              prop="status"
              label="状态"
              width="100"
            >
              <template #default="{ row }">
                <AlertStatus :status="row.status" />
              </template>
            </el-table-column>
            <el-table-column
              prop="priority"
              label="优先级"
              width="100"
            >
              <template #default="{ row }">
                <el-tag
                  :type="getPriorityTagType(row.priority)"
                  size="small"
                >
                  {{ getPriorityText(row.priority) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column
              prop="triggered_at"
              label="触发时间"
              width="160"
              sortable="custom"
            >
              <template #default="{ row }">
                <div class="time-info">
                  <div>{{ formatDateTime(row.triggered_at) }}</div>
                  <div class="time-ago">{{ getTimeAgo(row.triggered_at) }}</div>
                </div>
              </template>
            </el-table-column>
            <el-table-column
              prop="acknowledged"
              label="确认"
              width="80"
              align="center"
            >
              <template #default="{ row }">
                <el-icon
                  :class="row.acknowledged ? 'acknowledged' : 'unacknowledged'"
                  :color="row.acknowledged ? '#67C23A' : '#E6A23C'"
                >
                  <Check v-if="row.acknowledged" />
                  <Warning v-else />
                </el-icon>
              </template>
            </el-table-column>
            <el-table-column
              prop="duration"
              label="持续时间"
              width="120"
            >
              <template #default="{ row }">
                {{ formatDuration(row.duration) }}
              </template>
            </el-table-column>
            <el-table-column
              label="操作"
              width="150"
              fixed="right"
            >
              <template #default="{ row }">
                <el-button
                  type="primary"
                  size="small"
                  @click="showAlertDetail(row)"
                >
                  详情
                </el-button>
                <el-button
                  v-if="!row.acknowledged"
                  type="warning"
                  size="small"
                  @click="acknowledgeAlert(row.id)"
                >
                  确认
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <!-- 时间线视图 -->
        <div v-else-if="viewMode === 'timeline'" class="timeline-view">
          <el-timeline>
            <el-timeline-item
              v-for="alert in alertHistory"
              :key="alert.id"
              :timestamp="formatDateTime(alert.triggered_at)"
              :type="getTimelineType(alert.severity)"
              :size="alert.severity === 'critical' ? 'large' : 'normal'"
              placement="top"
            >
              <el-card class="timeline-card" :class="getSeverityClass(alert.severity)">
                <div class="timeline-header">
                  <div class="alert-info">
                    <AlertSeverity :severity="alert.severity" />
                    <el-tag :type="getTypeTagType(alert.type)" size="small" style="margin-left: 8px;">
                      {{ getTypeText(alert.type) }}
                    </el-tag>
                    <span class="alert-id">#{{ alert.id }}</span>
                  </div>
                  <AlertStatus :status="alert.status" />
                </div>
                <h4 class="alert-title">{{ alert.title }}</h4>
                <p class="alert-description">{{ alert.description }}</p>
                <div class="timeline-footer">
                  <div class="alert-meta">
                    <span>源: {{ alert.source }}</span>
                    <span>持续: {{ formatDuration(alert.duration) }}</span>
                    <span v-if="alert.operator">处理人: {{ alert.operator }}</span>
                  </div>
                  <div class="alert-actions">
                    <el-button size="small" @click="showAlertDetail(alert)">
                      详情
                    </el-button>
                  </div>
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
            :page-sizes="[10, 20, 50, 100]"
            :total="pagination.total"
            layout="total, sizes, prev, pager, next, jumper"
            @size-change="handlePageSizeChange"
            @current-change="handlePageChange"
          />
        </div>
      </el-card>
    </div>

    <!-- 批量操作工具栏 -->
    <div v-if="selectedAlerts.length > 0" class="batch-actions">
      <el-card shadow="never">
        <div class="batch-toolbar">
          <span>已选择 {{ selectedAlerts.length }} 条记录</span>
          <div class="actions">
            <el-button @click="batchAcknowledge">批量确认</el-button>
            <el-button @click="batchExport">批量导出</el-button>
            <el-button @click="batchDelete" type="danger">批量删除</el-button>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 告警详情对话框 -->
    <el-dialog
      v-model="detailDialogVisible"
      title="告警详情"
      width="80%"
      :close-on-click-modal="false"
    >
      <!-- <AlertDetail v-if="selectedAlert" :alert="selectedAlert" /> -->
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="detailDialogVisible = false">关闭</el-button>
          <el-button
            v-if="!selectedAlert?.acknowledged"
            type="warning"
            @click="acknowledgeAlert(selectedAlert?.id)"
          >
            确认告警
          </el-button>
          <el-button type="primary" @click="editAlert">编辑</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 显示设置对话框 -->
    <el-dialog
      v-model="showSettings"
      title="显示设置"
      width="600px"
    >
      <div class="settings-form">
        <el-form :model="displaySettings" label-width="120px">
          <el-form-item label="刷新间隔">
            <el-select v-model="displaySettings.refreshInterval">
              <el-option label="不自动刷新" :value="0" />
              <el-option label="30秒" :value="30000" />
              <el-option label="1分钟" :value="60000" />
              <el-option label="5分钟" :value="300000" />
              <el-option label="10分钟" :value="600000" />
            </el-select>
          </el-form-item>
          <el-form-item label="每页显示">
            <el-select v-model="displaySettings.pageSize">
              <el-option label="10条" :value="10" />
              <el-option label="20条" :value="20" />
              <el-option label="50条" :value="50" />
              <el-option label="100条" :value="100" />
            </el-select>
          </el-form-item>
          <el-form-item label="默认排序">
            <el-select v-model="displaySettings.defaultSort">
              <el-option label="触发时间倒序" value="triggered_at_desc" />
              <el-option label="触发时间正序" value="triggered_at_asc" />
              <el-option label="严重级别倒序" value="severity_desc" />
              <el-option label="ID倒序" value="id_desc" />
            </el-select>
          </el-form-item>
          <el-form-item label="显示列">
            <el-checkbox-group v-model="displaySettings.visibleColumns">
              <el-checkbox label="id">告警ID</el-checkbox>
              <el-checkbox label="severity">严重级别</el-checkbox>
              <el-checkbox label="type">类型</el-checkbox>
              <el-checkbox label="title">标题</el-checkbox>
              <el-checkbox label="source">告警源</el-checkbox>
              <el-checkbox label="status">状态</el-checkbox>
              <el-checkbox label="priority">优先级</el-checkbox>
              <el-checkbox label="triggered_at">触发时间</el-checkbox>
              <el-checkbox label="acknowledged">确认状态</el-checkbox>
              <el-checkbox label="duration">持续时间</el-checkbox>
            </el-checkbox-group>
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Clock, Refresh, Download, Setting, Search, Warning, Bell, Timer,
  Check, List, User
} from '@element-plus/icons-vue'
// import StatCard from '../../components/common/StatCard.vue' // 暂时注释掉不存在的组件
// import AlertSeverity from '../../components/business/AlertSeverity.vue' // 暂时注释掉不存在的组件
// import AlertStatus from '../../components/business/AlertStatus.vue' // 暂时注释掉不存在的组件
// import AlertDetail from '../../components/business/AlertDetail.vue' // 暂时注释掉不存在的组件

// 响应式数据
const loading = ref(false)
const viewMode = ref<'table' | 'timeline'>('table')
const showSettings = ref(false)
const detailDialogVisible = ref(false)
const selectedAlert = ref<any>(null)
const selectedAlerts = ref<any[]>([])
const refreshTimer = ref<NodeJS.Timeout | null>(null)

// 统计数据
const stats = ref({
  todayAlerts: 156,
  todayTrend: 12.5,
  weekAlerts: 892,
  weekTrend: -8.3,
  avgResponseTime: 45,
  responseTrend: -15.2,
  handleRate: 94.8,
  handleTrend: 3.2
})

// 筛选条件
const filters = ref({
  search: '',
  severity: '',
  status: '',
  type: '',
  dateRange: null as any,
  source: '',
  acknowledged: '',
  priority: '',
  operator: ''
})

// 分页信息
const pagination = ref({
  page: 1,
  size: 20,
  total: 0
})

// 排序信息
const sortInfo = ref({
  prop: 'triggered_at',
  order: 'descending'
})

// 显示设置
const displaySettings = ref({
  refreshInterval: 60000,
  pageSize: 20,
  defaultSort: 'triggered_at_desc',
  visibleColumns: ['id', 'severity', 'type', 'title', 'source', 'status', 'priority', 'triggered_at', 'acknowledged', 'duration']
})

// 告警源列表
const alertSources = ref([
  { id: 'driver_001', name: 'Modbus TCP Driver' },
  { id: 'driver_002', name: 'OPC UA Driver' },
  { id: 'conn_001', name: 'PLC Connection 1' },
  { id: 'conn_002', name: 'SCADA Connection' },
  { id: 'system', name: '系统监控' }
])

// 告警历史数据
const alertHistory = ref([
  {
    id: 'AL001234',
    severity: 'critical',
    type: 'datapoint',
    title: '温度传感器超出安全范围',
    description: '温度传感器 T001 读数超出设定安全范围 (85°C)',
    source: 'Modbus TCP Driver',
    status: 'handled',
    priority: 'high',
    triggered_at: '2024-01-20 14:30:25',
    resolved_at: '2024-01-20 14:45:10',
    acknowledged: true,
    acknowledged_by: 'admin',
    acknowledged_at: '2024-01-20 14:32:15',
    operator: '张工程师',
    duration: 885, // 秒
    value: 87.5,
    threshold: 85,
    tags: ['temperature', 'safety', 'critical']
  },
  {
    id: 'AL001235',
    severity: 'major',
    type: 'driver',
    title: 'OPC UA 连接异常',
    description: 'OPC UA 驱动连接到服务器失败，连续重试 3 次失败',
    source: 'OPC UA Driver',
    status: 'handling',
    priority: 'high',
    triggered_at: '2024-01-20 13:25:10',
    resolved_at: null,
    acknowledged: true,
    acknowledged_by: 'operator1',
    acknowledged_at: '2024-01-20 13:26:30',
    operator: '李技术员',
    duration: 4515,
    tags: ['connection', 'opc-ua', 'driver']
  },
  {
    id: 'AL001236',
    severity: 'warning',
    type: 'connection',
    title: '网络延迟过高',
    description: 'PLC 连接网络延迟超过 500ms，可能影响数据采集',
    source: 'PLC Connection 1',
    status: 'unhandled',
    priority: 'medium',
    triggered_at: '2024-01-20 12:15:45',
    resolved_at: null,
    acknowledged: false,
    acknowledged_by: null,
    acknowledged_at: null,
    operator: null,
    duration: 8745,
    value: 625,
    threshold: 500,
    tags: ['network', 'latency', 'performance']
  },
  {
    id: 'AL001237',
    severity: 'info',
    type: 'system',
    title: '系统维护完成',
    description: '定期系统维护任务已完成，所有服务恢复正常',
    source: '系统监控',
    status: 'handled',
    priority: 'low',
    triggered_at: '2024-01-20 10:00:00',
    resolved_at: '2024-01-20 10:05:30',
    acknowledged: true,
    acknowledged_by: 'system',
    acknowledged_at: '2024-01-20 10:00:05',
    operator: 'System',
    duration: 330,
    tags: ['maintenance', 'system', 'info']
  },
  {
    id: 'AL001238',
    severity: 'critical',
    type: 'datapoint',
    title: '压力传感器故障',
    description: '压力传感器 P003 无数据返回，设备可能已损坏',
    source: 'Modbus RTU Driver',
    status: 'handled',
    priority: 'high',
    triggered_at: '2024-01-20 09:30:15',
    resolved_at: '2024-01-20 11:20:45',
    acknowledged: true,
    acknowledged_by: 'admin',
    acknowledged_at: '2024-01-20 09:32:00',
    operator: '王维修工',
    duration: 6630,
    tags: ['sensor', 'pressure', 'hardware']
  }
])

// 计算属性
const filteredHistory = computed(() => {
  let result = [...alertHistory.value]

  // 应用筛选条件
  if (filters.value.search) {
    const search = filters.value.search.toLowerCase()
    result = result.filter(alert =>
      alert.title.toLowerCase().includes(search) ||
      alert.description.toLowerCase().includes(search) ||
      alert.source.toLowerCase().includes(search)
    )
  }

  if (filters.value.severity) {
    result = result.filter(alert => alert.severity === filters.value.severity)
  }

  if (filters.value.status) {
    result = result.filter(alert => alert.status === filters.value.status)
  }

  if (filters.value.type) {
    result = result.filter(alert => alert.type === filters.value.type)
  }

  if (filters.value.acknowledged !== '') {
    const isAcknowledged = filters.value.acknowledged === 'true'
    result = result.filter(alert => alert.acknowledged === isAcknowledged)
  }

  if (filters.value.priority) {
    result = result.filter(alert => alert.priority === filters.value.priority)
  }

  if (filters.value.operator) {
    const operator = filters.value.operator.toLowerCase()
    result = result.filter(alert =>
      alert.operator?.toLowerCase().includes(operator)
    )
  }

  if (filters.value.dateRange) {
    const [startDate, endDate] = filters.value.dateRange
    result = result.filter(alert => {
      const alertDate = new Date(alert.triggered_at)
      return alertDate >= new Date(startDate) && alertDate <= new Date(endDate)
    })
  }

  // 应用排序
  if (sortInfo.value.prop) {
    result.sort((a: any, b: any) => {
      let aVal = a[sortInfo.value.prop]
      let bVal = b[sortInfo.value.prop]

      if (sortInfo.value.prop === 'triggered_at') {
        aVal = new Date(aVal).getTime()
        bVal = new Date(bVal).getTime()
      }

      if (typeof aVal === 'string') {
        aVal = aVal.toLowerCase()
        bVal = bVal.toLowerCase()
      }

      if (sortInfo.value.order === 'ascending') {
        return aVal > bVal ? 1 : -1
      } else {
        return aVal < bVal ? 1 : -1
      }
    })
  }

  // 更新分页总数
  pagination.value.total = result.length

  // 应用分页
  const start = (pagination.value.page - 1) * pagination.value.size
  const end = start + pagination.value.size
  return result.slice(start, end)
})

// 方法
const refreshData = async () => {
  loading.value = true
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // 更新统计数据
    stats.value = {
      todayAlerts: Math.floor(Math.random() * 200) + 100,
      todayTrend: (Math.random() - 0.5) * 30,
      weekAlerts: Math.floor(Math.random() * 1000) + 500,
      weekTrend: (Math.random() - 0.5) * 20,
      avgResponseTime: Math.floor(Math.random() * 60) + 20,
      responseTrend: (Math.random() - 0.5) * 40,
      handleRate: 90 + Math.random() * 10,
      handleTrend: (Math.random() - 0.5) * 10
    }

    ElMessage.success('数据刷新成功')
  } catch (error) {
    ElMessage.error('数据刷新失败')
  } finally {
    loading.value = false
  }
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
    severity: '',
    status: '',
    type: '',
    dateRange: null,
    source: '',
    acknowledged: '',
    priority: '',
    operator: ''
  }
  pagination.value.page = 1
}

const handleSelectionChange = (selection: any[]) => {
  selectedAlerts.value = selection
}

const handleSortChange = ({ prop, order }: any) => {
  sortInfo.value = { prop, order }
}

const handlePageChange = (page: number) => {
  pagination.value.page = page
}

const handlePageSizeChange = (size: number) => {
  pagination.value.size = size
  pagination.value.page = 1
}

const showAlertDetail = (alert: any) => {
  selectedAlert.value = alert
  detailDialogVisible.value = true
}

const acknowledgeAlert = async (alertId: string) => {
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 500))
    
    const alert = alertHistory.value.find(a => a.id === alertId)
    if (alert) {
      alert.acknowledged = true
      alert.acknowledged_by = 'current_user'
      alert.acknowledged_at = new Date().toISOString().slice(0, 19).replace('T', ' ')
    }
    
    ElMessage.success('告警已确认')
    if (detailDialogVisible.value) {
      detailDialogVisible.value = false
    }
  } catch (error) {
    ElMessage.error('确认告警失败')
  }
}

const editAlert = () => {
  ElMessage.info('编辑功能开发中...')
}

const batchAcknowledge = async () => {
  try {
    await ElMessageBox.confirm(
      `确定要批量确认选中的 ${selectedAlerts.value.length} 条告警吗？`,
      '批量确认',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    // 模拟批量操作
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    selectedAlerts.value.forEach(alert => {
      const originalAlert = alertHistory.value.find(a => a.id === alert.id)
      if (originalAlert) {
        originalAlert.acknowledged = true
        originalAlert.acknowledged_by = 'current_user'
        originalAlert.acknowledged_at = new Date().toISOString().slice(0, 19).replace('T', ' ')
      }
    })
    
    selectedAlerts.value = []
    ElMessage.success('批量确认成功')
  } catch (error) {
    // 用户取消操作
  }
}

const batchExport = async () => {
  try {
    // 模拟导出操作
    await new Promise(resolve => setTimeout(resolve, 1000))
    ElMessage.success('导出成功')
  } catch (error) {
    ElMessage.error('导出失败')
  }
}

const batchDelete = async () => {
  try {
    await ElMessageBox.confirm(
      `确定要删除选中的 ${selectedAlerts.value.length} 条告警记录吗？此操作不可撤销。`,
      '批量删除',
      {
        confirmButtonText: '确定删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    // 模拟批量删除
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    const idsToDelete = selectedAlerts.value.map(alert => alert.id)
    alertHistory.value = alertHistory.value.filter(alert => !idsToDelete.includes(alert.id))
    
    selectedAlerts.value = []
    ElMessage.success('批量删除成功')
  } catch (error) {
    // 用户取消操作
  }
}

const exportHistory = async () => {
  try {
    // 模拟导出操作
    await new Promise(resolve => setTimeout(resolve, 1500))
    ElMessage.success('告警历史已导出到本地文件')
  } catch (error) {
    ElMessage.error('导出失败')
  }
}

const saveSettings = () => {
  // 应用设置
  pagination.value.size = displaySettings.value.pageSize
  
  // 设置自动刷新
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
    refreshTimer.value = null
  }
  
  if (displaySettings.value.refreshInterval > 0) {
    refreshTimer.value = setInterval(() => {
      refreshData()
    }, displaySettings.value.refreshInterval)
  }
  
  showSettings.value = false
  ElMessage.success('设置已保存')
}

// 工具函数
const formatDateTime = (dateStr: string) => {
  return dateStr
}

const getTimeAgo = (dateStr: string) => {
  const now = new Date()
  const alertTime = new Date(dateStr)
  const diff = now.getTime() - alertTime.getTime()
  
  const minutes = Math.floor(diff / (1000 * 60))
  const hours = Math.floor(diff / (1000 * 60 * 60))
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  
  if (minutes < 60) {
    return `${minutes}分钟前`
  } else if (hours < 24) {
    return `${hours}小时前`
  } else {
    return `${days}天前`
  }
}

const formatDuration = (seconds: number) => {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60
  
  if (hours > 0) {
    return `${hours}小时${minutes}分钟`
  } else if (minutes > 0) {
    return `${minutes}分钟${secs}秒`
  } else {
    return `${secs}秒`
  }
}

const getTypeText = (type: string) => {
  const typeMap: { [key: string]: string } = {
    datapoint: '数据点',
    driver: '驱动',
    connection: '连接',
    system: '系统'
  }
  return typeMap[type] || type
}

const getTypeTagType = (type: string) => {
  const tagTypeMap: { [key: string]: string } = {
    datapoint: 'primary',
    driver: 'success',
    connection: 'warning',
    system: 'info'
  }
  return tagTypeMap[type] || ''
}

const getPriorityText = (priority: string) => {
  const priorityMap: { [key: string]: string } = {
    high: '高',
    medium: '中',
    low: '低'
  }
  return priorityMap[priority] || priority
}

const getPriorityTagType = (priority: string) => {
  const tagTypeMap: { [key: string]: string } = {
    high: 'danger',
    medium: 'warning',
    low: 'info'
  }
  return tagTypeMap[priority] || ''
}

const getTimelineType = (severity: string) => {
  const typeMap: { [key: string]: string } = {
    critical: 'danger',
    major: 'warning',
    warning: 'primary',
    info: 'info'
  }
  return typeMap[severity] || 'primary'
}

const getSeverityClass = (severity: string) => {
  return `severity-${severity}`
}

// 生命周期
onMounted(() => {
  refreshData()
})

onUnmounted(() => {
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
  }
})
</script>

<style scoped lang="scss">
.alert-history {
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

  .stats-cards {
    margin-bottom: 24px;
  }

  .filters-container {
    margin-bottom: 24px;

    .filters {
      .el-row + .el-row {
        margin-top: 0;
      }
    }
  }

  .table-container {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .time-info {
      .time-ago {
        font-size: 12px;
        color: #909399;
        margin-top: 2px;
      }
    }

    .acknowledged {
      cursor: pointer;
    }

    .unacknowledged {
      cursor: pointer;
      opacity: 0.6;
    }
  }

  .timeline-view {
    .timeline-card {
      margin: 16px 0;

      &.severity-critical {
        border-left: 4px solid #f56c6c;
      }

      &.severity-major {
        border-left: 4px solid #e6a23c;
      }

      &.severity-warning {
        border-left: 4px solid #f4d03f;
      }

      &.severity-info {
        border-left: 4px solid #409eff;
      }

      .timeline-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 12px;

        .alert-info {
          display: flex;
          align-items: center;
          gap: 8px;

          .alert-id {
            color: #909399;
            font-size: 12px;
          }
        }
      }

      .alert-title {
        margin: 0 0 8px 0;
        font-size: 16px;
        font-weight: 600;
        color: #303133;
      }

      .alert-description {
        margin: 0 0 16px 0;
        color: #606266;
        line-height: 1.5;
      }

      .timeline-footer {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-top: 1px solid #ebeef5;
        padding-top: 12px;

        .alert-meta {
          display: flex;
          gap: 16px;
          font-size: 12px;
          color: #909399;

          span {
            display: flex;
            align-items: center;
          }
        }
      }
    }
  }

  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: 24px;
  }

  .batch-actions {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 1000;

    .batch-toolbar {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 24px;
      background: #409eff;
      color: white;
      border-radius: 8px;
      min-width: 400px;

      .actions {
        display: flex;
        gap: 12px;

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
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 12px;
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .alert-history {
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

    .filters {
      .el-col {
        margin-bottom: 12px;
      }
    }

    .timeline-card {
      .timeline-footer {
        flex-direction: column;
        gap: 12px;
        align-items: flex-start;

        .alert-meta {
          flex-direction: column;
          gap: 8px;
        }
      }
    }

    .batch-actions {
      left: 12px;
      right: 12px;
      transform: none;

      .batch-toolbar {
        min-width: auto;
        flex-direction: column;
        gap: 12px;
        text-align: center;

        .actions {
          justify-content: center;
        }
      }
    }
  }
}
</style>