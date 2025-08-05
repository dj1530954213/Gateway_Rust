<template>
  <div class="alert-history-records">
    <!-- 筛选面板 -->
    <el-card class="filter-panel" shadow="never">
      <div class="filter-form">
        <div class="filter-row">
          <div class="filter-item">
            <label class="filter-label">时间范围</label>
            <el-date-picker
              v-model="filterForm.dateRange"
              type="datetimerange"
              range-separator="至"
              start-placeholder="开始时间"
              end-placeholder="结束时间"
              format="YYYY-MM-DD HH:mm:ss"
              value-format="YYYY-MM-DD HH:mm:ss"
              style="width: 380px"
              @change="handleDateRangeChange"
            />
          </div>

          <div class="filter-item">
            <label class="filter-label">报警级别</label>
            <el-select
              v-model="filterForm.severity"
              placeholder="选择报警级别"
              clearable
              style="width: 150px"
            >
              <el-option label="全部" value="" />
              <el-option label="信息" value="info" />
              <el-option label="警告" value="warning" />
              <el-option label="严重" value="critical" />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">报警状态</label>
            <el-select
              v-model="filterForm.status"
              placeholder="选择报警状态"
              clearable
              style="width: 150px"
            >
              <el-option label="全部" value="" />
              <el-option label="触发中" value="firing" />
              <el-option label="已恢复" value="resolved" />
              <el-option label="已确认" value="acknowledged" />
              <el-option label="已抑制" value="suppressed" />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">设备</label>
            <el-select
              v-model="filterForm.deviceId"
              placeholder="选择设备"
              clearable
              filterable
              style="width: 200px"
            >
              <el-option
                v-for="device in availableDevices"
                :key="device.id"
                :label="device.name"
                :value="device.id"
              />
            </el-select>
          </div>
        </div>

        <div class="filter-row">
          <div class="filter-item">
            <label class="filter-label">报警规则</label>
            <el-select
              v-model="filterForm.ruleId"
              placeholder="选择报警规则"
              clearable
              filterable
              style="width: 200px"
            >
              <el-option
                v-for="rule in availableRules"
                :key="rule.id"
                :label="rule.name"
                :value="rule.id"
              />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">关键词</label>
            <el-input
              v-model="filterForm.keyword"
              placeholder="搜索报警消息或描述"
              clearable
              style="width: 250px"
            />
          </div>

          <div class="filter-actions">
            <el-button type="primary" @click="searchAlerts" :loading="loading">
              <el-icon><Search /></el-icon>
              查询
            </el-button>
            <el-button @click="resetFilters">
              <el-icon><RefreshLeft /></el-icon>
              重置
            </el-button>
            <el-button @click="exportAlerts" :loading="exporting">
              <el-icon><Download /></el-icon>
              导出
            </el-button>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 统计卡片 -->
    <div class="stats-cards">
      <el-card shadow="never" class="stat-card">
        <div class="stat-content">
          <div class="stat-number">{{ stats.total }}</div>
          <div class="stat-label">总报警数</div>
        </div>
      </el-card>
      
      <el-card shadow="never" class="stat-card firing">
        <div class="stat-content">
          <div class="stat-number">{{ stats.firing }}</div>
          <div class="stat-label">触发中</div>
        </div>
      </el-card>
      
      <el-card shadow="never" class="stat-card resolved">
        <div class="stat-content">
          <div class="stat-number">{{ stats.resolved }}</div>
          <div class="stat-label">已恢复</div>
        </div>
      </el-card>
      
      <el-card shadow="never" class="stat-card acknowledged">
        <div class="stat-content">
          <div class="stat-number">{{ stats.acknowledged }}</div>
          <div class="stat-label">已确认</div>
        </div>
      </el-card>
    </div>

    <!-- 报警列表 -->
    <el-card class="alerts-table" shadow="never">
      <template #header>
        <div class="table-header">
          <span class="table-title">报警记录</span>
          <div class="table-actions">
            <el-button-group>
              <el-button
                :type="viewMode === 'list' ? 'primary' : 'default'"
                @click="viewMode = 'list'"
              >
                <el-icon><List /></el-icon>
                列表
              </el-button>
              <el-button
                :type="viewMode === 'timeline' ? 'primary' : 'default'"
                @click="viewMode = 'timeline'"
              >
                <el-icon><Timer /></el-icon>
                时间轴
              </el-button>
            </el-button-group>

            <el-dropdown @command="handleBatchAction">
              <el-button>
                批量操作
                <el-icon><ArrowDown /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="acknowledge">确认报警</el-dropdown-item>
                  <el-dropdown-item command="resolve">标记已解决</el-dropdown-item>
                  <el-dropdown-item command="suppress">抑制报警</el-dropdown-item>
                  <el-dropdown-item command="delete" divided>删除记录</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
      </template>

      <!-- 列表视图 -->
      <div v-if="viewMode === 'list'">
        <el-table
          :data="alertList"
          :loading="loading"
          row-key="id"
          @selection-change="handleSelectionChange"
          @sort-change="handleSortChange"
        >
          <el-table-column type="selection" width="55" />
          
          <el-table-column label="报警级别" width="100" align="center">
            <template #default="{ row }">
              <el-tag :type="getSeverityType(row.severity)" size="small">
                {{ getSeverityLabel(row.severity) }}
              </el-tag>
            </template>
          </el-table-column>

          <el-table-column label="报警状态" width="100" align="center">
            <template #default="{ row }">
              <el-tag :type="getStatusType(row.status)" size="small">
                {{ getStatusLabel(row.status) }}
              </el-tag>
            </template>
          </el-table-column>

          <el-table-column label="报警消息" min-width="300">
            <template #default="{ row }">
              <div class="alert-message">
                <div class="message-title">{{ row.title }}</div>
                <div class="message-desc">{{ row.description }}</div>
                <div class="message-meta">
                  <span class="meta-item">规则：{{ row.ruleName }}</span>
                  <span class="meta-item">设备：{{ row.deviceName }}</span>
                  <span v-if="row.tagName" class="meta-item">标签：{{ row.tagName }}</span>
                </div>
              </div>
            </template>
          </el-table-column>

          <el-table-column label="触发时间" width="180" sortable="custom" prop="triggerTime">
            <template #default="{ row }">
              <div class="time-info">
                <div class="time-primary">{{ formatDateTime(row.triggerTime) }}</div>
                <div class="time-duration" v-if="row.status === 'firing'">
                  持续 {{ formatDuration(row.triggerTime) }}
                </div>
              </div>
            </template>
          </el-table-column>

          <el-table-column label="恢复时间" width="180">
            <template #default="{ row }">
              <div v-if="row.resolveTime" class="time-info">
                <div class="time-primary">{{ formatDateTime(row.resolveTime) }}</div>
                <div class="time-duration">
                  历时 {{ formatDuration(row.triggerTime, row.resolveTime) }}
                </div>
              </div>
              <span v-else class="text-muted">-</span>
            </template>
          </el-table-column>

          <el-table-column label="操作" width="120" fixed="right">
            <template #default="{ row }">
              <el-dropdown @command="(cmd) => handleRowAction(cmd, row)">
                <el-button type="text">
                  操作 <el-icon><ArrowDown /></el-icon>
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item command="detail">查看详情</el-dropdown-item>
                    <el-dropdown-item 
                      v-if="row.status === 'firing'" 
                      command="acknowledge"
                    >
                      确认报警
                    </el-dropdown-item>
                    <el-dropdown-item 
                      v-if="row.status === 'firing'" 
                      command="resolve"
                    >
                      标记已解决
                    </el-dropdown-item>
                    <el-dropdown-item command="suppress">抑制报警</el-dropdown-item>
                    <el-dropdown-item command="history" divided>查看历史</el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- 时间轴视图 -->
      <div v-else class="timeline-view">
        <el-timeline>
          <el-timeline-item
            v-for="alert in alertList"
            :key="alert.id"
            :timestamp="formatDateTime(alert.triggerTime)"
            :type="getSeverityType(alert.severity)"
          >
            <div class="timeline-card">
              <div class="timeline-header">
                <div class="timeline-title">
                  <el-tag :type="getSeverityType(alert.severity)" size="small">
                    {{ getSeverityLabel(alert.severity) }}
                  </el-tag>
                  <span class="alert-title">{{ alert.title }}</span>
                  <el-tag :type="getStatusType(alert.status)" size="small">
                    {{ getStatusLabel(alert.status) }}
                  </el-tag>
                </div>
              </div>
              
              <div class="timeline-content">
                <p class="alert-description">{{ alert.description }}</p>
                <div class="alert-details">
                  <div class="detail-item">
                    <strong>设备：</strong>{{ alert.deviceName }}
                  </div>
                  <div v-if="alert.tagName" class="detail-item">
                    <strong>标签：</strong>{{ alert.tagName }}
                  </div>
                  <div class="detail-item">
                    <strong>规则：</strong>{{ alert.ruleName }}
                  </div>
                  <div v-if="alert.resolveTime" class="detail-item">
                    <strong>恢复时间：</strong>{{ formatDateTime(alert.resolveTime) }}
                  </div>
                </div>
              </div>
              
              <div class="timeline-actions">
                <el-button type="text" @click="showAlertDetail(alert)">
                  查看详情
                </el-button>
              </div>
            </div>
          </el-timeline-item>
        </el-timeline>
      </div>

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.size"
          :total="pagination.total"
          :page-sizes="[20, 50, 100, 200]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- 报警详情对话框 -->
    <el-dialog
      v-model="showDetailDialog"
      title="报警详情"
      width="800px"
      :close-on-click-modal="false"
    >
      <div v-if="selectedAlert" class="alert-detail">
        <div class="detail-section">
          <h4 class="detail-title">基本信息</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <label>报警标题：</label>
              <span>{{ selectedAlert.title }}</span>
            </div>
            <div class="detail-item">
              <label>报警级别：</label>
              <el-tag :type="getSeverityType(selectedAlert.severity)">
                {{ getSeverityLabel(selectedAlert.severity) }}
              </el-tag>
            </div>
            <div class="detail-item">
              <label>报警状态：</label>
              <el-tag :type="getStatusType(selectedAlert.status)">
                {{ getStatusLabel(selectedAlert.status) }}
              </el-tag>
            </div>
            <div class="detail-item">
              <label>触发时间：</label>
              <span>{{ formatDateTime(selectedAlert.triggerTime) }}</span>
            </div>
            <div v-if="selectedAlert.resolveTime" class="detail-item">
              <label>恢复时间：</label>
              <span>{{ formatDateTime(selectedAlert.resolveTime) }}</span>
            </div>
            <div class="detail-item">
              <label>持续时间：</label>
              <span>{{ formatDuration(selectedAlert.triggerTime, selectedAlert.resolveTime) }}</span>
            </div>
          </div>
        </div>

        <div class="detail-section">
          <h4 class="detail-title">描述信息</h4>
          <p class="alert-description">{{ selectedAlert.description }}</p>
        </div>

        <div class="detail-section">
          <h4 class="detail-title">关联信息</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <label>设备名称：</label>
              <span>{{ selectedAlert.deviceName }}</span>
            </div>
            <div class="detail-item">
              <label>设备ID：</label>
              <span>{{ selectedAlert.deviceId }}</span>
            </div>
            <div v-if="selectedAlert.tagName" class="detail-item">
              <label>标签名称：</label>
              <span>{{ selectedAlert.tagName }}</span>
            </div>
            <div v-if="selectedAlert.tagId" class="detail-item">
              <label>标签ID：</label>
              <span>{{ selectedAlert.tagId }}</span>
            </div>
            <div class="detail-item">
              <label>报警规则：</label>
              <span>{{ selectedAlert.ruleName }}</span>
            </div>
            <div class="detail-item">
              <label>规则ID：</label>
              <span>{{ selectedAlert.ruleId }}</span>
            </div>
          </div>
        </div>

        <div v-if="selectedAlert.triggerValue !== undefined" class="detail-section">
          <h4 class="detail-title">触发数据</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <label>触发值：</label>
              <span class="trigger-value">{{ selectedAlert.triggerValue }}</span>
            </div>
            <div v-if="selectedAlert.threshold !== undefined" class="detail-item">
              <label>阈值：</label>
              <span>{{ selectedAlert.threshold }}</span>
            </div>
            <div v-if="selectedAlert.unit" class="detail-item">
              <label>单位：</label>
              <span>{{ selectedAlert.unit }}</span>
            </div>
          </div>
        </div>

        <div v-if="selectedAlert.labels && Object.keys(selectedAlert.labels).length > 0" class="detail-section">
          <h4 class="detail-title">标签信息</h4>
          <div class="labels-grid">
            <div 
              v-for="(value, key) in selectedAlert.labels" 
              :key="key"
              class="label-item"
            >
              <el-tag>{{ key }}: {{ value }}</el-tag>
            </div>
          </div>
        </div>

        <div v-if="selectedAlert.annotations && Object.keys(selectedAlert.annotations).length > 0" class="detail-section">
          <h4 class="detail-title">注释信息</h4>
          <div class="annotations">
            <div 
              v-for="(value, key) in selectedAlert.annotations" 
              :key="key"
              class="annotation-item"
            >
              <strong>{{ key }}：</strong>{{ value }}
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="showDetailDialog = false">关闭</el-button>
          <el-button 
            v-if="selectedAlert?.status === 'firing'" 
            type="warning" 
            @click="acknowledgeAlert(selectedAlert)"
          >
            确认报警
          </el-button>
          <el-button 
            v-if="selectedAlert?.status === 'firing'" 
            type="success" 
            @click="resolveAlert(selectedAlert)"
          >
            标记已解决
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * AlertHistoryRecords —— 报警历史记录组件
 *
 * 📝 Responsibilities:
 *  1. 展示报警历史记录列表
 *  2. 提供多维度筛选功能
 *  3. 支持列表和时间轴两种视图
 *  4. 报警详情查看和操作
 *  5. 批量操作管理
 *
 * 📦 Dependencies:
 *  - Element Plus 组件库
 *  - 时间格式化工具
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Search,
  RefreshLeft,
  Download,
  List,
  Timer,
  ArrowDown
} from '@element-plus/icons-vue'
import { alertsApi, devicesApi } from '@/api'

// ===== Props =====
const props = defineProps<{
  deviceId?: string
  ruleId?: string
  autoRefresh?: boolean
}>()

// ===== 响应式数据 =====
const loading = ref(false)
const exporting = ref(false)
const viewMode = ref<'list' | 'timeline'>('list')
const showDetailDialog = ref(false)
const selectedAlert = ref<any>(null)
const selectedRows = ref<any[]>([])

// 筛选表单
const filterForm = ref({
  dateRange: [] as string[],
  severity: '',
  status: '',
  deviceId: props.deviceId || '',
  ruleId: props.ruleId || '',
  keyword: ''
})

// 分页
const pagination = ref({
  page: 1,
  size: 20,
  total: 0
})

// 排序
const sortField = ref('triggerTime')
const sortOrder = ref('desc')

// 数据
const alertList = ref<any[]>([])
const availableDevices = ref<any[]>([])
const availableRules = ref<any[]>([])

// 统计数据
const stats = ref({
  total: 0,
  firing: 0,
  resolved: 0,
  acknowledged: 0
})

// ===== 计算属性 =====
const hasSelection = computed(() => {
  return selectedRows.value.length > 0
})

// ===== 方法 =====

/**
 * 初始化数据
 */
async function initializeData() {
  try {
    // 从API加载设备和规则数据
    await loadAvailableDevices()
    await loadAvailableRules()
    
    // 设置默认时间范围（最近7天）
    const now = new Date()
    const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
    filterForm.value.dateRange = [
      weekAgo.toISOString().slice(0, 19),
      now.toISOString().slice(0, 19)
    ]

    // 加载报警数据
    await searchAlerts()

  } catch (error) {
    console.error('初始化报警历史失败:', error)
    ElMessage.error('初始化失败')
  }
}

/**
 * 加载可用设备列表
 */
async function loadAvailableDevices() {
  try {
    const response = await devicesApi.list()
    availableDevices.value = response.items || []
  } catch (error) {
    console.error('加载设备列表失败:', error)
    availableDevices.value = []
  }
}

/**
 * 加载可用规则列表
 */
async function loadAvailableRules() {
  try {
    const response = await alertsApi.getRules()
    availableRules.value = response.items || []
  } catch (error) {
    console.error('加载报警规则失败:', error)
    availableRules.value = []
  }
}

/**
 * 搜索报警
 */
async function searchAlerts() {
  try {
    loading.value = true

    // 构建查询参数
    const queryParams = {
      page: pagination.value.page,
      size: pagination.value.size,
      severity: filterForm.value.severity || undefined,
      status: filterForm.value.status || undefined,
      device_id: filterForm.value.deviceId || undefined,
      rule_id: filterForm.value.ruleId || undefined,
      keyword: filterForm.value.keyword || undefined,
      start_time: filterForm.value.dateRange?.[0] || undefined,
      end_time: filterForm.value.dateRange?.[1] || undefined,
      sort_field: sortField.value,
      sort_order: sortOrder.value
    }

    // 调用后端API
    const response = await alertsApi.getAlertHistory(queryParams)
    
    alertList.value = response.items
    pagination.value.total = response.total

    // 更新统计
    updateStats(response.items)

    if (response.items.length === 0) {
      ElMessage.info('未找到符合条件的报警记录')
    }

  } catch (error) {
    console.error('查询报警记录失败:', error)
    ElMessage.error('查询失败')
  } finally {
    loading.value = false
  }
}

/**
 * 更新统计数据
 */
function updateStats(alerts: any[]) {
  stats.value = {
    total: alerts.length,
    firing: alerts.filter(a => a.status === 'firing').length,
    resolved: alerts.filter(a => a.status === 'resolved').length,
    acknowledged: alerts.filter(a => a.status === 'acknowledged').length
  }
}

/**
 * 重置筛选条件
 */
function resetFilters() {
  const now = new Date()
  const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
  
  filterForm.value = {
    dateRange: [
      weekAgo.toISOString().slice(0, 19),
      now.toISOString().slice(0, 19)
    ],
    severity: '',
    status: '',
    deviceId: props.deviceId || '',
    ruleId: props.ruleId || '',
    keyword: ''
  }

  pagination.value.page = 1
  searchAlerts()
}

/**
 * 导出报警数据
 */
async function exportAlerts() {
  try {
    exporting.value = true
    
    // 构建查询参数
    const queryParams = {
      severity: filterForm.value.severity || undefined,
      status: filterForm.value.status || undefined,
      device_id: filterForm.value.deviceId || undefined,
      rule_id: filterForm.value.ruleId || undefined,
      keyword: filterForm.value.keyword || undefined,
      start_time: filterForm.value.dateRange?.[0] || undefined,
      end_time: filterForm.value.dateRange?.[1] || undefined,
      format: 'xlsx'
    }
    
    // 调用导出API
    await alertsApi.exportAlertHistory(queryParams)
    
    ElMessage.success('报警数据导出成功')
    
  } catch (error) {
    console.error('导出失败:', error)
    ElMessage.error('导出失败')
  } finally {
    exporting.value = false
  }
}

/**
 * 处理日期范围变化
 */
function handleDateRangeChange() {
  pagination.value.page = 1
  searchAlerts()
}

/**
 * 处理选择变化
 */
function handleSelectionChange(selection: any[]) {
  selectedRows.value = selection
}

/**
 * 处理排序变化
 */
function handleSortChange({ prop, order }: { prop: string, order: string | null }) {
  if (prop && order) {
    sortField.value = prop
    sortOrder.value = order === 'ascending' ? 'asc' : 'desc'
    pagination.value.page = 1
    searchAlerts()
  }
}

/**
 * 处理页面变化
 */
function handlePageChange(page: number) {
  pagination.value.page = page
  searchAlerts()
}

/**
 * 处理页面大小变化
 */
function handleSizeChange(size: number) {
  pagination.value.size = size
  pagination.value.page = 1
  searchAlerts()
}

/**
 * 处理批量操作
 */
async function handleBatchAction(command: string) {
  if (selectedRows.value.length === 0) {
    ElMessage.warning('请先选择要操作的报警记录')
    return
  }

  const actionMap = {
    acknowledge: '确认',
    resolve: '标记已解决',
    suppress: '抑制',
    delete: '删除'
  }

  const actionName = actionMap[command as keyof typeof actionMap]
  
  try {
    await ElMessageBox.confirm(
      `确定要${actionName}选中的 ${selectedRows.value.length} 条报警记录吗？`,
      '批量操作确认',
      {
        type: 'warning'
      }
    )

    // 调用批量操作API
    const alertIds = selectedRows.value.map(row => row.id)
    await alertsApi.batchUpdateAlerts(alertIds, { action: command })
    
    ElMessage.success(`已${actionName} ${selectedRows.value.length} 条报警记录`)
    selectedRows.value = []
    await searchAlerts()

  } catch (error) {
    // 用户取消操作
  }
}

/**
 * 处理行操作
 */
async function handleRowAction(command: string, row: any) {
  switch (command) {
    case 'detail':
      showAlertDetail(row)
      break
    case 'acknowledge':
      await acknowledgeAlert(row)
      break
    case 'resolve':
      await resolveAlert(row)
      break
    case 'suppress':
      await suppressAlert(row)
      break
    case 'history':
      showAlertHistory(row)
      break
  }
}

/**
 * 显示报警详情
 */
function showAlertDetail(alert: any) {
  selectedAlert.value = alert
  showDetailDialog.value = true
}

/**
 * 确认报警
 */
async function acknowledgeAlert(alert: any) {
  try {
    await alertsApi.updateAlert(alert.id, { status: 'acknowledged' })
    ElMessage.success('报警已确认')
    await searchAlerts()
  } catch (error) {
    console.error('确认报警失败:', error)
    ElMessage.error('确认报警失败')
  }
}

/**
 * 标记报警已解决
 */
async function resolveAlert(alert: any) {
  try {
    await alertsApi.updateAlert(alert.id, { status: 'resolved' })
    ElMessage.success('报警已标记为已解决')
    await searchAlerts()
  } catch (error) {
    console.error('解决报警失败:', error)
    ElMessage.error('操作失败')
  }
}

/**
 * 抑制报警
 */
async function suppressAlert(alert: any) {
  try {
    await alertsApi.updateAlert(alert.id, { status: 'suppressed' })
    ElMessage.success('报警已抑制')
    await searchAlerts()
  } catch (error) {
    console.error('抑制报警失败:', error)
    ElMessage.error('抑制报警失败')
  }
}

/**
 * 显示报警历史
 */
function showAlertHistory(alert: any) {
  ElMessage.info('报警历史功能开发中...')
}

/**
 * 获取严重程度类型
 */
function getSeverityType(severity: string): string {
  const typeMap: { [key: string]: string } = {
    critical: 'danger',
    warning: 'warning',
    info: 'info'
  }
  return typeMap[severity] || 'info'
}

/**
 * 获取严重程度标签
 */
function getSeverityLabel(severity: string): string {
  const labelMap: { [key: string]: string } = {
    critical: '严重',
    warning: '警告',
    info: '信息'
  }
  return labelMap[severity] || severity
}

/**
 * 获取状态类型
 */
function getStatusType(status: string): string {
  const typeMap: { [key: string]: string } = {
    firing: 'danger',
    resolved: 'success',
    acknowledged: 'warning',
    suppressed: 'info'
  }
  return typeMap[status] || 'info'
}

/**
 * 获取状态标签
 */
function getStatusLabel(status: string): string {
  const labelMap: { [key: string]: string } = {
    firing: '触发中',
    resolved: '已恢复',
    acknowledged: '已确认',
    suppressed: '已抑制'
  }
  return labelMap[status] || status
}

/**
 * 格式化日期时间
 */
function formatDateTime(dateStr: string): string {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN')
}

/**
 * 格式化持续时间
 */
function formatDuration(startTime: string, endTime?: string | null): string {
  if (!startTime) return '-'
  
  const start = new Date(startTime)
  const end = endTime ? new Date(endTime) : new Date()
  const diffMs = end.getTime() - start.getTime()
  
  const minutes = Math.floor(diffMs / (1000 * 60))
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) {
    return `${days}天${hours % 24}小时`
  } else if (hours > 0) {
    return `${hours}小时${minutes % 60}分钟`
  } else {
    return `${minutes}分钟`
  }
}

// ===== 生命周期 =====
onMounted(async () => {
  await initializeData()
})

// ===== 监听器 =====
watch(() => [filterForm.value.severity, filterForm.value.status], () => {
  pagination.value.page = 1
  searchAlerts()
})

watch(() => [filterForm.value.deviceId, filterForm.value.ruleId], () => {
  pagination.value.page = 1
  searchAlerts()
})

// 自动刷新
let refreshTimer: NodeJS.Timeout | null = null
if (props.autoRefresh) {
  refreshTimer = setInterval(() => {
    searchAlerts()
  }, 30000) // 30秒刷新一次
}

// 清理定时器
onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<style scoped lang="scss">
.alert-history-records {
  .filter-panel {
    margin-bottom: 16px;
    
    .filter-form {
      .filter-row {
        display: flex;
        align-items: flex-end;
        gap: 20px;
        margin-bottom: 16px;
        flex-wrap: wrap;
        
        &:last-child {
          margin-bottom: 0;
        }
        
        .filter-item {
          display: flex;
          flex-direction: column;
          
          .filter-label {
            font-size: 13px;
            color: #606266;
            margin-bottom: 6px;
            font-weight: 500;
          }
        }
        
        .filter-actions {
          display: flex;
          gap: 12px;
          margin-left: auto;
        }
      }
    }
  }
  
  .stats-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    margin-bottom: 16px;
    
    .stat-card {
      border: none;
      
      &.firing {
        border-left: 4px solid #f56c6c;
      }
      
      &.resolved {
        border-left: 4px solid #67c23a;
      }
      
      &.acknowledged {
        border-left: 4px solid #e6a23c;
      }
      
      .stat-content {
        text-align: center;
        
        .stat-number {
          font-size: 28px;
          font-weight: 600;
          color: #303133;
          margin-bottom: 8px;
        }
        
        .stat-label {
          font-size: 14px;
          color: #606266;
        }
      }
    }
  }
  
  .alerts-table {
    .table-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      .table-title {
        font-size: 16px;
        font-weight: 600;
        color: #303133;
      }
      
      .table-actions {
        display: flex;
        gap: 12px;
      }
    }
    
    .alert-message {
      .message-title {
        font-weight: 500;
        color: #303133;
        margin-bottom: 4px;
      }
      
      .message-desc {
        font-size: 13px;
        color: #606266;
        margin-bottom: 6px;
        line-height: 1.4;
      }
      
      .message-meta {
        display: flex;
        gap: 12px;
        flex-wrap: wrap;
        
        .meta-item {
          font-size: 12px;
          color: #909399;
          background: #f5f7fa;
          padding: 2px 6px;
          border-radius: 3px;
        }
      }
    }
    
    .time-info {
      .time-primary {
        color: #303133;
        margin-bottom: 2px;
      }
      
      .time-duration {
        font-size: 12px;
        color: #909399;
      }
    }
    
    .text-muted {
      color: #c0c4cc;
    }
    
    .timeline-view {
      padding: 20px 0;
      
      .timeline-card {
        background: #fff;
        border: 1px solid #e4e7ed;
        border-radius: 8px;
        padding: 16px;
        margin-bottom: 16px;
        
        .timeline-header {
          .timeline-title {
            display: flex;
            align-items: center;
            gap: 12px;
            margin-bottom: 12px;
            
            .alert-title {
              font-weight: 500;
              color: #303133;
              flex: 1;
            }
          }
        }
        
        .timeline-content {
          .alert-description {
            color: #606266;
            margin-bottom: 12px;
            line-height: 1.5;
          }
          
          .alert-details {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 8px;
            
            .detail-item {
              font-size: 13px;
              color: #606266;
              
              strong {
                color: #303133;
              }
            }
          }
        }
        
        .timeline-actions {
          margin-top: 12px;
          text-align: right;
        }
      }
    }
    
    .pagination-wrapper {
      margin-top: 20px;
      text-align: center;
    }
  }
}

// 报警详情对话框样式
.alert-detail {
  .detail-section {
    margin-bottom: 24px;
    
    &:last-child {
      margin-bottom: 0;
    }
    
    .detail-title {
      font-size: 16px;
      font-weight: 600;
      color: #303133;
      margin: 0 0 16px 0;
      padding-bottom: 8px;
      border-bottom: 1px solid #e4e7ed;
    }
    
    .detail-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
      gap: 12px;
      
      .detail-item {
        display: flex;
        align-items: center;
        
        label {
          font-weight: 500;
          color: #606266;
          width: 100px;
          flex-shrink: 0;
        }
        
        span {
          color: #303133;
        }
        
        .trigger-value {
          font-weight: 600;
          color: #f56c6c;
        }
      }
    }
    
    .alert-description {
      color: #606266;
      line-height: 1.6;
      margin: 0;
    }
    
    .labels-grid {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      
      .label-item {
        .el-tag {
          font-family: monospace;
        }
      }
    }
    
    .annotations {
      .annotation-item {
        margin-bottom: 8px;
        color: #606266;
        
        &:last-child {
          margin-bottom: 0;
        }
        
        strong {
          color: #303133;
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .alert-history-records {
    .filter-panel .filter-form .filter-row {
      flex-direction: column;
      align-items: stretch;
      gap: 16px;
      
      .filter-item {
        width: 100%;
        
        :deep(.el-select),
        :deep(.el-input),
        :deep(.el-date-editor) {
          width: 100% !important;
        }
      }
      
      .filter-actions {
        margin-left: 0;
        justify-content: center;
      }
    }
    
    .stats-cards {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}

@media (max-width: 768px) {
  .alert-history-records {
    .stats-cards {
      grid-template-columns: 1fr;
    }
    
    .alerts-table .table-header {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
    }
    
    .timeline-view .timeline-card .alert-details {
      grid-template-columns: 1fr;
    }
  }
  
  .alert-detail .detail-section .detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>