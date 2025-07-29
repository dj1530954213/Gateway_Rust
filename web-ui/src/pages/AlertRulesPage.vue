<template>
  <div class="alert-rules-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="title-section">
          <h1 class="page-title">报警规则管理</h1>
          <p class="page-description">配置和管理设备数据的报警规则，实现智能监控和及时预警</p>
        </div>
        
        <div class="header-actions">
          <el-button type="primary" @click="showCreateDialog = true">
            <el-icon><Plus /></el-icon>
            新建规则
          </el-button>
          <el-button @click="refreshRules" :loading="loading">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
          <el-button @click="showImportDialog = true">
            <el-icon><Upload /></el-icon>
            导入规则
          </el-button>
          <el-button @click="exportRules" :disabled="selectedRules.length === 0">
            <el-icon><Download /></el-icon>
            导出规则
          </el-button>
        </div>
      </div>
    </div>

    <!-- 筛选和统计面板 -->
    <el-card class="filter-panel" shadow="never">
      <div class="filter-content">
        <!-- 筛选条件 -->
        <div class="filter-row">
          <div class="filter-item">
            <label class="filter-label">规则状态</label>
            <el-select v-model="filterForm.status" placeholder="选择状态" style="width: 150px">
              <el-option label="全部" value="" />
              <el-option label="启用" value="enabled" />
              <el-option label="禁用" value="disabled" />
              <el-option label="错误" value="error" />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">报警级别</label>
            <el-select v-model="filterForm.severity" placeholder="选择级别" style="width: 150px">
              <el-option label="全部" value="" />
              <el-option label="严重" value="critical" />
              <el-option label="警告" value="warning" />
              <el-option label="信息" value="info" />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">设备筛选</label>
            <el-select
              v-model="filterForm.deviceIds"
              multiple
              placeholder="选择设备"
              style="width: 300px"
              collapse-tags
              collapse-tags-tooltip
            >
              <el-option
                v-for="device in availableDevices"
                :key="device.id"
                :label="device.name"
                :value="device.id"
              />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">搜索</label>
            <el-input
              v-model="filterForm.keyword"
              placeholder="输入规则名称或描述"
              style="width: 250px"
              clearable
              @input="handleSearch"
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
          </div>

          <div class="filter-actions">
            <el-button @click="applyFilters">筛选</el-button>
            <el-button @click="resetFilters">重置</el-button>
          </div>
        </div>

        <!-- 统计信息 -->
        <div class="stats-row">
          <div class="stats-group">
            <div class="stat-item">
              <span class="stat-label">总规则数:</span>
              <span class="stat-value">{{ ruleStats.total }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">启用:</span>
              <span class="stat-value enabled">{{ ruleStats.enabled }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">禁用:</span>
              <span class="stat-value disabled">{{ ruleStats.disabled }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">错误:</span>
              <span class="stat-value error">{{ ruleStats.error }}</span>
            </div>
          </div>

          <div class="recent-alerts">
            <span class="recent-label">最近24小时触发:</span>
            <span class="recent-count">{{ ruleStats.recentAlerts }}</span>
            <el-button type="link" size="small" @click="viewRecentAlerts">
              查看详情
            </el-button>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 规则列表 -->
    <el-card class="rules-list-card" shadow="never">
      <!-- 列表工具栏 -->
      <div class="list-toolbar">
        <div class="toolbar-left">
          <el-checkbox
            v-model="selectAll"
            :indeterminate="isIndeterminate"
            @change="handleSelectAll"
          >
            全选
          </el-checkbox>
          <span class="selection-info" v-if="selectedRules.length > 0">
            已选择 {{ selectedRules.length }} 条规则
          </span>
        </div>

        <div class="toolbar-right">
          <div class="batch-actions" v-if="selectedRules.length > 0">
            <el-button size="small" @click="batchEnable">批量启用</el-button>
            <el-button size="small" @click="batchDisable">批量禁用</el-button>
            <el-button size="small" type="danger" @click="batchDelete">批量删除</el-button>
          </div>

          <div class="view-options">
            <el-radio-group v-model="viewMode" size="small">
              <el-radio-button label="list">列表</el-radio-button>
              <el-radio-button label="card">卡片</el-radio-button>
              <el-radio-button label="table">表格</el-radio-button>
            </el-radio-group>
          </div>
        </div>
      </div>

      <!-- 规则列表内容 -->
      <div class="rules-content">
        <!-- 列表视图 -->
        <div v-if="viewMode === 'list'" class="rules-list">
          <div
            v-for="rule in filteredRules"
            :key="rule.id"
            class="rule-item"
            :class="{ 'selected': selectedRules.includes(rule.id) }"
            @click="selectRule(rule.id)"
          >
            <div class="rule-checkbox">
              <el-checkbox
                :model-value="selectedRules.includes(rule.id)"
                @change="toggleRuleSelection(rule.id)"
                @click.stop
              />
            </div>

            <div class="rule-info">
              <div class="rule-header">
                <div class="rule-name">
                  <span class="name-text">{{ rule.name }}</span>
                  <el-tag
                    :type="getRuleStatusType(rule.status)"
                    size="small"
                  >
                    {{ getRuleStatusText(rule.status) }}
                  </el-tag>
                  <el-tag
                    :type="getSeverityType(rule.severity)"
                    size="small"
                  >
                    {{ getSeverityText(rule.severity) }}
                  </el-tag>
                </div>
                
                <div class="rule-actions">
                  <el-tooltip content="编辑规则">
                    <el-button type="link" size="small" @click.stop="editRule(rule)">
                      <el-icon><Edit /></el-icon>
                    </el-button>
                  </el-tooltip>
                  
                  <el-tooltip content="测试规则">
                    <el-button type="link" size="small" @click.stop="testRule(rule)">
                      <el-icon><VideoPlay /></el-icon>
                    </el-button>
                  </el-tooltip>
                  
                  <el-tooltip content="复制规则">
                    <el-button type="link" size="small" @click.stop="copyRule(rule)">
                      <el-icon><CopyDocument /></el-icon>
                    </el-button>
                  </el-tooltip>

                  <el-switch
                    :model-value="rule.status === 'enabled'"
                    @change="toggleRuleStatus(rule)"
                    @click.stop
                  />

                  <el-dropdown @click.stop>
                    <el-button type="link" size="small">
                      <el-icon><MoreFilled /></el-icon>
                    </el-button>
                    <template #dropdown>
                      <el-dropdown-menu>
                        <el-dropdown-item @click="viewRuleHistory(rule)">
                          查看历史
                        </el-dropdown-item>
                        <el-dropdown-item @click="exportRule(rule)">
                          导出规则
                        </el-dropdown-item>
                        <el-dropdown-item divided @click="deleteRule(rule)">
                          删除规则
                        </el-dropdown-item>
                      </el-dropdown-menu>
                    </template>
                  </el-dropdown>
                </div>
              </div>

              <div class="rule-description">
                {{ rule.description || '暂无描述' }}
              </div>

              <div class="rule-details">
                <div class="detail-item">
                  <span class="detail-label">监控设备:</span>
                  <span class="detail-value">{{ rule.deviceNames?.join(', ') || '未设置' }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">监控点位:</span>
                  <span class="detail-value">{{ rule.tagNames?.join(', ') || '未设置' }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">触发条件:</span>
                  <span class="detail-value">{{ formatConditions(rule.conditions) }}</span>
                </div>
              </div>

              <div class="rule-stats">
                <div class="stats-item">
                  <span class="stats-label">最近触发:</span>
                  <span class="stats-value">{{ formatDateTime(rule.lastTriggered) }}</span>
                </div>
                <div class="stats-item">
                  <span class="stats-label">触发次数:</span>
                  <span class="stats-value">{{ rule.triggerCount || 0 }}</span>
                </div>
                <div class="stats-item">
                  <span class="stats-label">创建时间:</span>
                  <span class="stats-value">{{ formatDateTime(rule.createdAt) }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-if="filteredRules.length === 0" class="empty-state">
            <el-empty
              description="暂无报警规则"
              :image-size="120"
            >
              <template #description>
                <p>还没有配置任何报警规则</p>
                <p>点击"新建规则"开始创建您的第一个报警规则</p>
              </template>
              <el-button type="primary" @click="showCreateDialog = true">
                新建规则
              </el-button>
            </el-empty>
          </div>
        </div>

        <!-- 卡片视图 -->
        <div v-else-if="viewMode === 'card'" class="rules-cards">
          <div
            v-for="rule in filteredRules"
            :key="rule.id"
            class="rule-card"
            :class="{ 'selected': selectedRules.includes(rule.id) }"
          >
            <div class="card-header">
              <div class="card-title">
                <el-checkbox
                  :model-value="selectedRules.includes(rule.id)"
                  @change="toggleRuleSelection(rule.id)"
                />
                <span class="title-text">{{ rule.name }}</span>
              </div>
              
              <div class="card-status">
                <el-tag
                  :type="getRuleStatusType(rule.status)"
                  size="small"
                >
                  {{ getRuleStatusText(rule.status) }}
                </el-tag>
              </div>
            </div>

            <div class="card-content">
              <div class="card-description">
                {{ rule.description || '暂无描述' }}
              </div>

              <div class="card-metrics">
                <div class="metric-item">
                  <span class="metric-label">级别</span>
                  <el-tag :type="getSeverityType(rule.severity)" size="small">
                    {{ getSeverityText(rule.severity) }}
                  </el-tag>
                </div>
                <div class="metric-item">
                  <span class="metric-label">触发次数</span>
                  <span class="metric-value">{{ rule.triggerCount || 0 }}</span>
                </div>
                <div class="metric-item">
                  <span class="metric-label">最近触发</span>
                  <span class="metric-value">{{ formatRelativeTime(rule.lastTriggered) }}</span>
                </div>
              </div>
            </div>

            <div class="card-actions">
              <el-button size="small" @click="editRule(rule)">编辑</el-button>
              <el-button size="small" @click="testRule(rule)">测试</el-button>
              <el-switch
                :model-value="rule.status === 'enabled'"
                @change="toggleRuleStatus(rule)"
              />
            </div>
          </div>
        </div>

        <!-- 表格视图 -->
        <div v-else-if="viewMode === 'table'" class="rules-table">
          <el-table
            :data="filteredRules"
            stripe
            border
            @selection-change="handleTableSelectionChange"
          >
            <el-table-column type="selection" width="50" />
            
            <el-table-column prop="name" label="规则名称" width="200">
              <template #default="{ row }">
                <div class="rule-name-cell">
                  <span class="name-text">{{ row.name }}</span>
                  <el-tag
                    :type="getRuleStatusType(row.status)"
                    size="small"
                  >
                    {{ getRuleStatusText(row.status) }}
                  </el-tag>
                </div>
              </template>
            </el-table-column>

            <el-table-column prop="severity" label="级别" width="100">
              <template #default="{ row }">
                <el-tag :type="getSeverityType(row.severity)" size="small">
                  {{ getSeverityText(row.severity) }}
                </el-tag>
              </template>
            </el-table-column>

            <el-table-column prop="description" label="描述" min-width="200" />

            <el-table-column label="监控对象" width="200">
              <template #default="{ row }">
                <div class="monitor-targets">
                  <div>设备: {{ row.deviceNames?.join(', ') || '未设置' }}</div>
                  <div>点位: {{ row.tagNames?.join(', ') || '未设置' }}</div>
                </div>
              </template>
            </el-table-column>

            <el-table-column prop="triggerCount" label="触发次数" width="100" />

            <el-table-column prop="lastTriggered" label="最近触发" width="160">
              <template #default="{ row }">
                {{ formatDateTime(row.lastTriggered) }}
              </template>
            </el-table-column>

            <el-table-column label="操作" width="200" fixed="right">
              <template #default="{ row }">
                <div class="table-actions">
                  <el-button type="link" size="small" @click="editRule(row)">
                    编辑
                  </el-button>
                  <el-button type="link" size="small" @click="testRule(row)">
                    测试
                  </el-button>
                  <el-switch
                    :model-value="row.status === 'enabled'"
                    @change="toggleRuleStatus(row)"
                    size="small"
                  />
                </div>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </div>

      <!-- 分页 -->
      <div class="pagination-container" v-if="filteredRules.length > 0">
        <el-pagination
          v-model:current-page="pagination.currentPage"
          v-model:page-size="pagination.pageSize"
          :total="pagination.total"
          :page-sizes="[10, 20, 50, 100]"
          background
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handlePageSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- 规则编辑对话框 -->
    <AlertRuleEditor
      v-model:visible="showCreateDialog"
      :rule-data="currentRule"
      @save="handleRuleSave"
    />

    <!-- 规则测试对话框 -->
    <AlertRuleTest
      v-model:visible="showTestDialog"
      :rule="testingRule"
    />

    <!-- 导入规则对话框 -->
    <AlertRuleImport
      v-model:visible="showImportDialog"
      @import="handleRuleImport"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * AlertRulesPage —— 报警规则管理页面
 *
 * 📝 Responsibilities:
 *  1. 报警规则列表展示和管理
 *  2. 规则创建、编辑、删除操作
 *  3. 规则状态控制和测试
 *  4. 批量操作和规则导入导出
 *  5. 规则筛选和搜索
 *
 * 📦 Dependencies:
 *  - AlertRuleEditor 规则编辑器
 *  - AlertRuleTest 规则测试组件
 *  - AlertRuleImport 规则导入组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Plus,
  Refresh,
  Upload,
  Download,
  Search,
  Edit,
  VideoPlay,
  CopyDocument,
  MoreFilled
} from '@element-plus/icons-vue'

import { useAlertsStore, useDevicesStore } from '@/stores'
import { formatDateTime, getRelativeTime } from '@/utils/date'

// 组件导入（这些组件将在后续步骤中创建）
import AlertRuleEditor from '@/components/alerts/AlertRuleEditor.vue'
import AlertRuleTest from '@/components/alerts/AlertRuleTest.vue'
import AlertRuleImport from '@/components/alerts/AlertRuleImport.vue'

// ===== 路由 =====
const router = useRouter()

// ===== Stores =====
const alertsStore = useAlertsStore()
const devicesStore = useDevicesStore()

// ===== 响应式数据 =====
const loading = ref(false)
const showCreateDialog = ref(false)
const showTestDialog = ref(false)
const showImportDialog = ref(false)

// 视图模式
const viewMode = ref<'list' | 'card' | 'table'>('list')

// 选择状态
const selectedRules = ref<string[]>([])
const selectAll = ref(false)

// 筛选表单
const filterForm = ref({
  status: '',
  severity: '',
  deviceIds: [] as string[],
  keyword: ''
})

// 分页
const pagination = ref({
  currentPage: 1,
  pageSize: 20,
  total: 0
})

// 当前操作的规则
const currentRule = ref(null)
const testingRule = ref(null)

// 可用设备列表
const availableDevices = ref([])

// 规则列表
const alertRules = ref([])

// ===== 计算属性 =====
const isIndeterminate = computed(() => {
  const selected = selectedRules.value.length
  const total = filteredRules.value.length
  return selected > 0 && selected < total
})

const filteredRules = computed(() => {
  let rules = alertRules.value

  // 状态筛选
  if (filterForm.value.status) {
    rules = rules.filter(rule => rule.status === filterForm.value.status)
  }

  // 级别筛选
  if (filterForm.value.severity) {
    rules = rules.filter(rule => rule.severity === filterForm.value.severity)
  }

  // 设备筛选
  if (filterForm.value.deviceIds.length > 0) {
    rules = rules.filter(rule => 
      rule.deviceIds?.some(id => filterForm.value.deviceIds.includes(id))
    )
  }

  // 关键词搜索
  if (filterForm.value.keyword) {
    const keyword = filterForm.value.keyword.toLowerCase()
    rules = rules.filter(rule => 
      rule.name.toLowerCase().includes(keyword) ||
      rule.description?.toLowerCase().includes(keyword)
    )
  }

  return rules
})

const ruleStats = computed(() => {
  const stats = {
    total: alertRules.value.length,
    enabled: 0,
    disabled: 0,
    error: 0,
    recentAlerts: 0
  }

  alertRules.value.forEach(rule => {
    switch (rule.status) {
      case 'enabled':
        stats.enabled++
        break
      case 'disabled':
        stats.disabled++
        break
      case 'error':
        stats.error++
        break
    }

    // 统计最近24小时触发的规则
    if (rule.lastTriggered) {
      const lastTriggered = new Date(rule.lastTriggered)
      const now = new Date()
      const diffHours = (now.getTime() - lastTriggered.getTime()) / (1000 * 60 * 60)
      if (diffHours <= 24) {
        stats.recentAlerts++
      }
    }
  })

  return stats
})

// ===== 方法 =====

/**
 * 初始化数据
 */
async function initializeData() {
  try {
    loading.value = true

    // 加载设备列表
    await devicesStore.fetchDevices({ page: 1, size: 1000 })
    availableDevices.value = devicesStore.state.devices

    // 加载报警规则
    await loadAlertRules()

  } catch (error) {
    console.error('初始化报警规则页面失败:', error)
    ElMessage.error('页面初始化失败')
  } finally {
    loading.value = false
  }
}

/**
 * 加载报警规则列表
 */
async function loadAlertRules() {
  try {
    // 这里应该调用实际的API
    // const rules = await alertsStore.fetchAlertRules()
    
    // 模拟数据
    alertRules.value = generateMockRules()
    pagination.value.total = alertRules.value.length

  } catch (error) {
    console.error('加载报警规则失败:', error)
    ElMessage.error('加载报警规则失败')
  }
}

/**
 * 生成模拟规则数据
 */
function generateMockRules() {
  const statuses = ['enabled', 'disabled', 'error']
  const severities = ['critical', 'warning', 'info']
  const rules = []

  for (let i = 1; i <= 25; i++) {
    rules.push({
      id: `rule_${i}`,
      name: `报警规则 ${i}`,
      description: `这是第 ${i} 个报警规则的描述信息`,
      status: statuses[i % statuses.length],
      severity: severities[i % severities.length],
      deviceIds: [`device_${(i % 3) + 1}`, `device_${(i % 5) + 1}`],
      deviceNames: [`设备-${(i % 3) + 1}`, `设备-${(i % 5) + 1}`],
      tagIds: [`tag_${i}`, `tag_${i + 10}`],
      tagNames: [`温度传感器-${i}`, `压力传感器-${i}`],
      conditions: [
        {
          field: 'temperature',
          operator: '>',
          value: 80 + (i % 20),
          unit: '°C'
        }
      ],
      triggerCount: Math.floor(Math.random() * 100),
      lastTriggered: i % 4 === 0 ? new Date(Date.now() - Math.random() * 24 * 60 * 60 * 1000).toISOString() : null,
      createdAt: new Date(Date.now() - i * 24 * 60 * 60 * 1000).toISOString(),
      updatedAt: new Date(Date.now() - Math.random() * 24 * 60 * 60 * 1000).toISOString()
    })
  }

  return rules
}

/**
 * 获取规则状态类型
 */
function getRuleStatusType(status: string): string {
  const typeMap: Record<string, string> = {
    'enabled': 'success',
    'disabled': 'info',
    'error': 'danger'
  }
  return typeMap[status] || 'info'
}

/**
 * 获取规则状态文本
 */
function getRuleStatusText(status: string): string {
  const textMap: Record<string, string> = {
    'enabled': '启用',
    'disabled': '禁用',
    'error': '错误'
  }
  return textMap[status] || status
}

/**
 * 获取严重程度类型
 */
function getSeverityType(severity: string): string {
  const typeMap: Record<string, string> = {
    'critical': 'danger',
    'warning': 'warning',
    'info': 'info'
  }
  return typeMap[severity] || 'info'
}

/**
 * 获取严重程度文本
 */
function getSeverityText(severity: string): string {
  const textMap: Record<string, string> = {
    'critical': '严重',
    'warning': '警告',
    'info': '信息'
  }
  return textMap[severity] || severity
}

/**
 * 格式化条件
 */
function formatConditions(conditions: any[]): string {
  if (!conditions || conditions.length === 0) return '未设置'
  
  return conditions.map(condition => {
    return `${condition.field} ${condition.operator} ${condition.value}${condition.unit || ''}`
  }).join(', ')
}

/**
 * 格式化相对时间
 */
function formatRelativeTime(timestamp: string): string {
  if (!timestamp) return '从未'
  return getRelativeTime(timestamp)
}

/**
 * 处理搜索
 */
function handleSearch() {
  // 防抖搜索
  clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    applyFilters()
  }, 300)
}

let searchTimeout: NodeJS.Timeout | null = null

/**
 * 应用筛选
 */
function applyFilters() {
  // 筛选逻辑在computed中处理
  pagination.value.currentPage = 1
}

/**
 * 重置筛选
 */
function resetFilters() {
  filterForm.value = {
    status: '',
    severity: '',
    deviceIds: [],
    keyword: ''
  }
  pagination.value.currentPage = 1
}

/**
 * 刷新规则列表
 */
async function refreshRules() {
  await loadAlertRules()
  ElMessage.success('规则列表已刷新')
}

/**
 * 全选/取消全选
 */
function handleSelectAll(value: boolean) {
  if (value) {
    selectedRules.value = filteredRules.value.map(rule => rule.id)
  } else {
    selectedRules.value = []
  }
}

/**
 * 选择规则
 */
function selectRule(ruleId: string) {
  if (selectedRules.value.includes(ruleId)) {
    selectedRules.value = selectedRules.value.filter(id => id !== ruleId)
  } else {
    selectedRules.value.push(ruleId)
  }
}

/**
 * 切换规则选择状态
 */
function toggleRuleSelection(ruleId: string) {
  selectRule(ruleId)
}

/**
 * 表格选择变化
 */
function handleTableSelectionChange(selection: any[]) {
  selectedRules.value = selection.map(rule => rule.id)
}

/**
 * 编辑规则
 */
function editRule(rule: any) {
  currentRule.value = { ...rule }
  showCreateDialog.value = true
}

/**
 * 测试规则
 */
function testRule(rule: any) {
  testingRule.value = rule
  showTestDialog.value = true
}

/**
 * 复制规则
 */
function copyRule(rule: any) {
  const copiedRule = {
    ...rule,
    id: undefined,
    name: `${rule.name  } (副本)`,
    status: 'disabled'
  }
  currentRule.value = copiedRule
  showCreateDialog.value = true
}

/**
 * 切换规则状态
 */
async function toggleRuleStatus(rule: any) {
  try {
    const newStatus = rule.status === 'enabled' ? 'disabled' : 'enabled'
    
    // 这里应该调用API更新状态
    // await alertsStore.updateRuleStatus(rule.id, newStatus)
    
    // 模拟更新
    rule.status = newStatus
    
    ElMessage.success(`规则已${newStatus === 'enabled' ? '启用' : '禁用'}`)
  } catch (error) {
    console.error('更新规则状态失败:', error)
    ElMessage.error('更新规则状态失败')
  }
}

/**
 * 删除规则
 */
async function deleteRule(rule: any) {
  try {
    await ElMessageBox.confirm(
      `确定要删除规则"${rule.name}"吗？此操作不可恢复。`,
      '确认删除',
      {
        type: 'warning'
      }
    )

    // 这里应该调用API删除
    // await alertsStore.deleteRule(rule.id)
    
    // 模拟删除
    alertRules.value = alertRules.value.filter(r => r.id !== rule.id)
    
    ElMessage.success('规则已删除')
  } catch (error) {
    if (error !== 'cancel') {
      console.error('删除规则失败:', error)
      ElMessage.error('删除规则失败')
    }
  }
}

/**
 * 批量启用
 */
async function batchEnable() {
  try {
    // 这里应该调用批量API
    selectedRules.value.forEach(ruleId => {
      const rule = alertRules.value.find(r => r.id === ruleId)
      if (rule) rule.status = 'enabled'
    })

    ElMessage.success(`已启用 ${selectedRules.value.length} 个规则`)
    selectedRules.value = []
  } catch (error) {
    console.error('批量启用失败:', error)
    ElMessage.error('批量启用失败')
  }
}

/**
 * 批量禁用
 */
async function batchDisable() {
  try {
    selectedRules.value.forEach(ruleId => {
      const rule = alertRules.value.find(r => r.id === ruleId)
      if (rule) rule.status = 'disabled'
    })

    ElMessage.success(`已禁用 ${selectedRules.value.length} 个规则`)
    selectedRules.value = []
  } catch (error) {
    console.error('批量禁用失败:', error)
    ElMessage.error('批量禁用失败')
  }
}

/**
 * 批量删除
 */
async function batchDelete() {
  try {
    await ElMessageBox.confirm(
      `确定要删除选中的 ${selectedRules.value.length} 个规则吗？此操作不可恢复。`,
      '确认批量删除',
      {
        type: 'warning'
      }
    )

    alertRules.value = alertRules.value.filter(rule => 
      !selectedRules.value.includes(rule.id)
    )

    ElMessage.success(`已删除 ${selectedRules.value.length} 个规则`)
    selectedRules.value = []
  } catch (error) {
    if (error !== 'cancel') {
      console.error('批量删除失败:', error)
      ElMessage.error('批量删除失败')
    }
  }
}

/**
 * 导出规则
 */
function exportRules() {
  const rulesToExport = selectedRules.value.length > 0 
    ? alertRules.value.filter(rule => selectedRules.value.includes(rule.id))
    : alertRules.value

  const exportData = {
    rules: rulesToExport,
    exportTime: new Date().toISOString(),
    version: '1.0'
  }

  const blob = new Blob([JSON.stringify(exportData, null, 2)], {
    type: 'application/json'
  })
  
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `alert_rules_${new Date().toISOString().slice(0, 10)}.json`
  link.click()
  
  URL.revokeObjectURL(url)
  ElMessage.success('规则导出完成')
}

/**
 * 导出单个规则
 */
function exportRule(rule: any) {
  selectedRules.value = [rule.id]
  exportRules()
  selectedRules.value = []
}

/**
 * 查看规则历史
 */
function viewRuleHistory(rule: any) {
  router.push(`/alerts/rules/${rule.id}/history`)
}

/**
 * 查看最近报警
 */
function viewRecentAlerts() {
  router.push('/alerts/history')
}

/**
 * 处理规则保存
 */
function handleRuleSave(ruleData: any) {
  if (ruleData.id) {
    // 更新现有规则
    const index = alertRules.value.findIndex(r => r.id === ruleData.id)
    if (index !== -1) {
      alertRules.value[index] = { ...ruleData }
    }
  } else {
    // 添加新规则
    ruleData.id = `rule_${Date.now()}`
    ruleData.createdAt = new Date().toISOString()
    alertRules.value.unshift(ruleData)
  }
  
  showCreateDialog.value = false
  currentRule.value = null
  ElMessage.success('规则保存成功')
}

/**
 * 处理规则导入
 */
function handleRuleImport(importedRules: any[]) {
  importedRules.forEach(rule => {
    rule.id = `rule_${Date.now()}_${Math.random()}`
    rule.createdAt = new Date().toISOString()
    alertRules.value.unshift(rule)
  })
  
  showImportDialog.value = false
  ElMessage.success(`已导入 ${importedRules.length} 个规则`)
}

/**
 * 分页变化
 */
function handlePageChange(page: number) {
  pagination.value.currentPage = page
}

/**
 * 页面大小变化
 */
function handlePageSizeChange(size: number) {
  pagination.value.pageSize = size
  pagination.value.currentPage = 1
}

// ===== 监听器 =====
watch(selectedRules, (newSelection) => {
  const total = filteredRules.value.length
  selectAll.value = newSelection.length === total && total > 0
})

// ===== 生命周期 =====
onMounted(async () => {
  await initializeData()
})
</script>

<style scoped lang="scss">
.alert-rules-page {
  padding: 24px;
  background: #f5f5f5;
  min-height: 100vh;

  .page-header {
    margin-bottom: 16px;
    
    .header-content {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      
      .title-section {
        .page-title {
          font-size: 28px;
          font-weight: 600;
          color: #303133;
          margin: 0 0 8px 0;
        }
        
        .page-description {
          font-size: 14px;
          color: #606266;
          margin: 0;
        }
      }
      
      .header-actions {
        display: flex;
        gap: 12px;
      }
    }
  }

  .filter-panel {
    margin-bottom: 16px;
    
    .filter-content {
      .filter-row {
        display: flex;
        align-items: center;
        gap: 20px;
        margin-bottom: 16px;
        flex-wrap: wrap;
        
        .filter-item {
          display: flex;
          align-items: center;
          gap: 8px;
          
          .filter-label {
            font-size: 13px;
            color: #606266;
            white-space: nowrap;
          }
        }
        
        .filter-actions {
          margin-left: auto;
          display: flex;
          gap: 8px;
        }
      }
      
      .stats-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding-top: 16px;
        border-top: 1px solid #ebeef5;
        
        .stats-group {
          display: flex;
          gap: 24px;
          
          .stat-item {
            display: flex;
            align-items: center;
            gap: 4px;
            
            .stat-label {
              font-size: 13px;
              color: #909399;
            }
            
            .stat-value {
              font-weight: 600;
              
              &.enabled { color: #67c23a; }
              &.disabled { color: #909399; }
              &.error { color: #f56c6c; }
            }
          }
        }
        
        .recent-alerts {
          display: flex;
          align-items: center;
          gap: 8px;
          
          .recent-label {
            font-size: 13px;
            color: #606266;
          }
          
          .recent-count {
            font-weight: 600;
            color: #e6a23c;
          }
        }
      }
    }
  }

  .rules-list-card {
    .list-toolbar {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;
      padding: 0 4px;
      
      .toolbar-left {
        display: flex;
        align-items: center;
        gap: 16px;
        
        .selection-info {
          font-size: 13px;
          color: #606266;
        }
      }
      
      .toolbar-right {
        display: flex;
        align-items: center;
        gap: 16px;
        
        .batch-actions {
          display: flex;
          gap: 8px;
        }
      }
    }
    
    .rules-content {
      .rules-list {
        .rule-item {
          display: flex;
          padding: 16px;
          border: 1px solid #ebeef5;
          border-radius: 6px;
          margin-bottom: 12px;
          background: white;
          cursor: pointer;
          transition: all 0.2s;
          
          &:hover {
            border-color: #c0c4cc;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
          }
          
          &.selected {
            border-color: #409eff;
            background: #f0f8ff;
          }
          
          .rule-checkbox {
            margin-right: 12px;
            display: flex;
            align-items: flex-start;
            padding-top: 2px;
          }
          
          .rule-info {
            flex: 1;
            
            .rule-header {
              display: flex;
              justify-content: space-between;
              align-items: flex-start;
              margin-bottom: 8px;
              
              .rule-name {
                display: flex;
                align-items: center;
                gap: 8px;
                
                .name-text {
                  font-size: 16px;
                  font-weight: 600;
                  color: #303133;
                }
              }
              
              .rule-actions {
                display: flex;
                align-items: center;
                gap: 8px;
              }
            }
            
            .rule-description {
              color: #606266;
              margin-bottom: 12px;
              line-height: 1.4;
            }
            
            .rule-details {
              display: grid;
              grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
              gap: 8px;
              margin-bottom: 12px;
              
              .detail-item {
                display: flex;
                gap: 8px;
                
                .detail-label {
                  font-size: 13px;
                  color: #909399;
                  white-space: nowrap;
                }
                
                .detail-value {
                  font-size: 13px;
                  color: #303133;
                }
              }
            }
            
            .rule-stats {
              display: flex;
              gap: 24px;
              
              .stats-item {
                display: flex;
                gap: 4px;
                
                .stats-label {
                  font-size: 12px;
                  color: #909399;
                }
                
                .stats-value {
                  font-size: 12px;
                  color: #303133;
                }
              }
            }
          }
        }
      }
      
      .rules-cards {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
        gap: 16px;
        
        .rule-card {
          border: 1px solid #ebeef5;
          border-radius: 8px;
          padding: 16px;
          background: white;
          transition: all 0.2s;
          
          &:hover {
            border-color: #c0c4cc;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
          }
          
          &.selected {
            border-color: #409eff;
            background: #f0f8ff;
          }
          
          .card-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 12px;
            
            .card-title {
              display: flex;
              align-items: center;
              gap: 8px;
              
              .title-text {
                font-weight: 600;
                color: #303133;
              }
            }
          }
          
          .card-content {
            margin-bottom: 16px;
            
            .card-description {
              color: #606266;
              margin-bottom: 12px;
              line-height: 1.4;
            }
            
            .card-metrics {
              display: flex;
              justify-content: space-between;
              
              .metric-item {
                text-align: center;
                
                .metric-label {
                  display: block;
                  font-size: 12px;
                  color: #909399;
                  margin-bottom: 4px;
                }
                
                .metric-value {
                  font-size: 13px;
                  color: #303133;
                  font-weight: 500;
                }
              }
            }
          }
          
          .card-actions {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding-top: 12px;
            border-top: 1px solid #f0f0f0;
          }
        }
      }
      
      .rules-table {
        .rule-name-cell {
          display: flex;
          align-items: center;
          gap: 8px;
          
          .name-text {
            font-weight: 500;
          }
        }
        
        .monitor-targets {
          font-size: 12px;
          
          div {
            margin-bottom: 2px;
            color: #606266;
          }
        }
        
        .table-actions {
          display: flex;
          align-items: center;
          gap: 8px;
        }
      }
      
      .empty-state {
        text-align: center;
        padding: 60px 20px;
      }
    }
    
    .pagination-container {
      margin-top: 16px;
      display: flex;
      justify-content: center;
    }
  }
}

@media (max-width: 1200px) {
  .alert-rules-page {
    .page-header .header-content {
      flex-direction: column;
      gap: 16px;
      align-items: stretch;
    }
    
    .filter-panel .filter-content .filter-row {
      flex-direction: column;
      align-items: stretch;
      gap: 12px;
      
      .filter-item {
        justify-content: space-between;
        
        :deep(.el-select) {
          width: 100% !important;
        }
      }
      
      .filter-actions {
        margin-left: 0;
        justify-content: center;
      }
    }
  }
}

@media (max-width: 768px) {
  .alert-rules-page {
    padding: 16px;
    
    .page-header .title-section .page-title {
      font-size: 24px;
    }
    
    .rules-list-card {
      .list-toolbar {
        flex-direction: column;
        gap: 12px;
        align-items: stretch;
        
        .toolbar-left,
        .toolbar-right {
          justify-content: center;
        }
      }
      
      .rules-content .rules-cards {
        grid-template-columns: 1fr;
      }
    }
  }
}
</style>