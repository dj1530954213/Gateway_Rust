<template>
  <div class="system-metrics">
    <!-- 页面标题和工具栏 -->
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">
          <el-icon><TrendCharts /></el-icon>
          性能指标仪表板
        </h1>
        <span class="page-description">监控系统性能指标和趋势分析</span>
      </div>
      <div class="header-actions">
        <el-button-group>
          <el-button
            :type="timeRange === '1h' ? 'primary' : ''"
            @click="setTimeRange('1h')"
          >
            1小时
          </el-button>
          <el-button
            :type="timeRange === '6h' ? 'primary' : ''"
            @click="setTimeRange('6h')"
          >
            6小时
          </el-button>
          <el-button
            :type="timeRange === '24h' ? 'primary' : ''"
            @click="setTimeRange('24h')"
          >
            24小时
          </el-button>
          <el-button
            :type="timeRange === '7d' ? 'primary' : ''"
            @click="setTimeRange('7d')"
          >
            7天
          </el-button>
        </el-button-group>
        <el-button type="primary" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          刷新数据
        </el-button>
        <el-button @click="exportMetrics">
          <el-icon><Download /></el-icon>
          导出报告
        </el-button>
        <el-button @click="showSettings = true">
          <el-icon><Setting /></el-icon>
          仪表板设置
        </el-button>
      </div>
    </div>

    <!-- 核心指标概览 -->
    <div class="overview-section">
      <el-row :gutter="20">
        <el-col :span="6">
          <MetricCard
            title="CPU 使用率"
            :value="metrics.cpu.current"
            :max="100"
            suffix="%"
            :trend="metrics.cpu.trend"
            :status="getMetricStatus(metrics.cpu.current, 80, 90)"
            :chart-data="metrics.cpu.history"
            color="#409EFF"
            icon="Cpu"
          />
        </el-col>
        <el-col :span="6">
          <MetricCard
            title="内存使用率"
            :value="metrics.memory.current"
            :max="100"
            suffix="%"
            :trend="metrics.memory.trend"
            :status="getMetricStatus(metrics.memory.current, 85, 95)"
            :chart-data="metrics.memory.history"
            color="#67C23A"
            icon="Memory"
          />
        </el-col>
        <el-col :span="6">
          <MetricCard
            title="磁盘使用率"
            :value="metrics.disk.current"
            :max="100"
            suffix="%"
            :trend="metrics.disk.trend"
            :status="getMetricStatus(metrics.disk.current, 80, 90)"
            :chart-data="metrics.disk.history"
            color="#E6A23C"
            icon="Folder"
          />
        </el-col>
        <el-col :span="6">
          <MetricCard
            title="网络吞吐量"
            :value="metrics.network.current"
            suffix="MB/s"
            :trend="metrics.network.trend"
            :status="getNetworkStatus(metrics.network.current)"
            :chart-data="metrics.network.history"
            color="#F56C6C"
            icon="Connection"
          />
        </el-col>
      </el-row>
    </div>

    <!-- 详细性能图表 -->
    <div class="charts-section">
      <el-row :gutter="20">
        <el-col :span="12">
          <el-card shadow="never" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>CPU 性能分析</span>
                <div class="chart-controls">
                  <el-checkbox v-model="chartOptions.cpu.showAverage">显示平均值</el-checkbox>
                  <el-checkbox v-model="chartOptions.cpu.showPeak">显示峰值</el-checkbox>
                </div>
              </div>
            </template>
            <div ref="cpuChart" class="chart-container"></div>
          </el-card>
        </el-col>
        <el-col :span="12">
          <el-card shadow="never" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>内存使用详情</span>
                <div class="chart-controls">
                  <el-checkbox v-model="chartOptions.memory.showSwap">显示交换内存</el-checkbox>
                  <el-checkbox v-model="chartOptions.memory.showBuffer">显示缓存</el-checkbox>
                </div>
              </div>
            </template>
            <div ref="memoryChart" class="chart-container"></div>
          </el-card>
        </el-col>
      </el-row>

      <el-row :gutter="20" style="margin-top: 20px;">
        <el-col :span="12">
          <el-card shadow="never" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>磁盘 I/O 监控</span>
                <div class="chart-controls">
                  <el-select v-model="chartOptions.disk.selectedDisk" size="small" style="width: 120px;">
                    <el-option label="系统盘" value="system" />
                    <el-option label="数据盘" value="data" />
                    <el-option label="全部" value="all" />
                  </el-select>
                </div>
              </div>
            </template>
            <div ref="diskChart" class="chart-container"></div>
          </el-card>
        </el-col>
        <el-col :span="12">
          <el-card shadow="never" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>网络流量监控</span>
                <div class="chart-controls">
                  <el-checkbox v-model="chartOptions.network.showInbound">入站流量</el-checkbox>
                  <el-checkbox v-model="chartOptions.network.showOutbound">出站流量</el-checkbox>
                </div>
              </div>
            </template>
            <div ref="networkChart" class="chart-container"></div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 进程监控 -->
    <div class="processes-section">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <span>进程监控</span>
            <div class="header-actions">
              <el-input
                v-model="processFilter"
                placeholder="筛选进程..."
                size="small"
                style="width: 200px;"
                clearable
              >
                <template #prefix>
                  <el-icon><Search /></el-icon>
                </template>
              </el-input>
              <el-button size="small" @click="refreshProcesses">
                <el-icon><Refresh /></el-icon>
                刷新进程
              </el-button>
            </div>
          </div>
        </template>

        <el-table :data="filteredProcesses" style="width: 100%" @sort-change="handleSortChange">
          <el-table-column prop="pid" label="PID" width="80" sortable="custom" />
          <el-table-column prop="name" label="进程名" width="200" sortable="custom">
            <template #default="{ row }">
              <div class="process-name">
                <el-icon class="process-icon">
                  <component :is="getProcessIcon(row.type)" />
                </el-icon>
                <span>{{ row.name }}</span>
              </div>
            </template>
          </el-table-column>
          <el-table-column prop="cpu" label="CPU使用" width="120" sortable="custom">
            <template #default="{ row }">
              <div class="metric-progress">
                <el-progress
                  :percentage="row.cpu"
                  :stroke-width="8"
                  :color="getProgressColor(row.cpu, 50, 80)"
                  :show-text="false"
                />
                <span class="metric-text">{{ row.cpu }}%</span>
              </div>
            </template>
          </el-table-column>
          <el-table-column prop="memory" label="内存使用" width="120" sortable="custom">
            <template #default="{ row }">
              <div class="metric-progress">
                <el-progress
                  :percentage="(row.memory / 1024) * 10"
                  :stroke-width="8"
                  :color="getProgressColor((row.memory / 1024) * 10, 50, 80)"
                  :show-text="false"
                />
                <span class="metric-text">{{ formatMemory(row.memory) }}</span>
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
          <el-table-column prop="uptime" label="运行时间" width="120" />
          <el-table-column prop="threads" label="线程数" width="100" />
          <el-table-column prop="priority" label="优先级" width="100" />
          <el-table-column label="操作" width="120" fixed="right">
            <template #default="{ row }">
              <el-button size="small" @click="showProcessDetail(row)">
                详情
              </el-button>
              <el-button
                size="small"
                type="danger"
                :disabled="row.critical"
                @click="terminateProcess(row.pid)"
              >
                终止
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </div>

    <!-- 系统资源详情 -->
    <div class="resources-section">
      <el-row :gutter="20">
        <el-col :span="12">
          <el-card shadow="never">
            <template #header>
              <span>系统信息</span>
            </template>
            <div class="system-info">
              <div class="info-group">
                <h4>硬件信息</h4>
                <div class="info-item">
                  <span class="info-label">处理器:</span>
                  <span class="info-value">{{ systemInfo.processor }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">核心数:</span>
                  <span class="info-value">{{ systemInfo.cores }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">总内存:</span>
                  <span class="info-value">{{ systemInfo.totalMemory }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">磁盘容量:</span>
                  <span class="info-value">{{ systemInfo.totalDisk }}</span>
                </div>
              </div>
              
              <div class="info-group">
                <h4>系统版本</h4>
                <div class="info-item">
                  <span class="info-label">操作系统:</span>
                  <span class="info-value">{{ systemInfo.os }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">内核版本:</span>
                  <span class="info-value">{{ systemInfo.kernel }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">运行时间:</span>
                  <span class="info-value">{{ systemInfo.uptime }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">负载平均:</span>
                  <span class="info-value">{{ systemInfo.loadAverage }}</span>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="12">
          <el-card shadow="never">
            <template #header>
              <span>告警阈值配置</span>
            </template>
            <div class="threshold-config">
              <el-form :model="thresholds" label-width="100px">
                <el-form-item label="CPU 警告">
                  <el-input-number
                    v-model="thresholds.cpu.warning"
                    :min="0"
                    :max="100"
                    :precision="1"
                  />
                  <span style="margin-left: 8px;">%</span>
                </el-form-item>
                <el-form-item label="CPU 严重">
                  <el-input-number
                    v-model="thresholds.cpu.critical"
                    :min="0"
                    :max="100"
                    :precision="1"
                  />
                  <span style="margin-left: 8px;">%</span>
                </el-form-item>
                <el-form-item label="内存 警告">
                  <el-input-number
                    v-model="thresholds.memory.warning"
                    :min="0"
                    :max="100"
                    :precision="1"
                  />
                  <span style="margin-left: 8px;">%</span>
                </el-form-item>
                <el-form-item label="内存 严重">
                  <el-input-number
                    v-model="thresholds.memory.critical"
                    :min="0"
                    :max="100"
                    :precision="1"
                  />
                  <span style="margin-left: 8px;">%</span>
                </el-form-item>
                <el-form-item label="磁盘 警告">
                  <el-input-number
                    v-model="thresholds.disk.warning"
                    :min="0"
                    :max="100"
                    :precision="1"
                  />
                  <span style="margin-left: 8px;">%</span>
                </el-form-item>
                <el-form-item label="磁盘 严重">
                  <el-input-number
                    v-model="thresholds.disk.critical"
                    :min="0"
                    :max="100"
                    :precision="1"
                  />
                  <span style="margin-left: 8px;">%</span>
                </el-form-item>
                <el-form-item>
                  <el-button type="primary" @click="saveThresholds">
                    保存配置
                  </el-button>
                  <el-button @click="resetThresholds">
                    重置默认
                  </el-button>
                </el-form-item>
              </el-form>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 性能趋势分析 -->
    <div class="trends-section">
      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <span>性能趋势分析</span>
            <div class="trend-controls">
              <el-select v-model="trendMetric" size="small" style="width: 120px;">
                <el-option label="CPU" value="cpu" />
                <el-option label="内存" value="memory" />
                <el-option label="磁盘" value="disk" />
                <el-option label="网络" value="network" />
              </el-select>
              <el-select v-model="trendPeriod" size="small" style="width: 120px;">
                <el-option label="小时" value="hour" />
                <el-option label="天" value="day" />
                <el-option label="周" value="week" />
                <el-option label="月" value="month" />
              </el-select>
            </div>
          </div>
        </template>
        <div ref="trendChart" class="trend-chart"></div>
        
        <div class="trend-analysis">
          <el-row :gutter="20">
            <el-col :span="8">
              <div class="analysis-item">
                <h4>平均值分析</h4>
                <div class="analysis-content">
                  <div class="metric-item">
                    <span>当前周期:</span>
                    <span class="metric-value">{{ trendAnalysis.currentAverage }}%</span>
                  </div>
                  <div class="metric-item">
                    <span>上个周期:</span>
                    <span class="metric-value">{{ trendAnalysis.previousAverage }}%</span>
                  </div>
                  <div class="metric-item">
                    <span>变化幅度:</span>
                    <span :class="getTrendClass(trendAnalysis.averageChange)">
                      {{ trendAnalysis.averageChange }}%
                    </span>
                  </div>
                </div>
              </div>
            </el-col>
            <el-col :span="8">
              <div class="analysis-item">
                <h4>峰值分析</h4>
                <div class="analysis-content">
                  <div class="metric-item">
                    <span>最高峰值:</span>
                    <span class="metric-value">{{ trendAnalysis.maxPeak }}%</span>
                  </div>
                  <div class="metric-item">
                    <span>最低值:</span>
                    <span class="metric-value">{{ trendAnalysis.minValue }}%</span>
                  </div>
                  <div class="metric-item">
                    <span>波动范围:</span>
                    <span class="metric-value">{{ trendAnalysis.volatility }}%</span>
                  </div>
                </div>
              </div>
            </el-col>
            <el-col :span="8">
              <div class="analysis-item">
                <h4>趋势预测</h4>
                <div class="analysis-content">
                  <div class="metric-item">
                    <span>趋势方向:</span>
                    <span :class="getTrendDirectionClass(trendAnalysis.direction)">
                      {{ getTrendDirectionText(trendAnalysis.direction) }}
                    </span>
                  </div>
                  <div class="metric-item">
                    <span>预测值:</span>
                    <span class="metric-value">{{ trendAnalysis.prediction }}%</span>
                  </div>
                  <div class="metric-item">
                    <span>置信度:</span>
                    <span class="metric-value">{{ trendAnalysis.confidence }}%</span>
                  </div>
                </div>
              </div>
            </el-col>
          </el-row>
        </div>
      </el-card>
    </div>

    <!-- 进程详情对话框 -->
    <el-dialog
      v-model="processDetailVisible"
      title="进程详情"
      width="700px"
      :close-on-click-modal="false"
    >
      <ProcessDetail v-if="selectedProcess" :process="selectedProcess" />
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="processDetailVisible = false">关闭</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 仪表板设置对话框 -->
    <el-dialog
      v-model="showSettings"
      title="仪表板设置"
      width="600px"
    >
      <div class="settings-form">
        <el-form :model="dashboardSettings" label-width="120px">
          <el-form-item label="刷新间隔">
            <el-select v-model="dashboardSettings.refreshInterval">
              <el-option label="5秒" :value="5000" />
              <el-option label="10秒" :value="10000" />
              <el-option label="30秒" :value="30000" />
              <el-option label="1分钟" :value="60000" />
              <el-option label="5分钟" :value="300000" />
            </el-select>
          </el-form-item>
          <el-form-item label="数据点数">
            <el-input-number
              v-model="dashboardSettings.dataPoints"
              :min="50"
              :max="500"
            />
          </el-form-item>
          <el-form-item label="显示模块">
            <el-checkbox-group v-model="dashboardSettings.visibleModules">
              <el-checkbox label="overview">性能概览</el-checkbox>
              <el-checkbox label="charts">详细图表</el-checkbox>
              <el-checkbox label="processes">进程监控</el-checkbox>
              <el-checkbox label="resources">系统资源</el-checkbox>
              <el-checkbox label="trends">趋势分析</el-checkbox>
            </el-checkbox-group>
          </el-form-item>
          <el-form-item label="自动缩放">
            <el-switch v-model="dashboardSettings.autoScale" />
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
  TrendCharts, Refresh, Download, Setting, Search, Cpu, Memory,
  Folder, Connection, Server, Tools, Document
} from '@element-plus/icons-vue'
import MetricCard from '../../components/common/MetricCard.vue'
import ProcessDetail from '../../components/business/ProcessDetail.vue'
import * as echarts from 'echarts'

// 响应式数据
const loading = ref(false)
const timeRange = ref('1h')
const processFilter = ref('')
const showSettings = ref(false)
const processDetailVisible = ref(false)
const selectedProcess = ref<any>(null)
const trendMetric = ref('cpu')
const trendPeriod = ref('hour')
const refreshTimer = ref<NodeJS.Timeout | null>(null)

// 图表实例引用
const cpuChart = ref<HTMLElement>()
const memoryChart = ref<HTMLElement>()
const diskChart = ref<HTMLElement>()
const networkChart = ref<HTMLElement>()
const trendChart = ref<HTMLElement>()

// 性能指标数据
const metrics = ref({
  cpu: {
    current: 45.6,
    trend: -2.3,
    history: [42, 45, 48, 46, 44, 47, 46]
  },
  memory: {
    current: 67.8,
    trend: 5.2,
    history: [65, 66, 68, 67, 69, 68, 68]
  },
  disk: {
    current: 23.4,
    trend: 0.8,
    history: [22, 23, 23, 24, 23, 23, 23]
  },
  network: {
    current: 125.6,
    trend: 12.4,
    history: [120, 118, 125, 130, 128, 126, 126]
  }
})

// 图表配置选项
const chartOptions = ref({
  cpu: {
    showAverage: true,
    showPeak: true
  },
  memory: {
    showSwap: true,
    showBuffer: true
  },
  disk: {
    selectedDisk: 'all'
  },
  network: {
    showInbound: true,
    showOutbound: true
  }
})

// 进程监控数据
const processes = ref([
  {
    pid: 1234,
    name: 'gateway-core',
    type: 'service',
    cpu: 12.5,
    memory: 256,
    status: 'running',
    uptime: '15d 8h',
    threads: 8,
    priority: 'normal',
    critical: true
  },
  {
    pid: 1235,
    name: 'modbus-driver',
    type: 'driver',
    cpu: 8.3,
    memory: 128,
    status: 'running',
    uptime: '15d 8h',
    threads: 4,
    priority: 'normal',
    critical: true
  },
  {
    pid: 1236,
    name: 'opcua-driver',
    type: 'driver',
    cpu: 25.7,
    memory: 512,
    status: 'running',
    uptime: '2h 15m',
    threads: 12,
    priority: 'high',
    critical: true
  },
  {
    pid: 1237,
    name: 'mqtt-broker',
    type: 'broker',
    cpu: 5.2,
    memory: 96,
    status: 'running',
    uptime: '15d 8h',
    threads: 6,
    priority: 'normal',
    critical: true
  },
  {
    pid: 1238,
    name: 'data-processor',
    type: 'service',
    cpu: 18.9,
    memory: 384,
    status: 'running',
    uptime: '15d 8h',
    threads: 16,
    priority: 'high',
    critical: true
  },
  {
    pid: 1239,
    name: 'web-server',
    type: 'web',
    cpu: 3.4,
    memory: 64,
    status: 'running',
    uptime: '15d 8h',
    threads: 2,
    priority: 'low',
    critical: false
  },
  {
    pid: 1240,
    name: 'log-collector',
    type: 'service',
    cpu: 1.8,
    memory: 32,
    status: 'running',
    uptime: '15d 8h',
    threads: 1,
    priority: 'low',
    critical: false
  }
])

// 系统信息
const systemInfo = ref({
  processor: 'Intel Core i7-9700K @ 3.60GHz',
  cores: '8核16线程',
  totalMemory: '16 GB',
  totalDisk: '1 TB SSD',
  os: 'Ubuntu 20.04.3 LTS',
  kernel: '5.4.0-91-generic',
  uptime: '15天 8小时 32分钟',
  loadAverage: '0.45, 0.52, 0.48'
})

// 告警阈值配置
const thresholds = ref({
  cpu: {
    warning: 70.0,
    critical: 90.0
  },
  memory: {
    warning: 80.0,
    critical: 95.0
  },
  disk: {
    warning: 85.0,
    critical: 95.0
  }
})

// 趋势分析数据
const trendAnalysis = ref({
  currentAverage: 45.6,
  previousAverage: 48.2,
  averageChange: -5.4,
  maxPeak: 89.2,
  minValue: 12.8,
  volatility: 76.4,
  direction: 'down',
  prediction: 42.3,
  confidence: 85.4
})

// 仪表板设置
const dashboardSettings = ref({
  refreshInterval: 10000,
  dataPoints: 100,
  visibleModules: ['overview', 'charts', 'processes', 'resources', 'trends'],
  autoScale: true
})

// 计算属性
const filteredProcesses = computed(() => {
  if (!processFilter.value) {
    return processes.value
  }
  const filter = processFilter.value.toLowerCase()
  return processes.value.filter(process =>
    process.name.toLowerCase().includes(filter) ||
    process.type.toLowerCase().includes(filter)
  )
})

// 方法
const setTimeRange = (range: string) => {
  timeRange.value = range
  // 重新加载数据
  refreshData()
}

const refreshData = async () => {
  loading.value = true
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // 更新指标数据
    metrics.value = {
      cpu: {
        current: 30 + Math.random() * 40,
        trend: (Math.random() - 0.5) * 10,
        history: Array.from({ length: 7 }, () => Math.floor(30 + Math.random() * 40))
      },
      memory: {
        current: 50 + Math.random() * 30,
        trend: (Math.random() - 0.5) * 8,
        history: Array.from({ length: 7 }, () => Math.floor(50 + Math.random() * 30))
      },
      disk: {
        current: 20 + Math.random() * 20,
        trend: (Math.random() - 0.5) * 5,
        history: Array.from({ length: 7 }, () => Math.floor(20 + Math.random() * 20))
      },
      network: {
        current: 100 + Math.random() * 50,
        trend: (Math.random() - 0.5) * 20,
        history: Array.from({ length: 7 }, () => Math.floor(100 + Math.random() * 50))
      }
    }
    
    // 更新进程数据
    processes.value.forEach(process => {
      if (process.status === 'running') {
        process.cpu = Math.random() * 30
        process.memory = 50 + Math.random() * 200
      }
    })
    
    // 更新趋势分析
    updateTrendAnalysis()
    
    // 更新图表
    updateCharts()
    
    ElMessage.success('数据刷新成功')
  } catch (error) {
    ElMessage.error('数据刷新失败')
  } finally {
    loading.value = false
  }
}

const updateTrendAnalysis = () => {
  const currentMetric = metrics.value[trendMetric.value as keyof typeof metrics.value]
  trendAnalysis.value = {
    currentAverage: currentMetric.current,
    previousAverage: currentMetric.current + (Math.random() - 0.5) * 20,
    averageChange: (Math.random() - 0.5) * 15,
    maxPeak: Math.max(...currentMetric.history) + Math.random() * 20,
    minValue: Math.min(...currentMetric.history) - Math.random() * 10,
    volatility: Math.random() * 30 + 20,
    direction: Math.random() > 0.5 ? 'up' : 'down',
    prediction: currentMetric.current + (Math.random() - 0.5) * 10,
    confidence: 70 + Math.random() * 25
  }
}

const refreshProcesses = async () => {
  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    // 模拟进程数据更新
    ElMessage.success('进程列表已刷新')
  } catch (error) {
    ElMessage.error('刷新进程失败')
  }
}

const handleSortChange = ({ prop, order }: any) => {
  // 实现排序逻辑
  if (!prop || !order) return
  
  processes.value.sort((a: any, b: any) => {
    const aVal = a[prop]
    const bVal = b[prop]
    
    if (typeof aVal === 'number' && typeof bVal === 'number') {
      return order === 'ascending' ? aVal - bVal : bVal - aVal
    } else {
      const result = String(aVal).localeCompare(String(bVal))
      return order === 'ascending' ? result : -result
    }
  })
}

const showProcessDetail = (process: any) => {
  selectedProcess.value = process
  processDetailVisible.value = true
}

const terminateProcess = async (pid: number) => {
  try {
    await ElMessageBox.confirm(
      `确定要终止进程 PID ${pid} 吗？此操作不可撤销。`,
      '终止进程',
      {
        confirmButtonText: '确定终止',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    // 模拟终止进程
    const processIndex = processes.value.findIndex(p => p.pid === pid)
    if (processIndex > -1) {
      processes.value[processIndex].status = 'terminated'
      processes.value[processIndex].cpu = 0
      processes.value[processIndex].memory = 0
    }
    
    ElMessage.success('进程已终止')
  } catch (error) {
    // 用户取消操作
  }
}

const saveThresholds = () => {
  ElMessage.success('阈值配置已保存')
}

const resetThresholds = () => {
  thresholds.value = {
    cpu: { warning: 70.0, critical: 90.0 },
    memory: { warning: 80.0, critical: 95.0 },
    disk: { warning: 85.0, critical: 95.0 }
  }
  ElMessage.success('阈值已重置为默认值')
}

const exportMetrics = async () => {
  try {
    ElMessage.info('正在生成性能报告...')
    await new Promise(resolve => setTimeout(resolve, 2000))
    ElMessage.success('性能报告已导出到本地文件')
  } catch (error) {
    ElMessage.error('导出报告失败')
  }
}

const saveSettings = () => {
  // 应用设置
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
  }
  
  refreshTimer.value = setInterval(() => {
    refreshData()
  }, dashboardSettings.value.refreshInterval)
  
  showSettings.value = false
  ElMessage.success('设置已保存')
}

const initCharts = () => {
  // 初始化CPU图表
  if (cpuChart.value) {
    const chart = echarts.init(cpuChart.value)
    const option = {
      tooltip: { trigger: 'axis' },
      legend: { data: ['CPU使用率', '平均值', '峰值'] },
      grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
      xAxis: { type: 'category', data: ['10:00', '10:30', '11:00', '11:30', '12:00', '12:30', '13:00'] },
      yAxis: { type: 'value', max: 100 },
      series: [
        {
          name: 'CPU使用率',
          type: 'line',
          data: [45, 48, 46, 52, 49, 47, 46],
          smooth: true,
          areaStyle: { opacity: 0.3 }
        },
        {
          name: '平均值',
          type: 'line',
          data: [47, 47, 47, 47, 47, 47, 47],
          lineStyle: { type: 'dashed' }
        },
        {
          name: '峰值',
          type: 'line',
          data: [80, 82, 78, 85, 83, 81, 80],
          lineStyle: { type: 'dotted' }
        }
      ]
    }
    chart.setOption(option)
  }
  
  // 初始化其他图表...
  // 这里省略其他图表的初始化代码，实际项目中会完整实现
}

const updateCharts = () => {
  // 更新所有图表数据
  // 实际项目中会根据新数据更新图表
}

// 工具函数
const getMetricStatus = (value: number, warning: number, critical: number) => {
  if (value >= critical) return 'critical'
  if (value >= warning) return 'warning'
  return 'healthy'
}

const getNetworkStatus = (value: number) => {
  if (value > 200) return 'critical'
  if (value > 150) return 'warning'
  return 'healthy'
}

const getProgressColor = (value: number, warning: number, critical: number) => {
  if (value >= critical) return '#f56c6c'
  if (value >= warning) return '#e6a23c'
  return '#67c23a'
}

const formatMemory = (mb: number) => {
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(1)} GB`
  }
  return `${mb} MB`
}

const getProcessIcon = (type: string) => {
  const iconMap: { [key: string]: any } = {
    service: Server,
    driver: Connection,
    broker: Connection,
    web: Document
  }
  return iconMap[type] || Tools
}

const getStatusTagType = (status: string) => {
  const tagTypeMap: { [key: string]: string } = {
    running: 'success',
    terminated: 'danger',
    sleeping: 'warning'
  }
  return tagTypeMap[status] || 'info'
}

const getStatusText = (status: string) => {
  const textMap: { [key: string]: string } = {
    running: '运行中',
    terminated: '已终止',
    sleeping: '休眠'
  }
  return textMap[status] || status
}

const getTrendClass = (value: number) => {
  if (value > 0) return 'trend-up'
  if (value < 0) return 'trend-down'
  return 'trend-stable'
}

const getTrendDirectionClass = (direction: string) => {
  return `trend-${direction}`
}

const getTrendDirectionText = (direction: string) => {
  const textMap: { [key: string]: string } = {
    up: '上升',
    down: '下降',
    stable: '稳定'
  }
  return textMap[direction] || direction
}

// 生命周期
onMounted(() => {
  refreshData()
  
  // 初始化图表
  nextTick(() => {
    initCharts()
  })
  
  // 设置自动刷新
  refreshTimer.value = setInterval(() => {
    refreshData()
  }, dashboardSettings.value.refreshInterval)
})

onUnmounted(() => {
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
  }
})
</script>

<style scoped lang="scss">
.system-metrics {
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
      align-items: center;
    }
  }

  .overview-section {
    margin-bottom: 24px;
  }

  .charts-section {
    margin-bottom: 24px;

    .chart-card {
      height: 400px;

      .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;

        .chart-controls {
          display: flex;
          gap: 12px;
          align-items: center;
        }
      }

      .chart-container {
        height: 300px;
        width: 100%;
      }
    }
  }

  .processes-section {
    margin-bottom: 24px;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .header-actions {
        display: flex;
        gap: 12px;
        align-items: center;
      }
    }

    .process-name {
      display: flex;
      align-items: center;
      gap: 8px;

      .process-icon {
        font-size: 16px;
        color: #409eff;
      }
    }

    .metric-progress {
      display: flex;
      align-items: center;
      gap: 8px;

      .metric-text {
        min-width: 60px;
        text-align: right;
        font-size: 12px;
      }
    }
  }

  .resources-section {
    margin-bottom: 24px;

    .system-info {
      .info-group {
        margin-bottom: 24px;

        h4 {
          margin: 0 0 16px 0;
          color: #303133;
          border-bottom: 1px solid #ebeef5;
          padding-bottom: 8px;
        }

        .info-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 0;
          border-bottom: 1px solid #f5f7fa;

          .info-label {
            color: #606266;
          }

          .info-value {
            color: #303133;
            font-weight: 500;
          }
        }
      }
    }

    .threshold-config {
      .el-form-item {
        margin-bottom: 20px;
      }
    }
  }

  .trends-section {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .trend-controls {
        display: flex;
        gap: 12px;
        align-items: center;
      }
    }

    .trend-chart {
      height: 300px;
      width: 100%;
      margin-bottom: 24px;
    }

    .trend-analysis {
      .analysis-item {
        padding: 16px;
        background: #fafbfc;
        border-radius: 6px;
        border: 1px solid #ebeef5;

        h4 {
          margin: 0 0 16px 0;
          color: #303133;
          font-size: 14px;
        }

        .analysis-content {
          .metric-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 12px;
            font-size: 13px;

            span:first-child {
              color: #606266;
            }

            .metric-value {
              color: #303133;
              font-weight: 500;
            }

            .trend-up {
              color: #f56c6c;
            }

            .trend-down {
              color: #67c23a;
            }

            .trend-stable {
              color: #909399;
            }
          }
        }
      }
    }
  }

  .settings-form {
    .el-form-item {
      margin-bottom: 24px;
    }

    .el-checkbox-group {
      display: flex;
      flex-direction: column;
      gap: 8px;
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .system-metrics {
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

    .charts-section {
      .chart-card {
        height: 350px;
        margin-bottom: 16px;

        .card-header {
          flex-direction: column;
          gap: 12px;
          align-items: flex-start;
        }

        .chart-container {
          height: 250px;
        }
      }
    }

    .trend-analysis {
      .el-row .el-col {
        margin-bottom: 16px;
      }
    }
  }
}
</style>