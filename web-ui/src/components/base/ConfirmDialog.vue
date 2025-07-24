<template>
  <el-dialog
    v-model="dialogVisible"
    :title="title"
    :width="width"
    :top="top"
    :modal="modal"
    :modal-class="modalClass"
    :append-to-body="appendToBody"
    :lock-scroll="lockScroll"
    :custom-class="customClass"
    :open-delay="openDelay"
    :close-delay="closeDelay"
    :close-on-click-modal="closeOnClickModal"
    :close-on-press-escape="closeOnPressEscape"
    :show-close="showClose"
    :before-close="handleBeforeClose"
    :center="center"
    :align-center="alignCenter"
    :destroy-on-close="destroyOnClose"
    @open="handleOpen"
    @opened="handleOpened"
    @close="handleClose"
    @closed="handleClosed"
  >
    <!-- 对话框内容 -->
    <div class="confirm-dialog-content">
      <!-- 图标区域 -->
      <div v-if="showIcon" class="confirm-icon" :class="`confirm-icon--${type}`">
        <el-icon :size="iconSize">
          <component :is="currentIcon" />
        </el-icon>
      </div>
      
      <!-- 内容区域 -->
      <div class="confirm-content">
        <!-- 主要消息 -->
        <div class="confirm-message" :class="messageClass">
          <slot name="message">
            {{ message }}
          </slot>
        </div>
        
        <!-- 详细描述 -->
        <div v-if="description || $slots.description" class="confirm-description">
          <slot name="description">
            {{ description }}
          </slot>
        </div>
        
        <!-- 额外内容 -->
        <div v-if="$slots.content" class="confirm-extra-content">
          <slot name="content" />
        </div>
        
        <!-- 输入框（用于确认输入） -->
        <div v-if="requireInput" class="confirm-input">
          <el-input
            v-model="inputValue"
            :placeholder="inputPlaceholder"
            :type="inputType"
            :validation-icon="inputValidationIcon"
            :validate-event="false"
            @input="handleInputChange"
            @keyup.enter="handleConfirm"
          />
          <div v-if="inputErrorMessage" class="input-error">
            {{ inputErrorMessage }}
          </div>
        </div>
        
        <!-- 复选框确认 -->
        <div v-if="requireCheckbox" class="confirm-checkbox">
          <el-checkbox v-model="checkboxValue" @change="handleCheckboxChange">
            {{ checkboxText }}
          </el-checkbox>
        </div>
        
        <!-- 警告信息 -->
        <div v-if="warningText || $slots.warning" class="confirm-warning">
          <el-alert
            :title="warningText"
            type="warning"
            effect="light"
            :show-icon="true"
            :closable="false"
          >
            <slot name="warning" />
          </el-alert>
        </div>
        
        <!-- 倒计时 -->
        <div v-if="countdown > 0" class="confirm-countdown">
          {{ countdownText }}（{{ remainingTime }}秒后自动{{ autoAction === 'confirm' ? '确认' : '取消' }}）
        </div>
      </div>
    </div>
    
    <!-- 对话框按钮 -->
    <template #footer>
      <div class="confirm-footer">
        <slot name="footer" :confirm="handleConfirm" :cancel="handleCancel">
          <el-button
            v-if="showCancelButton"
            :size="buttonSize"
            :disabled="loading"
            @click="handleCancel"
          >
            {{ cancelButtonText }}
          </el-button>
          
          <el-button
            :type="confirmButtonType"
            :size="buttonSize"
            :loading="loading"
            :disabled="isConfirmDisabled"
            @click="handleConfirm"
          >
            {{ confirmButtonText }}
          </el-button>
        </slot>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import {
  WarningFilled,
  InfoFilled,
  SuccessFilled,
  CircleCloseFilled,
  QuestionFilled,
} from '@element-plus/icons-vue'

export type ConfirmType = 'info' | 'success' | 'warning' | 'error' | 'question'
export type AutoAction = 'confirm' | 'cancel'

interface Props {
  modelValue?: boolean
  type?: ConfirmType
  title?: string
  message?: string
  description?: string
  warningText?: string
  
  // 外观样式
  width?: string | number
  top?: string
  customClass?: string
  center?: boolean
  alignCenter?: boolean
  
  // 图标
  showIcon?: boolean
  icon?: any
  iconSize?: string | number
  
  // 按钮
  showCancelButton?: boolean
  confirmButtonText?: string
  cancelButtonText?: string
  confirmButtonType?: 'primary' | 'success' | 'warning' | 'danger' | 'info'
  buttonSize?: 'large' | 'default' | 'small'
  
  // 行为控制
  modal?: boolean
  modalClass?: string
  appendToBody?: boolean
  lockScroll?: boolean
  openDelay?: number
  closeDelay?: number
  closeOnClickModal?: boolean
  closeOnPressEscape?: boolean
  showClose?: boolean
  destroyOnClose?: boolean
  
  // 加载状态
  loading?: boolean
  
  // 输入确认
  requireInput?: boolean
  inputPlaceholder?: string
  inputType?: string
  inputPattern?: RegExp
  inputValidator?: (value: string) => boolean | string
  inputValidationIcon?: boolean
  
  // 复选框确认
  requireCheckbox?: boolean
  checkboxText?: string
  
  // 倒计时
  countdown?: number
  countdownText?: string
  autoAction?: AutoAction
  
  // 样式类
  messageClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  type: 'warning',
  title: '确认操作',
  message: '确定要执行此操作吗？',
  width: '420px',
  top: '15vh',
  center: false,
  alignCenter: false,
  showIcon: true,
  iconSize: 24,
  showCancelButton: true,
  confirmButtonText: '确定',
  cancelButtonText: '取消',
  confirmButtonType: 'primary',
  buttonSize: 'default',
  modal: true,
  appendToBody: true,
  lockScroll: true,
  openDelay: 0,
  closeDelay: 0,
  closeOnClickModal: false,
  closeOnPressEscape: true,
  showClose: true,
  destroyOnClose: false,
  loading: false,
  requireInput: false,
  inputPlaceholder: '请输入',
  inputType: 'text',
  inputValidationIcon: true,
  requireCheckbox: false,
  checkboxText: '我已确认此操作',
  countdown: 0,
  countdownText: '操作确认',
  autoAction: 'cancel',
})

interface Emits {
  'update:modelValue': [value: boolean]
  confirm: [inputValue?: string]
  cancel: []
  open: []
  opened: []
  close: []
  closed: []
}

const emit = defineEmits<Emits>()

// 状态
const dialogVisible = ref(props.modelValue)
const inputValue = ref('')
const checkboxValue = ref(false)
const inputErrorMessage = ref('')
const remainingTime = ref(props.countdown)

// 倒计时定时器
let countdownTimer: NodeJS.Timeout | null = null

// 监听modelValue变化
watch(() => props.modelValue, (val) => {
  dialogVisible.value = val
  if (val) {
    startCountdown()
  } else {
    stopCountdown()
  }
})

// 监听dialogVisible变化
watch(dialogVisible, (val) => {
  emit('update:modelValue', val)
})

// 当前图标
const currentIcon = computed(() => {
  if (props.icon) return props.icon
  
  const iconMap = {
    info: InfoFilled,
    success: SuccessFilled,
    warning: WarningFilled,
    error: CircleCloseFilled,
    question: QuestionFilled,
  }
  
  return iconMap[props.type] || QuestionFilled
})

// 确认按钮是否禁用
const isConfirmDisabled = computed(() => {
  if (props.loading) return true
  
  // 输入验证
  if (props.requireInput) {
    if (!inputValue.value.trim()) return true
    if (inputErrorMessage.value) return true
  }
  
  // 复选框验证
  if (props.requireCheckbox && !checkboxValue.value) {
    return true
  }
  
  return false
})

// 启动倒计时
const startCountdown = () => {
  if (props.countdown <= 0) return
  
  remainingTime.value = props.countdown
  countdownTimer = setInterval(() => {
    remainingTime.value--
    
    if (remainingTime.value <= 0) {
      stopCountdown()
      if (props.autoAction === 'confirm') {
        handleConfirm()
      } else {
        handleCancel()
      }
    }
  }, 1000)
}

// 停止倒计时
const stopCountdown = () => {
  if (countdownTimer) {
    clearInterval(countdownTimer)
    countdownTimer = null
  }
}

// 验证输入
const validateInput = () => {
  inputErrorMessage.value = ''
  
  if (!inputValue.value.trim()) {
    inputErrorMessage.value = '请输入内容'
    return false
  }
  
  if (props.inputPattern && !props.inputPattern.test(inputValue.value)) {
    inputErrorMessage.value = '输入格式不正确'
    return false
  }
  
  if (props.inputValidator) {
    const result = props.inputValidator(inputValue.value)
    if (typeof result === 'string') {
      inputErrorMessage.value = result
      return false
    }
    if (!result) {
      inputErrorMessage.value = '输入验证失败'
      return false
    }
  }
  
  return true
}

// 事件处理
const handleBeforeClose = (done: () => void) => {
  if (!props.loading) {
    done()
  }
}

const handleOpen = () => {
  emit('open')
}

const handleOpened = () => {
  emit('opened')
}

const handleClose = () => {
  stopCountdown()
  emit('close')
}

const handleClosed = () => {
  // 重置状态
  inputValue.value = ''
  checkboxValue.value = false
  inputErrorMessage.value = ''
  remainingTime.value = props.countdown
  emit('closed')
}

const handleConfirm = () => {
  if (isConfirmDisabled.value) return
  
  if (props.requireInput && !validateInput()) {
    return
  }
  
  emit('confirm', props.requireInput ? inputValue.value : undefined)
  
  if (!props.loading) {
    dialogVisible.value = false
  }
}

const handleCancel = () => {
  if (!props.loading) {
    emit('cancel')
    dialogVisible.value = false
  }
}

const handleInputChange = () => {
  if (inputErrorMessage.value) {
    validateInput()
  }
}

const handleCheckboxChange = (value: boolean) => {
  checkboxValue.value = value
}

// 公开方法
const open = () => {
  dialogVisible.value = true
}

const close = () => {
  dialogVisible.value = false
}

// 生命周期
onMounted(() => {
  if (props.modelValue) {
    startCountdown()
  }
})

onUnmounted(() => {
  stopCountdown()
})

defineExpose({
  open,
  close,
})
</script>

<style scoped lang="scss">
.confirm-dialog-content {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  
  .confirm-icon {
    flex-shrink: 0;
    
    &.confirm-icon--info {
      color: var(--el-color-info);
    }
    
    &.confirm-icon--success {
      color: var(--el-color-success);
    }
    
    &.confirm-icon--warning {
      color: var(--el-color-warning);
    }
    
    &.confirm-icon--error {
      color: var(--el-color-danger);
    }
    
    &.confirm-icon--question {
      color: var(--el-color-primary);
    }
  }
  
  .confirm-content {
    flex: 1;
    min-width: 0;
    
    .confirm-message {
      font-size: 16px;
      font-weight: 500;
      color: var(--el-text-color-primary);
      margin-bottom: 8px;
      line-height: 1.4;
    }
    
    .confirm-description {
      font-size: 14px;
      color: var(--el-text-color-secondary);
      margin-bottom: 12px;
      line-height: 1.5;
    }
    
    .confirm-extra-content {
      margin: 12px 0;
    }
    
    .confirm-input {
      margin: 16px 0;
      
      .input-error {
        font-size: 12px;
        color: var(--el-color-danger);
        margin-top: 4px;
        line-height: 1.4;
      }
    }
    
    .confirm-checkbox {
      margin: 16px 0;
    }
    
    .confirm-warning {
      margin: 16px 0;
    }
    
    .confirm-countdown {
      margin-top: 16px;
      padding: 8px 12px;
      background: var(--el-color-warning-light-9);
      border: 1px solid var(--el-color-warning-light-5);
      border-radius: 4px;
      font-size: 13px;
      color: var(--el-color-warning-dark-2);
      text-align: center;
    }
  }
}

.confirm-footer {
  text-align: right;
  
  .el-button + .el-button {
    margin-left: 8px;
  }
}

// 当只有图标时的垂直居中
.confirm-dialog-content:has(.confirm-icon):not(:has(.confirm-description, .confirm-extra-content, .confirm-input, .confirm-checkbox, .confirm-warning, .confirm-countdown)) {
  align-items: center;
}

// 响应式设计
@media (max-width: 768px) {
  .confirm-dialog-content {
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 12px;
    
    .confirm-content {
      width: 100%;
    }
  }
  
  .confirm-footer {
    text-align: center;
    
    .el-button {
      margin: 0 4px;
    }
  }
}
</style>