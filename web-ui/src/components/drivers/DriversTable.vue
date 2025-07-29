<template>
  <div class="drivers-table">
    <el-table
      :data="drivers"
      v-loading="loading"
      @selection-change="handleSelectionChange"
      empty-text="暂无驱动数据"
      stripe
      style="width: 100%"
    >
      <!-- 选择列 -->
      <el-table-column type="selection" width="50" />
      
      <!-- 驱动名称 -->
      <el-table-column prop="filename" label="驱动文件" min-width="200">
        <template #default="{ row }">
          <div class="driver-name">
            <el-icon class="driver-icon">
              <Cpu />
            </el-icon>
            <div class="name-content">
              <div class="name-text">{{ row.filename }}</div>
              <div v-if="row.info?.name" class="description-text">{{ row.info.name }}</div>
            </div>
          </div>
        </template>
      </el-table-column>
      
      <!-- 版本信息 -->
      <el-table-column label="版本信息" min-width="150">
        <template #default="{ row }">
          <div class="version-info">
            <div v-if="row.info?.version" class="version-item">
              <span class="label">版本:</span>
              <span class="value">{{ row.info.version }}</span>
            </div>
            <div v-if="row.info?.author" class="version-item">
              <span class="label">作者:</span>
              <span class="value">{{ row.info.author }}</span>
            </div>
          </div>
        </template>
      </el-table-column>
      
      <!-- 支持协议 -->
      <el-table-column label="支持协议" width="120">
        <template #default="{ row }">
          <el-tag v-if="row.info?.protocol" :type="getProtocolTagType(row.info.protocol)" size="small">
            {{ getProtocolDisplayName(row.info.protocol) }}
          </el-tag>
          <span v-else class="no-data">未知</span>
        </template>
      </el-table-column>
      
      <!-- 加载状态 -->
      <el-table-column prop="status" label="加载状态" width="100">
        <template #default="{ row }">
          <el-tag :type="getStatusTagType(row.status)" size="small">
            <el-icon class="status-icon">
              <CircleCheck v-if="row.status === 'Loaded'" />
              <CircleClose v-else-if="row.status === 'Failed'" />
              <Warning v-else />
            </el-icon>
            {{ getStatusDisplayName(row.status) }}
          </el-tag>
        </template>
      </el-table-column>
      
      <!-- 文件大小 -->
      <el-table-column prop="file_size" label="文件大小" width="100">
        <template #default="{ row }">
          {{ formatFileSize(row.file_size) }}
        </template>
      </el-table-column>
      
      <!-- 上传时间 -->
      <el-table-column prop="uploaded_at" label="上传时间" width="160">
        <template #default="{ row }">
          {{ formatDateTime(row.uploaded_at) }}
        </template>
      </el-table-column>
      
      <!-- 最后加载时间 -->
      <el-table-column prop="last_loaded_at" label="最后加载" width="160">
        <template #default="{ row }">
          <span v-if="row.last_loaded_at">{{ formatDateTime(row.last_loaded_at) }}</span>
          <span v-else class="no-data">从未加载</span>
        </template>
      </el-table-column>
      
      <!-- 错误信息 -->
      <el-table-column label="错误信息" min-width="200">
        <template #default="{ row }">
          <div v-if="row.error_message" class="error-message">
            <el-tooltip :content="row.error_message" placement="top" :show-after="1000">
              <span class="error-text">{{ truncateText(row.error_message, 50) }}</span>
            </el-tooltip>
          </div>
          <span v-else class="no-data">无</span>
        </template>
      </el-table-column>
      
      <!-- 操作列 -->
      <el-table-column label="操作" width="180" fixed="right">
        <template #default="{ row }">
          <div class="action-buttons">
            <!-- 查看详情 -->
            <el-tooltip content="查看详情" placement="top">
              <el-button
                type="primary"
                link
                size="small"
                @click="$emit('view-details', row)"
              >
                <el-icon><View /></el-icon>
              </el-button>
            </el-tooltip>
            
            <!-- 重新加载 -->
            <el-tooltip content="重新加载" placement="top">
              <el-button
                type="warning"
                link
                size="small"
                :disabled="row.status === 'Loaded'"
                @click="$emit('reload-driver', row)"
              >
                <el-icon><Refresh /></el-icon>
              </el-button>
            </el-tooltip>
            
            <!-- 删除 -->
            <el-tooltip content="删除驱动" placement="top">
              <el-button
                type="danger"
                link
                size="small"
                @click="$emit('delete-driver', row)"
              >
                <el-icon><Delete /></el-icon>
              </el-button>
            </el-tooltip>
          </div>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<script setup lang="ts">
/**
 * DriversTable —— 驱动列表表格组件
 *
 * 📝 Responsibilities:
 *  1. 显示驱动列表数据
 *  2. 支持选择操作
 *  3. 提供驱动操作按钮
 *  4. 格式化显示驱动信息
 *
 * 📦 Dependencies:
 *  - Element Plus Table 组件
 *  - 日期格式化工具
 *  - 文件大小格式化工具
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import type { DriverVO } from '@/api/drivers'
import { 
  Cpu, 
  CircleCheck, 
  CircleClose, 
  Warning, 
  View, 
  Refresh, 
  Delete 
} from '@element-plus/icons-vue'
import { formatDateTime } from '@/utils/date'
import { formatFileSize, truncateText } from '@/utils/format'

// ===== Props =====
defineProps<{
  drivers: DriverVO[]
  loading?: boolean
  selectedDrivers?: DriverVO[]
}>()

// ===== Emits =====
const emit = defineEmits<{
  'selection-change': [drivers: DriverVO[]]
  'delete-driver': [driver: DriverVO]
  'reload-driver': [driver: DriverVO]
  'view-details': [driver: DriverVO]
}>()

// ===== 方法 =====

/**
 * 处理选择变更
 */
function handleSelectionChange(selection: DriverVO[]) {
  emit('selection-change', selection)
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
</script>

<style scoped lang="scss">
.drivers-table {
  .driver-name {
    display: flex;
    align-items: center;
    gap: 8px;
    
    .driver-icon {
      color: #409eff;
      font-size: 16px;
      flex-shrink: 0;
    }
    
    .name-content {
      min-width: 0;
      flex: 1;
      
      .name-text {
        font-weight: 500;
        color: #303133;
        margin-bottom: 2px;
        font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
      }
      
      .description-text {
        font-size: 12px;
        color: #909399;
        line-height: 1.2;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }
    }
  }
  
  .version-info {
    .version-item {
      margin-bottom: 4px;
      font-size: 13px;
      line-height: 1.4;
      
      &:last-child {
        margin-bottom: 0;
      }
      
      .label {
        color: #909399;
        margin-right: 4px;
      }
      
      .value {
        color: #303133;
        font-weight: 500;
      }
    }
  }
  
  .status-icon {
    margin-right: 4px;
    font-size: 12px;
  }
  
  .error-message {
    .error-text {
      color: #f56c6c;
      font-size: 12px;
      font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
      line-height: 1.3;
      cursor: help;
    }
  }
  
  .no-data {
    color: #c0c4cc;
    font-style: italic;
    font-size: 12px;
  }
  
  .action-buttons {
    display: flex;
    align-items: center;
    gap: 4px;
    
    .el-button {
      padding: 4px;
      
      .el-icon {
        font-size: 14px;
      }
      
      &:disabled {
        opacity: 0.5;
        cursor: not-allowed;
      }
    }
  }
}

// 表格样式优化
:deep(.el-table) {
  .el-table__header-wrapper {
    .el-table__header {
      th {
        background-color: #fafafa;
        border-bottom: 1px solid #ebeef5;
        
        .cell {
          font-weight: 600;
          color: #303133;
        }
      }
    }
  }
  
  .el-table__body-wrapper {
    .el-table__body {
      tr {
        &:hover {
          background-color: #f5f7fa;
        }
        
        td {
          border-bottom: 1px solid #f0f0f0;
          
          .cell {
            padding: 8px 0;
          }
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .drivers-table {
    :deep(.el-table) {
      .el-table__body-wrapper {
        overflow-x: auto;
      }
    }
    
    .driver-name {
      .name-content {
        .description-text {
          display: none;
        }
      }
    }
    
    .version-info {
      .version-item {
        font-size: 12px;
      }
    }
    
    .action-buttons {
      flex-direction: column;
      gap: 2px;
      
      .el-button {
        width: 100%;
        justify-content: center;
      }
    }
  }
}
</style>