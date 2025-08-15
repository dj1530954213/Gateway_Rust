<template>
  <div class="devices-table">
    <el-table
      v-loading="loading"
      :data="devices"
      empty-text="暂无设备数据"
      stripe
      style="width: 100%"
      @selection-change="handleSelectionChange"
    >
      <!-- 选择列 -->
      <el-table-column type="selection" width="50" />

      <!-- 设备名称 -->
      <el-table-column prop="name" label="设备名称" min-width="180">
        <template #default="{ row }">
          <div class="device-name">
            <el-icon class="device-icon">
              <Monitor />
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

      <!-- 协议类型 -->
      <el-table-column prop="protocol" label="协议类型" width="120">
        <template #default="{ row }">
          <el-tag :type="getProtocolTagType(row.protocol)" size="small">
            {{ getProtocolDisplayName(row.protocol) }}
          </el-tag>
        </template>
      </el-table-column>

      <!-- 连接配置 -->
      <el-table-column label="连接配置" min-width="200">
        <template #default="{ row }">
          <div class="connection-config">
            <div v-if="row.connection_config.host" class="config-item">
              <span class="config-label">主机:</span>
              <span class="config-value"
                >{{ row.connection_config.host }}:{{
                  row.connection_config.port || 502
                }}</span
              >
            </div>
            <div v-if="row.connection_config.slave_id" class="config-item">
              <span class="config-label">从站ID:</span>
              <span class="config-value">{{
                row.connection_config.slave_id
              }}</span>
            </div>
            <div v-if="row.connection_config.endpoint" class="config-item">
              <span class="config-label">端点:</span>
              <span class="config-value">{{
                row.connection_config.endpoint
              }}</span>
            </div>
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

      <!-- 创建时间 -->
      <el-table-column prop="created_at" label="创建时间" width="160">
        <template #default="{ row }">
          {{ formatDateTime(row.created_at) }}
        </template>
      </el-table-column>

      <!-- 更新时间 -->
      <el-table-column prop="updated_at" label="更新时间" width="160">
        <template #default="{ row }">
          {{ formatDateTime(row.updated_at) }}
        </template>
      </el-table-column>

      <!-- 操作列 -->
      <el-table-column label="操作" width="200" fixed="right">
        <template #default="{ row }">
          <div class="action-buttons">
            <!-- 测试连接 -->
            <el-tooltip content="测试连接" placement="top">
              <el-button
                type="primary"
                link
                size="small"
                @click="$emit('test-connection', row)"
              >
                <el-icon><Connection /></el-icon>
              </el-button>
            </el-tooltip>

            <!-- 启用/禁用 -->
            <el-tooltip
              :content="row.enabled ? '禁用设备' : '启用设备'"
              placement="top"
            >
              <el-button
                :type="row.enabled ? 'warning' : 'success'"
                link
                size="small"
                @click="$emit('toggle-device', row)"
              >
                <el-icon v-if="row.enabled"><VideoPlay /></el-icon>
                <el-icon v-else><VideoPause /></el-icon>
              </el-button>
            </el-tooltip>

            <!-- 编辑 -->
            <el-tooltip content="编辑设备" placement="top">
              <el-button
                type="primary"
                link
                size="small"
                @click="$emit('edit-device', row)"
              >
                <el-icon><Edit /></el-icon>
              </el-button>
            </el-tooltip>

            <!-- 删除 -->
            <el-tooltip content="删除设备" placement="top">
              <el-button
                type="danger"
                link
                size="small"
                @click="$emit('delete-device', row)"
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
 * DevicesTable —— 设备列表表格组件
 *
 * 📝 Responsibilities:
 *  1. 显示设备列表数据
 *  2. 支持选择操作
 *  3. 提供设备操作按钮
 *  4. 格式化显示设备信息
 *
 * 📦 Dependencies:
 *  - Element Plus Table 组件
 *  - 日期格式化工具
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Monitor,
  Connection,
  VideoPlay,
  VideoPause,
  Edit,
  Delete,
} from '@element-plus/icons-vue'

import type { DeviceVO } from '@/api/devices'
import { formatDateTime } from '@/utils/date'

// ===== Props =====
defineProps<{
  devices: DeviceVO[]
  loading?: boolean
  selectedDevices?: DeviceVO[]
}>()

// ===== Emits =====
const emit = defineEmits<{
  'selection-change': [devices: DeviceVO[]]
  'edit-device': [device: DeviceVO]
  'delete-device': [device: DeviceVO]
  'toggle-device': [device: DeviceVO]
  'test-connection': [device: DeviceVO]
}>()

// ===== 方法 =====

/**
 * 处理选择变更
 */
function handleSelectionChange(selection: DeviceVO[]) {
  // 直接发射事件到父组件
  emit('selection-change', selection)
}

/**
 * 获取协议类型标签样式
 */
function getProtocolTagType(protocol: string): string {
  const typeMap: Record<string, string> = {
    ModbusTcp: 'primary',
    ModbusRtu: 'success',
    OpcUa: 'warning',
    Mqtt: 'info',
  }
  return typeMap[protocol] || 'default'
}

/**
 * 获取协议显示名称
 */
function getProtocolDisplayName(protocol: string): string {
  const nameMap: Record<string, string> = {
    ModbusTcp: 'Modbus TCP',
    ModbusRtu: 'Modbus RTU',
    OpcUa: 'OPC UA',
    Mqtt: 'MQTT',
  }
  return nameMap[protocol] || protocol
}
</script>

<style scoped lang="scss">
.devices-table {
  .device-name {
    display: flex;
    align-items: center;
    gap: 8px;

    .device-icon {
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

  .connection-config {
    .config-item {
      margin-bottom: 4px;
      font-size: 13px;
      line-height: 1.4;

      &:last-child {
        margin-bottom: 0;
      }

      .config-label {
        color: #909399;
        margin-right: 4px;
      }

      .config-value {
        color: #303133;
        font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
      }
    }
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
  .devices-table {
    :deep(.el-table) {
      .el-table__body-wrapper {
        overflow-x: auto;
      }
    }

    .device-name {
      .name-content {
        .description-text {
          display: none;
        }
      }
    }

    .connection-config {
      .config-item {
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
