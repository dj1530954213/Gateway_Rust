<template>
  <div class="device-status-overview">
    <!-- 设备统计概览 -->
    <div class="device-summary">
      <div class="summary-stats">
        <div class="stat-item total">
          <div class="stat-icon">
            <el-icon><Monitor /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ deviceStats.total }}</div>
            <div class="stat-label">总设备数</div>
          </div>
        </div>

        <div class="stat-item online">
          <div class="stat-icon">
            <el-icon><CircleCheck /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ deviceStats.online }}</div>
            <div class="stat-label">在线设备</div>
          </div>
        </div>

        <div class="stat-item offline">
          <div class="stat-icon">
            <el-icon><CircleClose /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ deviceStats.offline }}</div>
            <div class="stat-label">离线设备</div>
          </div>
        </div>

        <div class="stat-item error">
          <div class="stat-icon">
            <el-icon><Warning /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ deviceStats.error }}</div>
            <div class="stat-label">异常设备</div>
          </div>
        </div>
      </div>

      <!-- 连接状态进度条 -->
      <div class="connection-progress">
        <div class="progress-header">
          <span class="progress-label">设备连接健康度</span>
          <span class="progress-value">{{ connectionHealthPercentage }}%</span>
        </div>
        <el-progress
          :percentage="connectionHealthPercentage"
          :status="connectionHealthStatus"
          :stroke-width="6"
          :show-text="false"
        />
      </div>
    </div>

    <!-- 设备列表 -->
    <div class="device-list">
      <div v-if="loading" class="list-loading">
        <el-skeleton :rows="3" animated />
      </div>

      <div v-else-if="devices.length === 0" class="list-empty">
        <el-empty description="暂无设备数据" :image-size="80" />
      </div>

      <div v-else class="device-items">
        <div
          v-for="device in devices"
          :key="device.id"
          class="device-item"
          :class="`status-${device.status}`"
          @click="handleDeviceClick(device)"
        >
          <div class="device-info">
            <div class="device-header">
              <div class="device-name">{{ device.name }}</div>
              <el-tag
                :type="getStatusTagType(device.status)"
                size="small"
                effect="plain"
              >
                {{ getStatusText(device.status) }}
              </el-tag>
            </div>

            <div class="device-details">
              <div class="detail-item">
                <span class="detail-label">协议:</span>
                <span class="detail-value">{{
                  getProtocolDisplayName(device.protocol)
                }}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">地址:</span>
                <span class="detail-value">{{
                  formatDeviceAddress(device)
                }}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">最后通信:</span>
                <span
                  class="detail-value"
                  :class="{ outdated: isOutdated(device.lastCommunication) }"
                >
                  {{ formatLastCommunication(device.lastCommunication) }}
                </span>
              </div>
            </div>
          </div>

          <div class="device-metrics">
            <div class="metric-item">
              <div class="metric-label">数据点</div>
              <div class="metric-value">{{ device.tagCount || 0 }}</div>
            </div>
            <div class="metric-item">
              <div class="metric-label">错误率</div>
              <div
                class="metric-value"
                :class="{ 'high-error': device.errorRate > 5 }"
              >
                {{ device.errorRate || 0 }}%
              </div>
            </div>
            <div class="metric-item">
              <div class="metric-label">响应时间</div>
              <div class="metric-value">{{ device.responseTime || 0 }}ms</div>
            </div>
          </div>

          <div class="device-actions">
            <el-button-group size="small">
              <el-tooltip content="测试连接" placement="top">
                <el-button
                  type="primary"
                  :loading="device.testing"
                  @click.stop="testConnection(device)"
                >
                  <el-icon><Connection /></el-icon>
                </el-button>
              </el-tooltip>

              <el-tooltip content="重启设备" placement="top">
                <el-button
                  type="warning"
                  :loading="device.restarting"
                  @click.stop="restartDevice(device)"
                >
                  <el-icon><Refresh /></el-icon>
                </el-button>
              </el-tooltip>

              <el-tooltip content="查看详情" placement="top">
                <el-button type="info" @click.stop="viewDetails(device)">
                  <el-icon><View /></el-icon>
                </el-button>
              </el-tooltip>
            </el-button-group>
          </div>

          <!-- 实时状态指示器 -->
          <div class="status-indicator">
            <div
              class="indicator-dot"
              :class="`dot-${device.status}`"
              :title="getStatusText(device.status)"
            ></div>
          </div>
        </div>
      </div>
    </div>

    <!-- 协议分布图表 -->
    <div class="protocol-distribution">
      <div class="distribution-header">
        <span>协议分布</span>
        <el-button type="text" size="small" @click="refreshProtocolStats">
          <el-icon><Refresh /></el-icon>
        </el-button>
      </div>

      <div class="protocol-chart">
        <div
          v-for="(count, protocol) in protocolStats"
          :key="protocol"
          class="protocol-bar"
        >
          <div class="protocol-info">
            <span class="protocol-name">{{
              getProtocolDisplayName(protocol)
            }}</span>
            <span class="protocol-count">{{ count }}</span>
          </div>
          <div class="protocol-progress">
            <div
              class="progress-bar"
              :style="{ width: `${(count / deviceStats.total) * 100}%` }"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * DeviceStatusOverview —— 设备连接状态概览
 *
 * 📝 Responsibilities:
 *  1. 显示设备连接统计信息
 *  2. 展示设备列表和状态
 *  3. 提供设备快速操作
 *  4. 协议分布可视化
 *  5. 实时状态更新
 *
 * 📦 Dependencies:
 *  - 设备Store
 *  - 日期格式化工具
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Monitor,
  CircleCheck,
  CircleClose,
  Warning,
  Connection,
  Refresh,
  View,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, watch } from 'vue'

import type { DeviceVO } from '@/api/devices'
import { formatDateTime } from '@/utils/date'

// ===== Props & Emits =====
const props = defineProps<{
  devices: DeviceVO[]
  loading?: boolean
}>()

const emit = defineEmits<{
  'device-click': [device: DeviceVO]
}>()

// ===== 响应式数据 =====
const protocolStats = ref<Record<string, number>>({})

// ===== 计算属性 =====
const deviceStats = computed(() => {
  const total = props.devices.length
  const online = props.devices.filter(d => d.status === 'online').length
  const offline = props.devices.filter(d => d.status === 'offline').length
  const error = props.devices.filter(d => d.status === 'error').length

  return {
    total,
    online,
    offline,
    error,
  }
})

const connectionHealthPercentage = computed(() => {
  const { total, online } = deviceStats.value
  return total > 0 ? Math.round((online / total) * 100) : 0
})

const connectionHealthStatus = computed(() => {
  const percentage = connectionHealthPercentage.value
  if (percentage >= 90) return 'success'
  if (percentage >= 70) return 'warning'
  return 'exception'
})

// ===== 方法 =====

/**
 * 获取状态标签类型
 */
function getStatusTagType(status: string): string {
  const typeMap: Record<string, string> = {
    online: 'success',
    offline: 'info',
    error: 'danger',
    connecting: 'warning',
  }
  return typeMap[status] || 'info'
}

/**
 * 获取状态文本
 */
function getStatusText(status: string): string {
  const textMap: Record<string, string> = {
    online: '在线',
    offline: '离线',
    error: '异常',
    connecting: '连接中',
  }
  return textMap[status] || status
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
    Http: 'HTTP',
  }
  return nameMap[protocol] || protocol
}

/**
 * 格式化设备地址
 */
function formatDeviceAddress(device: DeviceVO): string {
  const config = device.connection_config
  if (config.host) {
    return `${config.host}:${config.port || 502}`
  }
  if (config.endpoint) {
    return config.endpoint
  }
  if (config.serial_port) {
    return config.serial_port
  }
  return '未配置'
}

/**
 * 格式化最后通信时间
 */
function formatLastCommunication(time?: string): string {
  if (!time) return '从未通信'

  const now = new Date()
  const lastTime = new Date(time)
  const diffMs = now.getTime() - lastTime.getTime()
  const diffMinutes = Math.floor(diffMs / 60000)

  if (diffMinutes < 1) return '刚刚'
  if (diffMinutes < 60) return `${diffMinutes}分钟前`
  if (diffMinutes < 1440) return `${Math.floor(diffMinutes / 60)}小时前`
  return formatDateTime(time).split(' ')[1] // 只显示时间部分
}

/**
 * 检查通信时间是否过时
 */
function isOutdated(time?: string): boolean {
  if (!time) return true

  const now = new Date()
  const lastTime = new Date(time)
  const diffMs = now.getTime() - lastTime.getTime()

  return diffMs > 5 * 60 * 1000 // 5分钟无通信视为过时
}

/**
 * 处理设备点击
 */
function handleDeviceClick(device: DeviceVO) {
  emit('device-click', device)
}

/**
 * 测试连接
 */
async function testConnection(device: DeviceVO) {
  device.testing = true

  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 2000))

    // 模拟测试结果
    const success = Math.random() > 0.3

    if (success) {
      device.status = 'online'
      device.lastCommunication = new Date().toISOString()
      ElMessage.success(`设备 ${device.name} 连接测试成功`)
    } else {
      device.status = 'error'
      ElMessage.error(`设备 ${device.name} 连接测试失败`)
    }
  } catch (error) {
    ElMessage.error('连接测试失败')
  } finally {
    device.testing = false
  }
}

/**
 * 重启设备
 */
async function restartDevice(device: DeviceVO) {
  try {
    await ElMessageBox.confirm(
      `确定要重启设备 "${device.name}" 吗？`,
      '确认重启',
      {
        type: 'warning',
        confirmButtonText: '重启',
        cancelButtonText: '取消',
      }
    )

    device.restarting = true
    device.status = 'connecting'

    // 模拟重启过程
    await new Promise(resolve => setTimeout(resolve, 3000))

    device.status = 'online'
    device.lastCommunication = new Date().toISOString()

    ElMessage.success(`设备 ${device.name} 重启成功`)
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('设备重启失败')
    }
  } finally {
    device.restarting = false
  }
}

/**
 * 查看设备详情
 */
function viewDetails(device: DeviceVO) {
  // 这里可以打开设备详情对话框或跳转到详情页
  ElMessage.info(`查看设备 ${device.name} 详情功能开发中...`)
}

/**
 * 更新协议统计
 */
function updateProtocolStats() {
  const stats: Record<string, number> = {}

  props.devices.forEach(device => {
    const protocol = device.protocol
    stats[protocol] = (stats[protocol] || 0) + 1
  })

  protocolStats.value = stats
}

/**
 * 刷新协议统计
 */
function refreshProtocolStats() {
  updateProtocolStats()
  ElMessage.success('协议统计已刷新')
}

// ===== 监听器 =====
watch(
  () => props.devices,
  () => {
    updateProtocolStats()
  },
  { immediate: true, deep: true }
)
</script>

<style scoped lang="scss">
.device-status-overview {
  .device-summary {
    margin-bottom: 20px;

    .summary-stats {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
      gap: 12px;
      margin-bottom: 16px;

      .stat-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 12px;
        background: #f9f9f9;
        border-radius: 6px;

        .stat-icon {
          font-size: 20px;

          .el-icon {
            &.total {
              color: #409eff;
            }
            &.online {
              color: #67c23a;
            }
            &.offline {
              color: #909399;
            }
            &.error {
              color: #f56c6c;
            }
          }
        }

        .stat-content {
          .stat-value {
            font-size: 18px;
            font-weight: 600;
            color: #303133;
            margin-bottom: 2px;
          }

          .stat-label {
            font-size: 12px;
            color: #909399;
          }
        }
      }
    }

    .connection-progress {
      .progress-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;

        .progress-label {
          font-size: 14px;
          color: #606266;
        }

        .progress-value {
          font-size: 14px;
          font-weight: 600;
          color: #303133;
        }
      }
    }
  }

  .device-list {
    margin-bottom: 20px;

    .list-loading {
      padding: 16px;
    }

    .list-empty {
      text-align: center;
      padding: 40px 16px;
    }

    .device-items {
      .device-item {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 16px;
        border: 1px solid #ebeef5;
        border-radius: 6px;
        margin-bottom: 8px;
        cursor: pointer;
        transition: all 0.3s;
        position: relative;

        &:hover {
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          transform: translateY(-1px);
        }

        &.status-online {
          border-left: 4px solid #67c23a;
        }

        &.status-offline {
          border-left: 4px solid #909399;
        }

        &.status-error {
          border-left: 4px solid #f56c6c;
        }

        .device-info {
          flex: 1;
          min-width: 0;

          .device-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 8px;

            .device-name {
              font-size: 16px;
              font-weight: 600;
              color: #303133;
            }
          }

          .device-details {
            display: flex;
            flex-wrap: wrap;
            gap: 16px;

            .detail-item {
              display: flex;
              align-items: center;
              font-size: 13px;

              .detail-label {
                color: #909399;
                margin-right: 4px;
              }

              .detail-value {
                color: #606266;
                font-family: monospace;

                &.outdated {
                  color: #f56c6c;
                }
              }
            }
          }
        }

        .device-metrics {
          display: flex;
          gap: 16px;

          .metric-item {
            text-align: center;

            .metric-label {
              font-size: 11px;
              color: #909399;
              margin-bottom: 4px;
            }

            .metric-value {
              font-size: 14px;
              font-weight: 600;
              color: #303133;

              &.high-error {
                color: #f56c6c;
              }
            }
          }
        }

        .device-actions {
          .el-button-group {
            .el-button {
              padding: 6px 8px;
            }
          }
        }

        .status-indicator {
          position: absolute;
          top: 8px;
          right: 8px;

          .indicator-dot {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            animation: pulse 2s infinite;

            &.dot-online {
              background: #67c23a;
              box-shadow: 0 0 0 0 rgba(103, 194, 58, 0.7);
            }

            &.dot-offline {
              background: #909399;
              animation: none;
            }

            &.dot-error {
              background: #f56c6c;
              box-shadow: 0 0 0 0 rgba(245, 108, 108, 0.7);
            }

            &.dot-connecting {
              background: #e6a23c;
              box-shadow: 0 0 0 0 rgba(230, 162, 60, 0.7);
            }
          }
        }
      }
    }
  }

  .protocol-distribution {
    .distribution-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 12px;

      span {
        font-size: 14px;
        font-weight: 600;
        color: #303133;
      }
    }

    .protocol-chart {
      .protocol-bar {
        margin-bottom: 12px;

        &:last-child {
          margin-bottom: 0;
        }

        .protocol-info {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 4px;

          .protocol-name {
            font-size: 13px;
            color: #606266;
          }

          .protocol-count {
            font-size: 13px;
            font-weight: 600;
            color: #303133;
          }
        }

        .protocol-progress {
          height: 6px;
          background: #f0f0f0;
          border-radius: 3px;
          overflow: hidden;

          .progress-bar {
            height: 100%;
            background: linear-gradient(90deg, #409eff, #67c23a);
            border-radius: 3px;
            transition: width 0.3s;
          }
        }
      }
    }
  }
}

@keyframes pulse {
  0% {
    transform: scale(0.95);
    box-shadow: 0 0 0 0 rgba(103, 194, 58, 0.7);
  }

  70% {
    transform: scale(1);
    box-shadow: 0 0 0 4px rgba(103, 194, 58, 0);
  }

  100% {
    transform: scale(0.95);
    box-shadow: 0 0 0 0 rgba(103, 194, 58, 0);
  }
}

// 响应式设计
@media (max-width: 768px) {
  .device-status-overview {
    .device-summary {
      .summary-stats {
        grid-template-columns: repeat(2, 1fr);
        gap: 8px;

        .stat-item {
          padding: 8px;

          .stat-icon {
            font-size: 16px;
          }

          .stat-content {
            .stat-value {
              font-size: 16px;
            }
          }
        }
      }
    }

    .device-list {
      .device-items {
        .device-item {
          flex-direction: column;
          align-items: stretch;
          gap: 12px;

          .device-info {
            .device-details {
              flex-direction: column;
              gap: 8px;
            }
          }

          .device-metrics {
            justify-content: space-around;
          }

          .device-actions {
            align-self: center;
          }
        }
      }
    }
  }
}
</style>
