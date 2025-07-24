<template>
  <div class="health-status">
    <!-- 页面标题和工具栏 -->
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">
          <el-icon><Monitor /></el-icon>
          系统健康监控
        </h1>
        <span class="page-description">实时监控系统各组件健康状态</span>
      </div>
      <div class="header-actions">
        <el-button type="primary" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          刷新数据
        </el-button>
        <el-button @click="runHealthCheck">
          <el-icon><Tools /></el-icon>
          健康检查
        </el-button>
        <el-button @click="exportReport">
          <el-icon><Download /></el-icon>
          导出报告
        </el-button>
        <el-button @click="showSettings = true">
          <el-icon><Setting /></el-icon>
          监控设置
        </el-button>
      </div>
    </div>

    <!-- 系统总览 -->
    <div class="overview-section">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <span>系统总览</span>
            <div class="overall-status">
              <el-tag
                :type="getOverallStatusType(overallHealth.status)"
                size="large"
              >
                <el-icon style="margin-right: 4px;">
                  <Check v-if="overallHealth.status === 'healthy'" />
                  <Warning v-else-if="overallHealth.status === 'warning'" />
                  <Close v-else />
                </el-icon>
                {{ getStatusText(overallHealth.status) }}
              </el-tag>
            </div>
          </div>
        </template>

        <el-row :gutter="20">
          <el-col :span="6">
            <StatCard
              title="运行时间"
              :value="systemInfo.uptime"
              color="#67C23A"
              icon="Timer"
            />
          </el-col>
          <el-col :span="6">
            <StatCard
              title="CPU 使用率"
              :value="systemInfo.cpuUsage"
              suffix="%"
              :trend="systemInfo.cpuTrend"
              trend-label="较1小时前"
              color="#409EFF"
              icon="Cpu"
            />
          </el-col>
          <el-col :span="6">
            <StatCard
              title="内存使用率"
              :value="systemInfo.memoryUsage"
              suffix="%"
              :trend="systemInfo.memoryTrend"
              trend-label="较1小时前"
              color="#E6A23C"
              icon="Memory"
            />
          </el-col>
          <el-col :span="6">
            <StatCard
              title="磁盘使用率"
              :value="systemInfo.diskUsage"
              suffix="%"
              :trend="systemInfo.diskTrend"
              trend-label="较1小时前"
              color="#F56C6C"
              icon="HardDisk"
            />
          </el-col>
        </el-row>

        <div class="system-metrics" style="margin-top: 20px;">
          <el-row :gutter="20">
            <el-col :span="12">
              <div class="metric-chart">
                <h4>系统资源使用趋势</h4>
                <div ref="resourceChart" style="height: 300px;"></div>
              </div>
            </el-col>
            <el-col :span="12">
              <div class="metric-chart">
                <h4>网络流量监控</h4>
                <div ref="networkChart" style="height: 300px;"></div>
              </div>
            </el-col>
          </el-row>
        </div>
      </el-card>
    </div>

    <!-- 组件健康状态 -->
    <div class="components-section">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <span>组件健康状态</span>
            <div class="health-summary">
              <span class="health-item">
                <el-badge :value="healthStats.healthy" type="success">
                  健康
                </el-badge>
              </span>
              <span class="health-item">
                <el-badge :value="healthStats.warning" type="warning">
                  警告
                </el-badge>
              </span>
              <span class="health-item">
                <el-badge :value="healthStats.critical" type="danger">
                  异常
                </el-badge>
              </span>
            </div>
          </div>
        </template>

        <el-row :gutter="20">
          <el-col
            v-for="component in healthComponents"
            :key="component.id"
            :span="8"
          >
            <el-card class="component-card" :class="getComponentStatusClass(component.status)">
              <div class="component-header">
                <div class="component-info">
                  <el-icon class="component-icon">
                    <component :is="getComponentIcon(component.type)" />
                  </el-icon>
                  <div class="component-details">
                    <h4>{{ component.name }}</h4>
                    <span class="component-type">{{ component.type }}</span>
                  </div>
                </div>
                <div class="component-status">
                  <el-tag :type="getStatusTagType(component.status)" size="small">
                    {{ getStatusText(component.status) }}
                  </el-tag>
                </div>
              </div>
              
              <div class="component-metrics">
                <div class="metric-row">
                  <span class="metric-label">响应时间:</span>
                  <span class="metric-value">{{ component.responseTime }}ms</span>
                </div>
                <div class="metric-row">
                  <span class="metric-label">最后检查:</span>
                  <span class="metric-value">{{ formatDateTime(component.lastCheck) }}</span>
                </div>
                <div v-if="component.errorCount > 0" class="metric-row error">
                  <span class="metric-label">错误次数:</span>
                  <span class="metric-value">{{ component.errorCount }}</span>
                </div>
              </div>

              <div class="component-actions">
                <el-button size="small" @click="showComponentDetail(component)">
                  详情
                </el-button>
                <el-button
                  size="small"
                  type="primary"
                  @click="checkComponent(component.id)"
                >
                  检查
                </el-button>
                <el-button
                  v-if="component.status !== 'healthy'"
                  size="small"
                  type="warning"
                  @click="restartComponent(component.id)"
                >
                  重启
                </el-button>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </el-card>
    </div>

    <!-- 服务状态监控 -->
    <div class="services-section">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <span>服务状态监控</span>
            <el-button size="small" @click="refreshServices">
              <el-icon><Refresh /></el-icon>
              刷新服务
            </el-button>
          </div>
        </template>

        <el-table :data="services" style="width: 100%">
          <el-table-column prop="name" label="服务名称" width="200">
            <template #default="{ row }">
              <div class="service-name">
                <el-icon class="service-icon">
                  <component :is="getServiceIcon(row.type)" />
                </el-icon>
                <span>{{ row.name }}</span>
              </div>
            </template>
          </el-table-column>
          <el-table-column prop="status" label="状态" width="100">
            <template #default="{ row }">
              <el-tag :type="getStatusTagType(row.status)" size="small">
                {{ getStatusText(row.status) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="port" label="端口" width="100" />
          <el-table-column prop="cpu" label="CPU使用" width="120">
            <template #default="{ row }">
              <el-progress
                :percentage="row.cpu"
                :stroke-width="8"
                :color="getProgressColor(row.cpu)"
                :show-text="false"
              />
              <span style="margin-left: 8px;">{{ row.cpu }}%</span>
            </template>
          </el-table-column>
          <el-table-column prop="memory" label="内存使用" width="120">
            <template #default="{ row }">
              <el-progress
                :percentage="row.memory"
                :stroke-width="8"
                :color="getProgressColor(row.memory)"
                :show-text="false"
              />
              <span style="margin-left: 8px;">{{ row.memory }}%</span>
            </template>
          </el-table-column>
          <el-table-column prop="uptime" label="运行时间" width="150" />
          <el-table-column prop="requests" label="请求数" width="100" />
          <el-table-column prop="lastRestart" label="最后重启" width="160">
            <template #default="{ row }">
              {{ formatDateTime(row.lastRestart) }}
            </template>
          </el-table-column>
          <el-table-column label="操作" width="200" fixed="right">
            <template #default="{ row }">
              <el-button size="small" @click="showServiceLogs(row)">
                日志
              </el-button>
              <el-button
                size="small"
                type="primary"
                :disabled="row.status === 'stopped'"
                @click="restartService(row.id)"
              >
                重启
              </el-button>
              <el-button
                size="small"
                :type="row.status === 'running' ? 'danger' : 'success'"
                @click="toggleService(row)"
              >
                {{ row.status === 'running' ? '停止' : '启动' }}
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </div>

    <!-- 实时日志监控 -->
    <div class="logs-section">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <span>实时日志监控</span>
            <div class="log-controls">
              <el-select v-model="logLevel" placeholder="日志级别" size="small" style="width: 120px;">
                <el-option label="全部" value="all" />
                <el-option label="错误" value="error" />
                <el-option label="警告" value="warn" />
                <el-option label="信息" value="info" />
                <el-option label="调试" value="debug" />
              </el-select>
              <el-button
                size="small"
                :type="logPaused ? 'success' : 'warning'"
                @click="toggleLogPause"
              >
                {{ logPaused ? '继续' : '暂停' }}
              </el-button>
              <el-button size="small" @click="clearLogs">
                清空
              </el-button>
            </div>
          </div>
        </template>

        <div class="log-container">
          <div ref="logContent" class="log-content">
            <div
              v-for="(log, index) in filteredLogs"
              :key="index"
              class="log-entry"
              :class="getLogLevelClass(log.level)"
            >
              <span class="log-time">{{ formatLogTime(log.timestamp) }}</span>
              <span class="log-level">{{ log.level.toUpperCase() }}</span>
              <span class="log-source">{{ log.source }}</span>
              <span class="log-message">{{ log.message }}</span>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 健康检查历史 -->
    <div class="history-section">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <span>健康检查历史</span>
            <el-button size="small" @click="clearHistory">
              清空历史
            </el-button>
          </div>
        </template>

        <el-timeline>
          <el-timeline-item
            v-for="(check, index) in healthHistory"
            :key="index"
            :timestamp="formatDateTime(check.timestamp)"
            :type="getTimelineType(check.status)"
            placement="top"
          >
            <el-card class="history-card" :class="getStatusClass(check.status)">
              <div class="history-header">
                <span class="history-title">{{ check.title }}</span>
                <el-tag :type="getStatusTagType(check.status)" size="small">
                  {{ getStatusText(check.status) }}
                </el-tag>
              </div>
              <p class="history-description">{{ check.description }}</p>
              <div class="history-details">
                <span>检查组件: {{ check.componentCount }}</span>
                <span>发现问题: {{ check.issueCount }}</span>
                <span>耗时: {{ check.duration }}ms</span>
              </div>
            </el-card>
          </el-timeline-item>
        </el-timeline>
      </el-card>
    </div>

    <!-- 组件详情对话框 -->
    <el-dialog
      v-model="componentDetailVisible"
      title="组件详情"
      width="800px"
      :close-on-click-modal="false"
    >
      <ComponentDetail v-if="selectedComponent" :component="selectedComponent" />
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="componentDetailVisible = false">关闭</el-button>
          <el-button type="primary" @click="checkComponent(selectedComponent?.id)">
            健康检查
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 服务日志对话框 -->
    <el-dialog
      v-model="serviceLogsVisible"
      :title="`${selectedService?.name} - 服务日志`"
      width="90%"
      :close-on-click-modal="false"
    >
      <ServiceLogs v-if="selectedService" :service="selectedService" />
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="serviceLogsVisible = false">关闭</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 监控设置对话框 -->
    <el-dialog
      v-model="showSettings"
      title="监控设置"
      width="600px"
    >
      <div class="settings-form">
        <el-form :model="monitorSettings" label-width="140px">
          <el-form-item label="检查间隔">
            <el-select v-model="monitorSettings.checkInterval">
              <el-option label="10秒" :value="10000" />
              <el-option label="30秒" :value="30000" />
              <el-option label="1分钟" :value="60000" />
              <el-option label="5分钟" :value="300000" />
              <el-option label="10分钟" :value="600000" />
            </el-select>
          </el-form-item>
          <el-form-item label="告警阈值">
            <el-form :model="monitorSettings.thresholds" label-width="80px">
              <el-form-item label="CPU">
                <el-input-number
                  v-model="monitorSettings.thresholds.cpu"
                  :min="0"
                  :max="100"
                  :precision="1"
                />
                <span style="margin-left: 8px;">%</span>
              </el-form-item>
              <el-form-item label="内存">
                <el-input-number
                  v-model="monitorSettings.thresholds.memory"
                  :min="0"
                  :max="100"
                  :precision="1"
                />
                <span style="margin-left: 8px;">%</span>
              </el-form-item>
              <el-form-item label="磁盘">
                <el-input-number
                  v-model="monitorSettings.thresholds.disk"
                  :min="0"
                  :max="100"
                  :precision="1"
                />
                <span style="margin-left: 8px;">%</span>
              </el-form-item>
            </el-form>
          </el-form-item>
          <el-form-item label="日志保留">
            <el-select v-model="monitorSettings.logRetention">
              <el-option label="1小时" :value="3600000" />
              <el-option label="6小时" :value="21600000" />
              <el-option label="24小时" :value="86400000" />
              <el-option label="7天" :value="604800000" />
            </el-select>
          </el-form-item>
          <el-form-item label="自动重启">
            <el-switch v-model="monitorSettings.autoRestart" />
          </el-form-item>
        </el-form>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showSettings = false">取消</el-button>
          <el-button type="primary" @click="saveSettings">保存设置</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Monitor, Refresh, Tools, Download, Setting, Check, Warning, Close,
  Timer, Cpu, Memory, HardDisk, Connection, Database, Server, Link
} from '@element-plus/icons-vue'
import StatCard from '../../components/common/StatCard.vue'
import ComponentDetail from '../../components/business/ComponentDetail.vue'
import ServiceLogs from '../../components/business/ServiceLogs.vue'
import * as echarts from 'echarts'

// 响应式数据
const loading = ref(false)
const showSettings = ref(false)
const componentDetailVisible = ref(false)
const serviceLogsVisible = ref(false)
const selectedComponent = ref<any>(null)
const selectedService = ref<any>(null)
const logPaused = ref(false)
const logLevel = ref('all')
const healthCheckTimer = ref<NodeJS.Timeout | null>(null)
const logUpdateTimer = ref<NodeJS.Timeout | null>(null)

// 图表实例
const resourceChart = ref<any>(null)
const networkChart = ref<any>(null)
const logContent = ref<HTMLElement>()

// 系统总览数据
const overallHealth = ref({
  status: 'healthy' as 'healthy' | 'warning' | 'critical'
})

const systemInfo = ref({
  uptime: '15天 8小时',
  cpuUsage: 45.6,
  cpuTrend: -2.3,
  memoryUsage: 67.8,
  memoryTrend: 5.2,
  diskUsage: 23.4,
  diskTrend: 0.8
})

const healthStats = ref({
  healthy: 8,
  warning: 2,
  critical: 1
})

// 组件健康状态
const healthComponents = ref([
  {
    id: 'gateway_core',
    name: '网关核心',
    type: 'Core Service',
    status: 'healthy',
    responseTime: 12,
    lastCheck: '2024-01-20 15:30:25',
    errorCount: 0,
    description: 'IoT 网关核心服务，处理设备连接和数据路由'
  },
  {
    id: 'modbus_driver',
    name: 'Modbus 驱动',
    type: 'Protocol Driver',
    status: 'healthy',
    responseTime: 8,
    lastCheck: '2024-01-20 15:30:20',
    errorCount: 0,
    description: 'Modbus TCP/RTU 协议驱动服务'
  },
  {
    id: 'opcua_driver',
    name: 'OPC UA 驱动',
    type: 'Protocol Driver',
    status: 'warning',
    responseTime: 156,
    lastCheck: '2024-01-20 15:29:45',
    errorCount: 3,
    description: 'OPC UA 协议驱动服务，连接响应较慢'
  },
  {
    id: 'mqtt_broker',
    name: 'MQTT 代理',
    type: 'Message Broker',
    status: 'healthy',
    responseTime: 5,
    lastCheck: '2024-01-20 15:30:22',
    errorCount: 0,
    description: 'MQTT5 消息代理服务'
  },
  {
    id: 'data_processor',
    name: '数据处理器',
    type: 'Data Service',
    status: 'healthy',
    responseTime: 23,
    lastCheck: '2024-01-20 15:30:18',
    errorCount: 0,
    description: '数据预处理和转换服务'
  },
  {
    id: 'alert_engine',
    name: '告警引擎',
    type: 'Alert Service',
    status: 'critical',
    responseTime: 0,
    lastCheck: '2024-01-20 15:25:10',
    errorCount: 12,
    description: '告警规则引擎服务，当前离线'
  },
  {
    id: 'database',
    name: '时序数据库',
    type: 'Database',
    status: 'healthy',
    responseTime: 18,
    lastCheck: '2024-01-20 15:30:25',
    errorCount: 0,
    description: 'InfluxDB 时序数据存储'
  },
  {
    id: 'web_api',
    name: 'Web API',
    type: 'API Service',
    status: 'healthy',
    responseTime: 15,
    lastCheck: '2024-01-20 15:30:23',
    errorCount: 0,
    description: 'RESTful API 服务'
  },
  {
    id: 'file_manager',
    name: '文件管理器',
    type: 'Storage Service',
    status: 'warning',
    responseTime: 89,
    lastCheck: '2024-01-20 15:30:15',
    errorCount: 1,
    description: '配置文件和日志管理服务'
  }
])

// 服务状态监控
const services = ref([
  {
    id: 'gateway_main',
    name: 'Gateway Main',
    type: 'main',
    status: 'running',
    port: 8080,
    cpu: 12.5,
    memory: 256,
    uptime: '15天 8小时',
    requests: 15678,
    lastRestart: '2024-01-05 10:30:00'
  },
  {
    id: 'modbus_service',
    name: 'Modbus Service',
    type: 'driver',
    status: 'running',
    port: 502,
    cpu: 8.3,
    memory: 128,
    uptime: '15天 8小时',
    requests: 8942,
    lastRestart: '2024-01-05 10:30:15'
  },
  {
    id: 'opcua_service',
    name: 'OPC UA Service',
    type: 'driver',
    status: 'warning',
    port: 4840,
    cpu: 25.7,
    memory: 512,
    uptime: '2小时 15分钟',
    requests: 1234,
    lastRestart: '2024-01-20 13:15:30'
  },
  {
    id: 'mqtt_service',
    name: 'MQTT Service',
    type: 'broker',
    status: 'running',
    port: 1883,
    cpu: 5.2,
    memory: 96,
    uptime: '15天 8小时',
    requests: 25673,
    lastRestart: '2024-01-05 10:30:45'
  },
  {
    id: 'alert_service',
    name: 'Alert Service',
    type: 'service',
    status: 'stopped',
    port: 8081,
    cpu: 0,
    memory: 0,
    uptime: '0分钟',
    requests: 0,
    lastRestart: '2024-01-20 15:25:10'
  }
])

// 实时日志
const logs = ref([
  {
    timestamp: new Date(),
    level: 'info',
    source: 'Gateway Core',
    message: '系统启动完成，所有服务正常运行'
  },
  {
    timestamp: new Date(Date.now() - 1000),
    level: 'warn',
    source: 'OPC UA Driver',
    message: '连接到 OPC UA 服务器响应超时，正在重试...'
  },
  {
    timestamp: new Date(Date.now() - 2000),
    level: 'error',
    source: 'Alert Engine',
    message: '告警引擎服务连接失败，请检查服务状态'
  },
  {
    timestamp: new Date(Date.now() - 3000),
    level: 'info',
    source: 'Data Processor',
    message: '已处理 1000 条数据记录'
  },
  {
    timestamp: new Date(Date.now() - 4000),
    level: 'debug',
    source: 'MQTT Broker',
    message: '新客户端连接: client_12345'
  }
])

// 健康检查历史
const healthHistory = ref([
  {
    timestamp: '2024-01-20 15:30:00',
    title: '定期健康检查',
    description: '完成所有组件健康检查，发现1个严重问题，2个警告',
    status: 'warning',
    componentCount: 9,
    issueCount: 3,
    duration: 1250
  },
  {
    timestamp: '2024-01-20 15:15:00',
    title: '自动健康检查',
    description: '所有组件运行正常，系统状态良好',
    status: 'healthy',
    componentCount: 9,
    issueCount: 0,
    duration: 890
  },
  {
    timestamp: '2024-01-20 15:00:00',
    title: '告警触发检查',
    description: '告警引擎服务异常，已自动尝试重启',
    status: 'critical',
    componentCount: 9,
    issueCount: 1,
    duration: 2100
  }
])

// 监控设置
const monitorSettings = ref({
  checkInterval: 60000,
  thresholds: {
    cpu: 80.0,
    memory: 85.0,
    disk: 90.0
  },
  logRetention: 86400000,
  autoRestart: true
})

// 计算属性
const filteredLogs = computed(() => {
  if (logLevel.value === 'all') {
    return logs.value
  }
  return logs.value.filter(log => log.level === logLevel.value)
})

// 方法
const refreshData = async () => {
  loading.value = true
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // 更新系统信息
    systemInfo.value = {
      ...systemInfo.value,
      cpuUsage: 30 + Math.random() * 40,
      memoryUsage: 50 + Math.random() * 30,
      diskUsage: 20 + Math.random() * 20
    }
    
    // 更新组件状态
    healthComponents.value.forEach(component => {
      component.lastCheck = new Date().toISOString().slice(0, 19).replace('T', ' ')
      component.responseTime = Math.floor(Math.random() * 100) + 5
    })
    
    updateOverallHealth()
    ElMessage.success('数据刷新成功')
  } catch (error) {
    ElMessage.error('数据刷新失败')
  } finally {
    loading.value = false
  }
}

const runHealthCheck = async () => {
  loading.value = true
  try {
    ElMessage.info('开始执行健康检查...')
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // 模拟健康检查结果
    const checkResult = {
      timestamp: new Date().toISOString().slice(0, 19).replace('T', ' '),
      title: '手动健康检查',
      description: '用户触发的健康检查已完成',
      status: Math.random() > 0.7 ? 'healthy' : 'warning',
      componentCount: healthComponents.value.length,
      issueCount: Math.floor(Math.random() * 3),
      duration: 1000 + Math.floor(Math.random() * 1000)
    }
    
    healthHistory.value.unshift(checkResult)
    if (healthHistory.value.length > 10) {
      healthHistory.value = healthHistory.value.slice(0, 10)
    }
    
    updateOverallHealth()
    ElMessage.success('健康检查完成')
  } catch (error) {
    ElMessage.error('健康检查失败')
  } finally {
    loading.value = false
  }
}

const updateOverallHealth = () => {
  const criticalCount = healthComponents.value.filter(c => c.status === 'critical').length
  const warningCount = healthComponents.value.filter(c => c.status === 'warning').length
  
  if (criticalCount > 0) {
    overallHealth.value.status = 'critical'
  } else if (warningCount > 0) {
    overallHealth.value.status = 'warning'
  } else {
    overallHealth.value.status = 'healthy'
  }
  
  healthStats.value = {
    healthy: healthComponents.value.filter(c => c.status === 'healthy').length,
    warning: warningCount,
    critical: criticalCount
  }
}

const showComponentDetail = (component: any) => {
  selectedComponent.value = component
  componentDetailVisible.value = true
}

const checkComponent = async (componentId: string) => {
  try {
    ElMessage.info('正在检查组件...')
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    const component = healthComponents.value.find(c => c.id === componentId)
    if (component) {
      component.lastCheck = new Date().toISOString().slice(0, 19).replace('T', ' ')
      component.responseTime = Math.floor(Math.random() * 100) + 5
      
      // 随机更新状态
      const statuses = ['healthy', 'warning']
      component.status = statuses[Math.floor(Math.random() * statuses.length)]
    }
    
    updateOverallHealth()
    ElMessage.success('组件检查完成')
  } catch (error) {
    ElMessage.error('组件检查失败')
  }
}

const restartComponent = async (componentId: string) => {
  try {
    await ElMessageBox.confirm(
      '确定要重启此组件吗？重启过程中可能影响服务。',
      '重启组件',
      {
        confirmButtonText: '确定重启',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    ElMessage.info('正在重启组件...')
    await new Promise(resolve => setTimeout(resolve, 3000))
    
    const component = healthComponents.value.find(c => c.id === componentId)
    if (component) {
      component.status = 'healthy'
      component.errorCount = 0
      component.responseTime = Math.floor(Math.random() * 20) + 5
      component.lastCheck = new Date().toISOString().slice(0, 19).replace('T', ' ')
    }
    
    updateOverallHealth()
    ElMessage.success('组件重启成功')
  } catch (error) {
    // 用户取消操作
  }
}

const refreshServices = async () => {
  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    services.value.forEach(service => {
      if (service.status === 'running') {
        service.cpu = Math.random() * 30
        service.memory = 50 + Math.random() * 200
      }
    })
    ElMessage.success('服务状态已刷新')
  } catch (error) {
    ElMessage.error('刷新服务失败')
  }
}

const showServiceLogs = (service: any) => {
  selectedService.value = service
  serviceLogsVisible.value = true
}

const restartService = async (serviceId: string) => {
  try {
    await ElMessageBox.confirm(
      '确定要重启此服务吗？',
      '重启服务',
      {
        confirmButtonText: '确定重启',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    ElMessage.info('正在重启服务...')
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    const service = services.value.find(s => s.id === serviceId)
    if (service) {
      service.lastRestart = new Date().toISOString().slice(0, 19).replace('T', ' ')
      service.uptime = '刚刚启动'
      service.requests = 0
    }
    
    ElMessage.success('服务重启成功')
  } catch (error) {
    // 用户取消操作
  }
}

const toggleService = async (service: any) => {
  const action = service.status === 'running' ? '停止' : '启动'
  try {
    await ElMessageBox.confirm(
      `确定要${action}服务 "${service.name}" 吗？`,
      `${action}服务`,
      {
        confirmButtonText: `确定${action}`,
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    ElMessage.info(`正在${action}服务...`)
    await new Promise(resolve => setTimeout(resolve, 1500))
    
    if (service.status === 'running') {
      service.status = 'stopped'
      service.cpu = 0
      service.memory = 0
      service.uptime = '0分钟'
    } else {
      service.status = 'running'
      service.uptime = '刚刚启动'
    }
    
    ElMessage.success(`服务${action}成功`)
  } catch (error) {
    // 用户取消操作
  }
}

const toggleLogPause = () => {
  logPaused.value = !logPaused.value
  if (logPaused.value) {
    if (logUpdateTimer.value) {
      clearInterval(logUpdateTimer.value)
      logUpdateTimer.value = null
    }
    ElMessage.info('日志更新已暂停')
  } else {
    startLogUpdates()
    ElMessage.info('日志更新已继续')
  }
}

const clearLogs = () => {
  logs.value = []
  ElMessage.success('日志已清空')
}

const clearHistory = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要清空所有健康检查历史吗？',
      '清空历史',
      {
        confirmButtonText: '确定清空',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    healthHistory.value = []
    ElMessage.success('历史记录已清空')
  } catch (error) {
    // 用户取消操作
  }
}

const exportReport = async () => {
  try {
    ElMessage.info('正在生成健康报告...')
    await new Promise(resolve => setTimeout(resolve, 2000))
    ElMessage.success('健康报告已导出到本地文件')
  } catch (error) {
    ElMessage.error('导出报告失败')
  }
}

const saveSettings = () => {
  // 应用监控设置
  if (healthCheckTimer.value) {
    clearInterval(healthCheckTimer.value)
  }
  
  healthCheckTimer.value = setInterval(() => {
    if (!logPaused.value) {
      // 自动健康检查逻辑
      console.log('自动健康检查')
    }
  }, monitorSettings.value.checkInterval)
  
  showSettings.value = false
  ElMessage.success('设置已保存')
}

const startLogUpdates = () => {
  if (logUpdateTimer.value) {
    clearInterval(logUpdateTimer.value)
  }
  
  logUpdateTimer.value = setInterval(() => {
    if (!logPaused.value) {
      // 模拟新日志
      const levels = ['info', 'warn', 'error', 'debug']
      const sources = ['Gateway Core', 'Modbus Driver', 'OPC UA Driver', 'MQTT Broker', 'Data Processor']
      const messages = [
        '数据处理完成',
        '连接状态检查',
        '配置文件已重载',
        '客户端连接建立',
        '数据同步完成'
      ]
      
      const newLog = {
        timestamp: new Date(),
        level: levels[Math.floor(Math.random() * levels.length)],
        source: sources[Math.floor(Math.random() * sources.length)],
        message: messages[Math.floor(Math.random() * messages.length)]
      }
      
      logs.value.unshift(newLog)
      if (logs.value.length > 100) {
        logs.value = logs.value.slice(0, 100)
      }
      
      // 自动滚动到顶部
      nextTick(() => {
        if (logContent.value) {
          logContent.value.scrollTop = 0
        }
      })
    }
  }, 3000)
}

const initCharts = () => {
  // 初始化资源使用趋势图表
  if (resourceChart.value) {
    const chart = echarts.init(resourceChart.value)
    const option = {
      title: {
        text: ''
      },
      tooltip: {
        trigger: 'axis'
      },
      legend: {
        data: ['CPU', '内存', '磁盘']
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        containLabel: true
      },
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: ['10:00', '10:30', '11:00', '11:30', '12:00', '12:30', '13:00', '13:30', '14:00', '14:30', '15:00', '15:30']
      },
      yAxis: {
        type: 'value',
        max: 100
      },
      series: [
        {
          name: 'CPU',
          type: 'line',
          stack: 'Total',
          data: [25, 28, 32, 35, 38, 42, 45, 48, 46, 44, 42, 46]
        },
        {
          name: '内存',
          type: 'line',
          stack: 'Total',
          data: [60, 62, 65, 68, 70, 72, 68, 66, 64, 66, 68, 68]
        },
        {
          name: '磁盘',
          type: 'line',
          stack: 'Total',
          data: [20, 20, 21, 21, 22, 22, 23, 23, 23, 23, 24, 23]
        }
      ]
    }
    chart.setOption(option)
  }

  // 初始化网络流量图表
  if (networkChart.value) {
    const chart = echarts.init(networkChart.value)
    const option = {
      title: {
        text: ''
      },
      tooltip: {
        trigger: 'axis'
      },
      legend: {
        data: ['上行', '下行']
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        containLabel: true
      },
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: ['10:00', '10:30', '11:00', '11:30', '12:00', '12:30', '13:00', '13:30', '14:00', '14:30', '15:00', '15:30']
      },
      yAxis: {
        type: 'value'
      },
      series: [
        {
          name: '上行',
          type: 'line',
          data: [120, 132, 101, 134, 90, 230, 210, 182, 191, 234, 290, 330]
        },
        {
          name: '下行',
          type: 'line',
          data: [220, 282, 201, 234, 290, 330, 310, 123, 442, 321, 90, 149]
        }
      ]
    }
    chart.setOption(option)
  }
}

// 工具函数
const getOverallStatusType = (status: string) => {
  const typeMap: { [key: string]: string } = {
    healthy: 'success',
    warning: 'warning',
    critical: 'danger'
  }
  return typeMap[status] || 'info'
}

const getStatusText = (status: string) => {
  const textMap: { [key: string]: string } = {
    healthy: '健康',
    warning: '警告',
    critical: '异常',
    running: '运行中',
    stopped: '已停止'
  }
  return textMap[status] || status
}

const getStatusTagType = (status: string) => {
  const tagTypeMap: { [key: string]: string } = {
    healthy: 'success',
    warning: 'warning',
    critical: 'danger',
    running: 'success',
    stopped: 'danger'
  }
  return tagTypeMap[status] || 'info'
}

const getComponentStatusClass = (status: string) => {
  return `status-${status}`
}

const getComponentIcon = (type: string) => {
  const iconMap: { [key: string]: any } = {
    'Core Service': Server,
    'Protocol Driver': Connection,
    'Message Broker': Link,
    'Data Service': Database,
    'Alert Service': Warning,
    'Database': Database,
    'API Service': Server,
    'Storage Service': HardDisk
  }
  return iconMap[type] || Server
}

const getServiceIcon = (type: string) => {
  const iconMap: { [key: string]: any } = {
    main: Server,
    driver: Connection,
    broker: Link,
    service: Tools
  }
  return iconMap[type] || Server
}

const getProgressColor = (value: number) => {
  if (value > 80) return '#f56c6c'
  if (value > 60) return '#e6a23c'
  return '#67c23a'
}

const getLogLevelClass = (level: string) => {
  return `log-${level}`
}

const getTimelineType = (status: string) => {
  const typeMap: { [key: string]: string } = {
    healthy: 'success',
    warning: 'warning',
    critical: 'danger'
  }
  return typeMap[status] || 'info'
}

const getStatusClass = (status: string) => {
  return `status-${status}`
}

const formatDateTime = (dateStr: string) => {
  return dateStr
}

const formatLogTime = (date: Date) => {
  return date.toLocaleTimeString()
}

// 生命周期
onMounted(() => {
  updateOverallHealth()
  refreshData()
  startLogUpdates()
  
  // 初始化图表
  nextTick(() => {
    initCharts()
  })
})

onUnmounted(() => {
  if (healthCheckTimer.value) {
    clearInterval(healthCheckTimer.value)
  }
  if (logUpdateTimer.value) {
    clearInterval(logUpdateTimer.value)
  }
})
</script>

<style scoped lang="scss">
.health-status {
  padding: 24px;
  background: #f5f7fa;
  min-height: 100vh;

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
    background: white;
    padding: 20px 24px;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);

    .header-left {
      .page-title {
        margin: 0;
        font-size: 24px;
        color: #303133;
        display: flex;
        align-items: center;
        gap: 8px;
      }

      .page-description {
        color: #909399;
        margin-left: 32px;
      }
    }

    .header-actions {
      display: flex;
      gap: 12px;
    }
  }

  .overview-section {
    margin-bottom: 24px;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .overall-status {
      .el-tag {
        font-size: 16px;
        padding: 8px 16px;
      }
    }

    .system-metrics {
      .metric-chart {
        h4 {
          margin: 0 0 16px 0;
          color: #303133;
        }
      }
    }
  }

  .components-section {
    margin-bottom: 24px;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .health-summary {
        display: flex;
        gap: 16px;

        .health-item {
          display: flex;
          align-items: center;
        }
      }
    }

    .component-card {
      height: 200px;
      border-radius: 8px;
      transition: all 0.3s ease;

      &:hover {
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        transform: translateY(-2px);
      }

      &.status-healthy {
        border-left: 4px solid #67c23a;
      }

      &.status-warning {
        border-left: 4px solid #e6a23c;
      }

      &.status-critical {
        border-left: 4px solid #f56c6c;
      }

      .component-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 16px;

        .component-info {
          display: flex;
          align-items: center;
          gap: 12px;

          .component-icon {
            font-size: 24px;
            color: #409eff;
          }

          .component-details {
            h4 {
              margin: 0;
              font-size: 16px;
              color: #303133;
            }

            .component-type {
              font-size: 12px;
              color: #909399;
            }
          }
        }
      }

      .component-metrics {
        margin: 16px 0;

        .metric-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 8px;
          font-size: 14px;

          .metric-label {
            color: #606266;
          }

          .metric-value {
            color: #303133;
            font-weight: 500;
          }

          &.error {
            .metric-value {
              color: #f56c6c;
            }
          }
        }
      }

      .component-actions {
        margin-top: auto;
        display: flex;
        gap: 8px;
      }
    }
  }

  .services-section {
    margin-bottom: 24px;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .service-name {
      display: flex;
      align-items: center;
      gap: 8px;

      .service-icon {
        font-size: 16px;
        color: #409eff;
      }
    }
  }

  .logs-section {
    margin-bottom: 24px;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .log-controls {
        display: flex;
        gap: 12px;
        align-items: center;
      }
    }

    .log-container {
      .log-content {
        height: 300px;
        overflow-y: auto;
        background: #1e1e1e;
        border-radius: 4px;
        padding: 12px;
        font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;

        .log-entry {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 4px 0;
          border-bottom: 1px solid #333;
          font-size: 13px;

          .log-time {
            color: #888;
            min-width: 80px;
          }

          .log-level {
            min-width: 50px;
            font-weight: bold;
          }

          .log-source {
            color: #5dade2;
            min-width: 120px;
          }

          .log-message {
            flex: 1;
            color: #e8e8e8;
          }

          &.log-info {
            .log-level {
              color: #67c23a;
            }
          }

          &.log-warn {
            .log-level {
              color: #e6a23c;
            }
          }

          &.log-error {
            .log-level {
              color: #f56c6c;
            }
          }

          &.log-debug {
            .log-level {
              color: #909399;
            }
          }
        }
      }
    }
  }

  .history-section {
    .history-card {
      margin: 16px 0;

      &.status-healthy {
        border-left: 4px solid #67c23a;
      }

      &.status-warning {
        border-left: 4px solid #e6a23c;
      }

      &.status-critical {
        border-left: 4px solid #f56c6c;
      }

      .history-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;

        .history-title {
          font-weight: 600;
          color: #303133;
        }
      }

      .history-description {
        margin: 8px 0 16px 0;
        color: #606266;
        line-height: 1.5;
      }

      .history-details {
        display: flex;
        gap: 20px;
        font-size: 12px;
        color: #909399;

        span {
          display: flex;
          align-items: center;
        }
      }
    }
  }

  .settings-form {
    .el-form-item {
      margin-bottom: 24px;
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .health-status {
    padding: 12px;

    .page-header {
      flex-direction: column;
      gap: 16px;
      text-align: center;

      .header-actions {
        width: 100%;
        justify-content: center;
        flex-wrap: wrap;
      }
    }

    .component-card {
      margin-bottom: 16px;
    }

    .log-content {
      .log-entry {
        flex-direction: column;
        align-items: flex-start;
        gap: 4px;
        padding: 8px 0;

        .log-time,
        .log-level,
        .log-source {
          min-width: auto;
        }
      }
    }
  }
}
</style>