<template>
  <div class="action-buttons" :class="containerClass">
    <template v-for="action in visibleActions" :key="action.key">
      <!-- 普通按钮 -->
      <el-button
        v-if="!action.confirm && !action.dropdown"
        :type="action.type"
        :size="action.size || size"
        :icon="action.icon"
        :loading="getActionLoading(action)"
        :disabled="getActionDisabled(action)"
        :plain="action.plain"
        :round="action.round"
        :circle="action.circle"
        :text="action.text"
        :bg="action.bg"
        :link="action.link"
        :class="action.className"
        @click="handleActionClick(action)"
      >
        {{ action.label }}
      </el-button>
      
      <!-- 确认对话框按钮 -->
      <el-popconfirm
        v-else-if="action.confirm"
        :title="action.confirm.title || '确定要执行此操作吗？'"
        :confirm-button-text="action.confirm.confirmText || '确定'"
        :cancel-button-text="action.confirm.cancelText || '取消'"
        :confirm-button-type="action.confirm.confirmType || 'primary'"
        :cancel-button-type="action.confirm.cancelType || 'default'"
        :icon="action.confirm.icon"
        :icon-color="action.confirm.iconColor"
        :hide-icon="action.confirm.hideIcon"
        :width="action.confirm.width"
        @confirm="handleActionClick(action)"
        @cancel="handleActionCancel(action)"
      >
        <template #reference>
          <el-button
            :type="action.type"
            :size="action.size || size"
            :icon="action.icon"
            :loading="getActionLoading(action)"
            :disabled="getActionDisabled(action)"
            :plain="action.plain"
            :round="action.round"
            :circle="action.circle"
            :text="action.text"
            :bg="action.bg"
            :link="action.link"
            :class="action.className"
          >
            {{ action.label }}
          </el-button>
        </template>
      </el-popconfirm>
      
      <!-- 下拉菜单按钮 -->
      <el-dropdown
        v-else-if="action.dropdown"
        :trigger="action.dropdown.trigger || 'click'"
        :placement="action.dropdown.placement"
        :disabled="getActionDisabled(action)"
        @command="(command) => handleDropdownCommand(action, command)"
      >
        <el-button
          :type="action.type"
          :size="action.size || size"
          :icon="action.icon"
          :loading="getActionLoading(action)"
          :disabled="getActionDisabled(action)"
          :plain="action.plain"
          :round="action.round"
          :text="action.text"
          :bg="action.bg"
          :link="action.link"
          :class="action.className"
        >
          {{ action.label }}
          <el-icon class="el-icon--right">
            <ArrowDown />
          </el-icon>
        </el-button>
        
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item
              v-for="item in action.dropdown.items"
              :key="item.key"
              :command="item.key"
              :disabled="item.disabled"
              :divided="item.divided"
              :icon="item.icon"
            >
              {{ item.label }}
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
      
      <!-- 分割线 -->
      <el-divider
        v-if="action.divider"
        direction="vertical"
        class="action-divider"
      />
    </template>
    
    <!-- 更多操作按钮 -->
    <el-dropdown
      v-if="hasHiddenActions"
      :trigger="moreActionTrigger"
      placement="bottom-end"
      @command="handleMoreActionCommand"
    >
      <el-button
        :size="size"
        :type="moreActionType"
        :icon="MoreFilled"
        :text="moreActionText"
      >
        {{ moreActionLabel }}
      </el-button>
      
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item
            v-for="action in hiddenActions"
            :key="action.key"
            :command="action.key"
            :disabled="getActionDisabled(action)"
            :icon="action.icon"
          >
            {{ action.label }}
          </el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ArrowDown, MoreFilled } from '@element-plus/icons-vue'

export interface ActionButton {
  key: string
  label: string
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'default' | 'text'
  size?: 'large' | 'default' | 'small'
  icon?: any
  loading?: boolean | (() => boolean)
  disabled?: boolean | (() => boolean)
  visible?: boolean | (() => boolean)
  plain?: boolean
  round?: boolean
  circle?: boolean
  text?: boolean
  bg?: boolean
  link?: boolean
  className?: string
  
  // 确认对话框
  confirm?: {
    title?: string
    confirmText?: string
    cancelText?: string
    confirmType?: 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'default'
    cancelType?: 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'default'
    icon?: any
    iconColor?: string
    hideIcon?: boolean
    width?: string | number
  }
  
  // 下拉菜单
  dropdown?: {
    trigger?: 'click' | 'hover' | 'focus' | 'contextmenu'
    placement?: 'top' | 'top-start' | 'top-end' | 'bottom' | 'bottom-start' | 'bottom-end' | 'left' | 'left-start' | 'left-end' | 'right' | 'right-start' | 'right-end'
    items: DropdownItem[]
  }
  
  // 分割线
  divider?: boolean
}

export interface DropdownItem {
  key: string
  label: string
  disabled?: boolean
  divided?: boolean
  icon?: any
}

interface Props {
  actions: ActionButton[]
  size?: 'large' | 'default' | 'small'
  alignment?: 'left' | 'center' | 'right'
  direction?: 'horizontal' | 'vertical'
  wrap?: boolean
  spacing?: 'small' | 'default' | 'large'
  
  // 更多操作按钮
  maxVisible?: number
  moreActionLabel?: string
  moreActionType?: 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'default' | 'text'
  moreActionText?: boolean
  moreActionTrigger?: 'click' | 'hover'
}

const props = withDefaults(defineProps<Props>(), {
  size: 'default',
  alignment: 'left',
  direction: 'horizontal',
  wrap: false,
  spacing: 'default',
  maxVisible: 999,
  moreActionLabel: '更多',
  moreActionType: 'default',
  moreActionText: false,
  moreActionTrigger: 'click',
})

interface Emits {
  actionClick: [action: ActionButton]
  actionCancel: [action: ActionButton]
  dropdownCommand: [action: ActionButton, command: string]
}

const emit = defineEmits<Emits>()

// 可见的操作
const visibleActions = computed(() => {
  const filtered = props.actions.filter(action => {
    if (typeof action.visible === 'function') {
      return action.visible()
    }
    return action.visible !== false
  })
  
  if (props.maxVisible >= filtered.length) {
    return filtered
  }
  
  return filtered.slice(0, props.maxVisible)
})

// 隐藏的操作（显示在更多菜单中）
const hiddenActions = computed(() => {
  const filtered = props.actions.filter(action => {
    if (typeof action.visible === 'function') {
      return action.visible()
    }
    return action.visible !== false
  })
  
  if (props.maxVisible >= filtered.length) {
    return []
  }
  
  return filtered.slice(props.maxVisible)
})

// 是否有隐藏的操作
const hasHiddenActions = computed(() => hiddenActions.value.length > 0)

// 容器样式类
const containerClass = computed(() => {
  const classes = []
  
  classes.push(`action-buttons--${props.alignment}`)
  classes.push(`action-buttons--${props.direction}`)
  classes.push(`action-buttons--spacing-${props.spacing}`)
  
  if (props.wrap) {
    classes.push('action-buttons--wrap')
  }
  
  return classes.join(' ')
})

// 工具函数
const getActionLoading = (action: ActionButton) => {
  if (typeof action.loading === 'function') {
    return action.loading()
  }
  return action.loading === true
}

const getActionDisabled = (action: ActionButton) => {
  if (typeof action.disabled === 'function') {
    return action.disabled()
  }
  return action.disabled === true
}

// 事件处理
const handleActionClick = (action: ActionButton) => {
  if (!getActionDisabled(action)) {
    emit('actionClick', action)
  }
}

const handleActionCancel = (action: ActionButton) => {
  emit('actionCancel', action)
}

const handleDropdownCommand = (action: ActionButton, command: string) => {
  emit('dropdownCommand', action, command)
}

const handleMoreActionCommand = (command: string) => {
  const action = hiddenActions.value.find(a => a.key === command)
  if (action && !getActionDisabled(action)) {
    emit('actionClick', action)
  }
}
</script>

<style scoped lang="scss">
.action-buttons {
  display: flex;
  align-items: center;
  
  // 对齐方式
  &.action-buttons--left {
    justify-content: flex-start;
  }
  
  &.action-buttons--center {
    justify-content: center;
  }
  
  &.action-buttons--right {
    justify-content: flex-end;
  }
  
  // 方向
  &.action-buttons--horizontal {
    flex-direction: row;
  }
  
  &.action-buttons--vertical {
    flex-direction: column;
    align-items: stretch;
  }
  
  // 换行
  &.action-buttons--wrap {
    flex-wrap: wrap;
  }
  
  // 间距
  &.action-buttons--spacing-small {
    gap: 4px;
  }
  
  &.action-buttons--spacing-default {
    gap: 8px;
  }
  
  &.action-buttons--spacing-large {
    gap: 12px;
  }
  
  // 分割线
  .action-divider {
    height: 20px;
    margin: 0 4px;
  }
}

// 垂直方向时的特殊处理
.action-buttons--vertical {
  .action-divider {
    width: 100%;
    height: 1px;
    margin: 4px 0;
  }
  
  .el-button {
    width: 100%;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .action-buttons {
    &.action-buttons--horizontal {
      flex-wrap: wrap;
      gap: 6px;
    }
    
    .el-button {
      font-size: 12px;
      padding: 6px 12px;
    }
    
    .el-button--small {
      padding: 4px 8px;
    }
  }
}
</style>