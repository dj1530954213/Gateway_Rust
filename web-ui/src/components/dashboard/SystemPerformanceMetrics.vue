<template>
  <div class="system-performance-metrics">
    <div v-if="loading" class="metrics-loading">
      <el-skeleton :rows="4" animated />
    </div>
    
    <div v-else class="metrics-grid">
      <!-- CPU使用率 -->
      <div class="metric-card cpu">
        <div class="metric-header">
          <div class="metric-icon">
            <el-icon><Cpu /></el-icon>
          </div>
          <div class="metric-info">
            <div class="metric-title">CPU使用率</div>
            <div class="metric-subtitle">{{ metrics.cpu.cores }}核心</div>
          </div>
          <div class="metric-value">
            <span class="value-number" :class="getCpuStatusClass(metrics.cpu.usage)">
              {{ metrics.cpu.usage }}%
            </span>
          </div>
        </div>
        
        <div class="metric-progress">
          <el-progress
            :percentage="metrics.cpu.usage"
            :status="getCpuProgressStatus(metrics.cpu.usage)"
            :stroke-width="8"
            :show-text="false"
          />
        </div>
        
        <div class="metric-details">
          <div class="detail-item">
            <span class="detail-label">用户态:</span>
            <span class="detail-value">{{ metrics.cpu.user }}%</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">系统态:</span>
            <span class="detail-value">{{ metrics.cpu.system }}%</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">负载:</span>
            <span class="detail-value">{{ metrics.cpu.loadAverage }}</span>
          </div>
        </div>
      </div>
      
      <!-- 内存使用率 -->
      <div class="metric-card memory">
        <div class="metric-header">
          <div class="metric-icon">
            <el-icon><Memory /></el-icon>
          </div>
          <div class="metric-info">
            <div class="metric-title">内存使用</div>
            <div class="metric-subtitle">{{ formatFileSize(metrics.memory.total) }}</div>
          </div>
          <div class="metric-value">
            <span class="value-number" :class="getMemoryStatusClass(metrics.memory.usage)">
              {{ metrics.memory.usage }}%
            </span>
          </div>
        </div>
        
        <div class="metric-progress">
          <el-progress
            :percentage="metrics.memory.usage"
            :status="getMemoryProgressStatus(metrics.memory.usage)"
            :stroke-width="8"
            :show-text="false"
          />
        </div>
        
        <div class="metric-details">
          <div class="detail-item">
            <span class="detail-label">已用:</span>
            <span class="detail-value">{{ formatFileSize(metrics.memory.used) }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">可用:</span>
            <span class="detail-value">{{ formatFileSize(metrics.memory.available) }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">缓存:</span>
            <span class="detail-value">{{ formatFileSize(metrics.memory.cached) }}</span>
          </div>
        </div>
      </div>
      
      <!-- 磁盘使用率 -->
      <div class="metric-card disk">
        <div class="metric-header">
          <div class="metric-icon">
            <el-icon><Folder /></el-icon>
          </div>
          <div class="metric-info">
            <div class="metric-title">磁盘使用</div>
            <div class="metric-subtitle">{{ formatFileSize(metrics.disk.total) }}</div>
          </div>
          <div class="metric-value">
            <span class="value-number" :class="getDiskStatusClass(metrics.disk.usage)">
              {{ metrics.disk.usage }}%
            </span>
          </div>
        </div>
        
        <div class="metric-progress">
          <el-progress
            :percentage="metrics.disk.usage"
            :status="getDiskProgressStatus(metrics.disk.usage)"
            :stroke-width="8"
            :show-text="false"
          />
        </div>
        
        <div class="metric-details">
          <div class="detail-item">
            <span class="detail-label">已用:</span>
            <span class="detail-value">{{ formatFileSize(metrics.disk.used) }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">可用:</span>
            <span class="detail-value">{{ formatFileSize(metrics.disk.available) }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">IOPS:</span>
            <span class="detail-value">{{ metrics.disk.iops }}</span>
          </div>
        </div>
      </div>
      
      <!-- 网络流量 -->
      <div class="metric-card network">
        <div class="metric-header">
          <div class="metric-icon">
            <el-icon><Connection /></el-icon>
          </div>
          <div class="metric-info">
            <div class="metric-title">网络流量</div>
            <div class="metric-subtitle">{{ metrics.network.interface || 'eth0' }}</div>
          </div>
          <div class="metric-value">
            <div class="network-values">
              <div class="network-in">
                <span class="network-label">↓</span>
                <span class="network-speed">{{ formatNetworkSpeed(metrics.network.in) }}</span>
              </div>
              <div class="network-out">
                <span class="network-label">↑</span>
                <span class="network-speed">{{ formatNetworkSpeed(metrics.network.out) }}</span>
              </div>
            </div>
          </div>
        </div>
        
        <div class="network-chart">
          <div class="chart-legend">
            <div class="legend-item">
              <div class="legend-color in"></div>
              <span>入站</span>
            </div>
            <div class="legend-item">
              <div class="legend-color out"></div>
              <span>出站</span>
            </div>
          </div>
          
          <div class="mini-chart">
            <!-- 简单的网络流量迷你图表 -->
            <svg width="100%" height="60" viewBox="0 0 200 60">
              <polyline
                :points="generateNetworkPoints(metrics.network.history?.in || [])"
                fill="none"
                stroke="#67c23a"
                stroke-width="2"
              />
              <polyline
                :points="generateNetworkPoints(metrics.network.history?.out || [])"
                fill="none"
                stroke="#409eff"
                stroke-width="2"
              />
            </svg>
          </div>
        </div>
        
        <div class="metric-details">
          <div class="detail-item">
            <span class="detail-label">连接数:</span>
            <span class="detail-value">{{ metrics.network.connections || 0 }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">延迟:</span>
            <span class="detail-value">{{ metrics.network.latency || 0 }}ms</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">丢包率:</span>
            <span class="detail-value">{{ metrics.network.packetLoss || 0 }}%</span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 系统进程信息 -->
    <div class="process-info">
      <div class="process-header">
        <span class="process-title">资源占用Top进程</span>
        <el-button type="text" size="small" @click="refreshProcesses">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
      
      <div class="process-list">
        <div class="process-header-row">
          <div class="process-col">进程名</div>
          <div class="process-col">PID</div>
          <div class="process-col">CPU%</div>
          <div class="process-col">内存</div>
          <div class="process-col">状态</div>
        </div>
        
        <div
          v-for="process in topProcesses"
          :key="process.pid"
          class="process-row"
        >
          <div class="process-col process-name">
            <span class="name-text">{{ process.name }}</span>
          </div>
          <div class="process-col process-pid">{{ process.pid }}</div>
          <div class="process-col process-cpu">
            <span :class="{ 'high-usage': process.cpu > 50 }">{{ process.cpu }}%</span>
          </div>
          <div class="process-col process-memory">{{ formatFileSize(process.memory) }}</div>
          <div class="process-col process-status">
            <el-tag :type="getProcessStatusType(process.status)" size="small">
              {{ process.status }}
            </el-tag>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * SystemPerformanceMetrics —— 系统性能指标组件
 *
 * 📝 Responsibilities:
 *  1. 显示系统性能指标
 *  2. CPU、内存、磁盘、网络监控
 *  3. 进程资源占用信息
 *  4. 性能趋势图表
 *  5. 实时数据更新
 *
 * 📦 Dependencies:
 *  - 系统监控API
 *  - 格式化工具
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Cpu, Folder, Connection, Refresh } from '@element-plus/icons-vue'

import { formatFileSize, formatNetworkSpeed } from '@/utils/format'

// ===== 接口定义 =====
interface SystemMetrics {
  cpu: {
    usage: number
    cores: number
    user: number
    system: number
    loadAverage: number
  }
  memory: {
    usage: number
    total: number
    used: number
    available: number
    cached: number
  }
  disk: {
    usage: number
    total: number
    used: number
    available: number
    iops: number
  }
  network: {
    interface?: string
    in: number
    out: number
    connections?: number
    latency?: number
    packetLoss?: number
    history?: {
      in: number[]
      out: number[]
    }
  }
}

interface ProcessInfo {
  pid: number
  name: string
  cpu: number
  memory: number
  status: string
}

// ===== Props =====  
const props = defineProps<{
  metrics: SystemMetrics
  loading?: boolean
}>()

// ===== 响应式数据 =====
const topProcesses = ref<ProcessInfo[]>([
  { pid: 1234, name: 'web-gw-api', cpu: 15.6, memory: 128 * 1024 * 1024, status: 'running' },
  { pid: 1235, name: 'driver-manager', cpu: 8.2, memory: 64 * 1024 * 1024, status: 'running' },
  { pid: 1236, name: 'alert-engine', cpu: 5.1, memory: 32 * 1024 * 1024, status: 'running' },
  { pid: 1237, name: 'postgres', cpu: 12.3, memory: 256 * 1024 * 1024, status: 'running' },
  { pid: 1238, name: 'influxdb', cpu: 18.7, memory: 512 * 1024 * 1024, status: 'running' }
])

// ===== 方法 =====

/**
 * 获取CPU状态样式类
 */
function getCpuStatusClass(usage: number): string {
  if (usage >= 90) return 'critical'
  if (usage >= 70) return 'warning'
  return 'normal'
}

/**
 * 获取CPU进度条状态
 */
function getCpuProgressStatus(usage: number): string {
  if (usage >= 90) return 'exception'
  if (usage >= 70) return 'warning'
  return 'success'
}

/**
 * 获取内存状态样式类
 */
function getMemoryStatusClass(usage: number): string {
  if (usage >= 95) return 'critical'
  if (usage >= 80) return 'warning'
  return 'normal'
}

/**
 * 获取内存进度条状态
 */
function getMemoryProgressStatus(usage: number): string {
  if (usage >= 95) return 'exception'
  if (usage >= 80) return 'warning'
  return 'success'
}

/**
 * 获取磁盘状态样式类
 */
function getDiskStatusClass(usage: number): string {
  if (usage >= 95) return 'critical'
  if (usage >= 85) return 'warning'
  return 'normal'
}

/**
 * 获取磁盘进度条状态
 */
function getDiskProgressStatus(usage: number): string {
  if (usage >= 95) return 'exception'
  if (usage >= 85) return 'warning'
  return 'success'
}

/**
 * 获取进程状态标签类型
 */
function getProcessStatusType(status: string): string {
  const typeMap: Record<string, string> = {
    'running': 'success',
    'sleeping': 'info',
    'stopped': 'warning',
    'zombie': 'danger'
  }
  return typeMap[status] || 'info'
}

/**
 * 生成网络流量点坐标
 */
function generateNetworkPoints(data: number[]): string {
  if (!data || data.length === 0) return ''
  
  const width = 200
  const height = 60
  const padding = 5
  
  const maxValue = Math.max(...data, 1)
  const points = data.map((value, index) => {
    const x = (index / (data.length - 1)) * (width - 2 * padding) + padding
    const y = height - padding - ((value / maxValue) * (height - 2 * padding))
    return `${x},${y}`
  })
  
  return points.join(' ')
}

/**
 * 刷新进程信息
 */
function refreshProcesses() {
  // 模拟刷新进程数据
  topProcesses.value.forEach(process => {
    process.cpu = Math.max(0, process.cpu + (Math.random() - 0.5) * 10)
  })
  
  // 按CPU使用率排序
  topProcesses.value.sort((a, b) => b.cpu - a.cpu)
  
  ElMessage.success('进程信息已刷新')
}
</script>

<style scoped lang="scss">
.system-performance-metrics {
  .metrics-loading {
    padding: 20px;
  }
  
  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 16px;
    margin-bottom: 20px;
    
    .metric-card {
      padding: 16px;
      border: 1px solid #ebeef5;
      border-radius: 8px;
      background: white;
      
      &.cpu { border-left: 4px solid #409eff; }
      &.memory { border-left: 4px solid #67c23a; }
      &.disk { border-left: 4px solid #e6a23c; }
      &.network { border-left: 4px solid #909399; }
      
      .metric-header {
        display: flex;
        align-items: flex-start;
        margin-bottom: 12px;
        
        .metric-icon {
          width: 40px;
          height: 40px;
          border-radius: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          margin-right: 12px;
          
          .el-icon {
            font-size: 20px;
          }
        }
        
        .metric-info {
          flex: 1;
          min-width: 0;
          
          .metric-title {
            font-size: 16px;
            font-weight: 600;
            color: #303133;
            margin-bottom: 2px;
          }
          
          .metric-subtitle {
            font-size: 12px;
            color: #909399;
          }
        }
        
        .metric-value {
          .value-number {
            font-size: 24px;
            font-weight: 700;
            
            &.normal { color: #67c23a; }
            &.warning { color: #e6a23c; }
            &.critical { color: #f56c6c; }
          }
          
          .network-values {
            display: flex;
            flex-direction: column;
            gap: 4px;
            
            .network-in,
            .network-out {
              display: flex;
              align-items: center;
              gap: 4px;
              
              .network-label {
                font-size: 14px;
                font-weight: 600;
              }
              
              .network-speed {
                font-size: 12px;
                font-weight: 500;
              }
            }
            
            .network-in {
              color: #67c23a;
            }
            
            .network-out {
              color: #409eff;
            }
          }
        }
      }
      
      .metric-progress {
        margin-bottom: 12px;
      }
      
      .metric-details {
        display: flex;
        justify-content: space-between;
        
        .detail-item {
          display: flex;
          flex-direction: column;
          align-items: center;
          
          .detail-label {
            font-size: 11px;
            color: #909399;
            margin-bottom: 2px;
          }
          
          .detail-value {
            font-size: 12px;
            font-weight: 500;
            color: #303133;
          }
        }
      }
      
      .network-chart {
        margin-bottom: 12px;
        
        .chart-legend {
          display: flex;
          justify-content: center;
          gap: 16px;
          margin-bottom: 8px;
          
          .legend-item {
            display: flex;
            align-items: center;
            gap: 4px;
            font-size: 12px;
            color: #606266;
            
            .legend-color {
              width: 12px;
              height: 2px;
              
              &.in { background: #67c23a; }
              &.out { background: #409eff; }
            }
          }
        }
        
        .mini-chart {
          height: 60px;
          background: #f9f9f9;
          border-radius: 4px;
        }
      }
    }
    
    // 图标背景色
    .cpu .metric-icon { background: #e1f3ff; color: #409eff; }
    .memory .metric-icon { background: #f0f9ec; color: #67c23a; }
    .disk .metric-icon { background: #fdf6ec; color: #e6a23c; }
    .network .metric-icon { background: #f4f4f5; color: #909399; }
  }
  
  .process-info {
    padding: 16px;
    border: 1px solid #ebeef5;
    border-radius: 8px;
    background: white;
    
    .process-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;
      
      .process-title {
        font-size: 16px;
        font-weight: 600;
        color: #303133;
      }
    }
    
    .process-list {
      .process-header-row {
        display: grid;
        grid-template-columns: 2fr 80px 80px 100px 80px;
        gap: 12px;
        padding: 8px 0;
        border-bottom: 2px solid #ebeef5;
        font-weight: 600;
        color: #909399;
        font-size: 13px;
      }
      
      .process-row {
        display: grid;
        grid-template-columns: 2fr 80px 80px 100px 80px;
        gap: 12px;
        padding: 12px 0;
        border-bottom: 1px solid #f0f0f0;
        font-size: 13px;
        
        &:hover {
          background: #f5f7fa;
        }
        
        .process-col {
          display: flex;
          align-items: center;
          
          &.process-name {
            .name-text {
              font-weight: 500;
              color: #303133;
              font-family: monospace;
            }
          }
          
          &.process-pid {
            color: #606266;
            font-family: monospace;
          }
          
          &.process-cpu {
            .high-usage {
              color: #f56c6c;
              font-weight: 600;
            }
          }
          
          &.process-memory {
            color: #606266;
          }
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .system-performance-metrics {
    .metrics-grid {
      grid-template-columns: 1fr;
      gap: 12px;
      
      .metric-card {
        padding: 12px;
        
        .metric-header {
          .metric-icon {
            width: 32px;
            height: 32px;
            
            .el-icon {
              font-size: 16px;
            }
          }
          
          .metric-value .value-number {
            font-size: 20px;
          }
        }
        
        .metric-details {
          .detail-item {
            .detail-label,
            .detail-value {
              font-size: 11px;
            }
          }
        }
      }
    }
    
    .process-info {
      padding: 12px;
      
      .process-list {
        .process-header-row,
        .process-row {
          grid-template-columns: 1fr;
          gap: 8px;
          
          .process-col {
            justify-content: space-between;
            
            &::before {
              content: attr(data-label);
              font-weight: 600;
              color: #909399;
            }
          }
        }
        
        .process-header-row {
          display: none;
        }
        
        .process-row {
          .process-col {
            &.process-name::before { content: '进程: '; }
            &.process-pid::before { content: 'PID: '; }
            &.process-cpu::before { content: 'CPU: '; }
            &.process-memory::before { content: '内存: '; }
            &.process-status::before { content: '状态: '; }
          }
        }
      }
    }
  }
}
</style>