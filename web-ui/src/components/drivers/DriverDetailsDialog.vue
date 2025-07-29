<template>
  <el-dialog
    v-model="dialogVisible"
    :title="`驱动详情 - ${driver?.filename || ''}`"
    width="800px"
    @closed="handleDialogClosed"
  >
    <div v-if="driver" class="driver-details">
      <!-- 基本信息 -->
      <el-card class="detail-card" shadow="never">
        <template #header>
          <div class="card-header">
            <el-icon><InfoFilled /></el-icon>
            <span>基本信息</span>
          </div>
        </template>
        
        <div class="detail-grid">
          <div class="detail-item">
            <label class="detail-label">文件名:</label>
            <span class="detail-value filename">{{ driver.filename }}</span>
          </div>
          
          <div class="detail-item">
            <label class="detail-label">文件大小:</label>
            <span class="detail-value">{{ formatFileSize(driver.file_size) }}</span>
          </div>
          
          <div class="detail-item">
            <label class="detail-label">文件路径:</label>
            <span class="detail-value filepath">{{ driver.file_path || '未指定' }}</span>
          </div>
          
          <div class="detail-item">
            <label class="detail-label">加载状态:</label>
            <el-tag :type="getStatusTagType(driver.status)" size="default">
              <el-icon class="status-icon">
                <CircleCheck v-if="driver.status === 'Loaded'" />
                <CircleClose v-else-if="driver.status === 'Failed'" />
                <Warning v-else />
              </el-icon>
              {{ getStatusDisplayName(driver.status) }}
            </el-tag>
          </div>
          
          <div class="detail-item">
            <label class="detail-label">上传时间:</label>
            <span class="detail-value">{{ formatDateTime(driver.uploaded_at) }}</span>
          </div>
          
          <div class="detail-item">
            <label class="detail-label">最后加载:</label>
            <span class="detail-value">
              {{ driver.last_loaded_at ? formatDateTime(driver.last_loaded_at) : '从未加载' }}
            </span>
          </div>
        </div>
      </el-card>

      <!-- 驱动信息 -->
      <el-card v-if="driver.info" class="detail-card" shadow="never">
        <template #header>
          <div class="card-header">
            <el-icon><Cpu /></el-icon>
            <span>驱动信息</span>
          </div>
        </template>
        
        <div class="detail-grid">
          <div class="detail-item">
            <label class="detail-label">驱动名称:</label>
            <span class="detail-value">{{ driver.info.name || '未提供' }}</span>
          </div>
          
          <div class="detail-item">
            <label class="detail-label">版本:</label>
            <span class="detail-value version">{{ driver.info.version || '未提供' }}</span>
          </div>
          
          <div class="detail-item">
            <label class="detail-label">作者:</label>
            <span class="detail-value">{{ driver.info.author || '未提供' }}</span>
          </div>
          
          <div class="detail-item">
            <label class="detail-label">支持协议:</label>
            <el-tag v-if="driver.info.protocol" :type="getProtocolTagType(driver.info.protocol)">
              {{ getProtocolDisplayName(driver.info.protocol) }}
            </el-tag>
            <span v-else class="detail-value">未指定</span>
          </div>
          
          <div class="detail-item full-width">
            <label class="detail-label">描述:</label>
            <p class="detail-value description">
              {{ driver.info.description || '无描述信息' }}
            </p>
          </div>
        </div>
      </el-card>

      <!-- 配置信息 -->
      <el-card v-if="driver.config" class="detail-card" shadow="never">
        <template #header>
          <div class="card-header">
            <el-icon><Setting /></el-icon>
            <span>配置信息</span>
          </div>
        </template>
        
        <div class="config-content">
          <el-input
            type="textarea"
            :model-value="formatJsonConfig(driver.config)"
            :rows="8"
            readonly
            class="config-textarea"
          />
        </div>
      </el-card>

      <!-- 错误信息 -->
      <el-card v-if="driver.error_message" class="detail-card error-card" shadow="never">
        <template #header>
          <div class="card-header">
            <el-icon><WarningFilled /></el-icon>
            <span>错误信息</span>
          </div>
        </template>
        
        <div class="error-content">
          <el-alert
            type="error"
            :closable="false"
            show-icon
          >
            <div class="error-message">
              <pre>{{ driver.error_message }}</pre>
            </div>
          </el-alert>
        </div>
      </el-card>

      <!-- 操作历史 -->
      <el-card class="detail-card" shadow="never">
        <template #header>
          <div class="card-header">
            <el-icon><Clock /></el-icon>
            <span>操作历史</span>
          </div>
        </template>
        
        <div class="history-content">
          <el-timeline>
            <el-timeline-item
              timestamp="最后加载"
              :time="driver.last_loaded_at ? formatDateTime(driver.last_loaded_at) : '从未加载'"
              type="primary"
            >
              <span v-if="driver.last_loaded_at">驱动重新加载</span>
              <span v-else>尚未加载过</span>
            </el-timeline-item>
            
            <el-timeline-item
              timestamp="文件上传"
              :time="formatDateTime(driver.uploaded_at)"
              type="success"
            >
              驱动文件上传完成
            </el-timeline-item>
          </el-timeline>
        </div>
      </el-card>
    </div>

    <!-- 对话框底部按钮 -->
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="dialogVisible = false">
          关闭
        </el-button>
        <el-button
          v-if="driver?.status !== 'Loaded'"
          type="primary"
          @click="handleReload"
        >
          重新加载
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * DriverDetailsDialog —— 驱动详情对话框
 *
 * 📝 Responsibilities:
 *  1. 显示驱动完整信息
 *  2. 格式化展示配置数据
 *  3. 显示错误信息和历史
 *  4. 提供重新加载操作
 *
 * 📦 Dependencies:
 *  - Element Plus Dialog 组件
 *  - 日期格式化工具
 *  - 文件大小格式化工具
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { computed } from 'vue'
import { ElMessage } from 'element-plus'
import { 
  InfoFilled, 
  Cpu, 
  Setting, 
  WarningFilled, 
  Clock,
  CircleCheck,
  CircleClose,
  Warning
} from '@element-plus/icons-vue'
import { formatDateTime } from '@/utils/date'
import { formatFileSize } from '@/utils/format'
import { useDriversStore } from '@/stores'
import type { DriverVO } from '@/api/drivers'

// ===== Props =====
const props = defineProps<{
  visible: boolean
  driver: DriverVO | null
}>()

// ===== Emits =====
const emit = defineEmits<{
  'update:visible': [visible: boolean]
}>()

// ===== Store =====
const driversStore = useDriversStore()

// ===== 计算属性 =====
const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

// ===== 方法 =====

/**
 * 获取状态标签样式
 */
function getStatusTagType(status: string): string {
  const typeMap: Record<string, string> = {
    'Loaded': 'success',
    'Failed': 'danger',
    'Unloaded': 'warning',
  }
  return typeMap[status] || 'info'
}

/**
 * 获取状态显示名称
 */
function getStatusDisplayName(status: string): string {
  const nameMap: Record<string, string> = {
    'Loaded': '已加载',
    'Failed': '加载失败',
    'Unloaded': '未加载',
  }
  return nameMap[status] || status
}

/**
 * 获取协议类型标签样式
 */
function getProtocolTagType(protocol: string): string {
  const typeMap: Record<string, string> = {
    'ModbusTcp': 'primary',
    'ModbusRtu': 'success',
    'OpcUa': 'warning',
    'Mqtt': 'info',
    'Http': '',
    'Custom': 'info',
  }
  return typeMap[protocol] || 'default'
}

/**
 * 获取协议显示名称
 */
function getProtocolDisplayName(protocol: string): string {
  const nameMap: Record<string, string> = {
    'ModbusTcp': 'Modbus TCP',
    'ModbusRtu': 'Modbus RTU',
    'OpcUa': 'OPC UA',
    'Mqtt': 'MQTT',
    'Http': 'HTTP',
    'Custom': '自定义',
  }
  return nameMap[protocol] || protocol
}

/**
 * 格式化JSON配置
 */
function formatJsonConfig(config: any): string {
  try {
    return JSON.stringify(config, null, 2)
  } catch {
    return String(config)
  }
}

/**
 * 处理重新加载
 */
async function handleReload() {
  if (!props.driver) return
  
  const success = await driversStore.reloadDriver(props.driver.id)
  if (success) {
    ElMessage.success('驱动重新加载成功')
    dialogVisible.value = false
  }
}

/**
 * 处理对话框关闭
 */
function handleDialogClosed() {
  // 可在此处理清理逻辑
}
</script>

<style scoped lang="scss">
.driver-details {
  .detail-card {
    margin-bottom: 20px;
    
    &:last-child {
      margin-bottom: 0;
    }
    
    &.error-card {
      .el-card__header {
        background-color: #fef0f0;
        border-bottom: 1px solid #fde2e2;
      }
    }
    
    .card-header {
      display: flex;
      align-items: center;
      gap: 8px;
      font-weight: 600;
      color: #303133;
      
      .el-icon {
        font-size: 16px;
        color: #409eff;
      }
    }
  }
  
  .detail-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px 24px;
    
    .detail-item {
      display: flex;
      flex-direction: column;
      gap: 4px;
      
      &.full-width {
        grid-column: 1 / -1;
      }
      
      .detail-label {
        font-size: 13px;
        color: #909399;
        font-weight: 500;
        margin: 0;
      }
      
      .detail-value {
        font-size: 14px;
        color: #303133;
        word-break: break-all;
        
        &.filename {
          font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
          background-color: #f5f5f5;
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 13px;
        }
        
        &.filepath {
          font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
          font-size: 13px;
          color: #606266;
        }
        
        &.version {
          font-weight: 600;
          color: #409eff;
        }
        
        &.description {
          line-height: 1.5;
          margin: 0;
          padding: 8px 12px;
          background-color: #f9f9f9;
          border-radius: 4px;
          border-left: 3px solid #e1e3e9;
        }
      }
      
      .status-icon {
        margin-right: 4px;
        font-size: 12px;
      }
    }
  }
  
  .config-content {
    .config-textarea {
      :deep(.el-textarea__inner) {
        font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
        font-size: 13px;
        line-height: 1.4;
        background-color: #f8f8f8;
        border: 1px solid #e4e7ed;
      }
    }
  }
  
  .error-content {
    .error-message {
      pre {
        margin: 0;
        white-space: pre-wrap;
        word-wrap: break-word;
        font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
        font-size: 13px;
        line-height: 1.4;
      }
    }
  }
  
  .history-content {
    :deep(.el-timeline) {
      padding-left: 0;
      
      .el-timeline-item__timestamp {
        font-weight: 500;
        color: #606266;
      }
      
      .el-timeline-item__content {
        color: #303133;
      }
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

// 响应式设计
@media (max-width: 768px) {
  .driver-details {
    .detail-grid {
      grid-template-columns: 1fr;
      gap: 12px;
      
      .detail-item {
        &.full-width {
          grid-column: 1;
        }
        
        .detail-value {
          &.filename,
          &.filepath {
            font-size: 12px;
          }
        }
      }
    }
    
    .config-content {
      .config-textarea {
        :deep(.el-textarea__inner) {
          font-size: 12px;
        }
      }
    }
    
    .error-content {
      .error-message {
        pre {
          font-size: 12px;
        }
      }
    }
  }
}
</style>