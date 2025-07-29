<template>
  <el-dialog
    v-model="dialogVisible"
    title="报警规则测试"
    width="1000px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @close="handleClose"
  >
    <div class="rule-test-dialog">
      <!-- 规则信息概览 -->
      <el-card class="rule-overview" shadow="never">
        <template #header>
          <span class="card-title">规则信息</span>
        </template>
        
        <div class="rule-info">
          <div class="info-grid">
            <div class="info-item">
              <label>规则名称：</label>
              <span>{{ ruleData?.name || '未命名规则' }}</span>
            </div>
            <div class="info-item">
              <label>报警级别：</label>
              <el-tag :type="getSeverityType(ruleData?.severity)">
                {{ getSeverityLabel(ruleData?.severity) }}
              </el-tag>
            </div>
            <div class="info-item">
              <label>监控范围：</label>
              <span>{{ getScopeDescription() }}</span>
            </div>
            <div class="info-item">
              <label>条件数量：</label>
              <span>{{ ruleData?.conditions?.length || 0 }} 个</span>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 测试配置 -->
      <el-card class="test-config" shadow="never">
        <template #header>
          <div class="card-header">
            <span class="card-title">测试配置</span>
            <el-button type="primary" @click="runTest" :loading="testing">
              <el-icon><VideoPlay /></el-icon>
              {{ testing ? '测试中...' : '运行测试' }}
            </el-button>
          </div>
        </template>

        <div class="config-form">
          <div class="config-row">
            <div class="config-item">
              <label class="config-label">测试模式</label>
              <el-radio-group v-model="testConfig.mode">
                <el-radio label="realtime">实时数据测试</el-radio>
                <el-radio label="historical">历史数据回放</el-radio>
                <el-radio label="simulation">模拟数据测试</el-radio>
              </el-radio-group>
            </div>
          </div>

          <!-- 实时数据测试配置 -->
          <div v-if="testConfig.mode === 'realtime'" class="mode-config">
            <div class="config-row">
              <div class="config-item">
                <label class="config-label">测试时长</label>
                <div class="duration-input">
                  <el-input-number
                    v-model="testConfig.realtime.duration"
                    :min="1"
                    :max="60"
                    placeholder="分钟"
                  />
                  <span>分钟</span>
                </div>
              </div>
              
              <div class="config-item">
                <label class="config-label">数据采样间隔</label>
                <el-select v-model="testConfig.realtime.interval" placeholder="选择间隔">
                  <el-option label="1秒" value="1s" />
                  <el-option label="5秒" value="5s" />
                  <el-option label="10秒" value="10s" />
                  <el-option label="30秒" value="30s" />
                  <el-option label="1分钟" value="1m" />
                </el-select>
              </div>
            </div>
          </div>

          <!-- 历史数据回放配置 -->
          <div v-else-if="testConfig.mode === 'historical'" class="mode-config">
            <div class="config-row">
              <div class="config-item">
                <label class="config-label">时间范围</label>
                <el-date-picker
                  v-model="testConfig.historical.dateRange"
                  type="datetimerange"
                  range-separator="至"
                  start-placeholder="开始时间"
                  end-placeholder="结束时间"
                  format="YYYY-MM-DD HH:mm:ss"
                  value-format="YYYY-MM-DD HH:mm:ss"
                />
              </div>
              
              <div class="config-item">
                <label class="config-label">回放速度</label>
                <el-select v-model="testConfig.historical.speed" placeholder="选择速度">
                  <el-option label="1x (正常速度)" value="1x" />
                  <el-option label="2x (2倍速)" value="2x" />
                  <el-option label="5x (5倍速)" value="5x" />
                  <el-option label="10x (10倍速)" value="10x" />
                  <el-option label="最快" value="max" />
                </el-select>
              </div>
            </div>
          </div>

          <!-- 模拟数据测试配置 -->
          <div v-else-if="testConfig.mode === 'simulation'" class="mode-config">
            <div class="config-row">
              <div class="config-item">
                <label class="config-label">数据场景</label>
                <el-select v-model="testConfig.simulation.scenario" placeholder="选择场景">
                  <el-option label="正常运行" value="normal" />
                  <el-option label="逐步升高" value="rising" />
                  <el-option label="逐步降低" value="falling" />
                  <el-option label="周期性波动" value="periodic" />
                  <el-option label="随机变化" value="random" />
                  <el-option label="异常突增" value="spike" />
                  <el-option label="自定义" value="custom" />
                </el-select>
              </div>
              
              <div class="config-item">
                <label class="config-label">数据点数</label>
                <el-input-number
                  v-model="testConfig.simulation.dataPoints"
                  :min="10"
                  :max="1000"
                  placeholder="点数"
                />
              </div>
            </div>

            <!-- 自定义场景配置 -->
            <div v-if="testConfig.simulation.scenario === 'custom'" class="custom-scenario">
              <label class="config-label">自定义数据序列</label>
              <el-input
                v-model="testConfig.simulation.customData"
                type="textarea"
                :rows="3"
                placeholder="输入逗号分隔的数值序列，例如：10,15,20,25,30,35,40"
              />
            </div>
          </div>

          <!-- 高级选项 -->
          <div class="advanced-options">
            <el-checkbox v-model="testConfig.includeNotifications">
              测试通知发送
            </el-checkbox>
            <el-checkbox v-model="testConfig.recordResults">
              记录测试结果
            </el-checkbox>
            <el-checkbox v-model="testConfig.detailedLog">
              详细日志记录
            </el-checkbox>
          </div>
        </div>
      </el-card>

      <!-- 测试进度 -->
      <el-card v-if="testProgress.show" class="test-progress" shadow="never">
        <template #header>
          <div class="card-header">
            <span class="card-title">测试进度</span>
            <div class="progress-actions">
              <el-button 
                v-if="testing" 
                type="danger" 
                size="small" 
                @click="stopTest"
              >
                停止测试
              </el-button>
            </div>
          </div>
        </template>

        <div class="progress-content">
          <div class="progress-info">
            <div class="progress-item">
              <span class="progress-label">当前状态：</span>
              <span class="progress-value">{{ testProgress.status }}</span>
            </div>
            <div class="progress-item">
              <span class="progress-label">已处理：</span>
              <span class="progress-value">
                {{ testProgress.processed }} / {{ testProgress.total }}
              </span>
            </div>
            <div class="progress-item">
              <span class="progress-label">耗时：</span>
              <span class="progress-value">{{ formatDuration(testProgress.startTime) }}</span>
            </div>
          </div>
          
          <el-progress
            :percentage="testProgress.percentage"
            :status="testing ? 'active' : testProgress.success ? 'success' : 'exception'"
            :stroke-width="8"
          />
          
          <div v-if="testProgress.currentData" class="current-data">
            <div class="data-header">当前数据点</div>
            <div class="data-content">
              <div class="data-item">
                <span class="data-label">时间：</span>
                <span class="data-value">{{ formatDateTime(testProgress.currentData.timestamp) }}</span>
              </div>
              <div class="data-item">
                <span class="data-label">值：</span>
                <span class="data-value">{{ testProgress.currentData.value }}</span>
              </div>
              <div class="data-item">
                <span class="data-label">条件评估：</span>
                <el-tag :type="testProgress.currentData.conditionMet ? 'danger' : 'success'" size="small">
                  {{ testProgress.currentData.conditionMet ? '触发' : '正常' }}
                </el-tag>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 测试结果 -->
      <el-card v-if="testResults.length > 0" class="test-results" shadow="never">
        <template #header>
          <div class="card-header">
            <span class="card-title">测试结果</span>
            <div class="result-actions">
              <el-button type="text" @click="exportResults">
                <el-icon><Download /></el-icon>
                导出结果
              </el-button>
              <el-button type="text" @click="clearResults">
                <el-icon><Delete /></el-icon>
                清空结果
              </el-button>
            </div>
          </div>
        </template>

        <!-- 结果统计 -->
        <div class="result-stats">
          <div class="stat-item">
            <div class="stat-number">{{ testSummary.totalChecks }}</div>
            <div class="stat-label">总检查次数</div>
          </div>
          <div class="stat-item triggered">
            <div class="stat-number">{{ testSummary.triggeredCount }}</div>
            <div class="stat-label">触发次数</div>
          </div>
          <div class="stat-item">
            <div class="stat-number">{{ testSummary.successRate }}%</div>
            <div class="stat-label">规则准确率</div>
          </div>
          <div class="stat-item">
            <div class="stat-number">{{ testSummary.avgResponseTime }}ms</div>
            <div class="stat-label">平均响应时间</div>
          </div>
        </div>

        <!-- 结果详情切换 -->
        <div class="result-tabs">
          <el-tabs v-model="activeResultTab">
            <el-tab-pane label="触发记录" name="triggers">
              <div class="triggers-list">
                <div
                  v-for="(trigger, index) in triggeredResults"
                  :key="index"
                  class="trigger-item"
                >
                  <div class="trigger-header">
                    <div class="trigger-time">{{ formatDateTime(trigger.timestamp) }}</div>
                    <el-tag :type="getSeverityType(trigger.severity)" size="small">
                      {{ getSeverityLabel(trigger.severity) }}
                    </el-tag>
                  </div>
                  <div class="trigger-content">
                    <div class="trigger-condition">
                      <strong>触发条件：</strong>{{ trigger.conditionDescription }}
                    </div>
                    <div class="trigger-value">
                      <strong>实际值：</strong>{{ trigger.actualValue }}
                      <strong>阈值：</strong>{{ trigger.threshold }}
                    </div>
                    <div v-if="trigger.message" class="trigger-message">
                      <strong>消息：</strong>{{ trigger.message }}
                    </div>
                  </div>
                </div>
              </div>
            </el-tab-pane>

            <el-tab-pane label="时间线图表" name="timeline">
              <div class="timeline-chart">
                <div ref="timelineChartRef" style="width: 100%; height: 400px;"></div>
              </div>
            </el-tab-pane>

            <el-tab-pane label="详细日志" name="logs">
              <div class="test-logs">
                <div
                  v-for="(log, index) in testLogs"
                  :key="index"
                  class="log-item"
                  :class="log.level"
                >
                  <div class="log-time">{{ formatDateTime(log.timestamp) }}</div>
                  <div class="log-level">{{ log.level.toUpperCase() }}</div>
                  <div class="log-message">{{ log.message }}</div>
                </div>
              </div>
            </el-tab-pane>
          </el-tabs>
        </div>
      </el-card>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">关闭</el-button>
        <el-button 
          v-if="!testing && testResults.length > 0" 
          type="success" 
          @click="saveTestReport"
        >
          保存测试报告
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * AlertRuleTestDialog —— 报警规则测试对话框
 *
 * 📝 Responsibilities:
 *  1. 提供多种测试模式（实时、历史、模拟）
 *  2. 实时显示测试进度和当前状态
 *  3. 展示测试结果和统计信息
 *  4. 生成测试报告和日志
 *  5. 可视化展示测试时间线
 *
 * 📦 Dependencies:
 *  - Element Plus 组件库
 *  - ECharts 图表库
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  VideoPlay,
  Download,
  Delete
} from '@element-plus/icons-vue'
import * as echarts from 'echarts'

// ===== Props & Emits =====
const props = defineProps<{
  visible: boolean
  ruleData?: any
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  'test-complete': [results: any]
}>()

// ===== 响应式数据 =====
const dialogVisible = ref(props.visible)
const testing = ref(false)
const activeResultTab = ref('triggers')

// 测试配置
const testConfig = ref({
  mode: 'realtime',
  realtime: {
    duration: 5,
    interval: '5s'
  },
  historical: {
    dateRange: [] as string[],
    speed: '1x'
  },
  simulation: {
    scenario: 'normal',
    dataPoints: 100,
    customData: ''
  },
  includeNotifications: true,
  recordResults: true,
  detailedLog: true
})

// 测试进度
const testProgress = ref({
  show: false,
  status: '',
  processed: 0,
  total: 0,
  percentage: 0,
  startTime: '',
  success: false,
  currentData: null as any
})

// 测试结果
const testResults = ref<any[]>([])
const testLogs = ref<any[]>([])

// 图表实例
const timelineChartRef = ref<HTMLElement>()
let timelineChart: echarts.ECharts | null = null

// ===== 计算属性 =====
const triggeredResults = computed(() => {
  return testResults.value.filter(result => result.triggered)
})

const testSummary = computed(() => {
  const total = testResults.value.length
  const triggered = triggeredResults.value.length
  const successRate = total > 0 ? Math.round((triggered / total) * 100) : 0
  const avgResponseTime = total > 0 ? 
    Math.round(testResults.value.reduce((sum, r) => sum + (r.responseTime || 0), 0) / total) : 0

  return {
    totalChecks: total,
    triggeredCount: triggered,
    successRate,
    avgResponseTime
  }
})

// ===== 方法 =====

/**
 * 获取监控范围描述
 */
function getScopeDescription(): string {
  if (!props.ruleData) return ''
  
  const { scope, deviceIds, tagIds, groupIds } = props.ruleData
  
  switch (scope) {
    case 'device':
      return deviceIds?.length > 0 ? `${deviceIds.length} 个设备` : '未选择设备'
    case 'tag':
      return tagIds?.length > 0 ? `${tagIds.length} 个标签` : '未选择标签'
    case 'group':
      return groupIds?.length > 0 ? `${groupIds.length} 个分组` : '未选择分组'
    case 'all':
      return '全部设备'
    default:
      return '未设置'
  }
}

/**
 * 获取严重程度类型
 */
function getSeverityType(severity: string): string {
  const typeMap: { [key: string]: string } = {
    critical: 'danger',
    warning: 'warning',
    info: 'info'
  }
  return typeMap[severity] || 'info'
}

/**
 * 获取严重程度标签
 */
function getSeverityLabel(severity: string): string {
  const labelMap: { [key: string]: string } = {
    critical: '严重',
    warning: '警告',
    info: '信息'
  }
  return labelMap[severity] || severity
}

/**
 * 运行测试
 */
async function runTest() {
  if (!props.ruleData) {
    ElMessage.error('缺少规则数据')
    return
  }

  // 验证配置
  if (!validateTestConfig()) {
    return
  }

  testing.value = true
  testProgress.value = {
    show: true,
    status: '正在初始化测试...',
    processed: 0,
    total: getTestDataPoints(),
    percentage: 0,
    startTime: new Date().toISOString(),
    success: false,
    currentData: null
  }

  // 清空之前的结果
  testResults.value = []
  testLogs.value = []

  try {
    await executeTest()
    testProgress.value.success = true
    testProgress.value.status = '测试完成'
    
    ElMessage.success('规则测试完成')
    
  } catch (error) {
    console.error('测试执行失败:', error)
    testProgress.value.success = false
    testProgress.value.status = '测试失败'
    ElMessage.error('测试执行失败')
  } finally {
    testing.value = false
    testProgress.value.percentage = 100
  }
}

/**
 * 验证测试配置
 */
function validateTestConfig(): boolean {
  const { mode } = testConfig.value

  if (mode === 'historical') {
    if (!testConfig.value.historical.dateRange || testConfig.value.historical.dateRange.length !== 2) {
      ElMessage.error('请选择历史数据的时间范围')
      return false
    }
  }

  if (mode === 'simulation' && testConfig.value.simulation.scenario === 'custom') {
    if (!testConfig.value.simulation.customData.trim()) {
      ElMessage.error('请输入自定义数据序列')
      return false
    }
  }

  return true
}

/**
 * 获取测试数据点数量
 */
function getTestDataPoints(): number {
  const { mode } = testConfig.value

  switch (mode) {
    case 'realtime':
      const duration = testConfig.value.realtime.duration
      const intervalSeconds = parseInterval(testConfig.value.realtime.interval)
      return Math.ceil((duration * 60) / intervalSeconds)
    
    case 'historical':
      // 估算历史数据点数量
      return 1000
    
    case 'simulation':
      return testConfig.value.simulation.dataPoints
    
    default:
      return 100
  }
}

/**
 * 解析时间间隔
 */
function parseInterval(interval: string): number {
  const map: { [key: string]: number } = {
    '1s': 1,
    '5s': 5,
    '10s': 10,
    '30s': 30,
    '1m': 60
  }
  return map[interval] || 5
}

/**
 * 执行测试
 */
async function executeTest() {
  const { mode } = testConfig.value
  
  switch (mode) {
    case 'realtime':
      await executeRealtimeTest()
      break
    case 'historical':
      await executeHistoricalTest()
      break
    case 'simulation':
      await executeSimulationTest()
      break
  }
}

/**
 * 执行实时数据测试
 */
async function executeRealtimeTest() {
  const duration = testConfig.value.realtime.duration * 60 * 1000 // 转换为毫秒
  const intervalMs = parseInterval(testConfig.value.realtime.interval) * 1000
  const total = Math.ceil(duration / intervalMs)
  
  testProgress.value.status = '正在获取实时数据...'
  testProgress.value.total = total

  for (let i = 0; i < total && testing.value; i++) {
    // 模拟获取实时数据
    const currentData = generateRealtimeData()
    
    // 评估条件
    const result = await evaluateConditions(currentData)
    
    // 更新进度
    testProgress.value.processed = i + 1
    testProgress.value.percentage = Math.round(((i + 1) / total) * 100)
    testProgress.value.currentData = {
      timestamp: currentData.timestamp,
      value: currentData.value,
      conditionMet: result.triggered
    }
    testProgress.value.status = `正在测试... (${i + 1}/${total})`

    // 记录结果
    testResults.value.push(result)
    
    // 记录日志
    if (testConfig.value.detailedLog) {
      addTestLog('info', `数据点 ${i + 1}: 值=${currentData.value}, 条件=${result.triggered ? '触发' : '正常'}`)
    }

    // 等待下一个间隔
    await new Promise(resolve => setTimeout(resolve, Math.min(intervalMs, 500))) // 最多等待500ms用于演示
  }
}

/**
 * 执行历史数据测试
 */
async function executeHistoricalTest() {
  testProgress.value.status = '正在加载历史数据...'
  
  // 模拟加载历史数据
  const historicalData = generateHistoricalData()
  const total = historicalData.length
  testProgress.value.total = total

  for (let i = 0; i < total && testing.value; i++) {
    const data = historicalData[i]
    
    // 评估条件
    const result = await evaluateConditions(data)
    
    // 更新进度
    testProgress.value.processed = i + 1
    testProgress.value.percentage = Math.round(((i + 1) / total) * 100)
    testProgress.value.currentData = {
      timestamp: data.timestamp,
      value: data.value,
      conditionMet: result.triggered
    }
    testProgress.value.status = `正在回放历史数据... (${i + 1}/${total})`

    // 记录结果
    testResults.value.push(result)

    // 控制回放速度
    const speed = testConfig.value.historical.speed
    if (speed !== 'max') {
      const delay = speed === '1x' ? 100 : speed === '2x' ? 50 : 20
      await new Promise(resolve => setTimeout(resolve, delay))
    }
  }
}

/**
 * 执行模拟数据测试
 */
async function executeSimulationTest() {
  testProgress.value.status = '正在生成模拟数据...'
  
  const simulationData = generateSimulationData()
  const total = simulationData.length
  testProgress.value.total = total

  for (let i = 0; i < total && testing.value; i++) {
    const data = simulationData[i]
    
    // 评估条件
    const result = await evaluateConditions(data)
    
    // 更新进度
    testProgress.value.processed = i + 1
    testProgress.value.percentage = Math.round(((i + 1) / total) * 100)
    testProgress.value.currentData = {
      timestamp: data.timestamp,
      value: data.value,
      conditionMet: result.triggered
    }
    testProgress.value.status = `正在测试模拟数据... (${i + 1}/${total})`

    // 记录结果
    testResults.value.push(result)

    // 短暂延迟用于演示
    await new Promise(resolve => setTimeout(resolve, 50))
  }
}

/**
 * 生成实时数据
 */
function generateRealtimeData() {
  return {
    timestamp: new Date().toISOString(),
    value: Math.random() * 100,
    quality: 'good'
  }
}

/**
 * 生成历史数据
 */
function generateHistoricalData() {
  const data = []
  const [startTime, endTime] = testConfig.value.historical.dateRange
  const start = new Date(startTime).getTime()
  const end = new Date(endTime).getTime()
  const interval = (end - start) / 200 // 生成200个数据点

  for (let i = 0; i < 200; i++) {
    data.push({
      timestamp: new Date(start + i * interval).toISOString(),
      value: 50 + Math.sin(i * 0.1) * 30 + Math.random() * 10,
      quality: 'good'
    })
  }

  return data
}

/**
 * 生成模拟数据
 */
function generateSimulationData() {
  const { scenario, dataPoints, customData } = testConfig.value.simulation
  const data = []
  const now = Date.now()

  if (scenario === 'custom' && customData) {
    const values = customData.split(',').map(v => parseFloat(v.trim())).filter(v => !isNaN(v))
    return values.map((value, index) => ({
      timestamp: new Date(now + index * 1000).toISOString(),
      value,
      quality: 'good'
    }))
  }

  for (let i = 0; i < dataPoints; i++) {
    let value = 50

    switch (scenario) {
      case 'normal':
        value = 45 + Math.random() * 10
        break
      case 'rising':
        value = 30 + (i / dataPoints) * 50 + Math.random() * 5
        break
      case 'falling':
        value = 80 - (i / dataPoints) * 50 + Math.random() * 5
        break
      case 'periodic':
        value = 50 + Math.sin(i * 0.2) * 20 + Math.random() * 5
        break
      case 'random':
        value = Math.random() * 100
        break
      case 'spike':
        value = i === Math.floor(dataPoints / 2) ? 95 : 45 + Math.random() * 10
        break
    }

    data.push({
      timestamp: new Date(now + i * 1000).toISOString(),
      value,
      quality: 'good'
    })
  }

  return data
}

/**
 * 评估条件
 */
async function evaluateConditions(data: any) {
  const startTime = Date.now()
  
  // 模拟条件评估逻辑
  const conditions = props.ruleData?.conditions || []
  let triggered = false
  let conditionDescription = ''
  let threshold = 0

  if (conditions.length > 0) {
    // 简单的阈值检查示例
    const condition = conditions[0]
    threshold = condition.threshold || 80
    triggered = data.value > threshold
    conditionDescription = `值 > ${threshold}`
  }

  const responseTime = Date.now() - startTime

  const result = {
    timestamp: data.timestamp,
    value: data.value,
    triggered,
    conditionDescription,
    threshold,
    actualValue: data.value,
    severity: props.ruleData?.severity || 'warning',
    message: triggered ? '报警条件触发' : '正常',
    responseTime
  }

  // 发送通知（如果启用）
  if (triggered && testConfig.value.includeNotifications) {
    addTestLog('warning', `触发报警: ${conditionDescription}, 实际值: ${data.value}`)
  }

  return result
}

/**
 * 添加测试日志
 */
function addTestLog(level: string, message: string) {
  testLogs.value.push({
    timestamp: new Date().toISOString(),
    level,
    message
  })
}

/**
 * 停止测试
 */
async function stopTest() {
  try {
    await ElMessageBox.confirm('确定要停止当前测试吗？', '确认停止', {
      type: 'warning'
    })
    
    testing.value = false
    testProgress.value.status = '测试已停止'
    ElMessage.info('测试已停止')
    
  } catch (error) {
    // 用户取消
  }
}

/**
 * 清空结果
 */
function clearResults() {
  testResults.value = []
  testLogs.value = []
  testProgress.value.show = false
  ElMessage.success('测试结果已清空')
}

/**
 * 导出结果
 */
function exportResults() {
  const data = {
    ruleInfo: {
      name: props.ruleData?.name,
      severity: props.ruleData?.severity,
      scope: getScopeDescription()
    },
    testConfig: testConfig.value,
    summary: testSummary.value,
    results: testResults.value,
    logs: testLogs.value,
    exportTime: new Date().toISOString()
  }

  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `alert_rule_test_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.json`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)

  ElMessage.success('测试结果已导出')
}

/**
 * 保存测试报告
 */
function saveTestReport() {
  ElMessage.success('测试报告已保存')
  emit('test-complete', {
    summary: testSummary.value,
    results: testResults.value
  })
}

/**
 * 初始化时间线图表
 */
function initTimelineChart() {
  if (!timelineChartRef.value) return

  timelineChart = echarts.init(timelineChartRef.value)
  
  const option = {
    title: {
      text: '测试数据时间线'
    },
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const point = params[0]
        return `时间: ${formatDateTime(point.name)}<br/>
                值: ${point.value}<br/>
                状态: ${point.data.triggered ? '触发' : '正常'}`
      }
    },
    xAxis: {
      type: 'category',
      data: []
    },
    yAxis: {
      type: 'value'
    },
    series: [
      {
        name: '数据值',
        type: 'line',
        data: [],
        markLine: {
          data: [
            { yAxis: 80, name: '阈值线' }
          ]
        }
      }
    ]
  }

  timelineChart.setOption(option)
}

/**
 * 更新时间线图表
 */
function updateTimelineChart() {
  if (!timelineChart || testResults.value.length === 0) return

  const xData = testResults.value.map(r => r.timestamp)
  const seriesData = testResults.value.map(r => ({
    value: r.value,
    triggered: r.triggered,
    itemStyle: {
      color: r.triggered ? '#f56c6c' : '#67c23a'
    }
  }))

  timelineChart.setOption({
    xAxis: {
      data: xData
    },
    series: [{
      data: seriesData
    }]
  })
}

/**
 * 格式化日期时间
 */
function formatDateTime(dateStr: string): string {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN')
}

/**
 * 格式化持续时间
 */
function formatDuration(startTime: string): string {
  if (!startTime) return '0秒'
  
  const start = new Date(startTime)
  const now = new Date()
  const diffMs = now.getTime() - start.getTime()
  
  const seconds = Math.floor(diffMs / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  
  if (hours > 0) {
    return `${hours}小时${minutes % 60}分钟`
  } else if (minutes > 0) {
    return `${minutes}分钟${seconds % 60}秒`
  } else {
    return `${seconds}秒`
  }
}

/**
 * 处理对话框关闭
 */
function handleClose() {
  if (testing.value) {
    ElMessage.warning('测试正在进行中，请先停止测试')
    return
  }
  
  dialogVisible.value = false
}

// ===== 监听器 =====
watch(() => props.visible, (visible) => {
  dialogVisible.value = visible
  if (visible) {
    // 设置默认的历史数据时间范围
    const now = new Date()
    const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
    testConfig.value.historical.dateRange = [
      weekAgo.toISOString().slice(0, 19),
      now.toISOString().slice(0, 19)
    ]
  }
})

watch(dialogVisible, (visible) => {
  emit('update:visible', visible)
})

watch(activeResultTab, (tab) => {
  if (tab === 'timeline') {
    nextTick(() => {
      initTimelineChart()
      updateTimelineChart()
    })
  }
})

watch(() => testResults.value.length, () => {
  if (activeResultTab.value === 'timeline') {
    updateTimelineChart()
  }
})

// ===== 生命周期 =====
onMounted(() => {
  // 组件挂载时的初始化
})

onUnmounted(() => {
  if (timelineChart) {
    timelineChart.dispose()
    timelineChart = null
  }
})
</script>

<style scoped lang="scss">
.rule-test-dialog {
  .rule-overview {
    margin-bottom: 16px;
    
    .card-title {
      font-size: 16px;
      font-weight: 600;
      color: #303133;
    }
    
    .rule-info {
      .info-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 16px;
        
        .info-item {
          display: flex;
          align-items: center;
          
          label {
            font-weight: 500;
            color: #606266;
            margin-right: 8px;
          }
          
          span {
            color: #303133;
          }
        }
      }
    }
  }
  
  .test-config {
    margin-bottom: 16px;
    
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    
    .config-form {
      .config-row {
        display: flex;
        gap: 24px;
        margin-bottom: 20px;
        flex-wrap: wrap;
        
        &:last-child {
          margin-bottom: 0;
        }
        
        .config-item {
          flex: 1;
          min-width: 200px;
          
          .config-label {
            display: block;
            font-size: 13px;
            color: #606266;
            margin-bottom: 8px;
            font-weight: 500;
          }
        }
      }
      
      .mode-config {
        padding: 16px;
        background: #f8f9fa;
        border-radius: 6px;
        margin-bottom: 20px;
      }
      
      .custom-scenario {
        margin-top: 16px;
        
        .config-label {
          display: block;
          font-size: 13px;
          color: #606266;
          margin-bottom: 8px;
          font-weight: 500;
        }
      }
      
      .duration-input {
        display: flex;
        align-items: center;
        gap: 8px;
        
        span {
          font-size: 13px;
          color: #606266;
        }
      }
      
      .advanced-options {
        display: flex;
        gap: 20px;
        flex-wrap: wrap;
        padding: 16px;
        background: #f8f9fa;
        border-radius: 6px;
      }
    }
  }
  
  .test-progress {
    margin-bottom: 16px;
    
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    
    .progress-content {
      .progress-info {
        display: flex;
        gap: 24px;
        margin-bottom: 16px;
        flex-wrap: wrap;
        
        .progress-item {
          display: flex;
          align-items: center;
          
          .progress-label {
            font-size: 13px;
            color: #606266;
            margin-right: 8px;
          }
          
          .progress-value {
            font-weight: 500;
            color: #303133;
          }
        }
      }
      
      .current-data {
        margin-top: 20px;
        padding: 16px;
        background: #f8f9fa;
        border-radius: 6px;
        
        .data-header {
          font-size: 14px;
          font-weight: 600;
          color: #303133;
          margin-bottom: 12px;
        }
        
        .data-content {
          display: flex;
          gap: 20px;
          flex-wrap: wrap;
          
          .data-item {
            display: flex;
            align-items: center;
            
            .data-label {
              font-size: 13px;
              color: #606266;
              margin-right: 8px;
            }
            
            .data-value {
              font-weight: 500;
              color: #303133;
            }
          }
        }
      }
    }
  }
  
  .test-results {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      .result-actions {
        display: flex;
        gap: 12px;
      }
    }
    
    .result-stats {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
      gap: 16px;
      margin-bottom: 24px;
      
      .stat-item {
        text-align: center;
        padding: 16px;
        background: #f8f9fa;
        border-radius: 6px;
        
        &.triggered {
          background: #fef0f0;
          border: 1px solid #fbc4c4;
        }
        
        .stat-number {
          font-size: 24px;
          font-weight: 600;
          color: #303133;
          margin-bottom: 8px;
        }
        
        .stat-label {
          font-size: 13px;
          color: #606266;
        }
      }
    }
    
    .result-tabs {
      .triggers-list {
        max-height: 400px;
        overflow-y: auto;
        
        .trigger-item {
          border: 1px solid #e4e7ed;
          border-radius: 6px;
          padding: 16px;
          margin-bottom: 12px;
          
          &:last-child {
            margin-bottom: 0;
          }
          
          .trigger-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 12px;
            
            .trigger-time {
              font-size: 13px;
              color: #909399;
            }
          }
          
          .trigger-content {
            .trigger-condition,
            .trigger-value,
            .trigger-message {
              margin-bottom: 8px;
              font-size: 13px;
              color: #606266;
              
              &:last-child {
                margin-bottom: 0;
              }
              
              strong {
                color: #303133;
              }
            }
          }
        }
      }
      
      .timeline-chart {
        min-height: 400px;
      }
      
      .test-logs {
        max-height: 400px;
        overflow-y: auto;
        font-family: monospace;
        
        .log-item {
          display: flex;
          gap: 12px;
          padding: 8px 12px;
          border-bottom: 1px solid #f0f2f5;
          font-size: 13px;
          
          &.info {
            background: #f8f9fa;
          }
          
          &.warning {
            background: #fdf6ec;
          }
          
          &.error {
            background: #fef0f0;
          }
          
          .log-time {
            color: #909399;
            white-space: nowrap;
          }
          
          .log-level {
            color: #606266;
            font-weight: 600;
            width: 60px;
            text-align: center;
          }
          
          .log-message {
            flex: 1;
            color: #303133;
          }
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .rule-test-dialog {
    .rule-overview .rule-info .info-grid {
      grid-template-columns: 1fr;
    }
    
    .test-config .config-form .config-row {
      flex-direction: column;
      gap: 16px;
      
      .config-item {
        min-width: auto;
      }
    }
    
    .test-progress .progress-content .progress-info {
      flex-direction: column;
      gap: 12px;
    }
    
    .test-results .result-stats {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}
</style>