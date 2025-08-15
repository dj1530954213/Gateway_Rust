<template>
  <el-tag
    :type="tagType"
    :size="size"
    :effect="effect"
    :round="round"
    :hit="hit"
    :closable="closable"
    :disable-transitions="disableTransitions"
    :class="tagClass"
    @close="handleClose"
    @click="handleClick"
  >
    <el-icon v-if="icon" class="status-icon">
      <component :is="icon" />
    </el-icon>

    <span class="status-text">{{ displayText }}</span>

    <el-badge
      v-if="showBadge && badgeValue"
      :value="badgeValue"
      :type="badgeType"
      :is-dot="badgeIsDot"
      class="status-badge"
    />
  </el-tag>
</template>

<script setup lang="ts">
import {
  CircleCheck,
  CircleClose,
  Warning,
  Clock,
  Connection,
  // Disconnect, // 这个图标不存在，先移除
  Loading,
  QuestionFilled,
} from '@element-plus/icons-vue'
import { computed } from 'vue'

export interface StatusMapping {
  type: 'success' | 'info' | 'warning' | 'danger' | ''
  text: string
  icon?: any
  color?: string
}

interface Props {
  status: string | number | boolean
  mappings?: Record<string, StatusMapping>
  size?: 'large' | 'default' | 'small'
  effect?: 'dark' | 'light' | 'plain'
  round?: boolean
  hit?: boolean
  closable?: boolean
  disableTransitions?: boolean
  clickable?: boolean

  // 徽章相关
  showBadge?: boolean
  badgeValue?: string | number
  badgeType?: 'primary' | 'success' | 'warning' | 'danger' | 'info'
  badgeIsDot?: boolean

  // 自定义属性
  customClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  mappings: () => ({}),
  size: 'default',
  effect: 'light',
  round: false,
  hit: false,
  closable: false,
  disableTransitions: false,
  clickable: false,
  showBadge: false,
  badgeIsDot: false,
})

interface Emits {
  close: [event: Event]
  click: [event: Event]
}

const emit = defineEmits<Emits>()

// 预定义的状态映射
const defaultMappings: Record<string, StatusMapping> = {
  // 通用状态
  success: { type: 'success', text: '成功', icon: CircleCheck },
  error: { type: 'danger', text: '错误', icon: CircleClose },
  warning: { type: 'warning', text: '警告', icon: Warning },
  info: { type: 'info', text: '信息', icon: QuestionFilled },
  pending: { type: 'info', text: '等待中', icon: Clock },
  loading: { type: 'info', text: '加载中', icon: Loading },

  // 连接状态
  connected: { type: 'success', text: '已连接', icon: Connection },
  disconnected: { type: 'danger', text: '已断开', icon: Disconnect },
  connecting: { type: 'warning', text: '连接中', icon: Loading },

  // 运行状态
  running: { type: 'success', text: '运行中', icon: CircleCheck },
  stopped: { type: 'info', text: '已停止', icon: CircleClose },
  stopping: { type: 'warning', text: '停止中', icon: Loading },
  starting: { type: 'warning', text: '启动中', icon: Loading },

  // 布尔值
  true: { type: 'success', text: '是', icon: CircleCheck },
  false: { type: 'danger', text: '否', icon: CircleClose },

  // 数字状态
  '1': { type: 'success', text: '启用', icon: CircleCheck },
  '0': { type: 'danger', text: '禁用', icon: CircleClose },

  // 健康状态
  healthy: { type: 'success', text: '健康', icon: CircleCheck },
  unhealthy: { type: 'danger', text: '不健康', icon: CircleClose },
  degraded: { type: 'warning', text: '降级', icon: Warning },

  // 告警级别
  critical: { type: 'danger', text: '严重', icon: CircleClose },
  high: { type: 'danger', text: '高', icon: Warning },
  medium: { type: 'warning', text: '中', icon: Warning },
  low: { type: 'info', text: '低', icon: QuestionFilled },

  // 数据质量
  good: { type: 'success', text: '良好', icon: CircleCheck },
  bad: { type: 'danger', text: '异常', icon: CircleClose },
  uncertain: { type: 'warning', text: '不确定', icon: Warning },
}

// 获取状态配置
const statusConfig = computed(() => {
  const statusKey = String(props.status).toLowerCase()
  const mapping = props.mappings[statusKey] || defaultMappings[statusKey]

  if (mapping) {
    return mapping
  }

  // 如果没有匹配的映射，返回默认配置
  return {
    type: '' as const,
    text: String(props.status),
    icon: undefined,
  }
})

// 计算属性
const tagType = computed(() => statusConfig.value.type)
const displayText = computed(() => statusConfig.value.text)
const icon = computed(() => statusConfig.value.icon)

const tagClass = computed(() => {
  const classes = []

  if (props.clickable) {
    classes.push('status-tag-clickable')
  }

  if (props.customClass) {
    classes.push(props.customClass)
  }

  if (statusConfig.value.color) {
    classes.push('status-tag-custom-color')
  }

  return classes.join(' ')
})

// 事件处理
const handleClose = (event: Event) => {
  emit('close', event)
}

const handleClick = (event: Event) => {
  if (props.clickable) {
    emit('click', event)
  }
}
</script>

<style scoped lang="scss">
.el-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;

  .status-icon {
    font-size: 12px;
  }

  .status-text {
    line-height: 1;
  }

  .status-badge {
    margin-left: 4px;
  }

  &.status-tag-clickable {
    cursor: pointer;
    transition: all 0.3s;

    &:hover {
      transform: translateY(-1px);
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.12);
    }
  }

  &.status-tag-custom-color {
    // 自定义颜色样式可以在这里添加
  }
}

// 特定状态的动画效果
.el-tag {
  // 加载状态的旋转动画
  &:has(.status-icon .el-icon-loading) {
    .status-icon {
      animation: rotate 2s linear infinite;
    }
  }
}

@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

// 尺寸变体
.el-tag--large {
  .status-icon {
    font-size: 14px;
  }
}

.el-tag--small {
  .status-icon {
    font-size: 10px;
  }
}
</style>
