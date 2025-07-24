<template>
  <el-card 
    :class="cardClass" 
    :shadow="shadow"
    :body-style="bodyStyle"
  >
    <!-- 自定义头部插槽 -->
    <template v-if="$slots.header" #header>
      <slot name="header" />
    </template>

    <div class="loading-content" :class="contentClass">
      <!-- 加载状态 -->
      <div v-if="status === 'loading'" class="loading-state">
        <!-- 自定义加载动画插槽 -->
        <slot name="loading" :loadingText="loadingText">
          <!-- 默认加载动画 -->
          <div class="loading-animation" :class="animationType">
            <!-- 旋转器动画 -->
            <div v-if="animationType === 'spinner'" class="spinner">
              <el-icon :size="iconSize" class="loading-spinner">
                <Loading />
              </el-icon>
            </div>
            
            <!-- 点状动画 -->
            <div v-else-if="animationType === 'dots'" class="dots">
              <span class="dot"></span>
              <span class="dot"></span>
              <span class="dot"></span>
            </div>
            
            <!-- 波浪动画 -->
            <div v-else-if="animationType === 'wave'" class="wave">
              <span class="wave-bar"></span>
              <span class="wave-bar"></span>
              <span class="wave-bar"></span>
              <span class="wave-bar"></span>
            </div>
            
            <!-- 脉冲动画 -->
            <div v-else-if="animationType === 'pulse'" class="pulse">
              <div class="pulse-circle"></div>
            </div>
          </div>
          
          <!-- 加载文本 -->
          <div v-if="showText" class="loading-text">
            {{ loadingText }}
            <span v-if="showProgress && progress !== null" class="loading-progress">
              ({{ progress }}%)
            </span>
          </div>
        </slot>
      </div>
      
      <!-- 骨架屏状态 -->
      <div v-else-if="status === 'skeleton'" class="skeleton-state">
        <slot name="skeleton">
          <el-skeleton 
            :rows="skeletonRows" 
            :animated="skeletonAnimated"
            :avatar="skeletonAvatar"
            :avatar-size="skeletonAvatarSize"
            :loading="true"
          />
        </slot>
      </div>
      
      <!-- 错误状态 -->
      <div v-else-if="status === 'error'" class="error-state">
        <slot name="error" :retry="handleRetry" :errorMessage="errorMessage">
          <div class="error-content">
            <el-icon :size="iconSize" class="error-icon">
              <CircleCloseFilled />
            </el-icon>
            
            <div class="error-message">
              {{ errorMessage || '加载失败' }}
            </div>
            
            <el-button 
              v-if="showRetry"
              type="primary" 
              size="small" 
              @click="handleRetry"
              :loading="retrying"
            >
              重试
            </el-button>
          </div>
        </slot>
      </div>
      
      <!-- 空状态 -->
      <div v-else-if="status === 'empty'" class="empty-state">
        <slot name="empty">
          <div class="empty-content">
            <el-icon :size="iconSize" class="empty-icon">
              <DocumentDelete />
            </el-icon>
            
            <div class="empty-message">
              {{ emptyMessage || '暂无数据' }}
            </div>
          </div>
        </slot>
      </div>
      
      <!-- 成功状态 - 显示内容 -->
      <div v-else-if="status === 'success'" class="success-state">
        <slot name="content" />
      </div>
      
      <!-- 自定义状态 -->
      <div v-else class="custom-state">
        <slot />
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Loading, CircleCloseFilled, DocumentDelete } from '@element-plus/icons-vue'

export type LoadingStatus = 'loading' | 'skeleton' | 'error' | 'empty' | 'success' | 'custom'
export type AnimationType = 'spinner' | 'dots' | 'wave' | 'pulse'

interface Props {
  status: LoadingStatus
  
  // 加载配置
  loadingText?: string
  showText?: boolean
  showProgress?: boolean
  progress?: number | null
  animationType?: AnimationType
  
  // 外观配置
  size?: 'small' | 'default' | 'large'
  minHeight?: string
  iconSize?: number
  shadow?: 'always' | 'hover' | 'never'
  
  // 骨架屏配置
  skeletonRows?: number
  skeletonAnimated?: boolean
  skeletonAvatar?: boolean
  skeletonAvatarSize?: 'large' | 'default' | 'small'
  
  // 错误配置
  errorMessage?: string
  showRetry?: boolean
  retrying?: boolean
  
  // 空状态配置
  emptyMessage?: string
  
  // 自定义样式
  customClass?: string
  centerContent?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  status: 'loading',
  loadingText: '正在加载...',
  showText: true,
  showProgress: false,
  progress: null,
  animationType: 'spinner',
  size: 'default',
  minHeight: '200px',
  iconSize: 40,
  shadow: 'never',
  skeletonRows: 3,
  skeletonAnimated: true,
  skeletonAvatar: false,
  skeletonAvatarSize: 'default',
  showRetry: true,
  retrying: false,
  centerContent: true,
})

interface Emits {
  retry: []
}

const emit = defineEmits<Emits>()

// 计算样式类
const cardClass = computed(() => {
  const classes = ['loading-card']
  
  classes.push(`loading-card--${props.size}`)
  classes.push(`loading-card--${props.status}`)
  
  if (props.customClass) {
    classes.push(props.customClass)
  }
  
  return classes.join(' ')
})

const contentClass = computed(() => {
  const classes = []
  
  if (props.centerContent) {
    classes.push('loading-content--center')
  }
  
  return classes.join(' ')
})

// 动态body样式
const bodyStyle = computed(() => {
  return {
    minHeight: props.minHeight,
    display: 'flex',
    flexDirection: 'column',
    justifyContent: props.centerContent ? 'center' : 'flex-start',
  }
})

// 事件处理
const handleRetry = () => {
  emit('retry')
}
</script>

<style scoped lang="scss">
.loading-card {
  width: 100%;
  
  &.loading-card--small {
    .loading-text {
      font-size: 12px;
    }
    
    .error-message,
    .empty-message {
      font-size: 13px;
    }
  }
  
  &.loading-card--large {
    .loading-text {
      font-size: 16px;
    }
    
    .error-message,
    .empty-message {
      font-size: 15px;
    }
  }
}

.loading-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  
  &.loading-content--center {
    text-align: center;
  }
}

// 加载状态样式
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  
  .loading-animation {
    display: flex;
    align-items: center;
    justify-content: center;
    
    // 旋转器动画
    &.spinner {
      .loading-spinner {
        color: var(--el-color-primary);
        animation: spin 1.5s linear infinite;
      }
    }
    
    // 点状动画
    &.dots {
      display: flex;
      gap: 4px;
      
      .dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background-color: var(--el-color-primary);
        animation: dots-bounce 1.4s infinite ease-in-out;
        
        &:nth-child(1) { animation-delay: -0.32s; }
        &:nth-child(2) { animation-delay: -0.16s; }
        &:nth-child(3) { animation-delay: 0s; }
      }
    }
    
    // 波浪动画
    &.wave {
      display: flex;
      gap: 2px;
      align-items: center;
      
      .wave-bar {
        width: 4px;
        height: 20px;
        background-color: var(--el-color-primary);
        animation: wave-scale 1.2s infinite ease-in-out;
        
        &:nth-child(1) { animation-delay: -1.1s; }
        &:nth-child(2) { animation-delay: -1.0s; }
        &:nth-child(3) { animation-delay: -0.9s; }
        &:nth-child(4) { animation-delay: -0.8s; }
      }
    }
    
    // 脉冲动画
    &.pulse {
      .pulse-circle {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background-color: var(--el-color-primary);
        animation: pulse-scale 1.5s infinite;
      }
    }
  }
  
  .loading-text {
    color: var(--el-text-color-secondary);
    font-size: 14px;
    
    .loading-progress {
      color: var(--el-color-primary);
      font-weight: 500;
    }
  }
}

// 错误状态样式
.error-state {
  .error-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    
    .error-icon {
      color: var(--el-color-danger);
    }
    
    .error-message {
      color: var(--el-text-color-secondary);
      font-size: 14px;
      text-align: center;
    }
  }
}

// 空状态样式
.empty-state {
  .empty-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    
    .empty-icon {
      color: var(--el-text-color-placeholder);
    }
    
    .empty-message {
      color: var(--el-text-color-secondary);
      font-size: 14px;
      text-align: center;
    }
  }
}

// 动画定义
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes dots-bounce {
  0%, 80%, 100% {
    transform: scale(0);
  }
  40% {
    transform: scale(1);
  }
}

@keyframes wave-scale {
  0%, 40%, 100% {
    transform: scaleY(0.4);
  }
  20% {
    transform: scaleY(1);
  }
}

@keyframes pulse-scale {
  0% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.1);
    opacity: 0.7;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .loading-card {
    .loading-animation {
      transform: scale(0.8);
    }
    
    .loading-text,
    .error-message,
    .empty-message {
      font-size: 13px;
    }
  }
}
</style>