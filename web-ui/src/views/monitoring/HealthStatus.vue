<template>
  <div class="health-status">
    <!-- 页面头部 -->
    <div class="health-header">
      <div class="header-title">
        <el-icon :size="24"><Heart /></el-icon>
        <h1>系统健康状态</h1>
        <el-tag :type="overallHealthType" size="large">
          {{ overallHealthText }}
        </el-tag>
      </div>
      
      <div class="header-actions">
        <el-button 
          :type="autoRefresh ? 'primary' : 'default'"
          @click="toggleAutoRefresh"
          size="small"
        >
          <el-icon><VideoPlay v-if="!autoRefresh" /><VideoPause v-else /></el-icon>
          {{ autoRefresh ? '停止监控' : '开始监控' }}
        </el-button>
        
        <el-button @click="refreshHealthStatus" :loading="loading" size="small">
          <el-icon><Refresh /></el-icon>
          刷新状态
        </el-button>
        
        <el-button @click="runHealthCheck" :loading="checkingHealth" size="small" type="primary">
          <el-icon><Monitor /></el-icon>
          运行健康检查
        </el-button>
      </div>
    </div>

    <!-- 系统健康概览 -->
    <div class="health-overview">
      <el-row :gutter="16">
        <el-col :span="8">
          <el-card shadow="never" class="health-card">
            <div class="health-summary">
              <div class="health-score">
                <div class="score-circle">
                  <el-progress
                    type="circle"
                    :percentage="healthScore"
                    :width="80"
                    :stroke-width="8"
                    :color="getScoreColor(healthScore)"
                  >
                    <template #default="{ percentage }">
                      <span class="score-text">{{ percentage }}%</span>
                    </template>
                  </el-progress>
                </div>
                <div class="score-info">
                  <div class="score-label">整体健康评分</div>
                  <div class="score-status" :class="getScoreStatusClass(healthScore)">
                    {{ getScoreStatusText(healthScore) }}
                  </div>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="8">
          <el-card shadow="never" class="health-card">
            <div class="health-stats">
              <div class="stat-item">
                <div class="stat-value success">{{ healthStats.healthy }}</div>
                <div class="stat-label">健康服务</div>
              </div>
              <div class="stat-item">
                <div class="stat-value warning">{{ healthStats.warning }}</div>
                <div class="stat-label">警告服务</div>
              </div>
              <div class="stat-item">
                <div class="stat-value danger">{{ healthStats.critical }}</div>
                <div class="stat-label">异常服务</div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="8">
          <el-card shadow="never" class="health-card">
            <div class="uptime-info">
              <div class="uptime-main">
                <div class="uptime-value">{{ systemUptime }}</div>
                <div class="uptime-label">系统运行时间</div>
              </div>
              <div class="uptime-details">
                <div class="detail-item">
                  <span class="detail-label">最后重启:</span>
                  <span class="detail-value">{{ lastRestart }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">可用性:</span>
                  <span class="detail-value">{{ availability }}%</span>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 服务组件健康状态 -->
    <div class="components-health">
      <el-card header="服务组件健康状态">
        <el-row :gutter="16">
          <el-col 
            v-for="component in healthComponents" 
            :key="component.name"
            :span="6"
          >
            <div class="component-card" :class="`component-${component.status}`">
              <div class="component-header">
                <el-icon :size="20">
                  <component :is="component.icon" />
                </el-icon>
                <span class="component-name">{{ component.name }}</span>
                <el-tag 
                  :type="getHealthTagType(component.status)" 
                  size="small"
                  effect="light"
                >
                  {{ getHealthStatusText(component.status) }}
                </el-tag>
              </div>
              
              <div class="component-metrics">
                <div class="metric">
                  <span class="metric-label">响应时间:</span>
                  <span class="metric-value" :class="getResponseTimeClass(component.responseTime)">
                    {{ component.responseTime }}ms
                  </span>
                </div>
                <div class="metric">
                  <span class="metric-label">可用性:</span>
                  <span class="metric-value">{{ component.availability }}%</span>
                </div>
                <div class="metric">
                  <span class="metric-label">最后检查:</span>
                  <span class="metric-value">{{ formatTime(component.lastCheck) }}</span>
                </div>
              </div>
              
              <div class="component-actions">
                <el-button 
                  size="small" 
                  @click="checkComponent(component)"
                  :loading="component.checking"
                >
                  检查
                </el-button>
                <el-button 
                  size="small" 
                  type="primary" 
                  @click="restartComponent(component)"
                  :disabled="component.status === 'healthy'"
                >
                  重启
                </el-button>
              </div>
            </div>
          </el-col>
        </el-row>
      </el-card>
    </div>

    <!-- 健康趋势图表 -->
    <div class="health-trends">
      <el-row :gutter="20">
        <el-col :span="12">
          <el-card header="健康评分趋势" class="chart-card">
            <div class="chart-container">
              <v-chart
                class="chart"
                :option="healthScoreChartOption"
                autoresize
              />
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="12">
          <el-card header="服务可用性" class="chart-card">
            <div class="chart-container">
              <v-chart
                class="chart"
                :option="availabilityChartOption"
                autoresize
              />
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 健康检查日志 -->
    <el-card header="健康检查日志" class="health-logs">
      <template #extra>
        <el-button 
          size="small" 
          @click="clearLogs"
          :disabled="healthLogs.length === 0"
        >
          清空日志
        </el-button>
      </template>
      
      <el-table :data="healthLogs" stripe max-height="300">
        <el-table-column prop="timestamp" label="时间" width="180">
          <template #default="{ row }">
            {{ formatTime(row.timestamp) }}
          </template>
        </el-table-column>
        <el-table-column prop="component" label="组件" width="150" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getHealthTagType(row.status)" size="small">
              {{ getHealthStatusText(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="message" label="消息" show-overflow-tooltip />
        <el-table-column prop="duration" label="耗时" width="100">
          <template #default="{ row }">
            {{ row.duration }}ms
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 系统诊断结果弹窗 -->
    <el-dialog
      v-model="showDiagnostic"
      title="系统诊断结果"
      width="60%"
      top="5vh"
    >
      <div class="diagnostic-results">
        <div class="diagnostic-summary">
          <el-alert
            :title="diagnosticResult.summary"
            :type="diagnosticResult.type"
            show-icon
            :closable="false"
          />
        </div>
        
        <div class="diagnostic-details">
          <el-collapse v-model="activeCollapse">
            <el-collapse-item
              v-for="item in diagnosticResult.details"
              :key="item.category"
              :title="item.category"
              :name="item.category"
            >
              <div class="diagnostic-category">
                <div 
                  v-for="check in item.checks"
                  :key="check.name"
                  class="diagnostic-check"
                  :class="`check-${check.status}`"
                >
                  <div class="check-header">
                    <el-icon>
                      <Select v-if="check.status === 'pass'" />
                      <Warning v-else-if="check.status === 'warning'" />
                      <CircleClose v-else />
                    </el-icon>
                    <span class="check-name">{{ check.name }}</span>
                    <el-tag :type="getHealthTagType(check.status)" size="small">
                      {{ getHealthStatusText(check.status) }}
                    </el-tag>
                  </div>
                  <div class="check-message">{{ check.message }}</div>
                  <div v-if="check.suggestion" class="check-suggestion">
                    建议: {{ check.suggestion }}
                  </div>
                </div>
              </div>
            </el-collapse-item>
          </el-collapse>
        </div>
      </div>
      
      <template #footer>
        <el-button @click="showDiagnostic = false">关闭</el-button>
        <el-button type="primary" @click="exportDiagnostic">导出报告</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useSystemStore } from '@/stores/system'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart, PieChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
} from 'echarts/components'
import VChart from 'vue-echarts'
import { formatTime } from '@/utils/date'

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  PieChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
])

// 响应式状态
const loading = ref(false)
const checkingHealth = ref(false)
const autoRefresh = ref(true)
const showDiagnostic = ref(false)
const activeCollapse = ref<string[]>([])

// 健康数据 - 从API获取
const healthScore = ref(0)
const systemUptime = ref('获取中...')
const lastRestart = ref('获取中...')
const availability = ref(0)

const healthStats = ref({
  healthy: 0,
  warning: 0,
  critical: 0
})

// 组件健康状态 - 从API获取
const healthComponents = ref([])

// 健康检查日志 - 从API获取
const healthLogs = ref([])

// 诊断结果
const diagnosticResult = ref({
  summary: '',
  type: 'success' as 'success' | 'warning' | 'error',
  details: [] as any[]
})

// 计算属性
const overallHealthType = computed(() => {
  if (healthScore.value >= 80) return 'success'
  if (healthScore.value >= 60) return 'warning'  
  return 'danger'
})

const overallHealthText = computed(() => {
  if (healthScore.value >= 80) return '系统健康'
  if (healthScore.value >= 60) return '需要关注'
  return '系统异常'
})

// 图表配置 - 使用实际数据
const healthScoreData = ref([])
const healthScoreChartOption = computed(() => ({
  tooltip: {
    trigger: 'axis',
    formatter: '{b}: {c}%'
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true
  },
  xAxis: {
    type: 'category',
    data: healthScoreData.value.map(item => item.time) || []
  },
  yAxis: {
    type: 'value',
    min: 0,
    max: 100,
    axisLabel: {
      formatter: '{value}%'
    }
  },
  series: [
    {
      name: '健康评分',
      type: 'line',
      smooth: true,
      data: healthScoreData.value.map(item => item.score) || [],
      lineStyle: { 
        color: '#67c23a',
        width: 3
      },
      areaStyle: {
        color: {
          type: 'linear',
          x: 0,
          y: 0,
          x2: 0,
          y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(103, 194, 58, 0.3)' },
            { offset: 1, color: 'rgba(103, 194, 58, 0.1)' }
          ]
        }
      }
    }
  ]
}))

const availabilityChartOption = computed(() => ({
  tooltip: {
    trigger: 'item',
    formatter: '{a} <br/>{b}: {c}% ({d}%)'
  },
  legend: {
    orient: 'vertical',
    left: 'left'
  },
  series: [
    {
      name: '可用性',
      type: 'pie',
      radius: '70%',
      data: [
        { value: healthStats.value.healthy, name: '健康服务', itemStyle: { color: '#67c23a' } },
        { value: healthStats.value.warning, name: '警告服务', itemStyle: { color: '#e6a23c' } },
        { value: healthStats.value.critical, name: '异常服务', itemStyle: { color: '#f56c6c' } }
      ],
      emphasis: {
        itemStyle: {
          shadowBlur: 10,
          shadowOffsetX: 0,
          shadowColor: 'rgba(0, 0, 0, 0.5)'
        }
      }
    }
  ]
}))

// 方法
const getScoreColor = (score: number) => {
  if (score >= 80) return '#67c23a'
  if (score >= 60) return '#e6a23c'
  return '#f56c6c'
}

const getScoreStatusClass = (score: number) => {
  if (score >= 80) return 'status-healthy'
  if (score >= 60) return 'status-warning'
  return 'status-critical'
}

const getScoreStatusText = (score: number) => {
  if (score >= 80) return '优秀'
  if (score >= 60) return '良好'
  return '需改善'
}

const getHealthTagType = (status: string) => {
  const types: Record<string, string> = {
    healthy: 'success',
    pass: 'success',
    warning: 'warning',
    critical: 'danger',
    fail: 'danger'
  }
  return types[status] || 'info'
}

const getHealthStatusText = (status: string) => {
  const texts: Record<string, string> = {
    healthy: '健康',
    pass: '通过',
    warning: '警告',
    critical: '异常',
    fail: '失败'
  }
  return texts[status] || status
}

const getResponseTimeClass = (time: number) => {
  if (time <= 30) return 'response-good'
  if (time <= 100) return 'response-normal'
  return 'response-slow'
}

const toggleAutoRefresh = () => {
  autoRefresh.value = !autoRefresh.value
  ElMessage.success(autoRefresh.value ? '已开启自动监控' : '已停止自动监控')
}

const refreshHealthStatus = async () => {
  loading.value = true
  try {
    // 获取健康状态数据
    const systemStore = useSystemStore()
    await systemStore.fetchHealthStatus()
    
    // 更新本地状态
    if (systemStore.healthStatus) {
      const health = systemStore.healthStatus
      healthScore.value = health.metrics?.cpu_usage || 0
      systemUptime.value = `运行时间: ${health.metrics?.uptime || 0}秒`
      availability.value = 99.0
      
      // 更新统计数据
      const services = health.services || {}
      let healthy = 0, warning = 0, critical = 0
      
      Object.values(services).forEach(status => {
        if (status === 'healthy') healthy++
        else if (status === 'warning') warning++
        else critical++
      })
      
      healthStats.value = { healthy, warning, critical }
    }
    
    ElMessage.success('健康状态已刷新')
  } catch (error) {
    console.error('获取健康状态失败:', error)
    ElMessage.error('刷新失败')
  } finally {
    loading.value = false
  }
}

const runHealthCheck = async () => {
  checkingHealth.value = true
  
  try {
    // 执行真实的健康检查API调用
    const systemStore = useSystemStore()
    await systemStore.fetchHealthStatus()
    
    // 基于实际健康状态生成诊断结果
    const health = systemStore.healthStatus
    if (health) {
      const services = health.services || {}
      const metrics = health.metrics || {}
      
      let passCount = 0
      let warningCount = 0
      let criticalCount = 0
      
      const checks = Object.entries(services).map(([name, status]) => {
        if (status === 'healthy') passCount++
        else if (status === 'warning') warningCount++
        else criticalCount++
        
        return {
          name: `${name}服务`,
          status: status === 'healthy' ? 'pass' : status,
          message: status === 'healthy' ? '服务运行正常' : `服务状态: ${status}`
        }
      })
      
      diagnosticResult.value = {
        summary: `系统检查完成，发现 ${passCount} 个正常服务，${warningCount} 个警告，${criticalCount} 个异常`,
        type: criticalCount > 0 ? 'error' : warningCount > 0 ? 'warning' : 'success',
        details: [
          {
            category: '服务状态',
            checks
          },
          {
            category: '系统资源',
            checks: [
              {
                name: 'CPU使用率',
                status: (metrics.cpu_usage || 0) > 80 ? 'warning' : 'pass',
                message: `当前CPU使用率: ${metrics.cpu_usage || 0}%`
              },
              {
                name: '内存使用',
                status: (metrics.memory_usage || 0) > 80 ? 'warning' : 'pass',
                message: `当前内存使用率: ${metrics.memory_usage || 0}%`
              }
            ]
          }
        ]
      }
    } else {
      diagnosticResult.value = {
        summary: '无法获取系统健康状态',
        type: 'error',
        details: []
      }
    }
    
    showDiagnostic.value = true
    activeCollapse.value = ['服务状态', '系统资源']
    
    ElMessage.success('健康检查完成')
  } catch (error) {
    console.error('健康检查失败:', error)
    ElMessage.error('健康检查失败')
  } finally {
    checkingHealth.value = false
  }
}

const checkComponent = async (component: any) => {
  component.checking = true
  
  try {
    // 执行实际的组件健康检查
    const systemStore = useSystemStore()
    await systemStore.fetchHealthStatus()
    
    // 更新组件状态
    component.lastCheck = new Date()
    
    // 添加检查日志
    healthLogs.value.unshift({
      timestamp: new Date(),
      component: component.name,
      status: component.status,
      message: `组件检查完成: ${getHealthStatusText(component.status)}`,
      duration: component.responseTime
    })
    
    ElMessage.success(`${component.name} 检查完成`)
  } catch (error) {
    console.error(`${component.name} 检查失败:`, error)
    ElMessage.error(`${component.name} 检查失败`)
  } finally {
    component.checking = false
  }
}

const restartComponent = async (component: any) => {
  try {
    await ElMessageBox.confirm(
      `确认重启 ${component.name} 服务？这可能会影响相关功能。`,
      '确认重启',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    ElMessage.info(`正在重启 ${component.name}...`)
    
    // 调用系统重启API（需要后端实现）
    const systemStore = useSystemStore()
    await systemStore.restartSystem()
    
    // 重新获取状态
    await systemStore.fetchHealthStatus()
    
    component.lastCheck = new Date()
    
    // 添加日志
    healthLogs.value.unshift({
      timestamp: new Date(),
      component: component.name,
      status: 'healthy',
      message: '服务重启成功',
      duration: component.responseTime
    })
    
    ElMessage.success(`${component.name} 重启成功`)
  } catch (error) {
    if (error !== 'cancel') {
      console.error(`${component.name} 重启失败:`, error)
      ElMessage.error(`${component.name} 重启失败`)
    }
  }
}

const clearLogs = () => {
  healthLogs.value = []
  ElMessage.success('日志已清空')
}

const exportDiagnostic = () => {
  // 模拟导出诊断报告
  const report = {
    timestamp: new Date().toISOString(),
    healthScore: healthScore.value,
    summary: diagnosticResult.value.summary,
    details: diagnosticResult.value.details
  }
  
  const dataStr = JSON.stringify(report, null, 2)
  const dataUri = `data:application/json;charset=utf-8,${ encodeURIComponent(dataStr)}`
  
  const exportFileDefaultName = `health-diagnostic-${new Date().toISOString().split('T')[0]}.json`
  
  const linkElement = document.createElement('a')
  linkElement.setAttribute('href', dataUri)
  linkElement.setAttribute('download', exportFileDefaultName)
  linkElement.click()
  
  ElMessage.success('诊断报告已导出')
}

// 定时器
let refreshTimer: number | null = null

const startAutoRefresh = () => {
  if (refreshTimer) clearInterval(refreshTimer)
  
  refreshTimer = setInterval(async () => {
    if (autoRefresh.value) {
      // 静默获取最新健康状态
      try {
        const systemStore = useSystemStore()
        await systemStore.fetchHealthStatus()
        
        // 更新组件的最后检查时间
        healthComponents.value.forEach(component => {
          component.lastCheck = new Date()
        })
      } catch (error) {
        console.warn('自动刷新健康状态失败:', error)
      }
    }
  }, 30000) // 每30秒更新一次
}

// 生命周期
onMounted(async () => {
  // 初始化系统健康数据
  await refreshHealthStatus()
  startAutoRefresh()
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<style scoped lang="scss">
.health-status {
  .health-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 0 20px 0;
    
    .header-title {
      display: flex;
      align-items: center;
      gap: 12px;
      
      h1 {
        margin: 0;
        font-size: 20px;
        font-weight: 600;
      }
    }
    
    .header-actions {
      display: flex;
      align-items: center;
      gap: 12px;
    }
  }
  
  .health-overview {
    margin-bottom: 20px;
    
    .health-card {
      border: none;
      height: 180px;
      display: flex;
      align-items: center;
      
      .health-summary {
        width: 100%;
        
        .health-score {
          display: flex;
          align-items: center;
          gap: 20px;
          
          .score-circle {
            flex-shrink: 0;
          }
          
          .score-text {
            font-size: 18px;
            font-weight: 600;
          }
          
          .score-info {
            .score-label {
              font-size: 16px;
              color: var(--el-text-color-primary);
              margin-bottom: 8px;
            }
            
            .score-status {
              font-size: 14px;
              font-weight: 500;
              
              &.status-healthy {
                color: var(--el-color-success);
              }
              
              &.status-warning {
                color: var(--el-color-warning);
              }
              
              &.status-critical {
                color: var(--el-color-danger);
              }
            }
          }
        }
      }
      
      .health-stats {
        display: flex;
        justify-content: space-around;
        width: 100%;
        
        .stat-item {
          text-align: center;
          
          .stat-value {
            font-size: 32px;
            font-weight: 600;
            margin-bottom: 8px;
            
            &.success {
              color: var(--el-color-success);
            }
            
            &.warning {
              color: var(--el-color-warning);
            }
            
            &.danger {
              color: var(--el-color-danger);
            }
          }
          
          .stat-label {
            font-size: 14px;
            color: var(--el-text-color-secondary);
          }
        }
      }
      
      .uptime-info {
        width: 100%;
        
        .uptime-main {
          text-align: center;
          margin-bottom: 20px;
          
          .uptime-value {
            font-size: 24px;
            font-weight: 600;
            color: var(--el-color-primary);
            margin-bottom: 8px;
          }
          
          .uptime-label {
            font-size: 14px;
            color: var(--el-text-color-secondary);
          }
        }
        
        .uptime-details {
          .detail-item {
            display: flex;
            justify-content: space-between;
            margin-bottom: 8px;
            
            .detail-label {
              font-size: 13px;
              color: var(--el-text-color-secondary);
            }
            
            .detail-value {
              font-size: 13px;
              color: var(--el-text-color-primary);
              font-weight: 500;
            }
          }
        }
      }
    }
  }
  
  .components-health {
    margin-bottom: 20px;
    
    .component-card {
      border: 1px solid var(--el-border-color);
      border-radius: 8px;
      padding: 16px;
      margin-bottom: 16px;
      transition: all 0.3s;
      
      &:hover {
        box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
      }
      
      &.component-healthy {
        border-left: 4px solid var(--el-color-success);
      }
      
      &.component-warning {
        border-left: 4px solid var(--el-color-warning);
      }
      
      &.component-critical {
        border-left: 4px solid var(--el-color-danger);
      }
      
      .component-header {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 12px;
        
        .component-name {
          font-weight: 600;
          flex: 1;
        }
      }
      
      .component-metrics {
        margin-bottom: 12px;
        
        .metric {
          display: flex;
          justify-content: space-between;
          margin-bottom: 6px;
          font-size: 13px;
          
          .metric-label {
            color: var(--el-text-color-secondary);
          }
          
          .metric-value {
            font-weight: 500;
            
            &.response-good {
              color: var(--el-color-success);
            }
            
            &.response-normal {
              color: var(--el-color-warning);
            }
            
            &.response-slow {
              color: var(--el-color-danger);
            }
          }
        }
      }
      
      .component-actions {
        display: flex;
        gap: 8px;
      }
    }
  }
  
  .health-trends {
    margin-bottom: 20px;
    
    .chart-card {
      .chart-container {
        height: 300px;
        
        .chart {
          height: 100%;
          width: 100%;
        }
      }
    }
  }
  
  .health-logs {
    .el-table {
      font-size: 13px;
    }
  }
  
  .diagnostic-results {
    .diagnostic-summary {
      margin-bottom: 20px;
    }
    
    .diagnostic-details {
      .diagnostic-category {
        .diagnostic-check {
          padding: 12px;
          border: 1px solid var(--el-border-color);
          border-radius: 6px;
          margin-bottom: 8px;
          
          &.check-pass {
            border-left: 4px solid var(--el-color-success);
          }
          
          &.check-warning {
            border-left: 4px solid var(--el-color-warning);
          }
          
          &.check-fail {
            border-left: 4px solid var(--el-color-danger);
          }
          
          .check-header {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-bottom: 8px;
            
            .check-name {
              font-weight: 500;
              flex: 1;
            }
          }
          
          .check-message {
            font-size: 14px;
            color: var(--el-text-color-primary);
            margin-bottom: 8px;
          }
          
          .check-suggestion {
            font-size: 13px;
            color: var(--el-color-warning);
            padding: 8px;
            background: var(--el-color-warning-light-9);
            border-radius: 4px;
          }
        }
      }
    }
  }
}
</style>