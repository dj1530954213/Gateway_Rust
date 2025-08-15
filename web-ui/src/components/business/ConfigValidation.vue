<template>
  <div class="config-validation" :class="containerClass">
    <!-- 验证头部 -->
    <div class="validation-header">
      <div class="header-title">
        <el-icon :size="18">
          <CircleCheck />
        </el-icon>
        <span>{{ title }}</span>
        <el-tag
          v-if="showValidationStatus"
          :type="overallStatusType"
          size="small"
        >
          {{ overallStatusText }}
        </el-tag>
      </div>

      <div class="header-actions">
        <!-- 验证按钮 -->
        <el-button
          type="primary"
          size="small"
          :loading="validating"
          :disabled="!hasConfigToValidate"
          @click="handleValidate"
        >
          {{ validating ? '验证中...' : '开始验证' }}
        </el-button>

        <!-- 更多操作 -->
        <el-dropdown v-if="showActions" trigger="click" @command="handleAction">
          <el-button type="text" size="small" :icon="MoreFilled" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="export" :icon="Download">
                导出验证报告
              </el-dropdown-item>
              <el-dropdown-item command="rules" :icon="Setting">
                管理验证规则
              </el-dropdown-item>
              <el-dropdown-item command="history" :icon="Clock">
                验证历史
              </el-dropdown-item>
              <el-dropdown-item command="clear" :icon="Delete">
                清空结果
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- 验证进度 -->
    <div v-if="validating" class="validation-progress">
      <el-progress
        :percentage="validationProgress"
        :status="validationProgressStatus"
        :stroke-width="6"
      >
        <template #default="{ percentage }">
          <span class="progress-text">{{ progressText }}</span>
        </template>
      </el-progress>
    </div>

    <!-- 验证概览 -->
    <div
      v-if="showSummary && validationResults.length > 0"
      class="validation-summary"
    >
      <el-row :gutter="16">
        <el-col :span="6">
          <el-statistic title="总计" :value="totalValidations" />
        </el-col>
        <el-col :span="6">
          <el-statistic title="通过" :value="passedValidations" suffix="项">
            <template #prefix>
              <el-icon style="color: var(--el-color-success)">
                <CircleCheck />
              </el-icon>
            </template>
          </el-statistic>
        </el-col>
        <el-col :span="6">
          <el-statistic title="警告" :value="warningValidations" suffix="项">
            <template #prefix>
              <el-icon style="color: var(--el-color-warning)">
                <Warning />
              </el-icon>
            </template>
          </el-statistic>
        </el-col>
        <el-col :span="6">
          <el-statistic title="错误" :value="errorValidations" suffix="项">
            <template #prefix>
              <el-icon style="color: var(--el-color-danger)">
                <CircleClose />
              </el-icon>
            </template>
          </el-statistic>
        </el-col>
      </el-row>
    </div>

    <!-- 验证结果 -->
    <div class="validation-content">
      <el-tabs v-model="activeTab" type="border-card">
        <!-- 验证结果 -->
        <el-tab-pane label="验证结果" name="results">
          <div class="validation-results">
            <!-- 筛选器 -->
            <div class="results-filter">
              <el-radio-group
                v-model="filterType"
                size="small"
                @change="handleFilterChange"
              >
                <el-radio-button label="all">全部</el-radio-button>
                <el-radio-button label="error">错误</el-radio-button>
                <el-radio-button label="warning">警告</el-radio-button>
                <el-radio-button label="success">通过</el-radio-button>
              </el-radio-group>

              <SearchBox
                v-model="searchKeyword"
                placeholder="搜索验证项目"
                size="small"
                style="width: 200px; margin-left: 12px"
                @search="handleSearch"
              />
            </div>

            <!-- 结果列表 -->
            <div class="results-list">
              <el-empty
                v-if="filteredResults.length === 0 && !validating"
                :description="
                  hasValidationResults
                    ? '没有匹配的验证结果'
                    : '请点击开始验证按钮'
                "
                :image-size="100"
              />

              <div
                v-for="(result, index) in filteredResults"
                :key="index"
                class="result-item"
                :class="`result-item--${result.type}`"
              >
                <div class="result-header">
                  <div class="result-icon">
                    <el-icon :size="20">
                      <component :is="getResultIcon(result.type)" />
                    </el-icon>
                  </div>

                  <div class="result-info">
                    <div class="result-title">{{ result.title }}</div>
                    <div class="result-category">{{ result.category }}</div>
                  </div>

                  <div class="result-status">
                    <StatusTag :status="result.type" size="small" />
                  </div>

                  <div class="result-actions">
                    <el-button
                      v-if="result.type === 'error' && result.fixable"
                      type="text"
                      size="small"
                      @click="handleAutoFix(result)"
                    >
                      自动修复
                    </el-button>

                    <el-button
                      type="text"
                      size="small"
                      @click="handleShowDetail(result)"
                    >
                      详情
                    </el-button>
                  </div>
                </div>

                <div class="result-content">
                  <div class="result-message">{{ result.message }}</div>

                  <div v-if="result.details" class="result-details">
                    <el-collapse v-model="activeDetails[index]">
                      <el-collapse-item name="detail">
                        <template #title>
                          <span class="detail-title">详细信息</span>
                        </template>

                        <div class="detail-content">
                          <div v-if="result.details.path" class="detail-item">
                            <label>配置路径:</label>
                            <code>{{ result.details.path }}</code>
                          </div>

                          <div
                            v-if="result.details.expected"
                            class="detail-item"
                          >
                            <label>期望值:</label>
                            <code>{{ result.details.expected }}</code>
                          </div>

                          <div v-if="result.details.actual" class="detail-item">
                            <label>实际值:</label>
                            <code>{{ result.details.actual }}</code>
                          </div>

                          <div
                            v-if="result.details.suggestion"
                            class="detail-item"
                          >
                            <label>修复建议:</label>
                            <div class="suggestion-text">
                              {{ result.details.suggestion }}
                            </div>
                          </div>
                        </div>
                      </el-collapse-item>
                    </el-collapse>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- 验证规则 -->
        <el-tab-pane label="验证规则" name="rules">
          <div class="validation-rules">
            <div class="rules-header">
              <el-button type="primary" size="small" @click="handleAddRule">
                添加规则
              </el-button>
              <el-button size="small" @click="handleImportRules">
                导入规则
              </el-button>
              <el-button size="small" @click="handleExportRules">
                导出规则
              </el-button>
            </div>

            <BaseTable
              :data="validationRules"
              :columns="ruleColumns"
              @action="handleRuleAction"
            />
          </div>
        </el-tab-pane>

        <!-- 验证历史 -->
        <el-tab-pane v-if="showHistory" label="验证历史" name="history">
          <div class="validation-history">
            <BaseTable
              :data="validationHistory"
              :columns="historyColumns"
              @action="handleHistoryAction"
            />
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <!-- 验证规则编辑对话框 -->
    <el-dialog
      v-model="ruleDialogVisible"
      :title="editingRule ? '编辑验证规则' : '添加验证规则'"
      width="600px"
      :before-close="handleRuleDialogClose"
    >
      <BaseForm
        ref="ruleFormRef"
        v-model="ruleForm"
        :fields="ruleFormFields"
        :rules="ruleFormRules"
        label-width="100px"
      />

      <template #footer>
        <el-button @click="ruleDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleRuleSave"> 保存 </el-button>
      </template>
    </el-dialog>

    <!-- 验证详情对话框 -->
    <el-dialog
      v-model="detailDialogVisible"
      title="验证详情"
      width="500px"
      :before-close="handleDetailDialogClose"
    >
      <div v-if="selectedResult" class="validation-detail">
        <el-descriptions border column="1">
          <el-descriptions-item label="验证项目">
            {{ selectedResult.title }}
          </el-descriptions-item>
          <el-descriptions-item label="验证类别">
            {{ selectedResult.category }}
          </el-descriptions-item>
          <el-descriptions-item label="验证状态">
            <StatusTag :status="selectedResult.type" />
          </el-descriptions-item>
          <el-descriptions-item label="错误信息">
            {{ selectedResult.message }}
          </el-descriptions-item>
          <el-descriptions-item
            v-if="selectedResult.details?.path"
            label="配置路径"
          >
            <code>{{ selectedResult.details.path }}</code>
          </el-descriptions-item>
          <el-descriptions-item
            v-if="selectedResult.details?.suggestion"
            label="修复建议"
          >
            {{ selectedResult.details.suggestion }}
          </el-descriptions-item>
        </el-descriptions>
      </div>

      <template #footer>
        <el-button @click="detailDialogVisible = false">关闭</el-button>
        <el-button
          v-if="selectedResult?.fixable"
          type="primary"
          @click="handleAutoFix(selectedResult)"
        >
          自动修复
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import {
  CircleCheck,
  CircleClose,
  Warning,
  MoreFilled,
  Download,
  Setting,
  Clock,
  Delete,
  InfoFilled,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, reactive, watch } from 'vue'

import { SearchBox, BaseTable, BaseForm, StatusTag } from '../base'

import { configApi } from '@/api'

export type ValidationType = 'success' | 'warning' | 'error'

export interface ValidationResult {
  id: string
  title: string
  category: string
  type: ValidationType
  message: string
  fixable?: boolean
  details?: {
    path?: string
    expected?: any
    actual?: any
    suggestion?: string
  }
}

export interface ValidationRule {
  id: string
  name: string
  category: string
  description: string
  enabled: boolean
  severity: 'error' | 'warning'
  validator: string
  config?: any
}

export interface ValidationHistoryRecord {
  id: string
  timestamp: Date
  totalCount: number
  passedCount: number
  warningCount: number
  errorCount: number
  duration: number
}

interface Props {
  modelValue?: any
  title?: string

  // 功能控制
  showActions?: boolean
  showSummary?: boolean
  showHistory?: boolean
  autoValidate?: boolean

  // 验证配置
  validationRules?: ValidationRule[]

  // 外观配置
  size?: 'small' | 'default' | 'large'

  // 自定义样式
  customClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: '配置验证',
  showActions: true,
  showSummary: true,
  showHistory: true,
  autoValidate: false,
  validationRules: () => [],
  size: 'default',
})

interface Emits {
  'update:modelValue': [value: any]
  validate: [config: any]
  'auto-fix': [result: ValidationResult]
  'rule-change': [rules: ValidationRule[]]
}

const emit = defineEmits<Emits>()

// 状态
const validating = ref(false)
const validationProgress = ref(0)
const validationProgressStatus = ref<'success' | 'exception' | 'warning' | ''>(
  ''
)
const progressText = ref('')
const activeTab = ref('results')
const filterType = ref('all')
const searchKeyword = ref('')
const activeDetails = ref<Record<number, string[]>>({})

// 对话框状态
const ruleDialogVisible = ref(false)
const detailDialogVisible = ref(false)
const editingRule = ref<ValidationRule | null>(null)
const selectedResult = ref<ValidationResult | null>(null)

// 验证结果
const validationResults = ref<ValidationResult[]>([])

// 验证规则
const validationRules = ref<ValidationRule[]>([
  {
    id: '1',
    name: '连接地址验证',
    category: '连接配置',
    description: '验证IP地址格式是否正确',
    enabled: true,
    severity: 'error',
    validator: 'ipAddress',
  },
  {
    id: '2',
    name: '端口范围验证',
    category: '连接配置',
    description: '验证端口号是否在有效范围内',
    enabled: true,
    severity: 'error',
    validator: 'portRange',
  },
  {
    id: '3',
    name: '超时时间验证',
    category: '高级配置',
    description: '验证超时时间是否合理',
    enabled: true,
    severity: 'warning',
    validator: 'timeoutValue',
  },
])

// 验证历史
const validationHistory = ref<ValidationHistoryRecord[]>([
  {
    id: '1',
    timestamp: new Date(Date.now() - 3600000),
    totalCount: 15,
    passedCount: 12,
    warningCount: 2,
    errorCount: 1,
    duration: 1250,
  },
])

// 规则表单
const ruleForm = reactive({
  name: '',
  category: '',
  description: '',
  enabled: true,
  severity: 'error' as 'error' | 'warning',
  validator: '',
})

const ruleFormRef = ref()

// 表格列定义
const ruleColumns = [
  { key: 'name', label: '规则名称', width: 150 },
  { key: 'category', label: '分类', width: 120 },
  { key: 'description', label: '描述' },
  { key: 'severity', label: '严重级别', width: 100, type: 'tag' },
  { key: 'enabled', label: '状态', width: 80, type: 'switch' },
  { key: 'actions', label: '操作', width: 120, type: 'action' },
]

const historyColumns = [
  { key: 'timestamp', label: '验证时间', type: 'datetime', width: 160 },
  { key: 'totalCount', label: '总计', width: 80 },
  { key: 'passedCount', label: '通过', width: 80 },
  { key: 'warningCount', label: '警告', width: 80 },
  { key: 'errorCount', label: '错误', width: 80 },
  { key: 'duration', label: '耗时(ms)', width: 100 },
  { key: 'actions', label: '操作', width: 100, type: 'action' },
]

// 规则表单字段
const ruleFormFields = [
  {
    key: 'name',
    label: '规则名称',
    type: 'text',
    required: true,
    placeholder: '输入规则名称',
  },
  {
    key: 'category',
    label: '分类',
    type: 'select',
    required: true,
    options: [
      { label: '连接配置', value: '连接配置' },
      { label: '协议配置', value: '协议配置' },
      { label: '安全配置', value: '安全配置' },
      { label: '高级配置', value: '高级配置' },
    ],
  },
  {
    key: 'description',
    label: '描述',
    type: 'textarea',
    placeholder: '输入规则描述',
  },
  {
    key: 'severity',
    label: '严重级别',
    type: 'select',
    required: true,
    options: [
      { label: '错误', value: 'error' },
      { label: '警告', value: 'warning' },
    ],
  },
  {
    key: 'validator',
    label: '验证器',
    type: 'select',
    required: true,
    options: [
      { label: 'IP地址验证', value: 'ipAddress' },
      { label: '端口范围验证', value: 'portRange' },
      { label: '超时时间验证', value: 'timeoutValue' },
      { label: '自定义验证', value: 'custom' },
    ],
  },
  {
    key: 'enabled',
    label: '启用',
    type: 'switch',
    defaultValue: true,
  },
]

const ruleFormRules = {
  name: [{ required: true, message: '请输入规则名称', trigger: 'blur' }],
  category: [{ required: true, message: '请选择分类', trigger: 'change' }],
  severity: [{ required: true, message: '请选择严重级别', trigger: 'change' }],
  validator: [{ required: true, message: '请选择验证器', trigger: 'change' }],
}

// 计算属性
const containerClass = computed(() => {
  const classes = []

  classes.push(`config-validation--${props.size}`)

  if (props.customClass) {
    classes.push(props.customClass)
  }

  return classes.join(' ')
})

const hasConfigToValidate = computed(() => {
  return props.modelValue && Object.keys(props.modelValue).length > 0
})

const hasValidationResults = computed(() => {
  return validationResults.value.length > 0
})

const totalValidations = computed(() => validationResults.value.length)

const passedValidations = computed(() => {
  return validationResults.value.filter(r => r.type === 'success').length
})

const warningValidations = computed(() => {
  return validationResults.value.filter(r => r.type === 'warning').length
})

const errorValidations = computed(() => {
  return validationResults.value.filter(r => r.type === 'error').length
})

const overallStatusType = computed(() => {
  if (errorValidations.value > 0) return 'danger'
  if (warningValidations.value > 0) return 'warning'
  if (passedValidations.value > 0) return 'success'
  return 'info'
})

const overallStatusText = computed(() => {
  if (!hasValidationResults.value) return '未验证'
  if (errorValidations.value > 0) return '验证失败'
  if (warningValidations.value > 0) return '有警告'
  return '验证通过'
})

const showValidationStatus = computed(() => {
  return hasValidationResults.value || validating.value
})

const filteredResults = computed(() => {
  let results = validationResults.value

  // 按类型筛选
  if (filterType.value !== 'all') {
    results = results.filter(r => r.type === filterType.value)
  }

  // 按关键词搜索
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase()
    results = results.filter(
      r =>
        r.title.toLowerCase().includes(keyword) ||
        r.message.toLowerCase().includes(keyword) ||
        r.category.toLowerCase().includes(keyword)
    )
  }

  return results
})

// 方法
const getResultIcon = (type: ValidationType) => {
  const iconMap = {
    success: CircleCheck,
    warning: Warning,
    error: CircleClose,
  }
  return iconMap[type] || InfoFilled
}

// 事件处理
const handleValidate = async () => {
  if (!hasConfigToValidate.value) {
    ElMessage.warning('没有可验证的配置')
    return
  }

  validating.value = true
  validationProgress.value = 0
  progressText.value = '开始验证...'
  const startTime = Date.now()

  try {
    progressText.value = '开始验证配置...'

    // 调用配置验证API
    const response = await configApi.validateConfig(props.modelValue)

    validationResults.value = response.results || []
    validationProgress.value = 100

    // 记录验证历史
    const historyRecord: ValidationHistoryRecord = {
      id: Date.now().toString(),
      timestamp: new Date(),
      totalCount: validationResults.value.length,
      passedCount: validationResults.value.filter(r => r.type === 'success')
        .length,
      warningCount: validationResults.value.filter(r => r.type === 'warning')
        .length,
      errorCount: validationResults.value.filter(r => r.type === 'error')
        .length,
      duration: Date.now() - startTime,
    }

    validationHistory.value.unshift(historyRecord)

    emit('validate', props.modelValue)

    ElMessage.success('验证完成')
  } catch (error: any) {
    console.error('配置验证失败:', error)
    ElMessage.error(error.message || '验证过程中发生错误')
  } finally {
    validating.value = false
    validationProgress.value = 100
    validationProgressStatus.value =
      errorValidations.value > 0 ? 'exception' : 'success'
  }
}

const handleAction = (command: string) => {
  switch (command) {
    case 'export':
      handleExportReport()
      break
    case 'rules':
      activeTab.value = 'rules'
      break
    case 'history':
      activeTab.value = 'history'
      break
    case 'clear':
      handleClearResults()
      break
  }
}

const handleExportReport = () => {
  // 导出验证报告
  ElMessage.success('验证报告导出成功')
}

const handleClearResults = () => {
  validationResults.value = []
  ElMessage.success('验证结果已清空')
}

const handleFilterChange = () => {
  // 筛选变化处理
}

const handleSearch = () => {
  // 搜索处理
}

const handleAutoFix = (result: ValidationResult) => {
  emit('auto-fix', result)
  ElMessage.success('已执行自动修复')
}

const handleShowDetail = (result: ValidationResult) => {
  selectedResult.value = result
  detailDialogVisible.value = true
}

const handleDetailDialogClose = () => {
  detailDialogVisible.value = false
  selectedResult.value = null
}

const handleAddRule = () => {
  editingRule.value = null
  Object.assign(ruleForm, {
    name: '',
    category: '',
    description: '',
    enabled: true,
    severity: 'error',
    validator: '',
  })
  ruleDialogVisible.value = true
}

const handleImportRules = () => {
  // 导入规则
  ElMessage.success('规则导入成功')
}

const handleExportRules = () => {
  // 导出规则
  ElMessage.success('规则导出成功')
}

const handleRuleAction = (action: string, row: ValidationRule) => {
  switch (action) {
    case 'edit':
      editingRule.value = row
      Object.assign(ruleForm, row)
      ruleDialogVisible.value = true
      break
    case 'delete':
      const index = validationRules.value.findIndex(r => r.id === row.id)
      if (index > -1) {
        validationRules.value.splice(index, 1)
        ElMessage.success('规则删除成功')
      }
      break
    case 'toggle':
      row.enabled = !row.enabled
      break
  }
}

const handleRuleDialogClose = () => {
  ruleDialogVisible.value = false
  editingRule.value = null
}

const handleRuleSave = async () => {
  const valid = await ruleFormRef.value?.validate()
  if (valid) {
    if (editingRule.value) {
      // 编辑规则
      Object.assign(editingRule.value, ruleForm)
    } else {
      // 添加规则
      const newRule: ValidationRule = {
        id: Date.now().toString(),
        ...ruleForm,
      }
      validationRules.value.push(newRule)
    }

    emit('rule-change', validationRules.value)
    ruleDialogVisible.value = false
    ElMessage.success('规则保存成功')
  }
}

const handleHistoryAction = (action: string, row: ValidationHistoryRecord) => {
  if (action === 'delete') {
    const index = validationHistory.value.findIndex(h => h.id === row.id)
    if (index > -1) {
      validationHistory.value.splice(index, 1)
      ElMessage.success('历史记录删除成功')
    }
  }
}

// 监听
watch(
  () => props.modelValue,
  () => {
    if (props.autoValidate && hasConfigToValidate.value) {
      handleValidate()
    }
  },
  { deep: true }
)
</script>

<style scoped lang="scss">
.config-validation {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 6px;

  &.config-validation--small {
    .validation-header {
      padding: 8px 12px;
    }
  }

  &.config-validation--large {
    .validation-header {
      padding: 16px 20px;
    }
  }
}

.validation-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);

  .header-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    color: var(--el-text-color-primary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
}

.validation-progress {
  padding: 16px;

  .progress-text {
    font-size: 12px;
    color: var(--el-text-color-secondary);
  }
}

.validation-summary {
  padding: 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.validation-content {
  padding: 16px;

  :deep(.el-tabs__content) {
    padding-top: 16px;
  }
}

.validation-results {
  .results-filter {
    display: flex;
    align-items: center;
    margin-bottom: 16px;
  }

  .results-list {
    .result-item {
      border: 1px solid var(--el-border-color);
      border-radius: 6px;
      margin-bottom: 12px;
      overflow: hidden;

      &.result-item--success {
        border-color: var(--el-color-success-light-5);
        background: var(--el-color-success-light-9);
      }

      &.result-item--warning {
        border-color: var(--el-color-warning-light-5);
        background: var(--el-color-warning-light-9);
      }

      &.result-item--error {
        border-color: var(--el-color-danger-light-5);
        background: var(--el-color-danger-light-9);
      }

      .result-header {
        display: flex;
        align-items: center;
        padding: 12px 16px;

        .result-icon {
          margin-right: 12px;

          &.result-item--success .el-icon {
            color: var(--el-color-success);
          }

          &.result-item--warning .el-icon {
            color: var(--el-color-warning);
          }

          &.result-item--error .el-icon {
            color: var(--el-color-danger);
          }
        }

        .result-info {
          flex: 1;

          .result-title {
            font-size: 14px;
            font-weight: 500;
            color: var(--el-text-color-primary);
          }

          .result-category {
            font-size: 12px;
            color: var(--el-text-color-secondary);
            margin-top: 2px;
          }
        }

        .result-status {
          margin-right: 12px;
        }

        .result-actions {
          display: flex;
          gap: 8px;
        }
      }

      .result-content {
        padding: 0 16px 12px 40px;

        .result-message {
          font-size: 13px;
          color: var(--el-text-color-regular);
          margin-bottom: 8px;
        }

        .result-details {
          .detail-title {
            font-size: 12px;
            color: var(--el-text-color-secondary);
          }

          .detail-content {
            .detail-item {
              display: flex;
              align-items: flex-start;
              margin-bottom: 8px;

              label {
                font-size: 12px;
                color: var(--el-text-color-secondary);
                width: 80px;
                flex-shrink: 0;
              }

              code {
                background: var(--el-fill-color-light);
                padding: 2px 4px;
                border-radius: 2px;
                font-size: 11px;
              }

              .suggestion-text {
                font-size: 12px;
                color: var(--el-text-color-regular);
                line-height: 1.4;
              }
            }
          }
        }
      }
    }
  }
}

.validation-rules {
  .rules-header {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }
}

.validation-detail {
  .el-descriptions {
    :deep(
      .el-descriptions__body .el-descriptions__table .el-descriptions__cell
    ) {
      padding: 8px 12px;
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .config-validation {
    .validation-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .results-filter {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .result-item {
      .result-header {
        flex-direction: column;
        align-items: flex-start;
        gap: 8px;
      }
    }
  }
}
</style>
