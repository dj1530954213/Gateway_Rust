<template>
  <div class="tags-table">
    <el-table
      v-loading="loading"
      :data="tags"
      empty-text="暂无点位数据"
      stripe
      style="width: 100%"
      @selection-change="handleSelectionChange"
    >
      <!-- 选择列 -->
      <el-table-column type="selection" width="50" />

      <!-- 点位名称 -->
      <el-table-column prop="name" label="点位名称" min-width="180">
        <template #default="{ row }">
          <div class="tag-name">
            <el-icon class="tag-icon">
              <DataLine />
            </el-icon>
            <div class="name-content">
              <div class="name-text">{{ row.name }}</div>
              <div v-if="row.description" class="description-text">
                {{ row.description }}
              </div>
            </div>
          </div>
        </template>
      </el-table-column>

      <!-- 所属设备 -->
      <el-table-column label="所属设备" width="160">
        <template #default="{ row }">
          <div class="device-info">
            <el-tag type="info" size="small">
              {{ getDeviceName(row.device_id) }}
            </el-tag>
          </div>
        </template>
      </el-table-column>

      <!-- 数据类型 -->
      <el-table-column prop="data_type" label="数据类型" width="100">
        <template #default="{ row }">
          <el-tag :type="getDataTypeTagType(row.data_type)" size="small">
            {{ getDataTypeDisplayName(row.data_type) }}
          </el-tag>
        </template>
      </el-table-column>

      <!-- 访问模式 -->
      <el-table-column prop="access_mode" label="访问模式" width="100">
        <template #default="{ row }">
          <el-tag :type="getAccessModeTagType(row.access_mode)" size="small">
            {{ getAccessModeDisplayName(row.access_mode) }}
          </el-tag>
        </template>
      </el-table-column>

      <!-- 地址配置 -->
      <el-table-column label="地址配置" min-width="150">
        <template #default="{ row }">
          <div class="address-config">
            <div class="address-item">
              <span class="address-label">地址:</span>
              <span class="address-value">{{ row.address }}</span>
            </div>
            <div v-if="row.register_type" class="address-item">
              <span class="address-label">寄存器:</span>
              <span class="address-value">{{ row.register_type }}</span>
            </div>
          </div>
        </template>
      </el-table-column>

      <!-- 当前值 -->
      <el-table-column label="当前值" width="120">
        <template #default="{ row }">
          <div class="current-value">
            <span v-if="row.current_value !== null" class="value">
              {{ formatTagValue(row.current_value, row.data_type) }}
            </span>
            <span v-else class="no-value">--</span>
          </div>
        </template>
      </el-table-column>

      <!-- 状态 -->
      <el-table-column prop="enabled" label="状态" width="80">
        <template #default="{ row }">
          <el-tag :type="row.enabled ? 'success' : 'danger'" size="small">
            {{ row.enabled ? '已启用' : '已禁用' }}
          </el-tag>
        </template>
      </el-table-column>

      <!-- 最后更新 -->
      <el-table-column prop="updated_at" label="最后更新" width="160">
        <template #default="{ row }">
          {{ formatDateTime(row.updated_at) }}
        </template>
      </el-table-column>

      <!-- 操作列 -->
      <el-table-column label="操作" width="220" fixed="right">
        <template #default="{ row }">
          <div class="action-buttons">
            <!-- 测试读取 -->
            <el-tooltip content="测试读取" placement="top">
              <el-button
                type="primary"
                link
                size="small"
                :disabled="!canRead(row.access_mode)"
                @click="$emit('test-read', row)"
              >
                <el-icon><View /></el-icon>
              </el-button>
            </el-tooltip>

            <!-- 测试写入 -->
            <el-tooltip content="测试写入" placement="top">
              <el-button
                type="warning"
                link
                size="small"
                :disabled="!canWrite(row.access_mode)"
                @click="$emit('test-write', row)"
              >
                <el-icon><EditPen /></el-icon>
              </el-button>
            </el-tooltip>

            <!-- 启用/禁用 -->
            <el-tooltip
              :content="row.enabled ? '禁用点位' : '启用点位'"
              placement="top"
            >
              <el-button
                :type="row.enabled ? 'warning' : 'success'"
                link
                size="small"
                @click="$emit('toggle-tag', row)"
              >
                <el-icon v-if="row.enabled"><VideoPlay /></el-icon>
                <el-icon v-else><VideoPause /></el-icon>
              </el-button>
            </el-tooltip>

            <!-- 编辑 -->
            <el-tooltip content="编辑点位" placement="top">
              <el-button
                type="primary"
                link
                size="small"
                @click="$emit('edit-tag', row)"
              >
                <el-icon><Edit /></el-icon>
              </el-button>
            </el-tooltip>

            <!-- 删除 -->
            <el-tooltip content="删除点位" placement="top">
              <el-button
                type="danger"
                link
                size="small"
                @click="$emit('delete-tag', row)"
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
 * TagsTable —— 点位列表表格组件
 *
 * 📝 Responsibilities:
 *  1. 显示点位列表数据
 *  2. 支持选择操作
 *  3. 提供点位操作按钮
 *  4. 格式化显示点位信息
 *
 * 📦 Dependencies:
 *  - Element Plus Table 组件
 *  - 日期格式化工具
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  DataLine,
  View,
  EditPen,
  VideoPlay,
  VideoPause,
  Edit,
  Delete,
} from '@element-plus/icons-vue'
import { computed } from 'vue'

import type { DeviceVO } from '@/api/devices'
import type { TagVO } from '@/api/tags'
import { useDevicesStore } from '@/stores'
import { formatDateTime } from '@/utils/date'

// ===== Props =====
defineProps<{
  tags: TagVO[]
  loading?: boolean
  selectedTags?: TagVO[]
}>()

// ===== Emits =====
const emit = defineEmits<{
  'selection-change': [tags: TagVO[]]
  'edit-tag': [tag: TagVO]
  'delete-tag': [tag: TagVO]
  'toggle-tag': [tag: TagVO]
  'test-read': [tag: TagVO]
  'test-write': [tag: TagVO]
}>()

// ===== Store =====
const devicesStore = useDevicesStore()

// ===== 计算属性 =====
const devicesMap = computed(() => {
  const map: Record<string, DeviceVO> = {}
  devicesStore.state.devices.forEach(device => {
    map[device.id] = device
  })
  return map
})

// ===== 方法 =====

/**
 * 处理选择变更
 */
function handleSelectionChange(selection: TagVO[]) {
  emit('selection-change', selection)
}

/**
 * 获取设备名称
 */
function getDeviceName(deviceId: string): string {
  return devicesMap.value[deviceId]?.name || '未知设备'
}

/**
 * 获取数据类型标签样式
 */
function getDataTypeTagType(dataType: string): string {
  const typeMap: Record<string, string> = {
    Bool: 'success',
    Int32: 'primary',
    Float32: 'warning',
    String: 'info',
  }
  return typeMap[dataType] || 'default'
}

/**
 * 获取数据类型显示名称
 */
function getDataTypeDisplayName(dataType: string): string {
  const nameMap: Record<string, string> = {
    Bool: '布尔',
    Int32: '整数',
    Float32: '浮点',
    String: '字符串',
  }
  return nameMap[dataType] || dataType
}

/**
 * 获取访问模式标签样式
 */
function getAccessModeTagType(accessMode: string): string {
  const typeMap: Record<string, string> = {
    ReadOnly: 'info',
    WriteOnly: 'warning',
    ReadWrite: 'success',
  }
  return typeMap[accessMode] || 'default'
}

/**
 * 获取访问模式显示名称
 */
function getAccessModeDisplayName(accessMode: string): string {
  const nameMap: Record<string, string> = {
    ReadOnly: '只读',
    WriteOnly: '只写',
    ReadWrite: '读写',
  }
  return nameMap[accessMode] || accessMode
}

/**
 * 格式化点位值
 */
function formatTagValue(value: any, dataType: string): string {
  if (value === null || value === undefined) {
    return '--'
  }

  switch (dataType) {
    case 'Bool':
      return value ? '真' : '假'
    case 'Int32':
      return value.toString()
    case 'Float32':
      return parseFloat(value).toFixed(2)
    case 'String':
      return value.toString()
    default:
      return value.toString()
  }
}

/**
 * 检查是否可读
 */
function canRead(accessMode: string): boolean {
  return accessMode === 'ReadOnly' || accessMode === 'ReadWrite'
}

/**
 * 检查是否可写
 */
function canWrite(accessMode: string): boolean {
  return accessMode === 'WriteOnly' || accessMode === 'ReadWrite'
}
</script>

<style scoped lang="scss">
.tags-table {
  .tag-name {
    display: flex;
    align-items: center;
    gap: 8px;

    .tag-icon {
      color: #67c23a;
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

  .device-info {
    .el-tag {
      max-width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
    }
  }

  .address-config {
    .address-item {
      margin-bottom: 4px;
      font-size: 13px;
      line-height: 1.4;

      &:last-child {
        margin-bottom: 0;
      }

      .address-label {
        color: #909399;
        margin-right: 4px;
      }

      .address-value {
        color: #303133;
        font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
        background-color: #f5f7fa;
        padding: 1px 4px;
        border-radius: 2px;
        font-size: 12px;
      }
    }
  }

  .current-value {
    .value {
      font-weight: 500;
      color: #409eff;
      font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    }

    .no-value {
      color: #c0c4cc;
      font-style: italic;
    }
  }

  .action-buttons {
    display: flex;
    align-items: center;
    gap: 4px;

    .el-button {
      padding: 4px;

      &.is-disabled {
        opacity: 0.3;
      }

      .el-icon {
        font-size: 14px;
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

// 数据类型标签样式
:deep(.el-tag) {
  &.el-tag--success {
    background-color: #f0f9ff;
    border-color: #c6f6d5;
    color: #22543d;
  }

  &.el-tag--primary {
    background-color: #eff6ff;
    border-color: #bfdbfe;
    color: #1e3a8a;
  }

  &.el-tag--warning {
    background-color: #fffbeb;
    border-color: #fed7aa;
    color: #92400e;
  }

  &.el-tag--info {
    background-color: #f8fafc;
    border-color: #cbd5e1;
    color: #475569;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .tags-table {
    :deep(.el-table) {
      .el-table__body-wrapper {
        overflow-x: auto;
      }
    }

    .tag-name {
      .name-content {
        .description-text {
          display: none;
        }
      }
    }

    .address-config {
      .address-item {
        font-size: 12px;

        .address-value {
          font-size: 11px;
        }
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
