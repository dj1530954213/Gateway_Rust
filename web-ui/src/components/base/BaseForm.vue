<template>
  <el-form
    ref="formRef"
    :model="formData"
    :rules="formRules"
    :label-width="labelWidth"
    :label-position="labelPosition"
    :size="size"
    :disabled="disabled"
    :validate-on-rule-change="validateOnRuleChange"
    :hide-required-asterisk="hideRequiredAsterisk"
    :show-message="showMessage"
    :inline-message="inlineMessage"
    :status-icon="statusIcon"
    v-bind="$attrs"
    @validate="handleValidate"
  >
    <template v-for="field in processedFields" :key="field.key">
      <!-- 分组标题 -->
      <div v-if="field.type === 'group'" class="form-group">
        <h4 class="form-group-title">{{ field.label }}</h4>
        <el-divider v-if="field.divider !== false" />
      </div>

      <!-- 常规表单项 -->
      <el-form-item
        v-else
        :prop="field.key"
        :label="field.label"
        :label-width="field.labelWidth"
        :required="field.required"
        :rules="getFieldRules(field)"
        :error="getFieldError(field.key)"
        :show-message="field.showMessage !== false"
        :inline-message="field.inlineMessage"
        :size="field.size || size"
        :class="field.className"
      >
        <!-- 输入框 -->
        <el-input
          v-if="
            field.type === 'text' ||
            field.type === 'password' ||
            field.type === 'textarea'
          "
          v-model="formData[field.key]"
          :type="field.type === 'password' ? 'password' : 'text'"
          :placeholder="field.placeholder"
          :disabled="isFieldDisabled(field)"
          :readonly="field.readonly"
          :clearable="field.clearable !== false"
          :show-password="field.type === 'password'"
          :rows="field.rows"
          :autosize="field.autosize"
          :prefix-icon="field.prefixIcon"
          :suffix-icon="field.suffixIcon"
          :maxlength="field.maxlength"
          :show-word-limit="field.showWordLimit"
          @input="handleFieldChange(field.key, $event)"
          @blur="handleFieldBlur(field.key)"
          @focus="handleFieldFocus(field.key)"
        />

        <!-- 数字输入框 -->
        <el-input-number
          v-else-if="field.type === 'number'"
          v-model="formData[field.key]"
          :placeholder="field.placeholder"
          :disabled="isFieldDisabled(field)"
          :readonly="field.readonly"
          :min="field.min"
          :max="field.max"
          :step="field.step"
          :precision="field.precision"
          :controls="field.controls !== false"
          :controls-position="field.controlsPosition"
          @change="handleFieldChange(field.key, $event)"
        />

        <!-- 选择器 -->
        <el-select
          v-else-if="field.type === 'select' || field.type === 'multiSelect'"
          v-model="formData[field.key]"
          :placeholder="field.placeholder"
          :disabled="isFieldDisabled(field)"
          :clearable="field.clearable !== false"
          :multiple="field.type === 'multiSelect'"
          :collapse-tags="field.collapseTags"
          :collapse-tags-tooltip="field.collapseTagsTooltip"
          :filterable="field.filterable"
          :allow-create="field.allowCreate"
          :remote="field.remote"
          :remote-method="field.remoteMethod"
          :loading="field.loading"
          @change="handleFieldChange(field.key, $event)"
          @visible-change="handleSelectVisibleChange(field.key, $event)"
        >
          <el-option
            v-for="option in getFieldOptions(field)"
            :key="option.value"
            :label="option.label"
            :value="option.value"
            :disabled="option.disabled"
          />
        </el-select>

        <!-- 日期选择器 -->
        <el-date-picker
          v-else-if="
            field.type === 'date' ||
            field.type === 'datetime' ||
            field.type === 'daterange'
          "
          v-model="formData[field.key]"
          :type="getDatePickerType(field.type)"
          :placeholder="field.placeholder"
          :disabled="isFieldDisabled(field)"
          :readonly="field.readonly"
          :clearable="field.clearable !== false"
          :format="field.format"
          :value-format="field.valueFormat"
          :range-separator="field.rangeSeparator"
          :start-placeholder="field.startPlaceholder"
          :end-placeholder="field.endPlaceholder"
          :picker-options="field.pickerOptions"
          @change="handleFieldChange(field.key, $event)"
        />

        <!-- 时间选择器 -->
        <el-time-picker
          v-else-if="field.type === 'time'"
          v-model="formData[field.key]"
          :placeholder="field.placeholder"
          :disabled="isFieldDisabled(field)"
          :readonly="field.readonly"
          :clearable="field.clearable !== false"
          :format="field.format"
          :value-format="field.valueFormat"
          @change="handleFieldChange(field.key, $event)"
        />

        <!-- 开关 -->
        <el-switch
          v-else-if="field.type === 'switch'"
          v-model="formData[field.key]"
          :disabled="isFieldDisabled(field)"
          :active-text="field.activeText"
          :inactive-text="field.inactiveText"
          :active-value="field.activeValue"
          :inactive-value="field.inactiveValue"
          @change="handleFieldChange(field.key, $event)"
        />

        <!-- 单选框组 -->
        <el-radio-group
          v-else-if="field.type === 'radio'"
          v-model="formData[field.key]"
          :disabled="isFieldDisabled(field)"
          @change="handleFieldChange(field.key, $event)"
        >
          <el-radio
            v-for="option in getFieldOptions(field)"
            :key="option.value"
            :label="option.value"
            :disabled="option.disabled"
          >
            {{ option.label }}
          </el-radio>
        </el-radio-group>

        <!-- 复选框组 -->
        <el-checkbox-group
          v-else-if="field.type === 'checkbox'"
          v-model="formData[field.key]"
          :disabled="isFieldDisabled(field)"
          @change="handleFieldChange(field.key, $event)"
        >
          <el-checkbox
            v-for="option in getFieldOptions(field)"
            :key="option.value"
            :label="option.value"
            :disabled="option.disabled"
          >
            {{ option.label }}
          </el-checkbox>
        </el-checkbox-group>

        <!-- 单个复选框 -->
        <el-checkbox
          v-else-if="field.type === 'singleCheckbox'"
          v-model="formData[field.key]"
          :disabled="isFieldDisabled(field)"
          :true-label="field.trueLabel"
          :false-label="field.falseLabel"
          @change="handleFieldChange(field.key, $event)"
        >
          {{ field.checkboxText }}
        </el-checkbox>

        <!-- 文件上传 -->
        <el-upload
          v-else-if="field.type === 'upload'"
          :action="field.action"
          :headers="field.headers"
          :data="field.uploadData"
          :name="field.name"
          :multiple="field.multiple"
          :accept="field.accept"
          :before-upload="field.beforeUpload"
          :on-success="
            (response, file, fileList) =>
              handleUploadSuccess(field.key, response, file, fileList)
          "
          :on-error="
            (error, file, fileList) =>
              handleUploadError(field.key, error, file, fileList)
          "
          :on-remove="
            (file, fileList) => handleUploadRemove(field.key, file, fileList)
          "
          :file-list="formData[field.key] || []"
          :disabled="isFieldDisabled(field)"
          :list-type="field.listType"
          :drag="field.drag"
        >
          <slot :name="`upload-${field.key}`">
            <el-button :icon="field.uploadIcon || 'Upload'">
              {{ field.uploadText || '选择文件' }}
            </el-button>
          </slot>
        </el-upload>

        <!-- JSON编辑器 -->
        <el-input
          v-else-if="field.type === 'json'"
          v-model="jsonValues[field.key]"
          type="textarea"
          :rows="field.rows || 6"
          :placeholder="field.placeholder || '请输入JSON格式数据'"
          :disabled="isFieldDisabled(field)"
          @blur="handleJsonBlur(field.key)"
        />

        <!-- 自定义插槽 -->
        <slot
          v-else-if="field.type === 'slot'"
          :name="field.slotName || field.key"
          :field="field"
          :value="formData[field.key]"
          :disabled="isFieldDisabled(field)"
        />

        <!-- 字段说明 -->
        <div v-if="field.help" class="field-help">
          <el-text type="info" size="small">
            <el-icon><InfoFilled /></el-icon>
            {{ field.help }}
          </el-text>
        </div>
      </el-form-item>
    </template>

    <!-- 表单操作按钮 -->
    <el-form-item v-if="showActions" class="form-actions">
      <slot name="actions" :validate="validate" :reset-fields="resetFields">
        <el-button :disabled="disabled" @click="handleReset">
          {{ resetText }}
        </el-button>
        <el-button
          type="primary"
          :loading="submitLoading"
          :disabled="disabled"
          @click="handleSubmit"
        >
          {{ submitText }}
        </el-button>
      </slot>
    </el-form-item>
  </el-form>
</template>

<script setup lang="ts">
import { InfoFilled } from '@element-plus/icons-vue'
import type {
  FormInstance,
  FormRules,
  FormValidateCallback,
} from 'element-plus'
import { ref, reactive, computed, watch, nextTick } from 'vue'

export interface FormField {
  key: string
  label?: string
  type:
    | 'text'
    | 'password'
    | 'textarea'
    | 'number'
    | 'select'
    | 'multiSelect'
    | 'date'
    | 'datetime'
    | 'daterange'
    | 'time'
    | 'switch'
    | 'radio'
    | 'checkbox'
    | 'singleCheckbox'
    | 'upload'
    | 'json'
    | 'slot'
    | 'group'
  required?: boolean
  disabled?: boolean | ((formData: any) => boolean)
  readonly?: boolean
  placeholder?: string
  help?: string
  className?: string

  // 验证规则
  rules?: any[]
  validator?: (rule: any, value: any, callback: any) => void

  // 输入框特有属性
  clearable?: boolean
  maxlength?: number
  showWordLimit?: boolean
  rows?: number
  autosize?: boolean | { minRows?: number; maxRows?: number }
  prefixIcon?: any
  suffixIcon?: any

  // 数字输入框特有属性
  min?: number
  max?: number
  step?: number
  precision?: number
  controls?: boolean
  controlsPosition?: 'right' | ''

  // 选择器特有属性
  options?: SelectOption[]
  optionsLoader?: () => Promise<SelectOption[]>
  collapseTags?: boolean
  collapseTagsTooltip?: boolean
  filterable?: boolean
  allowCreate?: boolean
  remote?: boolean
  remoteMethod?: (query: string) => void
  loading?: boolean

  // 日期选择器特有属性
  format?: string
  valueFormat?: string
  rangeSeparator?: string
  startPlaceholder?: string
  endPlaceholder?: string
  pickerOptions?: any

  // 开关特有属性
  activeText?: string
  inactiveText?: string
  activeValue?: any
  inactiveValue?: any

  // 复选框特有属性
  checkboxText?: string
  trueLabel?: any
  falseLabel?: any

  // 上传组件特有属性
  action?: string
  headers?: Record<string, any>
  uploadData?: Record<string, any>
  name?: string
  multiple?: boolean
  accept?: string
  beforeUpload?: (file: File) => boolean | Promise<boolean>
  listType?: 'text' | 'picture' | 'picture-card'
  drag?: boolean
  uploadIcon?: any
  uploadText?: string

  // 布局属性
  labelWidth?: string
  showMessage?: boolean
  inlineMessage?: boolean
  size?: 'large' | 'default' | 'small'

  // 自定义插槽
  slotName?: string

  // 分组
  divider?: boolean
}

export interface SelectOption {
  label: string
  value: any
  disabled?: boolean
}

interface Props {
  fields: FormField[]
  modelValue: Record<string, any>
  rules?: FormRules
  labelWidth?: string
  labelPosition?: 'left' | 'right' | 'top'
  size?: 'large' | 'default' | 'small'
  disabled?: boolean
  validateOnRuleChange?: boolean
  hideRequiredAsterisk?: boolean
  showMessage?: boolean
  inlineMessage?: boolean
  statusIcon?: boolean

  // 操作按钮
  showActions?: boolean
  submitText?: string
  resetText?: string
  submitLoading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  rules: () => ({}),
  labelWidth: '120px',
  labelPosition: 'right',
  size: 'default',
  disabled: false,
  validateOnRuleChange: true,
  hideRequiredAsterisk: false,
  showMessage: true,
  inlineMessage: false,
  statusIcon: false,
  showActions: true,
  submitText: '提交',
  resetText: '重置',
  submitLoading: false,
})

interface Emits {
  'update:modelValue': [value: Record<string, any>]
  submit: [formData: Record<string, any>]
  reset: []
  fieldChange: [key: string, value: any]
  fieldBlur: [key: string]
  fieldFocus: [key: string]
  validate: [prop: string, isValid: boolean, message: string]
}

const emit = defineEmits<Emits>()

// 表单引用
const formRef = ref<FormInstance>()

// 表单数据
const formData = reactive({ ...props.modelValue })

// JSON字段的字符串值
const jsonValues = reactive<Record<string, string>>({})

// 字段错误信息
const fieldErrors = reactive<Record<string, string>>({})

// 监听props.modelValue变化
watch(
  () => props.modelValue,
  newValue => {
    Object.assign(formData, newValue)

    // 初始化JSON字段
    props.fields.forEach(field => {
      if (field.type === 'json' && newValue[field.key]) {
        try {
          jsonValues[field.key] = JSON.stringify(newValue[field.key], null, 2)
        } catch {
          jsonValues[field.key] = ''
        }
      }
    })
  },
  { immediate: true, deep: true }
)

// 监听formData变化，同步到父组件
watch(
  formData,
  newValue => {
    emit('update:modelValue', { ...newValue })
  },
  { deep: true }
)

// 处理后的字段列表
const processedFields = computed(() => {
  return props.fields.map(field => {
    // 为选择器字段加载选项
    if (
      (field.type === 'select' || field.type === 'multiSelect') &&
      field.optionsLoader &&
      !field.options
    ) {
      field.optionsLoader().then(options => {
        field.options = options
      })
    }

    return field
  })
})

// 表单验证规则
const formRules = computed(() => {
  const rules: FormRules = { ...props.rules }

  props.fields.forEach(field => {
    if (field.rules || field.required || field.validator) {
      const fieldRules = []

      if (field.required) {
        fieldRules.push({
          required: true,
          message: `请输入${field.label}`,
          trigger: ['blur', 'change'],
        })
      }

      if (field.rules) {
        fieldRules.push(...field.rules)
      }

      if (field.validator) {
        fieldRules.push({
          validator: field.validator,
          trigger: ['blur', 'change'],
        })
      }

      rules[field.key] = fieldRules
    }
  })

  return rules
})

// 获取字段规则
const getFieldRules = (field: FormField) => {
  return formRules.value[field.key] || []
}

// 获取字段错误信息
const getFieldError = (key: string) => {
  return fieldErrors[key] || ''
}

// 判断字段是否禁用
const isFieldDisabled = (field: FormField) => {
  if (props.disabled) return true
  if (typeof field.disabled === 'function') {
    return field.disabled(formData)
  }
  return field.disabled === true
}

// 获取字段选项
const getFieldOptions = (field: FormField) => {
  return field.options || []
}

// 获取日期选择器类型
const getDatePickerType = (type: string) => {
  const typeMap: Record<string, string> = {
    date: 'date',
    datetime: 'datetime',
    daterange: 'daterange',
  }
  return typeMap[type] || 'date'
}

// 事件处理
const handleFieldChange = (key: string, value: any) => {
  formData[key] = value
  emit('fieldChange', key, value)
}

const handleFieldBlur = (key: string) => {
  emit('fieldBlur', key)
}

const handleFieldFocus = (key: string) => {
  emit('fieldFocus', key)
}

const handleJsonBlur = (key: string) => {
  try {
    const parsed = JSON.parse(jsonValues[key] || '{}')
    formData[key] = parsed
    fieldErrors[key] = ''
  } catch (error) {
    fieldErrors[key] = 'JSON格式错误'
  }
}

const handleSelectVisibleChange = (key: string, visible: boolean) => {
  if (visible) {
    const field = props.fields.find(f => f.key === key)
    if (field?.optionsLoader && !field.options) {
      field.optionsLoader().then(options => {
        field.options = options
      })
    }
  }
}

const handleUploadSuccess = (
  key: string,
  response: any,
  file: any,
  fileList: any[]
) => {
  formData[key] = fileList
  emit('fieldChange', key, fileList)
}

const handleUploadError = (
  key: string,
  error: any,
  file: any,
  fileList: any[]
) => {
  console.error('Upload error:', error)
}

const handleUploadRemove = (key: string, file: any, fileList: any[]) => {
  formData[key] = fileList
  emit('fieldChange', key, fileList)
}

const handleValidate = (prop: string, isValid: boolean, message: string) => {
  emit('validate', prop, isValid, message)
}

const handleSubmit = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
    emit('submit', { ...formData })
  } catch (error) {
    console.error('Form validation failed:', error)
  }
}

const handleReset = () => {
  formRef.value?.resetFields()
  emit('reset')
}

// 公开方法
const validate = (callback?: FormValidateCallback) => {
  return formRef.value?.validate(callback)
}

const validateField = (
  props: string | string[],
  callback?: FormValidateCallback
) => {
  return formRef.value?.validateField(props, callback)
}

const resetFields = () => {
  formRef.value?.resetFields()
}

const clearValidate = (props?: string | string[]) => {
  formRef.value?.clearValidate(props)
}

const scrollToField = (prop: string) => {
  formRef.value?.scrollToField(prop)
}

defineExpose({
  validate,
  validateField,
  resetFields,
  clearValidate,
  scrollToField,
  formRef,
})
</script>

<style scoped lang="scss">
.form-group {
  margin: 24px 0 16px 0;

  .form-group-title {
    margin: 0 0 8px 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--el-text-color-primary);
  }
}

.field-help {
  margin-top: 4px;

  .el-text {
    display: flex;
    align-items: center;
    gap: 4px;
  }
}

.form-actions {
  margin-top: 24px;

  .el-button + .el-button {
    margin-left: 12px;
  }
}

// JSON编辑器样式
.el-textarea {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
}
</style>
