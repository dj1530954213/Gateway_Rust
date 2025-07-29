<template>
  <div class="alert-notification-list">
    <div v-if="loading" class="list-loading">
      <el-skeleton :rows="3" animated />
    </div>
    
    <div v-else-if="alerts.length === 0" class="list-empty">
      <el-empty description="暂无告警信息" :image-size="80">
        <template #description>
          <p>系统运行正常，暂无告警</p>
        </template>
      </el-empty>
    </div>
    
    <div v-else class="alert-items">
      <div
        v-for="alert in alerts"
        :key="alert.id"
        class="alert-item"
        :class="[
          `level-${alert.level}`,
          { 'acknowledged': alert.acknowledged, 'unread': !alert.read }
        ]"
        @click="handleAlertClick(alert)"
      >
        <!-- 告警级别指示器 -->
        <div class="alert-indicator">
          <div class="indicator-icon" :class="`icon-${alert.level}`">
            <el-icon>
              <Warning v-if="alert.level === 'warning'" />
              <CircleClose v-else-if="alert.level === 'error'" />
              <QuestionFilled v-else-if="alert.level === 'critical'" />
              <InfoFilled v-else />
            </el-icon>
          </div>
          
          <!-- 未读标识 -->
          <div v-if="!alert.read" class="unread-badge"></div>
        </div>
        
        <!-- 告警内容 -->
        <div class="alert-content">
          <div class="alert-header">
            <div class="alert-title">{{ alert.title || alert.message }}</div>
            <div class="alert-meta">
              <el-tag 
                :type="getLevelTagType(alert.level)" 
                size="small"
                effect="plain"
              >
                {{ getLevelText(alert.level) }}
              </el-tag>
              <span class="alert-time">{{ formatRelativeTime(alert.created_at) }}</span>
            </div>
          </div>
          
          <div class="alert-body">
            <div class="alert-message">{{ alert.message }}</div>
            
            <div v-if="alert.source" class="alert-source">
              <el-icon><Link /></el-icon>
              <span>{{ alert.source }}</span>
            </div>
            
            <div v-if="alert.details" class="alert-details">
              <div class="details-toggle" @click.stop="toggleDetails(alert.id)">
                <el-icon>
                  <ArrowDown v-if="!expandedAlerts.has(alert.id)" />
                  <ArrowUp v-else />
                </el-icon>
                <span>详细信息</span>
              </div>
              
              <div v-if="expandedAlerts.has(alert.id)" class="details-content">
                <pre>{{ formatDetails(alert.details) }}</pre>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 告警操作 -->
        <div class="alert-actions">
          <el-dropdown
            trigger="click"
            placement="bottom-end"
            @command="handleAction"
          >
            <el-button type="text" size="small" @click.stop>
              <el-icon><MoreFilled /></el-icon>
            </el-button>
            
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item 
                  v-if="!alert.acknowledged"
                  :command="{ action: 'acknowledge', alert }"
                >
                  <el-icon><Check /></el-icon>
                  确认
                </el-dropdown-item>
                <el-dropdown-item 
                  v-if="!alert.resolved"
                  :command="{ action: 'resolve', alert }"
                >
                  <el-icon><CircleCheck /></el-icon>
                  解决
                </el-dropdown-item>
                <el-dropdown-item 
                  :command="{ action: 'mute', alert }"
                  divided
                >
                  <el-icon><Mute /></el-icon>
                  静音
                </el-dropdown-item>
                <el-dropdown-item 
                  :command="{ action: 'details', alert }"
                >
                  <el-icon><View /></el-icon>
                  查看详情
                </el-dropdown-item>
                <el-dropdown-item 
                  :command="{ action: 'delete', alert }"
                  class="danger-item"
                  divided
                >
                  <el-icon><Delete /></el-icon>
                  删除
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
        
        <!-- 处理状态标识 -->
        <div v-if="alert.acknowledged || alert.resolved" class="status-badges">
          <el-tag v-if="alert.acknowledged" type="info" size="small" effect="plain">
            已确认
          </el-tag>
          <el-tag v-if="alert.resolved" type="success" size="small" effect="plain">
            已解决
          </el-tag>
        </div>
      </div>
    </div>
    
    <!-- 批量操作栏 -->
    <div v-if="selectedAlerts.length > 0" class="batch-actions">
      <div class="batch-info">
        <span>已选择 {{ selectedAlerts.length }} 个告警</span>
        <el-button type="text" size="small" @click="clearSelection">
          取消选择
        </el-button>
      </div>
      
      <div class="batch-buttons">
        <el-button 
          size="small" 
          @click="batchAcknowledge"
          :disabled="selectedAlerts.every(a => a.acknowledged)"
        >
          批量确认
        </el-button>
        <el-button 
          size="small" 
          @click="batchResolve"
          :disabled="selectedAlerts.every(a => a.resolved)"
        >
          批量解决
        </el-button>
        <el-button 
          size="small" 
          type="danger" 
          @click="batchDelete"
        >
          批量删除
        </el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * AlertNotificationList —— 报警通知列表组件
 *
 * 📝 Responsibilities:
 *  1. 显示告警列表
 *  2. 提供告警操作功能
 *  3. 支持批量操作
 *  4. 告警详情展示
 *  5. 实时状态更新
 *
 * 📦 Dependencies:
 *  - 告警Store
 *  - 日期格式化工具
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { 
  Warning, 
  CircleClose, 
  QuestionFilled, 
  InfoFilled,
  Link,
  ArrowDown,
  ArrowUp,
  MoreFilled,
  Check,
  CircleCheck,
  Mute,
  View,
  Delete
} from '@element-plus/icons-vue'

import { formatDateTime } from '@/utils/date'

// ===== 接口定义 =====
interface AlertItem {
  id: string
  title?: string
  message: string
  level: 'info' | 'warning' | 'error' | 'critical'
  source?: string
  details?: any
  created_at: string
  updated_at?: string
  acknowledged: boolean
  resolved: boolean
  read: boolean
  muted?: boolean
}

// ===== Props & Emits =====
const props = defineProps<{
  alerts: AlertItem[]
  loading?: boolean
}>()

const emit = defineEmits<{
  'alert-action': [action: string, alertId: string]
}>()

// ===== 响应式数据 =====
const expandedAlerts = ref(new Set<string>())
const selectedAlerts = ref<AlertItem[]>([])

// ===== 计算属性 =====
const unreadCount = computed(() => 
  props.alerts.filter(alert => !alert.read).length
)

// ===== 方法 =====

/**
 * 获取级别标签类型
 */
function getLevelTagType(level: string): string {
  const typeMap: Record<string, string> = {
    'info': 'info',
    'warning': 'warning',
    'error': 'danger',
    'critical': 'danger'
  }
  return typeMap[level] || 'info'
}

/**
 * 获取级别文本
 */
function getLevelText(level: string): string {
  const textMap: Record<string, string> = {
    'info': '信息',
    'warning': '警告',
    'error': '错误',
    'critical': '严重'
  }
  return textMap[level] || level
}

/**
 * 格式化相对时间
 */
function formatRelativeTime(time: string): string {
  const now = new Date()
  const alertTime = new Date(time)
  const diffMs = now.getTime() - alertTime.getTime()
  const diffMinutes = Math.floor(diffMs / 60000)
  
  if (diffMinutes < 1) return '刚刚'
  if (diffMinutes < 60) return `${diffMinutes}分钟前`
  if (diffMinutes < 1440) return `${Math.floor(diffMinutes / 60)}小时前`
  if (diffMinutes < 10080) return `${Math.floor(diffMinutes / 1440)}天前`
  
  return formatDateTime(time).split(' ')[0] // 只显示日期部分
}

/**
 * 格式化详细信息
 */
function formatDetails(details: any): string {
  if (typeof details === 'string') return details
  if (typeof details === 'object') {
    return JSON.stringify(details, null, 2)
  }
  return String(details)
}

/**
 * 切换详情展开
 */
function toggleDetails(alertId: string) {
  if (expandedAlerts.value.has(alertId)) {
    expandedAlerts.value.delete(alertId)
  } else {
    expandedAlerts.value.add(alertId)
  }
}

/**
 * 处理告警点击
 */
function handleAlertClick(alert: AlertItem) {
  // 标记为已读
  if (!alert.read) {
    alert.read = true
    emit('alert-action', 'mark-read', alert.id)
  }
  
  // 可以展开详情或跳转到详情页
  toggleDetails(alert.id)
}

/**
 * 处理操作命令
 */
async function handleAction(command: { action: string, alert: AlertItem }) {
  const { action, alert } = command
  
  try {
    switch (action) {
      case 'acknowledge':
        await acknowledgeAlert(alert)
        break
      case 'resolve':
        await resolveAlert(alert)
        break
      case 'mute':
        await muteAlert(alert)
        break
      case 'details':
        viewAlertDetails(alert)
        break
      case 'delete':
        await deleteAlert(alert)
        break
    }
  } catch (error) {
    ElMessage.error(`操作失败: ${error}`)
  }
}

/**
 * 确认告警
 */
async function acknowledgeAlert(alert: AlertItem) {
  alert.acknowledged = true
  emit('alert-action', 'acknowledge', alert.id)
  ElMessage.success('告警已确认')
}

/**
 * 解决告警
 */
async function resolveAlert(alert: AlertItem) {
  await ElMessageBox.confirm(
    '确定将此告警标记为已解决吗？',
    '确认解决',
    {
      type: 'info',
      confirmButtonText: '解决',
      cancelButtonText: '取消'
    }
  )
  
  alert.resolved = true
  alert.acknowledged = true
  emit('alert-action', 'resolve', alert.id)
  ElMessage.success('告警已解决')
}

/**
 * 静音告警
 */
async function muteAlert(alert: AlertItem) {
  const { value: duration } = await ElMessageBox.prompt(
    '请输入静音时长（分钟）：',
    '静音告警',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      inputPattern: /^\d+$/,
      inputErrorMessage: '请输入有效的数字'
    }
  )
  
  alert.muted = true
  emit('alert-action', 'mute', alert.id)
  ElMessage.success(`告警已静音 ${duration} 分钟`)
}

/**
 * 查看告警详情
 */
function viewAlertDetails(alert: AlertItem) {
  ElMessage.info('查看告警详情功能开发中...')
}

/**
 * 删除告警
 */
async function deleteAlert(alert: AlertItem) {
  await ElMessageBox.confirm(
    '确定要删除此告警吗？删除后无法恢复。',
    '确认删除',
    {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      confirmButtonClass: 'el-button--danger'
    }
  )
  
  emit('alert-action', 'delete', alert.id)
  ElMessage.success('告警已删除')
}

/**
 * 清除选择
 */
function clearSelection() {
  selectedAlerts.value = []
}

/**
 * 批量确认
 */
async function batchAcknowledge() {
  const unacknowledged = selectedAlerts.value.filter(a => !a.acknowledged)
  
  for (const alert of unacknowledged) {
    alert.acknowledged = true
    emit('alert-action', 'acknowledge', alert.id)
  }
  
  ElMessage.success(`批量确认了 ${unacknowledged.length} 个告警`)
  clearSelection()
}

/**
 * 批量解决
 */
async function batchResolve() {
  try {
    await ElMessageBox.confirm(
      `确定将选中的 ${selectedAlerts.value.length} 个告警标记为已解决吗？`,
      '批量解决确认',
      {
        type: 'info',
        confirmButtonText: '解决',
        cancelButtonText: '取消'
      }
    )
    
    const unresolved = selectedAlerts.value.filter(a => !a.resolved)
    
    for (const alert of unresolved) {
      alert.resolved = true
      alert.acknowledged = true
      emit('alert-action', 'resolve', alert.id)
    }
    
    ElMessage.success(`批量解决了 ${unresolved.length} 个告警`)
    clearSelection()
  } catch (error) {
    // 用户取消操作
  }
}

/**
 * 批量删除
 */
async function batchDelete() {
  try {
    await ElMessageBox.confirm(
      `确定要删除选中的 ${selectedAlerts.value.length} 个告警吗？删除后无法恢复。`,
      '批量删除确认',
      {
        type: 'warning',
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        confirmButtonClass: 'el-button--danger'
      }
    )
    
    for (const alert of selectedAlerts.value) {
      emit('alert-action', 'delete', alert.id)
    }
    
    ElMessage.success(`批量删除了 ${selectedAlerts.value.length} 个告警`)
    clearSelection()
  } catch (error) {
    // 用户取消操作
  }
}
</script>

<style scoped lang="scss">
.alert-notification-list {
  .list-loading {
    padding: 16px;
  }
  
  .list-empty {
    text-align: center;
    padding: 40px 16px;
  }
  
  .alert-items {
    max-height: 400px;
    overflow-y: auto;
    
    .alert-item {
      display: flex;
      gap: 12px;
      padding: 16px;
      border: 1px solid #ebeef5;
      border-radius: 6px;
      margin-bottom: 8px;
      cursor: pointer;
      transition: all 0.3s;
      position: relative;
      
      &:hover {
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
      }
      
      &.unread {
        background: #f0f9ff;
        border-color: #409eff;
      }
      
      &.acknowledged {
        opacity: 0.8;
      }
      
      &.level-info {
        border-left: 4px solid #409eff;
      }
      
      &.level-warning {
        border-left: 4px solid #e6a23c;
      }
      
      &.level-error {
        border-left: 4px solid #f56c6c;
      }
      
      &.level-critical {
        border-left: 4px solid #f56c6c;
        background: #fdf6f6;
      }
      
      .alert-indicator {
        position: relative;
        
        .indicator-icon {
          width: 32px;
          height: 32px;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          
          .el-icon {
            font-size: 16px;
          }
          
          &.icon-info {
            background: #e1f3ff;
            color: #409eff;
          }
          
          &.icon-warning {
            background: #fdf6ec;
            color: #e6a23c;
          }
          
          &.icon-error,
          &.icon-critical {
            background: #fef0f0;
            color: #f56c6c;
          }
        }
        
        .unread-badge {
          position: absolute;
          top: -2px;
          right: -2px;
          width: 8px;
          height: 8px;
          background: #f56c6c;
          border-radius: 50%;
          border: 2px solid white;
        }
      }
      
      .alert-content {
        flex: 1;
        min-width: 0;
        
        .alert-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 8px;
          
          .alert-title {
            font-size: 15px;
            font-weight: 600;
            color: #303133;
            margin-bottom: 4px;
          }
          
          .alert-meta {
            display: flex;
            align-items: center;
            gap: 8px;
            
            .alert-time {
              font-size: 12px;
              color: #909399;
            }
          }
        }
        
        .alert-body {
          .alert-message {
            font-size: 14px;
            color: #606266;
            line-height: 1.4;
            margin-bottom: 8px;
          }
          
          .alert-source {
            display: flex;
            align-items: center;
            gap: 4px;
            font-size: 12px;
            color: #909399;
            margin-bottom: 8px;
            
            .el-icon {
              font-size: 12px;
            }
          }
          
          .alert-details {
            .details-toggle {
              display: flex;
              align-items: center;
              gap: 4px;
              font-size: 12px;
              color: #409eff;
              cursor: pointer;
              margin-bottom: 8px;
              
              &:hover {
                color: #66b1ff;
              }
              
              .el-icon {
                font-size: 12px;
              }
            }
            
            .details-content {
              background: #f5f7fa;
              border: 1px solid #e4e7ed;
              border-radius: 4px;
              padding: 8px;
              
              pre {
                font-size: 11px;
                color: #606266;
                margin: 0;
                white-space: pre-wrap;
                word-wrap: break-word;
                font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
              }
            }
          }
        }
      }
      
      .alert-actions {
        .el-button {
          padding: 4px;
        }
      }
      
      .status-badges {
        position: absolute;
        top: 8px;
        right: 8px;
        display: flex;
        gap: 4px;
      }
    }
  }
  
  .batch-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #f5f7fa;
    border: 1px solid #e4e7ed;
    border-radius: 4px;
    margin-top: 16px;
    
    .batch-info {
      display: flex;
      align-items: center;
      gap: 12px;
      font-size: 14px;
      color: #606266;
    }
    
    .batch-buttons {
      display: flex;
      gap: 8px;
    }
  }
}

// 下拉菜单危险项样式
:deep(.danger-item) {
  color: #f56c6c !important;
  
  &:hover {
    background: #fef0f0 !important;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .alert-notification-list {
    .alert-items {
      .alert-item {
        flex-direction: column;
        gap: 8px;
        
        .alert-indicator {
          align-self: flex-start;
        }
        
        .alert-content {
          .alert-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 8px;
          }
        }
        
        .alert-actions {
          align-self: flex-end;
        }
        
        .status-badges {
          position: static;
          align-self: flex-start;
        }
      }
    }
    
    .batch-actions {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
      
      .batch-buttons {
        justify-content: center;
      }
    }
  }
}
</style>