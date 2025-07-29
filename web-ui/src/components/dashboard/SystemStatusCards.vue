<template>
  <div class="system-status-cards">
    <div class="status-grid">
      <div
        v-for="component in systemComponents"
        :key="component.name"
        class="status-card"
        :class="`status-${component.status}`"
      >
        <div class="card-header">
          <div class="component-info">
            <div class="component-name">{{ component.name }}</div>
            <div class="component-description">{{ component.description }}</div>
          </div>
          <el-tag
            :type="getStatusTagType(component.status)"
            :effect="component.status === 'running' ? 'dark' : 'plain'"
            size="small"
          >
            <el-icon class="status-icon">
              <CircleCheck v-if="component.status === 'running'" />
              <Warning v-else-if="component.status === 'warning'" />
              <CircleClose v-else-if="component.status === 'error'" />
              <Loading v-else-if="component.status === 'starting'" />
              <Remove v-else />
            </el-icon>
            {{ getStatusText(component.status) }}
          </el-tag>
        </div>
        
        <div class="card-metrics">
          <div class="metric-row">
            <div class="metric-item">
              <span class="metric-label">运行时间</span>
              <span class="metric-value">{{ formatUptime(component.uptime) }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">进程PID</span>
              <span class="metric-value">{{ component.pid || 'N/A' }}</span>
            </div>
          </div>
          
          <div class="metric-row">
            <div class="metric-item">
              <span class="metric-label">内存使用</span>
              <span class="metric-value">{{ formatMemory(component.memoryUsage) }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">CPU使用率</span>
              <span class="metric-value">{{ component.cpuUsage }}%</span>
            </div>
          </div>
          
          <div class="metric-row">
            <div class="metric-item">
              <span class="metric-label">错误计数</span>
              <span class="metric-value error-count" :class="{ 'has-errors': component.errorCount > 0 }">
                {{ component.errorCount }}
              </span>
            </div>
            <div class="metric-item">
              <span class="metric-label">最后检查</span>
              <span class="metric-value">{{ formatDateTime(component.lastCheck) }}</span>
            </div>
          </div>
        </div>
        
        <div class="card-actions">
          <el-button-group size="small">
            <el-button
              v-if="component.status === 'stopped'"
              type="success"
              @click="startComponent(component.name)"
              :loading="component.loading"
            >
              启动
            </el-button>
            <el-button
              v-else-if="component.status === 'running'"
              type="warning"
              @click="restartComponent(component.name)"
              :loading="component.loading"
            >
              重启
            </el-button>
            <el-button
              v-if="component.status !== 'stopped'"
              type="danger"
              @click="stopComponent(component.name)"
              :loading="component.loading"
            >
              停止
            </el-button>
            <el-button
              type="info"
              @click="viewLogs(component.name)"
            >
              日志
            </el-button>
          </el-button-group>
        </div>
        
        <!-- 错误详情面板 -->
        <div v-if="component.lastError && component.status === 'error'" class="error-panel">
          <div class="error-header">
            <el-icon><Warning /></el-icon>
            <span>最近错误</span>
          </div>
          <div class="error-content">
            <div class="error-time">{{ formatDateTime(component.lastError.time) }}</div>
            <div class="error-message">{{ component.lastError.message }}</div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 系统总览 -->
    <div class="system-overview">
      <el-card class="overview-card" shadow="never">
        <template #header>
          <div class="overview-header">
            <span>系统概览</span>
            <el-button type="text" size="small" @click="refreshAll" :loading="loading">
              <el-icon><Refresh /></el-icon>
              刷新全部
            </el-button>
          </div>
        </template>
        
        <div class="overview-stats">
          <div class="stat-item running">
            <div class="stat-value">{{ runningCount }}</div>
            <div class="stat-label">运行中</div>
          </div>
          <div class="stat-item warning">
            <div class="stat-value">{{ warningCount }}</div>
            <div class="stat-label">警告</div>
          </div>
          <div class="stat-item error">
            <div class="stat-value">{{ errorCount }}</div>
            <div class="stat-label">错误</div>
          </div>
          <div class="stat-item stopped">
            <div class="stat-value">{{ stoppedCount }}</div>
            <div class="stat-label">已停止</div>
          </div>
        </div>
        
        <div class="overview-progress">
          <div class="progress-label">系统健康度</div>
          <el-progress
            :percentage="systemHealthPercentage"
            :status="systemHealthStatus"
            :stroke-width="8"
            :show-text="true"
          />
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * SystemStatusCards —— 系统状态监控卡片
 *
 * 📝 Responsibilities:
 *  1. 显示各系统组件运行状态
 *  2. 提供组件启停控制
 *  3. 展示关键运行指标
 *  4. 错误信息提示
 *  5. 系统健康度概览
 *
 * 📦 Dependencies:
 *  - 系统监控API
 *  - 日期格式化工具
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { 
  CircleCheck, 
  Warning, 
  CircleClose, 
  Loading, 
  Remove, 
  Refresh 
} from '@element-plus/icons-vue'

import { formatDateTime, formatUptime } from '@/utils/date'
import { formatFileSize } from '@/utils/format'

// ===== Props & Emits =====
defineProps<{
  loading?: boolean
}>()

const emit = defineEmits<{
  'refresh': []
}>()

// ===== 响应式数据 =====
const systemComponents = ref([
  {
    name: 'data-collector',
    description: '数据采集服务',
    status: 'running',
    uptime: 86400000, // 毫秒
    pid: 1234,
    memoryUsage: 128 * 1024 * 1024, // 字节
    cpuUsage: 15.6,
    errorCount: 0,
    lastCheck: new Date().toISOString(),
    loading: false,
    lastError: null
  },
  {
    name: 'driver-manager',
    description: '驱动管理器',
    status: 'running',
    uptime: 86200000,
    pid: 1235,
    memoryUsage: 64 * 1024 * 1024,
    cpuUsage: 8.2,
    errorCount: 1,
    lastCheck: new Date().toISOString(),
    loading: false,
    lastError: null
  },
  {
    name: 'alert-engine',
    description: '告警引擎',
    status: 'warning',
    uptime: 85000000,
    pid: 1236,
    memoryUsage: 32 * 1024 * 1024,
    cpuUsage: 5.1,
    errorCount: 3,
    lastCheck: new Date().toISOString(),
    loading: false,
    lastError: {
      time: new Date(Date.now() - 60000).toISOString(),
      message: 'SMTP服务器连接超时'
    }
  },
  {
    name: 'web-gateway',
    description: 'Web网关服务',
    status: 'running',
    uptime: 86400000,
    pid: 1237,
    memoryUsage: 96 * 1024 * 1024,
    cpuUsage: 12.3,
    errorCount: 0,
    lastCheck: new Date().toISOString(),
    loading: false,
    lastError: null
  },
  {
    name: 'storage-service',
    description: '存储服务',
    status: 'error',
    uptime: 0,
    pid: null,
    memoryUsage: 0,
    cpuUsage: 0,
    errorCount: 15,
    lastCheck: new Date().toISOString(),
    loading: false,
    lastError: {
      time: new Date(Date.now() - 300000).toISOString(),
      message: '数据库连接失败: connection refused'
    }
  },
  {
    name: 'message-broker',
    description: '消息代理',
    status: 'stopped',
    uptime: 0,
    pid: null,
    memoryUsage: 0,
    cpuUsage: 0,
    errorCount: 0,
    lastCheck: new Date().toISOString(),
    loading: false,
    lastError: null
  }
])

// ===== 计算属性 =====
const runningCount = computed(() => 
  systemComponents.value.filter(c => c.status === 'running').length
)

const warningCount = computed(() => 
  systemComponents.value.filter(c => c.status === 'warning').length
)

const errorCount = computed(() => 
  systemComponents.value.filter(c => c.status === 'error').length
)

const stoppedCount = computed(() => 
  systemComponents.value.filter(c => c.status === 'stopped').length
)

const systemHealthPercentage = computed(() => {
  const total = systemComponents.value.length
  const healthy = runningCount.value + warningCount.value * 0.5
  return Math.round((healthy / total) * 100)
})

const systemHealthStatus = computed(() => {
  const percentage = systemHealthPercentage.value
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
    'running': 'success',
    'warning': 'warning', 
    'error': 'danger',
    'starting': 'info',
    'stopped': 'info'
  }
  return typeMap[status] || 'info'
}

/**
 * 获取状态文本
 */
function getStatusText(status: string): string {
  const textMap: Record<string, string> = {
    'running': '运行中',
    'warning': '警告',
    'error': '错误',
    'starting': '启动中',
    'stopped': '已停止'
  }
  return textMap[status] || status
}

/**
 * 格式化内存大小
 */
function formatMemory(bytes: number): string {
  return formatFileSize(bytes)
}

/**
 * 启动组件
 */
async function startComponent(componentName: string) {
  const component = systemComponents.value.find(c => c.name === componentName)
  if (!component) return
  
  try {
    await ElMessageBox.confirm(
      `确定要启动 "${component.description}" 服务吗？`,
      '确认启动',
      {
        type: 'info',
        confirmButtonText: '启动',
        cancelButtonText: '取消'
      }
    )
    
    component.loading = true
    
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    component.status = 'running'
    component.uptime = 0
    component.pid = Math.floor(Math.random() * 9000) + 1000
    component.lastCheck = new Date().toISOString()
    
    ElMessage.success(`${component.description} 启动成功`)
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('启动服务失败')
    }
  } finally {
    component.loading = false
  }
}

/**
 * 停止组件
 */
async function stopComponent(componentName: string) {
  const component = systemComponents.value.find(c => c.name === componentName)
  if (!component) return
  
  try {
    await ElMessageBox.confirm(
      `确定要停止 "${component.description}" 服务吗？`,
      '确认停止',
      {
        type: 'warning',
        confirmButtonText: '停止',
        cancelButtonText: '取消',
        confirmButtonClass: 'el-button--danger'
      }
    )
    
    component.loading = true
    
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1500))
    
    component.status = 'stopped'
    component.uptime = 0
    component.pid = null
    component.cpuUsage = 0
    component.memoryUsage = 0
    component.lastCheck = new Date().toISOString()
    
    ElMessage.success(`${component.description} 已停止`)
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('停止服务失败')
    }
  } finally {
    component.loading = false
  }
}

/**
 * 重启组件
 */
async function restartComponent(componentName: string) {
  const component = systemComponents.value.find(c => c.name === componentName)
  if (!component) return
  
  try {
    await ElMessageBox.confirm(
      `确定要重启 "${component.description}" 服务吗？`,
      '确认重启',
      {
        type: 'warning',
        confirmButtonText: '重启',
        cancelButtonText: '取消'
      }
    )
    
    component.loading = true
    component.status = 'starting'
    
    // 模拟重启过程
    await new Promise(resolve => setTimeout(resolve, 3000))
    
    component.status = 'running'
    component.uptime = 0
    component.errorCount = 0
    component.lastError = null
    component.lastCheck = new Date().toISOString()
    
    ElMessage.success(`${component.description} 重启成功`)
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('重启服务失败')
    }
  } finally {
    component.loading = false
  }
}

/**
 * 查看日志
 */
function viewLogs(componentName: string) {
  ElMessage.info(`查看 ${componentName} 日志功能开发中...`)
}

/**
 * 刷新所有组件状态
 */
async function refreshAll() {
  emit('refresh')
  
  // 模拟刷新过程
  for (const component of systemComponents.value) {
    component.lastCheck = new Date().toISOString()
    if (component.status === 'running') {
      component.uptime += 10000 // 增加10秒
      component.cpuUsage = Math.max(0, component.cpuUsage + (Math.random() - 0.5) * 5)
    }
  }
}

// ===== 生命周期 =====
onMounted(() => {
  // 启动定时更新
  const updateInterval = setInterval(() => {
    systemComponents.value.forEach(component => {
      if (component.status === 'running') {
        component.uptime += 5000 // 增加5秒
        component.cpuUsage = Math.max(0, Math.min(100, 
          component.cpuUsage + (Math.random() - 0.5) * 2
        ))
        component.lastCheck = new Date().toISOString()
      }
    })
  }, 5000)
  
  // 清理定时器
  onUnmounted(() => {
    clearInterval(updateInterval)
  })
})
</script>

<style scoped lang="scss">
.system-status-cards {
  .status-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 16px;
    margin-bottom: 20px;
    
    .status-card {
      padding: 16px;
      border: 1px solid #ebeef5;
      border-radius: 8px;
      background: #fff;
      transition: all 0.3s;
      
      &:hover {
        box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
      }
      
      &.status-running {
        border-left: 4px solid #67c23a;
      }
      
      &.status-warning {
        border-left: 4px solid #e6a23c;
      }
      
      &.status-error {
        border-left: 4px solid #f56c6c;
      }
      
      &.status-stopped {
        border-left: 4px solid #909399;
        background: #f9f9f9;
      }
      
      .card-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 16px;
        
        .component-info {
          flex: 1;
          
          .component-name {
            font-size: 16px;
            font-weight: 600;
            color: #303133;
            margin-bottom: 4px;
          }
          
          .component-description {
            font-size: 13px;
            color: #909399;
          }
        }
        
        .status-icon {
          margin-right: 4px;
          font-size: 12px;
        }
      }
      
      .card-metrics {
        margin-bottom: 16px;
        
        .metric-row {
          display: flex;
          justify-content: space-between;
          margin-bottom: 8px;
          
          &:last-child {
            margin-bottom: 0;
          }
          
          .metric-item {
            flex: 1;
            display: flex;
            flex-direction: column;
            
            .metric-label {
              font-size: 12px;
              color: #909399;
              margin-bottom: 4px;
            }
            
            .metric-value {
              font-size: 13px;
              font-weight: 500;
              color: #303133;
              
              &.error-count.has-errors {
                color: #f56c6c;
                font-weight: 600;
              }
            }
          }
        }
      }
      
      .card-actions {
        display: flex;
        justify-content: center;
        
        .el-button-group {
          width: 100%;
          
          .el-button {
            flex: 1;
          }
        }
      }
      
      .error-panel {
        margin-top: 12px;
        padding: 12px;
        background: #fef0f0;
        border: 1px solid #fde2e2;
        border-radius: 4px;
        
        .error-header {
          display: flex;
          align-items: center;
          margin-bottom: 8px;
          color: #f56c6c;
          font-weight: 500;
          font-size: 13px;
          
          .el-icon {
            margin-right: 6px;
          }
        }
        
        .error-content {
          .error-time {
            font-size: 11px;
            color: #909399;
            margin-bottom: 4px;
          }
          
          .error-message {
            font-size: 12px;
            color: #606266;
            font-family: monospace;
            line-height: 1.4;
          }
        }
      }
    }
  }
  
  .system-overview {
    .overview-card {
      .overview-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-weight: 600;
        color: #303133;
      }
      
      .overview-stats {
        display: flex;
        justify-content: space-around;
        margin-bottom: 20px;
        
        .stat-item {
          text-align: center;
          
          .stat-value {
            font-size: 24px;
            font-weight: 600;
            margin-bottom: 4px;
          }
          
          .stat-label {
            font-size: 12px;
            color: #909399;
          }
          
          &.running .stat-value { color: #67c23a; }
          &.warning .stat-value { color: #e6a23c; }
          &.error .stat-value { color: #f56c6c; }
          &.stopped .stat-value { color: #909399; }
        }
      }
      
      .overview-progress {
        .progress-label {
          font-size: 14px;
          color: #606266;
          margin-bottom: 8px;
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .system-status-cards {
    .status-grid {
      grid-template-columns: 1fr;
      gap: 12px;
      
      .status-card {
        padding: 12px;
        
        .card-header {
          flex-direction: column;
          align-items: flex-start;
          gap: 8px;
        }
        
        .card-metrics {
          .metric-row {
            flex-direction: column;
            gap: 8px;
            
            .metric-item {
              flex-direction: row;
              justify-content: space-between;
              align-items: baseline;
              
              .metric-label {
                margin-bottom: 0;
              }
            }
          }
        }
      }
    }
    
    .system-overview {
      .overview-stats {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 16px;
      }
    }
  }
}
</style>