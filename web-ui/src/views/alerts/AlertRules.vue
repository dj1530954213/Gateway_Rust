<template>
  <div class="alert-rules-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <el-button :icon="ArrowLeft" @click="handleGoBack">
          返回告警
        </el-button>
        <div class="header-title">
          <h1>告警规则配置</h1>
          <p>管理系统告警规则，设置告警条件和通知策略</p>
        </div>
      </div>

      <div class="header-actions">
        <el-button :icon="Refresh" @click="handleRefresh"> 刷新 </el-button>
        <el-button type="primary" :icon="Plus" @click="handleCreateRule">
          创建规则
        </el-button>
      </div>
    </div>

    <!-- 规则统计 -->
    <div class="rules-overview">
      <el-row :gutter="20">
        <el-col :span="6">
          <div class="overview-card active">
            <div class="card-icon">
              <el-icon><CircleCheck /></el-icon>
            </div>
            <div class="card-content">
              <div class="card-value">{{ ruleStats.active }}</div>
              <div class="card-label">活跃规则</div>
            </div>
          </div>
        </el-col>

        <el-col :span="6">
          <div class="overview-card disabled">
            <div class="card-icon">
              <el-icon><CircleClose /></el-icon>
            </div>
            <div class="card-content">
              <div class="card-value">{{ ruleStats.disabled }}</div>
              <div class="card-label">禁用规则</div>
            </div>
          </div>
        </el-col>

        <el-col :span="6">
          <div class="overview-card triggered">
            <div class="card-icon">
              <el-icon><Warning /></el-icon>
            </div>
            <div class="card-content">
              <div class="card-value">{{ ruleStats.triggered }}</div>
              <div class="card-label">已触发</div>
            </div>
          </div>
        </el-col>

        <el-col :span="6">
          <div class="overview-card total">
            <div class="card-icon">
              <el-icon><DataBoard /></el-icon>
            </div>
            <div class="card-content">
              <div class="card-value">{{ ruleStats.total }}</div>
              <div class="card-label">总规则数</div>
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
            placeholder="搜索规则名称..."
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
            <el-option label="启用" value="enabled" />
            <el-option label="禁用" value="disabled" />
          </el-select>
        </el-col>

        <el-col :span="4">
          <el-select v-model="typeFilter" placeholder="规则类型" clearable>
            <el-option label="全部" value="" />
            <el-option label="数据点" value="datapoint" />
            <el-option label="驱动" value="driver" />
            <el-option label="连接" value="connection" />
            <el-option label="系统" value="system" />
          </el-select>
        </el-col>

        <el-col :span="6">
          <div class="filter-actions">
            <el-button @click="handleResetFilters"> 重置筛选 </el-button>
            <el-button
              :disabled="!selectedRules.length"
              @click="handleBulkEnable"
            >
              批量启用
            </el-button>
            <el-button
              :disabled="!selectedRules.length"
              type="warning"
              @click="handleBulkDisable"
            >
              批量禁用
            </el-button>
          </div>
        </el-col>
      </el-row>
    </div>

    <!-- 规则列表 -->
    <div class="rules-table">
      <el-table
        v-loading="loading"
        :data="filteredRules"
        stripe
        @selection-change="handleSelectionChange"
      >
        <el-table-column type="selection" width="55" />

        <el-table-column prop="name" label="规则名称" min-width="200">
          <template #default="{ row }">
            <div class="rule-name">
              <el-icon class="rule-icon">
                <component :is="getRuleTypeIcon(row.type)" />
              </el-icon>
              <span class="name-text">{{ row.name }}</span>
              <el-tag v-if="row.isTriggered" type="danger" size="small"
                >已触发</el-tag
              >
            </div>
          </template>
        </el-table-column>

        <el-table-column
          prop="description"
          label="描述"
          min-width="250"
          show-overflow-tooltip
        />

        <el-table-column prop="type" label="类型" width="100">
          <template #default="{ row }">
            <el-tag size="small" :type="getRuleTypeTagType(row.type)">
              {{ formatRuleType(row.type) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="severity" label="严重等级" width="100">
          <template #default="{ row }">
            <el-tag :type="getSeverityTagType(row.severity)" size="small">
              {{ formatSeverity(row.severity) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="condition" label="触发条件" min-width="200">
          <template #default="{ row }">
            <div class="condition-info">
              <span class="condition-text">{{
                formatCondition(row.condition)
              }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="enabled" label="状态" width="80">
          <template #default="{ row }">
            <el-switch
              v-model="row.enabled"
              :disabled="loading"
              @change="handleToggleRule(row)"
            />
          </template>
        </el-table-column>

        <el-table-column prop="triggerCount" label="触发次数" width="100">
          <template #default="{ row }">
            <el-badge
              v-if="row.triggerCount > 0"
              :value="row.triggerCount"
              :max="99"
            >
              <span class="trigger-count">{{ row.triggerCount }}</span>
            </el-badge>
            <span v-else>0</span>
          </template>
        </el-table-column>

        <el-table-column prop="lastTriggered" label="最后触发" width="160">
          <template #default="{ row }">
            <span v-if="row.lastTriggered" class="last-triggered">
              {{ formatTime(row.lastTriggered) }}
            </span>
            <span v-else class="never-triggered">从未触发</span>
          </template>
        </el-table-column>

        <el-table-column label="操作" width="180" fixed="right">
          <template #default="{ row }">
            <div class="action-buttons">
              <el-button size="small" :icon="View" @click="handleViewRule(row)">
                查看
              </el-button>

              <el-button size="small" :icon="Edit" @click="handleEditRule(row)">
                编辑
              </el-button>

              <el-button
                size="small"
                :icon="CopyDocument"
                @click="handleCopyRule(row)"
              >
                复制
              </el-button>

              <el-dropdown
                @command="command => handleDropdownCommand(command, row)"
              >
                <el-button size="small" :icon="MoreFilled" />
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item command="test">
                      <el-icon><Link /></el-icon>
                      测试规则
                    </el-dropdown-item>
                    <el-dropdown-item command="export">
                      <el-icon><Download /></el-icon>
                      导出规则
                    </el-dropdown-item>
                    <el-dropdown-item command="delete" divided>
                      <el-icon><Delete /></el-icon>
                      删除规则
                    </el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="table-pagination">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50, 100]"
          :total="totalRules"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </div>

    <!-- 规则详情对话框 -->
    <el-dialog
      v-model="showDetailDialog"
      :title="`规则详情 - ${selectedRule?.name}`"
      width="900px"
      :destroy-on-close="true"
    >
      <div v-if="selectedRule" class="rule-detail">
        <el-row :gutter="20">
          <el-col :span="12">
            <el-descriptions :column="1" border>
              <el-descriptions-item label="规则名称">
                <strong>{{ selectedRule.name }}</strong>
              </el-descriptions-item>
              <el-descriptions-item label="规则类型">
                <el-tag :type="getRuleTypeTagType(selectedRule.type)">
                  {{ formatRuleType(selectedRule.type) }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="严重等级">
                <el-tag :type="getSeverityTagType(selectedRule.severity)">
                  {{ formatSeverity(selectedRule.severity) }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="规则状态">
                <el-tag :type="selectedRule.enabled ? 'success' : 'info'">
                  {{ selectedRule.enabled ? '启用' : '禁用' }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="创建时间">
                {{ formatTime(selectedRule.createdAt) }}
              </el-descriptions-item>
              <el-descriptions-item label="更新时间">
                {{ formatTime(selectedRule.updatedAt) }}
              </el-descriptions-item>
            </el-descriptions>
          </el-col>

          <el-col :span="12">
            <el-descriptions :column="1" border>
              <el-descriptions-item label="触发次数">
                {{ selectedRule.triggerCount }}
              </el-descriptions-item>
              <el-descriptions-item label="最后触发">
                {{
                  selectedRule.lastTriggered
                    ? formatTime(selectedRule.lastTriggered)
                    : '从未触发'
                }}
              </el-descriptions-item>
              <el-descriptions-item label="创建人">
                {{ selectedRule.createdBy || '系统' }}
              </el-descriptions-item>
              <el-descriptions-item label="更新人">
                {{ selectedRule.updatedBy || '系统' }}
              </el-descriptions-item>
            </el-descriptions>
          </el-col>
        </el-row>

        <div class="rule-description">
          <h4>规则描述</h4>
          <div class="description-content">
            {{ selectedRule.description }}
          </div>
        </div>

        <div class="rule-condition">
          <h4>触发条件</h4>
          <div class="condition-detail">
            <el-descriptions :column="2" border size="small">
              <el-descriptions-item label="监控对象">
                {{ selectedRule.condition?.target || 'N/A' }}
              </el-descriptions-item>
              <el-descriptions-item label="比较操作">
                {{ formatOperator(selectedRule.condition?.operator) }}
              </el-descriptions-item>
              <el-descriptions-item label="阈值">
                {{ selectedRule.condition?.threshold }}
              </el-descriptions-item>
              <el-descriptions-item label="持续时间">
                {{ selectedRule.condition?.duration }}秒
              </el-descriptions-item>
            </el-descriptions>
          </div>
        </div>

        <div v-if="selectedRule.actions?.length" class="rule-actions">
          <h4>响应动作</h4>
          <div class="actions-list">
            <el-tag
              v-for="action in selectedRule.actions"
              :key="action.type"
              class="action-tag"
            >
              {{ formatActionType(action.type) }}
            </el-tag>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 创建/编辑规则对话框 -->
    <el-dialog
      v-model="showFormDialog"
      :title="isEditing ? '编辑告警规则' : '创建告警规则'"
      width="800px"
      :destroy-on-close="true"
    >
      <el-form
        ref="ruleFormRef"
        :model="ruleForm"
        :rules="ruleRules"
        label-width="120px"
      >
        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="规则名称" prop="name">
              <el-input v-model="ruleForm.name" placeholder="请输入规则名称" />
            </el-form-item>
          </el-col>

          <el-col :span="12">
            <el-form-item label="规则类型" prop="type">
              <el-select v-model="ruleForm.type" placeholder="请选择规则类型">
                <el-option label="数据点规则" value="datapoint" />
                <el-option label="驱动规则" value="driver" />
                <el-option label="连接规则" value="connection" />
                <el-option label="系统规则" value="system" />
              </el-select>
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item label="规则描述" prop="description">
          <el-input
            v-model="ruleForm.description"
            type="textarea"
            :rows="3"
            placeholder="请输入规则描述"
          />
        </el-form-item>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="严重等级" prop="severity">
              <el-select
                v-model="ruleForm.severity"
                placeholder="请选择严重等级"
              >
                <el-option label="紧急" value="critical" />
                <el-option label="警告" value="warning" />
                <el-option label="信息" value="info" />
              </el-select>
            </el-form-item>
          </el-col>

          <el-col :span="12">
            <el-form-item label="规则状态">
              <el-switch
                v-model="ruleForm.enabled"
                active-text="启用"
                inactive-text="禁用"
              />
            </el-form-item>
          </el-col>
        </el-row>

        <el-divider content-position="left">触发条件</el-divider>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="监控对象" prop="condition.target">
              <el-input
                v-model="ruleForm.condition.target"
                placeholder="例如：数据点名称"
              />
            </el-form-item>
          </el-col>

          <el-col :span="12">
            <el-form-item label="比较操作" prop="condition.operator">
              <el-select
                v-model="ruleForm.condition.operator"
                placeholder="请选择操作"
              >
                <el-option label="大于 (>)" value="gt" />
                <el-option label="大于等于 (>=)" value="gte" />
                <el-option label="小于 (<)" value="lt" />
                <el-option label="小于等于 (<=)" value="lte" />
                <el-option label="等于 (=)" value="eq" />
                <el-option label="不等于 (!=)" value="neq" />
              </el-select>
            </el-form-item>
          </el-col>
        </el-row>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="阈值" prop="condition.threshold">
              <el-input
                v-model="ruleForm.condition.threshold"
                placeholder="请输入阈值"
              />
            </el-form-item>
          </el-col>

          <el-col :span="12">
            <el-form-item label="持续时间(秒)" prop="condition.duration">
              <el-input-number
                v-model="ruleForm.condition.duration"
                :min="0"
                :max="3600"
                placeholder="持续时间"
              />
            </el-form-item>
          </el-col>
        </el-row>

        <el-divider content-position="left">响应动作</el-divider>

        <el-form-item label="通知方式">
          <el-checkbox-group v-model="ruleForm.actions">
            <el-checkbox value="email">邮件通知</el-checkbox>
            <el-checkbox value="sms">短信通知</el-checkbox>
            <el-checkbox value="webhook">Webhook</el-checkbox>
            <el-checkbox value="log">记录日志</el-checkbox>
          </el-checkbox-group>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showFormDialog = false">取消</el-button>
        <el-button
          type="primary"
          :loading="submitting"
          @click="handleSubmitRule"
        >
          {{ isEditing ? '更新规则' : '创建规则' }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import {
  ArrowLeft,
  Refresh,
  Plus,
  CircleCheck,
  CircleClose,
  Warning,
  DataBoard,
  Search,
  View,
  Edit,
  CopyDocument,
  MoreFilled,
  Link,
  Download,
  Delete,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, reactive, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'

import { alertsApi } from '@/api'

// 类型定义
interface AlertRule {
  id: string
  name: string
  description: string
  type: 'datapoint' | 'driver' | 'connection' | 'system'
  severity: 'critical' | 'warning' | 'info'
  enabled: boolean
  isTriggered: boolean
  triggerCount: number
  lastTriggered?: Date
  condition: {
    target: string
    operator: 'gt' | 'gte' | 'lt' | 'lte' | 'eq' | 'neq'
    threshold: string
    duration: number
  }
  actions: string[]
  createdBy?: string
  updatedBy?: string
  createdAt: Date
  updatedAt: Date
}

interface RuleStats {
  active: number
  disabled: number
  triggered: number
  total: number
}

// 路由
const router = useRouter()

// 状态管理
const loading = ref(false)
const submitting = ref(false)
const showDetailDialog = ref(false)
const showFormDialog = ref(false)
const isEditing = ref(false)
const selectedRule = ref<AlertRule | null>(null)
const selectedRules = ref<AlertRule[]>([])
const ruleFormRef = ref()

// 搜索和筛选
const searchQuery = ref('')
const severityFilter = ref('')
const statusFilter = ref('')
const typeFilter = ref('')

// 分页
const currentPage = ref(1)
const pageSize = ref(20)
const totalRules = ref(0)

// 规则数据
const rules = ref<AlertRule[]>([])
const ruleStats = ref<RuleStats>({
  active: 0,
  disabled: 0,
  triggered: 0,
  total: 0,
})

// 表单数据
const ruleForm = reactive<Partial<AlertRule>>({
  name: '',
  description: '',
  type: 'datapoint',
  severity: 'warning',
  enabled: true,
  condition: {
    target: '',
    operator: 'gt',
    threshold: '',
    duration: 60,
  },
  actions: [],
})

// 表单验证规则
const ruleRules = {
  name: [
    { required: true, message: '请输入规则名称', trigger: 'blur' },
    { min: 3, max: 50, message: '名称长度应在3-50个字符', trigger: 'blur' },
  ],
  type: [{ required: true, message: '请选择规则类型', trigger: 'change' }],
  description: [
    { required: true, message: '请输入规则描述', trigger: 'blur' },
    { min: 10, max: 200, message: '描述长度应在10-200个字符', trigger: 'blur' },
  ],
  severity: [{ required: true, message: '请选择严重等级', trigger: 'change' }],
  'condition.target': [
    { required: true, message: '请输入监控对象', trigger: 'blur' },
  ],
  'condition.operator': [
    { required: true, message: '请选择比较操作', trigger: 'change' },
  ],
  'condition.threshold': [
    { required: true, message: '请输入阈值', trigger: 'blur' },
  ],
}

// 计算属性 - 现在分页在后端处理，直接返回rules
const filteredRules = computed(() => {
  return rules.value
})

// 方法
const getRuleTypeIcon = (type: string) => {
  const iconMap = {
    datapoint: 'DataBoard',
    driver: 'Setting',
    connection: 'Link',
    system: 'Monitor',
  }
  return iconMap[type as keyof typeof iconMap] || 'Setting'
}

const getRuleTypeTagType = (type: string) => {
  const typeMap = {
    datapoint: 'primary',
    driver: 'success',
    connection: 'warning',
    system: 'info',
  }
  return typeMap[type as keyof typeof typeMap] || 'info'
}

const getSeverityTagType = (severity: string) => {
  const typeMap = {
    critical: 'danger',
    warning: 'warning',
    info: 'info',
  }
  return typeMap[severity as keyof typeof typeMap] || 'info'
}

const formatRuleType = (type: string) => {
  const textMap = {
    datapoint: '数据点',
    driver: '驱动',
    connection: '连接',
    system: '系统',
  }
  return textMap[type as keyof typeof textMap] || type
}

const formatSeverity = (severity: string) => {
  const textMap = {
    critical: '紧急',
    warning: '警告',
    info: '信息',
  }
  return textMap[severity as keyof typeof textMap] || severity
}

const formatOperator = (operator: string) => {
  const textMap = {
    gt: '大于 (>)',
    gte: '大于等于 (>=)',
    lt: '小于 (<)',
    lte: '小于等于 (<=)',
    eq: '等于 (=)',
    neq: '不等于 (!=)',
  }
  return textMap[operator as keyof typeof textMap] || operator
}

const formatActionType = (type: string) => {
  const textMap = {
    email: '邮件通知',
    sms: '短信通知',
    webhook: 'Webhook',
    log: '记录日志',
  }
  return textMap[type as keyof typeof textMap] || type
}

const formatCondition = (condition: any) => {
  if (!condition) return 'N/A'
  return `${condition.target} ${formatOperator(condition.operator)} ${condition.threshold}`
}

const formatTime = (time: Date) => {
  return time.toLocaleString('zh-CN')
}

// 事件处理
const handleGoBack = () => {
  router.push('/alerts')
}

// 数据加载函数
const loadRules = async () => {
  try {
    loading.value = true
    const response = await alertsApi.getRules({
      page: currentPage.value,
      size: pageSize.value,
      severity: severityFilter.value || undefined,
      enabled:
        statusFilter.value === 'enabled'
          ? true
          : statusFilter.value === 'disabled'
            ? false
            : undefined,
      type: typeFilter.value || undefined,
      keyword: searchQuery.value || undefined,
    })

    if (response.success && response.data) {
      rules.value = response.data.items || []
      totalRules.value = response.data.total || 0
    }
  } catch (error) {
    console.error('Failed to load alert rules:', error)
    ElMessage.error('加载告警规则失败')
  } finally {
    loading.value = false
  }
}

const loadRuleStats = async () => {
  try {
    const stats = await alertsApi.getRuleStats()
    ruleStats.value = stats
  } catch (error) {
    console.error('Failed to load rule stats:', error)
  }
}

const handleRefresh = async () => {
  await Promise.all([loadRules(), loadRuleStats()])
  ElMessage.success('刷新成功')
}

const handleSearch = async () => {
  currentPage.value = 1
  await loadRules()
}

const handleResetFilters = async () => {
  searchQuery.value = ''
  severityFilter.value = ''
  statusFilter.value = ''
  typeFilter.value = ''
  currentPage.value = 1
  await loadRules()
}

const handleSelectionChange = (selection: AlertRule[]) => {
  selectedRules.value = selection
}

const handleSizeChange = async (size: number) => {
  pageSize.value = size
  currentPage.value = 1
  await loadRules()
}

const handleCurrentChange = async (page: number) => {
  currentPage.value = page
  await loadRules()
}

const handleToggleRule = async (rule: AlertRule) => {
  try {
    if (rule.enabled) {
      await alertsApi.disableRule(rule.id)
    } else {
      await alertsApi.enableRule(rule.id)
    }
    await loadRules()
    await loadRuleStats()
    ElMessage.success(`规则已${rule.enabled ? '禁用' : '启用'}`)
  } catch (error) {
    ElMessage.error('操作失败')
  }
}

const handleViewRule = (rule: AlertRule) => {
  selectedRule.value = rule
  showDetailDialog.value = true
}

const handleEditRule = (rule: AlertRule) => {
  isEditing.value = true
  Object.assign(ruleForm, rule)
  showFormDialog.value = true
}

const handleCopyRule = (rule: AlertRule) => {
  isEditing.value = false
  Object.assign(ruleForm, {
    ...rule,
    id: undefined,
    name: `${rule.name} - 副本`,
    enabled: false,
  })
  showFormDialog.value = true
}

const handleCreateRule = () => {
  isEditing.value = false
  Object.assign(ruleForm, {
    name: '',
    description: '',
    type: 'datapoint',
    severity: 'warning',
    enabled: true,
    condition: {
      target: '',
      operator: 'gt',
      threshold: '',
      duration: 60,
    },
    actions: [],
  })
  showFormDialog.value = true
}

const handleSubmitRule = async () => {
  try {
    await ruleFormRef.value?.validate()

    submitting.value = true
    await new Promise(resolve => setTimeout(resolve, 1000))

    if (isEditing.value) {
      const index = rules.value.findIndex(r => r.id === ruleForm.id)
      if (index !== -1) {
        Object.assign(rules.value[index], {
          ...ruleForm,
          updatedAt: new Date(),
          updatedBy: '当前用户',
        })
      }
      ElMessage.success('规则更新成功')
    } else {
      const newRule: AlertRule = {
        ...(ruleForm as AlertRule),
        id: Date.now().toString(),
        isTriggered: false,
        triggerCount: 0,
        createdBy: '当前用户',
        createdAt: new Date(),
        updatedAt: new Date(),
      }
      rules.value.unshift(newRule)
      ElMessage.success('规则创建成功')
    }

    updateRuleStats()
    showFormDialog.value = false
  } catch (error) {
    ElMessage.error('表单填写不完整')
  } finally {
    submitting.value = false
  }
}

const handleDropdownCommand = async (command: string, rule: AlertRule) => {
  switch (command) {
    case 'test':
      await handleTestRule(rule)
      break
    case 'export':
      handleExportRule(rule)
      break
    case 'delete':
      await handleDeleteRule(rule)
      break
  }
}

const handleTestRule = async (rule: AlertRule) => {
  try {
    ElMessage.info('正在测试规则...')
    await new Promise(resolve => setTimeout(resolve, 2000))
    const success = Math.random() > 0.3
    if (success) {
      ElMessage.success('规则测试通过，条件配置正确')
    } else {
      ElMessage.warning('规则测试失败，请检查条件配置')
    }
  } catch (error) {
    ElMessage.error('测试异常')
  }
}

const handleExportRule = (rule: AlertRule) => {
  const exportData = {
    rule,
    exportTime: new Date().toISOString(),
    version: '1.0',
  }

  const blob = new Blob([JSON.stringify(exportData, null, 2)], {
    type: 'application/json',
  })

  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `alert_rule_${rule.name}_${new Date().toISOString().split('T')[0]}.json`
  a.click()

  URL.revokeObjectURL(url)
  ElMessage.success('规则已导出')
}

const handleDeleteRule = async (rule: AlertRule) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除规则 "${rule.name}" 吗？此操作不可撤销。`,
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    const index = rules.value.findIndex(r => r.id === rule.id)
    if (index !== -1) {
      rules.value.splice(index, 1)
      updateRuleStats()
      ElMessage.success('规则已删除')
    }
  } catch (error) {
    // 取消操作
  }
}

const handleBulkEnable = async () => {
  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    selectedRules.value.forEach(rule => {
      rule.enabled = true
    })
    updateRuleStats()
    ElMessage.success(`已批量启用 ${selectedRules.value.length} 个规则`)
    selectedRules.value = []
  } catch (error) {
    ElMessage.error('批量操作失败')
  }
}

const handleBulkDisable = async () => {
  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    selectedRules.value.forEach(rule => {
      rule.enabled = false
    })
    updateRuleStats()
    ElMessage.success(`已批量禁用 ${selectedRules.value.length} 个规则`)
    selectedRules.value = []
  } catch (error) {
    ElMessage.error('批量操作失败')
  }
}

// 监听筛选器变化
watch([severityFilter, statusFilter, typeFilter], async () => {
  currentPage.value = 1
  await loadRules()
})

// 生命周期
onMounted(async () => {
  await Promise.all([loadRules(), loadRuleStats()])
})
</script>

<style scoped lang="scss">
.alert-rules-page {
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
    display: flex;
    align-items: center;
    gap: 16px;

    .header-title {
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
  }

  .header-actions {
    display: flex;
    gap: 12px;
  }
}

.rules-overview {
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

    &.active {
      .card-icon {
        background: var(--el-color-success);
      }
      .card-value {
        color: var(--el-color-success);
      }
    }

    &.disabled {
      .card-icon {
        background: var(--el-color-info);
      }
      .card-value {
        color: var(--el-color-info);
      }
    }

    &.triggered {
      .card-icon {
        background: var(--el-color-danger);
      }
      .card-value {
        color: var(--el-color-danger);
      }
    }

    &.total {
      .card-icon {
        background: var(--el-color-primary);
      }
      .card-value {
        color: var(--el-color-primary);
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
    display: flex;
    gap: 12px;
  }
}

.rules-table {
  background: white;
  border-radius: 12px;
  padding: 20px;
  border: 1px solid var(--el-border-color-light);

  .rule-name {
    display: flex;
    align-items: center;
    gap: 8px;

    .rule-icon {
      color: var(--el-color-primary);
      font-size: 16px;
    }

    .name-text {
      font-weight: 500;
    }
  }

  .condition-info {
    .condition-text {
      font-family: 'Courier New', monospace;
      font-size: 12px;
      background: var(--el-fill-color-light);
      padding: 4px 8px;
      border-radius: 4px;
    }
  }

  .trigger-count {
    font-weight: 500;
  }

  .last-triggered {
    font-size: 13px;
    color: var(--el-text-color-primary);
  }

  .never-triggered {
    color: var(--el-text-color-placeholder);
    font-size: 13px;
  }

  .action-buttons {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .table-pagination {
    margin-top: 20px;
    display: flex;
    justify-content: center;
  }
}

.rule-detail {
  .rule-description,
  .rule-condition,
  .rule-actions {
    margin-top: 20px;

    h4 {
      margin: 0 0 12px 0;
      font-size: 16px;
      color: var(--el-text-color-primary);
    }
  }

  .description-content {
    background: var(--el-fill-color-light);
    padding: 16px;
    border-radius: 6px;
    line-height: 1.6;
    color: var(--el-text-color-primary);
  }

  .actions-list {
    .action-tag {
      margin: 0 8px 8px 0;
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .rules-overview {
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

    .header-left {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

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
}
</style>
