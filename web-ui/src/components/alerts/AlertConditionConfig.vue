<template>
  <div class="alert-condition-config">
    <!-- 条件列表 -->
    <div class="conditions-list">
      <div
        v-for="(condition, index) in conditions"
        :key="condition.id"
        class="condition-item"
        :class="{ 'has-error': condition.hasError }"
      >
        <div class="condition-header">
          <div class="condition-title">
            <span class="condition-index">条件 {{ index + 1 }}</span>
            <el-tag
              v-if="condition.type"
              :type="getConditionTypeTag(condition.type)"
              size="small"
            >
              {{ getConditionTypeText(condition.type) }}
            </el-tag>
          </div>

          <div class="condition-actions">
            <el-tooltip content="复制条件">
              <el-button type="link" size="small" @click="copyCondition(index)">
                <el-icon><CopyDocument /></el-icon>
              </el-button>
            </el-tooltip>

            <el-tooltip content="删除条件">
              <el-button
                type="link"
                size="small"
                :disabled="conditions.length <= 1"
                @click="removeCondition(index)"
              >
                <el-icon><Delete /></el-icon>
              </el-button>
            </el-tooltip>
          </div>
        </div>

        <div class="condition-config">
          <!-- 条件类型选择 -->
          <div class="config-row">
            <div class="config-item">
              <label class="config-label">条件类型</label>
              <el-select
                v-model="condition.type"
                placeholder="选择条件类型"
                @change="handleConditionTypeChange(index)"
              >
                <el-option
                  v-for="type in conditionTypes"
                  :key="type.value"
                  :label="type.label"
                  :value="type.value"
                >
                  <div class="type-option">
                    <span class="type-name">{{ type.label }}</span>
                    <span class="type-desc">{{ type.description }}</span>
                  </div>
                </el-option>
              </el-select>
            </div>
          </div>

          <!-- 阈值条件配置 -->
          <div v-if="condition.type === 'threshold'" class="threshold-config">
            <div class="config-row">
              <div class="config-item">
                <label class="config-label">监控字段</label>
                <el-select
                  v-model="condition.field"
                  placeholder="选择要监控的字段"
                  @change="handleFieldChange(index)"
                >
                  <el-option
                    v-for="field in getAvailableFields()"
                    :key="field.value"
                    :label="field.label"
                    :value="field.value"
                  >
                    <div class="field-option">
                      <span class="field-name">{{ field.label }}</span>
                      <span class="field-type">{{ field.dataType }}</span>
                      <span v-if="field.unit" class="field-unit"
                        >({{ field.unit }})</span
                      >
                    </div>
                  </el-option>
                </el-select>
              </div>

              <div class="config-item">
                <label class="config-label">比较运算符</label>
                <el-select
                  v-model="condition.operator"
                  placeholder="选择运算符"
                >
                  <el-option
                    v-for="op in getOperatorOptions(condition.field)"
                    :key="op.value"
                    :label="op.label"
                    :value="op.value"
                  >
                    <div class="operator-option">
                      <span class="operator-symbol">{{ op.symbol }}</span>
                      <span class="operator-desc">{{ op.description }}</span>
                    </div>
                  </el-option>
                </el-select>
              </div>

              <div class="config-item">
                <label class="config-label">阈值</label>
                <div class="threshold-input">
                  <el-input-number
                    v-if="isNumericField(condition.field)"
                    v-model="condition.value"
                    :precision="getFieldPrecision(condition.field)"
                    :step="getFieldStep(condition.field)"
                    placeholder="输入阈值"
                  />
                  <el-input
                    v-else-if="isStringField(condition.field)"
                    v-model="condition.value"
                    placeholder="输入文本值"
                  />
                  <el-select
                    v-else-if="isBooleanField(condition.field)"
                    v-model="condition.value"
                    placeholder="选择布尔值"
                  >
                    <el-option label="True" value="true" />
                    <el-option label="False" value="false" />
                  </el-select>
                  <span
                    v-if="getFieldUnit(condition.field)"
                    class="threshold-unit"
                  >
                    {{ getFieldUnit(condition.field) }}
                  </span>
                </div>
              </div>
            </div>

            <!-- 高级阈值选项 -->
            <div class="advanced-threshold">
              <el-checkbox v-model="condition.useHysteresis">
                启用滞回（防止抖动）
              </el-checkbox>

              <div v-if="condition.useHysteresis" class="hysteresis-config">
                <div class="config-item">
                  <label class="config-label">滞回值</label>
                  <el-input-number
                    v-model="condition.hysteresis"
                    :precision="getFieldPrecision(condition.field)"
                    :min="0"
                    placeholder="滞回值"
                  />
                  <span class="hysteresis-tip">
                    恢复阈值 = 触发阈值 ± 滞回值
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- 变化率条件配置 -->
          <div v-else-if="condition.type === 'rate'" class="rate-config">
            <div class="config-row">
              <div class="config-item">
                <label class="config-label">监控字段</label>
                <el-select v-model="condition.field" placeholder="选择字段">
                  <el-option
                    v-for="field in getNumericFields()"
                    :key="field.value"
                    :label="field.label"
                    :value="field.value"
                  />
                </el-select>
              </div>

              <div class="config-item">
                <label class="config-label">变化类型</label>
                <el-select
                  v-model="condition.rateType"
                  placeholder="选择变化类型"
                >
                  <el-option label="增长率" value="increase" />
                  <el-option label="下降率" value="decrease" />
                  <el-option label="变化率（绝对值）" value="absolute" />
                </el-select>
              </div>

              <div class="config-item">
                <label class="config-label">时间窗口</label>
                <div class="time-window-input">
                  <el-input-number
                    v-model="condition.timeWindow.value"
                    :min="1"
                    :max="3600"
                    placeholder="时间值"
                  />
                  <el-select
                    v-model="condition.timeWindow.unit"
                    style="width: 80px"
                  >
                    <el-option label="秒" value="seconds" />
                    <el-option label="分钟" value="minutes" />
                    <el-option label="小时" value="hours" />
                  </el-select>
                </div>
              </div>

              <div class="config-item">
                <label class="config-label">变化阈值</label>
                <div class="rate-threshold">
                  <el-input-number
                    v-model="condition.rateThreshold"
                    :precision="2"
                    placeholder="变化阈值"
                  />
                  <el-select v-model="condition.rateUnit" style="width: 100px">
                    <el-option label="%" value="percent" />
                    <el-option label="绝对值" value="absolute" />
                  </el-select>
                </div>
              </div>
            </div>
          </div>

          <!-- 范围条件配置 -->
          <div v-else-if="condition.type === 'range'" class="range-config">
            <div class="config-row">
              <div class="config-item">
                <label class="config-label">监控字段</label>
                <el-select v-model="condition.field" placeholder="选择字段">
                  <el-option
                    v-for="field in getNumericFields()"
                    :key="field.value"
                    :label="field.label"
                    :value="field.value"
                  />
                </el-select>
              </div>

              <div class="config-item">
                <label class="config-label">范围类型</label>
                <el-radio-group v-model="condition.rangeType">
                  <el-radio label="within">在范围内</el-radio>
                  <el-radio label="outside">在范围外</el-radio>
                </el-radio-group>
              </div>

              <div class="config-item">
                <label class="config-label">最小值</label>
                <el-input-number
                  v-model="condition.minValue"
                  :precision="getFieldPrecision(condition.field)"
                  placeholder="最小值"
                />
              </div>

              <div class="config-item">
                <label class="config-label">最大值</label>
                <el-input-number
                  v-model="condition.maxValue"
                  :precision="getFieldPrecision(condition.field)"
                  placeholder="最大值"
                />
              </div>
            </div>
          </div>

          <!-- 状态条件配置 -->
          <div v-else-if="condition.type === 'status'" class="status-config">
            <div class="config-row">
              <div class="config-item">
                <label class="config-label">监控对象</label>
                <el-select
                  v-model="condition.target"
                  placeholder="选择监控对象"
                >
                  <el-option label="设备状态" value="device_status" />
                  <el-option label="连接状态" value="connection_status" />
                  <el-option label="数据质量" value="data_quality" />
                  <el-option label="通信错误" value="communication_error" />
                </el-select>
              </div>

              <div class="config-item">
                <label class="config-label">期望状态</label>
                <el-select
                  v-model="condition.expectedStatus"
                  placeholder="选择状态"
                >
                  <el-option
                    v-for="status in getStatusOptions(condition.target)"
                    :key="status.value"
                    :label="status.label"
                    :value="status.value"
                  />
                </el-select>
              </div>

              <div class="config-item">
                <label class="config-label">持续时间</label>
                <div class="duration-input">
                  <el-input-number
                    v-model="condition.duration.value"
                    :min="1"
                    :max="3600"
                    placeholder="持续时间"
                  />
                  <el-select
                    v-model="condition.duration.unit"
                    style="width: 80px"
                  >
                    <el-option label="秒" value="seconds" />
                    <el-option label="分钟" value="minutes" />
                    <el-option label="小时" value="hours" />
                  </el-select>
                </div>
              </div>
            </div>
          </div>

          <!-- 表达式条件配置 -->
          <div
            v-else-if="condition.type === 'expression'"
            class="expression-config"
          >
            <div class="config-row">
              <div class="config-item full-width">
                <label class="config-label">表达式</label>
                <el-input
                  v-model="condition.expression"
                  type="textarea"
                  :rows="3"
                  placeholder="输入JavaScript表达式，例如: value > 100 && quality === 'good'"
                />
                <div class="expression-help">
                  <span>可用变量: value, quality, timestamp, device, tag</span>
                  <el-button
                    type="link"
                    size="small"
                    @click="showExpressionHelp"
                  >
                    查看帮助
                  </el-button>
                </div>
              </div>
            </div>

            <div class="config-row">
              <div class="config-item">
                <label class="config-label">表达式验证</label>
                <el-button @click="validateExpression(index)"
                  >验证表达式</el-button
                >
                <div
                  v-if="condition.validationResult"
                  class="validation-result"
                >
                  <el-tag
                    :type="
                      condition.validationResult.valid ? 'success' : 'danger'
                    "
                    size="small"
                  >
                    {{
                      condition.validationResult.valid
                        ? '表达式有效'
                        : '表达式无效'
                    }}
                  </el-tag>
                  <span
                    v-if="condition.validationResult.error"
                    class="validation-error"
                  >
                    {{ condition.validationResult.error }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- 条件预览 -->
          <div class="condition-preview">
            <div class="preview-title">条件预览:</div>
            <div class="preview-content">
              {{ getConditionPreview(condition) }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加条件按钮 -->
    <div class="add-condition">
      <el-button @click="addCondition">
        <el-icon><Plus /></el-icon>
        添加条件
      </el-button>
    </div>

    <!-- 条件逻辑设置 -->
    <div v-if="conditions.length > 1" class="condition-logic">
      <div class="logic-title">条件逻辑关系</div>
      <el-radio-group v-model="logicOperator">
        <el-radio label="AND"> 所有条件都满足时触发（AND） </el-radio>
        <el-radio label="OR"> 任一条件满足时触发（OR） </el-radio>
      </el-radio-group>

      <div class="logic-preview">
        <span class="logic-text">
          {{ getLogicPreview() }}
        </span>
      </div>
    </div>

    <!-- 表达式帮助对话框 -->
    <el-dialog v-model="showHelpDialog" title="表达式帮助" width="600px">
      <div class="expression-help-content">
        <h4>可用变量</h4>
        <ul>
          <li><code>value</code> - 当前数据值</li>
          <li><code>quality</code> - 数据质量 ('good', 'uncertain', 'bad')</li>
          <li><code>timestamp</code> - 数据时间戳</li>
          <li><code>device</code> - 设备信息对象</li>
          <li><code>tag</code> - 标签信息对象</li>
        </ul>

        <h4>示例表达式</h4>
        <div class="example-expressions">
          <div class="example-item">
            <code>value > 100</code>
            <span>数值大于100</span>
          </div>
          <div class="example-item">
            <code>value > 80 && quality === 'good'</code>
            <span>数值大于80且数据质量良好</span>
          </div>
          <div class="example-item">
            <code>Math.abs(value - 50) > 10</code>
            <span>数值偏离50超过10</span>
          </div>
          <div class="example-item">
            <code>device.status === 'offline'</code>
            <span>设备离线</span>
          </div>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * AlertConditionConfig —— 报警条件配置组件
 *
 * 📝 Responsibilities:
 *  1. 多种类型的报警条件配置
 *  2. 阈值、变化率、范围、状态条件支持
 *  3. 自定义表达式条件
 *  4. 条件验证和预览
 *  5. 条件逻辑关系设置
 *
 * 📦 Dependencies:
 *  - Element Plus 表单组件
 *  - 表达式验证工具
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { Plus, Delete, CopyDocument } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, watch } from 'vue'

// ===== Props & Emits =====
const props = defineProps<{
  modelValue: any[]
  availableTags: any[]
  scope: string
  selectedDevices: string[]
  selectedTags: string[]
}>()

const emit = defineEmits<{
  'update:modelValue': [conditions: any[]]
}>()

// ===== 响应式数据 =====
const conditions = ref<any[]>(props.modelValue || [])
const logicOperator = ref('AND')
const showHelpDialog = ref(false)

// 条件类型选项
const conditionTypes = [
  {
    value: 'threshold',
    label: '阈值条件',
    description: '数值达到指定阈值时触发',
  },
  {
    value: 'rate',
    label: '变化率条件',
    description: '数值变化率超过阈值时触发',
  },
  {
    value: 'range',
    label: '范围条件',
    description: '数值在指定范围内或外时触发',
  },
  {
    value: 'status',
    label: '状态条件',
    description: '设备或连接状态变化时触发',
  },
  {
    value: 'expression',
    label: '表达式条件',
    description: '自定义JavaScript表达式',
  },
]

// ===== 计算属性 =====
const availableFields = computed(() => {
  let fields: any[] = []

  if (props.scope === 'device' && props.selectedDevices.length > 0) {
    // 基于选中设备的标签
    fields = props.availableTags
      .filter(tag => props.selectedDevices.includes(tag.deviceId))
      .map(tag => ({
        value: tag.id,
        label: tag.name,
        dataType: tag.dataType,
        unit: tag.unit,
        deviceName: tag.deviceName,
      }))
  } else if (props.scope === 'tag' && props.selectedTags.length > 0) {
    // 选中的特定标签
    fields = props.availableTags
      .filter(tag => props.selectedTags.includes(tag.id))
      .map(tag => ({
        value: tag.id,
        label: tag.name,
        dataType: tag.dataType,
        unit: tag.unit,
        deviceName: tag.deviceName,
      }))
  } else {
    // 所有可用标签
    fields = props.availableTags.map(tag => ({
      value: tag.id,
      label: tag.name,
      dataType: tag.dataType,
      unit: tag.unit,
      deviceName: tag.deviceName,
    }))
  }

  return fields
})

// ===== 方法 =====

/**
 * 初始化条件
 */
function initializeConditions() {
  if (conditions.value.length === 0) {
    addCondition()
  }
}

/**
 * 添加条件
 */
function addCondition() {
  const newCondition = {
    id: Date.now().toString(),
    type: 'threshold',
    field: '',
    operator: 'gt',
    value: null,
    useHysteresis: false,
    hysteresis: 0,
    rateType: 'increase',
    timeWindow: { value: 5, unit: 'minutes' },
    rateThreshold: 10,
    rateUnit: 'percent',
    rangeType: 'outside',
    minValue: null,
    maxValue: null,
    target: 'device_status',
    expectedStatus: 'offline',
    duration: { value: 30, unit: 'seconds' },
    expression: '',
    validationResult: null,
    hasError: false,
  }

  conditions.value.push(newCondition)
  updateModelValue()
}

/**
 * 移除条件
 */
function removeCondition(index: number) {
  conditions.value.splice(index, 1)
  updateModelValue()
}

/**
 * 复制条件
 */
function copyCondition(index: number) {
  const originalCondition = conditions.value[index]
  const copiedCondition = {
    ...originalCondition,
    id: Date.now().toString(),
  }
  conditions.value.splice(index + 1, 0, copiedCondition)
  updateModelValue()
}

/**
 * 处理条件类型变化
 */
function handleConditionTypeChange(index: number) {
  const condition = conditions.value[index]

  // 重置条件特定的字段
  condition.field = ''
  condition.operator = 'gt'
  condition.value = null
  condition.hasError = false

  updateModelValue()
}

/**
 * 处理字段变化
 */
function handleFieldChange(index: number) {
  const condition = conditions.value[index]
  const field = availableFields.value.find(f => f.value === condition.field)

  if (field) {
    // 根据字段类型设置默认运算符
    if (field.dataType === 'boolean') {
      condition.operator = 'eq'
    } else if (field.dataType === 'string') {
      condition.operator = 'eq'
    } else {
      condition.operator = 'gt'
    }
  }

  updateModelValue()
}

/**
 * 获取可用字段
 */
function getAvailableFields() {
  return availableFields.value
}

/**
 * 获取数值字段
 */
function getNumericFields() {
  return availableFields.value.filter(
    field => field.dataType === 'float' || field.dataType === 'integer'
  )
}

/**
 * 获取运算符选项
 */
function getOperatorOptions(fieldId: string) {
  const field = availableFields.value.find(f => f.value === fieldId)

  if (!field) {
    return []
  }

  const commonOperators = [
    { value: 'eq', label: '等于', symbol: '=', description: '等于指定值' },
    { value: 'ne', label: '不等于', symbol: '≠', description: '不等于指定值' },
  ]

  if (field.dataType === 'float' || field.dataType === 'integer') {
    return [
      ...commonOperators,
      { value: 'gt', label: '大于', symbol: '>', description: '大于指定值' },
      {
        value: 'gte',
        label: '大于等于',
        symbol: '≥',
        description: '大于或等于指定值',
      },
      { value: 'lt', label: '小于', symbol: '<', description: '小于指定值' },
      {
        value: 'lte',
        label: '小于等于',
        symbol: '≤',
        description: '小于或等于指定值',
      },
    ]
  } else if (field.dataType === 'string') {
    return [
      ...commonOperators,
      {
        value: 'contains',
        label: '包含',
        symbol: '⊇',
        description: '包含指定文本',
      },
      {
        value: 'starts_with',
        label: '开始于',
        symbol: '▶',
        description: '以指定文本开始',
      },
      {
        value: 'ends_with',
        label: '结束于',
        symbol: '◀',
        description: '以指定文本结束',
      },
      {
        value: 'regex',
        label: '正则匹配',
        symbol: '~',
        description: '匹配正则表达式',
      },
    ]
  } else {
    return commonOperators
  }
}

/**
 * 获取状态选项
 */
function getStatusOptions(target: string) {
  const statusMap: Record<string, any[]> = {
    device_status: [
      { value: 'online', label: '在线' },
      { value: 'offline', label: '离线' },
      { value: 'error', label: '错误' },
    ],
    connection_status: [
      { value: 'connected', label: '已连接' },
      { value: 'disconnected', label: '已断开' },
      { value: 'connecting', label: '连接中' },
    ],
    data_quality: [
      { value: 'good', label: '良好' },
      { value: 'uncertain', label: '可疑' },
      { value: 'bad', label: '错误' },
    ],
    communication_error: [
      { value: 'timeout', label: '超时' },
      { value: 'protocol_error', label: '协议错误' },
      { value: 'network_error', label: '网络错误' },
    ],
  }

  return statusMap[target] || []
}

/**
 * 检查字段类型
 */
function isNumericField(fieldId: string): boolean {
  const field = availableFields.value.find(f => f.value === fieldId)
  return field && (field.dataType === 'float' || field.dataType === 'integer')
}

function isStringField(fieldId: string): boolean {
  const field = availableFields.value.find(f => f.value === fieldId)
  return field && field.dataType === 'string'
}

function isBooleanField(fieldId: string): boolean {
  const field = availableFields.value.find(f => f.value === fieldId)
  return field && field.dataType === 'boolean'
}

/**
 * 获取字段属性
 */
function getFieldPrecision(fieldId: string): number {
  const field = availableFields.value.find(f => f.value === fieldId)
  return field && field.dataType === 'float' ? 2 : 0
}

function getFieldStep(fieldId: string): number {
  const field = availableFields.value.find(f => f.value === fieldId)
  return field && field.dataType === 'float' ? 0.01 : 1
}

function getFieldUnit(fieldId: string): string {
  const field = availableFields.value.find(f => f.value === fieldId)
  return field?.unit || ''
}

/**
 * 获取条件类型标签
 */
function getConditionTypeTag(type: string): string {
  const typeMap: Record<string, string> = {
    threshold: 'primary',
    rate: 'warning',
    range: 'success',
    status: 'info',
    expression: 'danger',
  }
  return typeMap[type] || 'info'
}

/**
 * 获取条件类型文本
 */
function getConditionTypeText(type: string): string {
  const condition = conditionTypes.find(t => t.value === type)
  return condition?.label || type
}

/**
 * 验证表达式
 */
function validateExpression(index: number) {
  const condition = conditions.value[index]

  try {
    // 创建测试上下文
    const testContext = {
      value: 100,
      quality: 'good',
      timestamp: new Date().toISOString(),
      device: { status: 'online', name: 'test-device' },
      tag: { name: 'test-tag', type: 'float' },
    }

    // 创建函数来评估表达式
    const func = new Function(
      'value',
      'quality',
      'timestamp',
      'device',
      'tag',
      'Math',
      `return ${condition.expression}`
    )

    // 测试表达式
    const result = func(
      testContext.value,
      testContext.quality,
      testContext.timestamp,
      testContext.device,
      testContext.tag,
      Math
    )

    condition.validationResult = {
      valid: typeof result === 'boolean',
      error: typeof result !== 'boolean' ? '表达式必须返回布尔值' : null,
    }

    if (condition.validationResult.valid) {
      ElMessage.success('表达式验证通过')
      condition.hasError = false
    } else {
      ElMessage.warning('表达式必须返回布尔值')
      condition.hasError = true
    }
  } catch (error) {
    condition.validationResult = {
      valid: false,
      error: (error as Error).message,
    }
    condition.hasError = true
    ElMessage.error('表达式语法错误')
  }

  updateModelValue()
}

/**
 * 显示表达式帮助
 */
function showExpressionHelp() {
  showHelpDialog.value = true
}

/**
 * 获取条件预览
 */
function getConditionPreview(condition: any): string {
  if (!condition.type) {
    return '请配置条件类型'
  }

  const field = availableFields.value.find(f => f.value === condition.field)
  const fieldName = field?.label || '未选择字段'

  switch (condition.type) {
    case 'threshold':
      if (!condition.field || condition.value === null) {
        return '请完成阈值条件配置'
      }
      const operator = getOperatorOptions(condition.field).find(
        op => op.value === condition.operator
      )
      const operatorSymbol = operator?.symbol || condition.operator
      const unit = getFieldUnit(condition.field)
      return `当 ${fieldName} ${operatorSymbol} ${condition.value}${unit} 时触发`

    case 'rate':
      if (!condition.field) {
        return '请完成变化率条件配置'
      }
      const rateTypeText =
        {
          increase: '增长率',
          decrease: '下降率',
          absolute: '变化率',
        }[condition.rateType] || '变化率'
      return `当 ${fieldName} ${rateTypeText} > ${condition.rateThreshold}${condition.rateUnit === 'percent' ? '%' : ''} (${condition.timeWindow.value}${condition.timeWindow.unit}) 时触发`

    case 'range':
      if (
        !condition.field ||
        condition.minValue === null ||
        condition.maxValue === null
      ) {
        return '请完成范围条件配置'
      }
      const rangeTypeText = condition.rangeType === 'within' ? '在' : '不在'
      return `当 ${fieldName} ${rangeTypeText} [${condition.minValue}, ${condition.maxValue}] 范围内时触发`

    case 'status':
      if (!condition.target || !condition.expectedStatus) {
        return '请完成状态条件配置'
      }
      const targetText =
        {
          device_status: '设备状态',
          connection_status: '连接状态',
          data_quality: '数据质量',
          communication_error: '通信错误',
        }[condition.target] || condition.target
      const statusOption = getStatusOptions(condition.target).find(
        s => s.value === condition.expectedStatus
      )
      const statusText = statusOption?.label || condition.expectedStatus
      return `当 ${targetText} 为 ${statusText} 持续 ${condition.duration.value}${condition.duration.unit} 时触发`

    case 'expression':
      if (!condition.expression) {
        return '请输入表达式'
      }
      return `当表达式 "${condition.expression}" 为真时触发`

    default:
      return '未知条件类型'
  }
}

/**
 * 获取逻辑预览
 */
function getLogicPreview(): string {
  if (conditions.value.length <= 1) {
    return ''
  }

  const conditionPreviews = conditions.value.map(
    (condition, index) => `条件${index + 1}`
  )

  if (logicOperator.value === 'AND') {
    return `${conditionPreviews.join(' 且 ')} 都满足时触发`
  } else {
    return `${conditionPreviews.join(' 或 ')} 任一满足时触发`
  }
}

/**
 * 更新模型值
 */
function updateModelValue() {
  emit('update:modelValue', conditions.value)
}

// ===== 监听器 =====
watch(
  () => props.modelValue,
  newValue => {
    conditions.value = newValue || []
  },
  { deep: true }
)

watch(
  conditions,
  () => {
    updateModelValue()
  },
  { deep: true }
)

watch(logicOperator, () => {
  updateModelValue()
})

// ===== 初始化 =====
initializeConditions()
</script>

<style scoped lang="scss">
.alert-condition-config {
  .conditions-list {
    .condition-item {
      border: 1px solid #ebeef5;
      border-radius: 6px;
      margin-bottom: 16px;
      background: white;
      transition: border-color 0.2s;

      &.has-error {
        border-color: #f56c6c;
      }

      .condition-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 12px 16px;
        background: #f8f9fa;
        border-bottom: 1px solid #ebeef5;
        border-radius: 6px 6px 0 0;

        .condition-title {
          display: flex;
          align-items: center;
          gap: 8px;

          .condition-index {
            font-weight: 600;
            color: #303133;
          }
        }

        .condition-actions {
          display: flex;
          gap: 4px;
        }
      }

      .condition-config {
        padding: 16px;

        .config-row {
          display: flex;
          gap: 16px;
          margin-bottom: 16px;
          flex-wrap: wrap;

          &:last-child {
            margin-bottom: 0;
          }

          .config-item {
            flex: 1;
            min-width: 200px;

            &.full-width {
              flex: 100%;
            }

            .config-label {
              display: block;
              font-size: 13px;
              color: #606266;
              margin-bottom: 6px;
              font-weight: 500;
            }
          }
        }

        .type-option,
        .field-option,
        .operator-option {
          display: flex;
          justify-content: space-between;
          align-items: center;
          width: 100%;

          .type-name,
          .field-name,
          .operator-symbol {
            font-weight: 500;
          }

          .type-desc,
          .field-type,
          .field-unit,
          .operator-desc {
            font-size: 12px;
            color: #909399;
          }
        }

        .threshold-input,
        .time-window-input,
        .rate-threshold,
        .duration-input {
          display: flex;
          align-items: center;
          gap: 8px;

          .threshold-unit,
          .hysteresis-tip {
            font-size: 12px;
            color: #909399;
            white-space: nowrap;
          }
        }

        .advanced-threshold {
          margin-top: 12px;

          .hysteresis-config {
            margin-top: 8px;
            padding: 12px;
            background: #f8f9fa;
            border-radius: 4px;
          }
        }

        .expression-help {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-top: 6px;
          font-size: 12px;
          color: #909399;
        }

        .validation-result {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-top: 8px;

          .validation-error {
            font-size: 12px;
            color: #f56c6c;
          }
        }

        .condition-preview {
          margin-top: 16px;
          padding: 12px;
          background: #f0f8ff;
          border-radius: 4px;
          border-left: 3px solid #409eff;

          .preview-title {
            font-size: 13px;
            color: #606266;
            margin-bottom: 4px;
          }

          .preview-content {
            font-size: 14px;
            color: #303133;
            font-weight: 500;
          }
        }
      }
    }
  }

  .add-condition {
    text-align: center;
    margin: 16px 0;
  }

  .condition-logic {
    margin-top: 24px;
    padding: 16px;
    background: #f8f9fa;
    border-radius: 6px;

    .logic-title {
      font-size: 14px;
      font-weight: 600;
      color: #303133;
      margin-bottom: 12px;
    }

    .logic-preview {
      margin-top: 12px;
      padding: 8px 12px;
      background: white;
      border-radius: 4px;
      border: 1px solid #dcdfe6;

      .logic-text {
        font-size: 13px;
        color: #606266;
        font-style: italic;
      }
    }
  }
}

.expression-help-content {
  h4 {
    color: #303133;
    margin: 16px 0 8px 0;

    &:first-child {
      margin-top: 0;
    }
  }

  ul {
    margin: 0 0 16px 20px;

    li {
      margin-bottom: 4px;

      code {
        background: #f1f1f1;
        padding: 2px 4px;
        border-radius: 2px;
        font-family: monospace;
      }
    }
  }

  .example-expressions {
    .example-item {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 8px 12px;
      margin-bottom: 8px;
      background: #f8f9fa;
      border-radius: 4px;

      code {
        background: #e6f7ff;
        color: #1890ff;
        padding: 4px 8px;
        border-radius: 4px;
        font-family: monospace;
        font-weight: 500;
      }

      span {
        font-size: 13px;
        color: #606266;
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .alert-condition-config {
    .conditions-list .condition-item .condition-config .config-row {
      flex-direction: column;
      gap: 12px;

      .config-item {
        min-width: auto;
      }
    }

    .threshold-input,
    .time-window-input,
    .rate-threshold,
    .duration-input {
      flex-wrap: wrap;
      gap: 4px;
    }
  }
}
</style>
