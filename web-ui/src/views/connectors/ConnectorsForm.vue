<template>
  <div class="connectors-form-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <el-button :icon="ArrowLeft" @click="handleGoBack">
          返回列表
        </el-button>
        
        <div class="header-title">
          <el-icon :size="24">
            <Connection />
          </el-icon>
          <h1>{{ isEditing ? '编辑连接' : '创建连接' }}</h1>
          <el-tag v-if="connectionForm.id" type="info">ID: {{ connectionForm.id }}</el-tag>
        </div>
      </div>
      
      <div class="header-actions">
        <el-button :icon="Link" @click="handleTestConnection" :loading="testing">
          测试连接
        </el-button>
        
        <el-button @click="handleLoadTemplate">
          <el-icon><DocumentCopy /></el-icon>
          从模板加载
        </el-button>
        
        <el-button type="primary" :icon="Check" @click="handleSave" :loading="saving">
          {{ isEditing ? '更新连接' : '创建连接' }}
        </el-button>
      </div>
    </div>

    <div class="form-container">
      <el-row :gutter="20">
        <!-- 左侧：主要配置 -->
        <el-col :span="16">
          <div class="form-section">
            <h3>基本信息</h3>
            <BaseForm
              v-model="connectionForm"
              :fields="basicFields"
              :rules="formRules"
              label-width="140px"
              ref="basicFormRef"
            />
          </div>

          <!-- 连接特定配置 -->
          <div class="form-section">
            <h3>
              <el-icon><Setting /></el-icon>
              {{ connectionConfig.title }}
              <el-tag size="small" type="info">{{ connectionConfig.type }}</el-tag>
            </h3>
            
            <ProtocolConfig
              v-model="connectionForm.config"
              :protocol-type="connectionForm.protocol"
              @validate="handleConfigValidate"
            />
          </div>

          <!-- 高级设置 -->
          <el-collapse v-model="expandedSections" class="advanced-config">
            <el-collapse-item title="安全配置" name="security">
              <BaseForm
                v-model="connectionForm"
                :fields="securityFields"
                :rules="securityRules"
                label-width="140px"
              />
            </el-collapse-item>

            <el-collapse-item title="连接池设置" name="pool">
              <BaseForm
                v-model="connectionForm"
                :fields="poolFields"
                :rules="poolRules" 
                label-width="140px"
              />
            </el-collapse-item>

            <el-collapse-item title="重连策略" name="retry">
              <BaseForm
                v-model="connectionForm"
                :fields="retryFields"
                :rules="retryRules"
                label-width="140px"
              />
            </el-collapse-item>

            <el-collapse-item title="监控设置" name="monitoring">
              <BaseForm
                v-model="connectionForm"
                :fields="monitoringFields"
                label-width="140px"
              />
            </el-collapse-item>
          </el-collapse>
        </el-col>

        <!-- 右侧：预览和状态 -->
        <el-col :span="8">
          <!-- 配置预览 -->
          <div class="preview-section">
            <h3>
              <el-icon><View /></el-icon>
              配置预览
              <el-button size="small" text @click="refreshPreview">
                <el-icon><Refresh /></el-icon>
              </el-button>
            </h3>
            
            <div class="config-preview">
              <el-tabs v-model="previewTab" size="small">
                <el-tab-pane label="JSON" name="json">
                  <div class="json-preview">
                    <pre><code>{{ formatConfigJson() }}</code></pre>
                  </div>
                </el-tab-pane>
                
                <el-tab-pane label="YAML" name="yaml">
                  <div class="yaml-preview">
                    <pre><code>{{ formatConfigYaml() }}</code></pre>
                  </div>
                </el-tab-pane>
                
                <el-tab-pane label="验证" name="validation">
                  <div class="config-validation">
                    <ConfigValidation 
                      :config="connectionForm"
                      :validation-rules="validationRules"
                    />
                  </div>
                </el-tab-pane>
              </el-tabs>
            </div>
          </div>

          <!-- 连接状态 -->
          <div class="status-section" v-if="isEditing">
            <h3>
              <el-icon><Monitor /></el-icon>
              连接状态
            </h3>
            
            <ConnectionStatus
              :status="connectionStatus.status"
              :name="connectionForm.name"
              :type="connectionForm.connectionType"
              :show-metrics="true"
              :metrics="connectionStatus.metrics"
            />
            
            <div class="status-details">
              <el-descriptions :column="1" size="small" border>
                <el-descriptions-item label="建立时间">
                  {{ formatTime(connectionStatus.establishedAt) }}
                </el-descriptions-item>
                <el-descriptions-item label="活跃连接">
                  {{ connectionStatus.activeConnections }} / {{ connectionForm.pool?.maxConnections || 1 }}
                </el-descriptions-item>
                <el-descriptions-item label="传输字节">
                  {{ formatBytes(connectionStatus.bytesTransferred) }}
                </el-descriptions-item>
                <el-descriptions-item label="最后活动">
                  {{ formatTime(connectionStatus.lastActivity) }}
                </el-descriptions-item>
              </el-descriptions>
            </div>
          </div>

          <!-- 快速操作 -->
          <div class="quick-actions">
            <h3>
              <el-icon><Operation /></el-icon>
              快速操作
            </h3>
            
            <div class="action-buttons">
              <el-button-group>
                <el-button :icon="Document" @click="handleExportConfig">
                  导出配置
                </el-button>
                <el-button :icon="Upload" @click="handleImportConfig">
                  导入配置
                </el-button>
              </el-button-group>
              
              <el-button-group>
                <el-button :icon="CopyDocument" @click="handleCopyConfig">
                  复制配置
                </el-button>
                <el-button :icon="DeleteFilled" @click="handleResetForm">
                  重置表单
                </el-button>
              </el-button-group>

              <el-button-group v-if="isEditing">
                <el-button :icon="VideoPlay" @click="handleStartConnection">
                  启动连接
                </el-button>
                <el-button :icon="VideoPause" @click="handleStopConnection">
                  停止连接
                </el-button>
              </el-button-group>
            </div>
          </div>
        </el-col>
      </el-row>
    </div>

    <!-- 连接测试对话框 -->
    <el-dialog
      v-model="testDialogVisible"
      title="连接测试结果"
      width="600px"
    >
      <div class="test-result">
        <div class="result-header">
          <el-icon :size="24" :class="testResult.success ? 'success' : 'error'">
            <component :is="testResult.success ? 'CircleCheck' : 'CircleClose'" />
          </el-icon>
          <div class="result-info">
            <h3>{{ testResult.success ? '连接成功' : '连接失败' }}</h3>
            <p>{{ testResult.message }}</p>
          </div>
        </div>

        <div class="test-details">
          <el-descriptions :column="2" border>
            <el-descriptions-item label="测试时间">
              {{ formatTime(testResult.timestamp) }}
            </el-descriptions-item>
            <el-descriptions-item label="响应时间">
              {{ testResult.responseTime }}ms
            </el-descriptions-item>
            <el-descriptions-item label="网络延迟">
              {{ testResult.latency }}ms
            </el-descriptions-item>
            <el-descriptions-item label="带宽测试">
              {{ testResult.bandwidth || 'N/A' }}
            </el-descriptions-item>
          </el-descriptions>

          <div v-if="testResult.logs" class="test-logs">
            <h4>详细日志:</h4>
            <el-scrollbar height="200px">
              <pre class="log-content">{{ testResult.logs }}</pre>
            </el-scrollbar>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 模板选择对话框 -->
    <el-dialog
      v-model="templateDialogVisible"
      title="选择连接模板"
      width="800px"
    >
      <div class="template-grid">
        <div 
          v-for="template in connectionTemplates"
          :key="template.id"
          class="template-card"
          :class="{ active: selectedTemplate?.id === template.id }"
          @click="selectedTemplate = template"
        >
          <div class="template-header">
            <el-icon class="template-icon">
              <component :is="getConnectionTypeIcon(template.connectionType)" />
            </el-icon>
            <div class="template-info">
              <h4>{{ template.name }}</h4>
              <p>{{ template.description }}</p>
            </div>
            <el-tag size="small" :type="getConnectionTypeTagType(template.connectionType)">
              {{ template.connectionType }}
            </el-tag>
          </div>
          
          <div class="template-config">
            <div class="config-items">
              <span class="config-item" v-for="item in template.configItems" :key="item">
                {{ item }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <el-button @click="templateDialogVisible = false">取消</el-button>
        <el-button 
          type="primary" 
          @click="handleApplyTemplate"
          :disabled="!selectedTemplate"
        >
          应用模板
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowLeft,
  Connection,
  Check,
  Link,
  DocumentCopy,
  Setting,
  View,
  Refresh,
  Monitor,
  Operation,
  Document,
  Upload,
  CopyDocument,
  DeleteFilled,
  VideoPlay,
  VideoPause,
  CircleCheck,
  CircleClose
} from '@element-plus/icons-vue'

// 导入组件
import { BaseForm } from '../../components/base'
import { 
  ProtocolConfig,
  ConfigValidation,
  ConnectionStatus
} from '../../components/business'

// 类型定义
interface ConnectionForm {
  id: string
  name: string
  description: string
  connectionType: 'tcp' | 'serial' | 'websocket' | 'http' | 'https'
  protocol: 'modbus_tcp' | 'modbus_rtu' | 'opcua' | 'mqtt' | 'ethernet_ip' | 'bacnet' | 'http'
  enabled: boolean
  autoReconnect: boolean
  config: Record<string, any>
  security: {
    enableTls: boolean
    verifyCertificate: boolean
    clientCertPath?: string
    clientKeyPath?: string
    caCertPath?: string
    username?: string
    password?: string
  }
  pool: {
    maxConnections: number
    minConnections: number
    connectionTimeout: number
    idleTimeout: number
    maxIdleTime: number
  }
  retry: {
    maxRetries: number
    retryInterval: number
    exponentialBackoff: boolean
    maxRetryInterval: number
  }
  monitoring: {
    enableHealthCheck: boolean
    healthCheckInterval: number
    enableMetrics: boolean
    metricsInterval: number
  }
}

interface TestResult {
  success: boolean
  message: string
  timestamp: Date
  responseTime: number
  latency: number
  bandwidth?: string
  logs?: string
}

interface ConnectionStatus {
  status: 'connected' | 'disconnected' | 'connecting' | 'error'
  establishedAt: Date
  activeConnections: number
  bytesTransferred: number
  lastActivity: Date
  metrics: Record<string, any>
}

interface ConnectionTemplate {
  id: string
  name: string
  description: string
  connectionType: string
  protocol: string
  config: Record<string, any>
  configItems: string[]
}

// 路由
const route = useRoute()
const router = useRouter()

// 状态管理
const saving = ref(false)
const testing = ref(false)
const testDialogVisible = ref(false)
const templateDialogVisible = ref(false)
const previewTab = ref('json')
const expandedSections = ref(['security'])

const basicFormRef = ref()
const selectedTemplate = ref<ConnectionTemplate | null>(null)

// 表单数据
const connectionForm = ref<ConnectionForm>({
  id: '',
  name: '',
  description: '',
  connectionType: 'tcp',
  protocol: 'modbus_tcp',
  enabled: true,
  autoReconnect: true,
  config: {},
  security: {
    enableTls: false,
    verifyCertificate: true
  },
  pool: {
    maxConnections: 10,
    minConnections: 1,
    connectionTimeout: 5000,
    idleTimeout: 30000,
    maxIdleTime: 300000
  },
  retry: {
    maxRetries: 3,
    retryInterval: 1000,
    exponentialBackoff: true,
    maxRetryInterval: 10000
  },
  monitoring: {
    enableHealthCheck: true,
    healthCheckInterval: 30000,
    enableMetrics: true,
    metricsInterval: 5000
  }
})

const testResult = ref<TestResult>({
  success: false,
  message: '',
  timestamp: new Date(),
  responseTime: 0,
  latency: 0
})

const connectionStatus = ref<ConnectionStatus>({
  status: 'disconnected',
  establishedAt: new Date(),
  activeConnections: 0,
  bytesTransferred: 0,
  lastActivity: new Date(),
  metrics: {}
})

// 模拟模板数据
const connectionTemplates = ref<ConnectionTemplate[]>([
  {
    id: '1',
    name: 'TCP 连接模板',
    description: '标准的TCP网络连接模板，适用于大多数以太网设备',
    connectionType: 'TCP',
    protocol: 'Modbus TCP',
    config: {
      host: '192.168.1.100',
      port: 502,
      timeout: 5000
    },
    configItems: ['主机地址', '端口号', '超时时间', '保持连接']
  },
  {
    id: '2',
    name: '串口连接模板',
    description: 'RS232/RS485串口连接模板',
    connectionType: 'Serial',
    protocol: 'Modbus RTU',
    config: {
      serialPort: 'COM1',
      baudRate: 9600,
      dataBits: 8,
      parity: 'None',
      stopBits: 1
    },
    configItems: ['串口号', '波特率', '数据位', '校验位', '停止位']
  },
  {
    id: '3',
    name: 'WebSocket 连接',
    description: 'WebSocket实时通信连接模板',
    connectionType: 'WebSocket',
    protocol: 'WebSocket',
    config: {
      url: 'ws://localhost:8080/ws',
      protocols: ['mqtt', 'wamp']
    },
    configItems: ['WebSocket URL', '子协议', '重连机制', '心跳检测']
  },
  {
    id: '4',
    name: 'HTTPS 安全连接',
    description: 'HTTPS安全连接模板，支持证书认证',
    connectionType: 'HTTPS',
    protocol: 'HTTPS',
    config: {
      baseUrl: 'https://api.example.com',
      timeout: 10000,
      maxRedirects: 5
    },
    configItems: ['基础URL', '超时时间', '重定向', 'SSL证书']
  }
])

// 计算属性
const isEditing = computed(() => !!route.params.id && route.params.id !== 'new')

const connectionConfig = computed(() => {
  const configs = {
    tcp: { title: 'TCP 连接配置', type: 'TCP' },
    serial: { title: '串口连接配置', type: 'Serial' },
    websocket: { title: 'WebSocket 连接配置', type: 'WebSocket' },
    http: { title: 'HTTP 连接配置', type: 'HTTP' },
    https: { title: 'HTTPS 连接配置', type: 'HTTPS' }
  }
  return configs[connectionForm.value.connectionType] || configs.tcp
})

// 表单字段配置
const basicFields = computed(() => [
  {
    name: 'name',
    label: '连接名称',
    type: 'input',
    required: true,
    placeholder: '请输入连接名称'
  },
  {
    name: 'description',
    label: '连接描述',
    type: 'textarea',
    placeholder: '请输入连接描述（可选）'
  },
  {
    name: 'connectionType',
    label: '连接类型',
    type: 'select',
    required: true,
    options: [
      { label: 'TCP 网络连接', value: 'tcp' },
      { label: '串口连接', value: 'serial' },
      { label: 'WebSocket 连接', value: 'websocket' },
      { label: 'HTTP 连接', value: 'http' },
      { label: 'HTTPS 安全连接', value: 'https' }
    ]
  },
  {
    name: 'protocol',
    label: '应用协议',
    type: 'select',
    required: true,
    options: [
      { label: 'Modbus TCP', value: 'modbus_tcp' },
      { label: 'Modbus RTU', value: 'modbus_rtu' },
      { label: 'OPC UA', value: 'opcua' },
      { label: 'MQTT', value: 'mqtt' },
      { label: 'EtherNet/IP', value: 'ethernet_ip' },
      { label: 'BACnet', value: 'bacnet' },
      { label: 'HTTP', value: 'http' }
    ]
  },
  {
    name: 'enabled',
    label: '启用状态',
    type: 'switch',
    activeText: '启用',
    inactiveText: '禁用'
  },
  {
    name: 'autoReconnect',
    label: '自动重连',
    type: 'switch',
    activeText: '启用',
    inactiveText: '禁用'
  }
])

const securityFields = [
  {
    name: 'security.enableTls',
    label: '启用TLS/SSL',
    type: 'switch'
  },
  {
    name: 'security.verifyCertificate',
    label: '验证证书',
    type: 'switch'
  },
  {
    name: 'security.username',
    label: '用户名',
    type: 'input',
    placeholder: '连接用户名'
  },
  {
    name: 'security.password',
    label: '密码',
    type: 'password',
    placeholder: '连接密码'
  },
  {
    name: 'security.clientCertPath',
    label: '客户端证书路径',
    type: 'input',
    placeholder: '客户端证书文件路径'
  },
  {
    name: 'security.clientKeyPath',
    label: '客户端密钥路径',
    type: 'input',
    placeholder: '客户端密钥文件路径'
  },
  {
    name: 'security.caCertPath',
    label: 'CA证书路径',
    type: 'input',
    placeholder: 'CA证书文件路径'
  }
]

const poolFields = [
  {
    name: 'pool.maxConnections',
    label: '最大连接数',
    type: 'number',
    min: 1,
    max: 100,
    placeholder: '连接池最大连接数'
  },
  {
    name: 'pool.minConnections',
    label: '最小连接数',
    type: 'number',
    min: 0,
    max: 50,
    placeholder: '连接池最小连接数'
  },
  {
    name: 'pool.connectionTimeout',
    label: '连接超时(ms)',
    type: 'number',
    min: 1000,
    max: 60000,
    placeholder: '建立连接超时时间'
  },
  {
    name: 'pool.idleTimeout',
    label: '空闲超时(ms)',
    type: 'number',
    min: 5000,
    max: 300000,
    placeholder: '连接空闲超时时间'
  },
  {
    name: 'pool.maxIdleTime',
    label: '最大空闲时间(ms)',
    type: 'number',
    min: 60000,
    max: 1800000,
    placeholder: '连接最大空闲时间'
  }
]

const retryFields = [
  {
    name: 'retry.maxRetries',
    label: '最大重试次数',
    type: 'number',
    min: 0,
    max: 10
  },
  {
    name: 'retry.retryInterval',
    label: '重试间隔(ms)',
    type: 'number',
    min: 100,
    max: 10000
  },
  {
    name: 'retry.exponentialBackoff',
    label: '指数退避',
    type: 'switch',
    activeText: '启用',
    inactiveText: '禁用'
  },
  {
    name: 'retry.maxRetryInterval',
    label: '最大重试间隔(ms)',
    type: 'number',
    min: 1000,
    max: 60000
  }
]

const monitoringFields = [
  {
    name: 'monitoring.enableHealthCheck',
    label: '启用健康检查',
    type: 'switch'
  },
  {
    name: 'monitoring.healthCheckInterval',
    label: '健康检查间隔(ms)',
    type: 'number',
    min: 5000,
    max: 300000
  },
  {
    name: 'monitoring.enableMetrics',
    label: '启用指标收集',
    type: 'switch'
  },
  {
    name: 'monitoring.metricsInterval',
    label: '指标收集间隔(ms)',
    type: 'number',
    min: 1000,
    max: 60000
  }
]

// 验证规则
const formRules = {
  name: [
    { required: true, message: '请输入连接名称', trigger: 'blur' },
    { min: 2, max: 50, message: '名称长度应在2-50字符之间', trigger: 'blur' }
  ],
  connectionType: [
    { required: true, message: '请选择连接类型', trigger: 'change' }
  ],
  protocol: [
    { required: true, message: '请选择应用协议', trigger: 'change' }
  ]
}

const securityRules = {
  'security.username': [
    { min: 3, max: 50, message: '用户名长度应在3-50字符之间', trigger: 'blur' }
  ],
  'security.password': [
    { min: 6, message: '密码长度不能少于6个字符', trigger: 'blur' }
  ]
}

const poolRules = {
  'pool.maxConnections': [
    { required: true, message: '请输入最大连接数', trigger: 'blur' }
  ],
  'pool.connectionTimeout': [
    { required: true, message: '请输入连接超时时间', trigger: 'blur' }
  ]
}

const retryRules = {
  'retry.maxRetries': [
    { required: true, message: '请输入最大重试次数', trigger: 'blur' }
  ],
  'retry.retryInterval': [
    { required: true, message: '请输入重试间隔', trigger: 'blur' }
  ]
}

const validationRules = computed(() => ({
  required: ['name', 'connectionType', 'protocol'],
  ranges: {
    'pool.maxConnections': { min: 1, max: 100 },
    'pool.connectionTimeout': { min: 1000, max: 60000 },
    'retry.maxRetries': { min: 0, max: 10 }
  }
}))

// 方法
const formatTime = (time: Date) => {
  return time.toLocaleString('zh-CN')
}

const formatBytes = (bytes: number) => {
  const sizes = ['B', 'KB', 'MB', 'GB']
  if (bytes === 0) return '0 B'
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return `${Math.round(bytes / Math.pow(1024, i) * 100) / 100  } ${  sizes[i]}`
}

const formatConfigJson = () => {
  try {
    return JSON.stringify(connectionForm.value, null, 2)
  } catch (error) {
    return '配置格式错误'
  }
}

const formatConfigYaml = () => {
  try {
    const obj = connectionForm.value
    let yaml = ''
    
    const toYaml = (obj: any, indent = 0) => {
      const spaces = '  '.repeat(indent)
      for (const [key, value] of Object.entries(obj)) {
        if (typeof value === 'object' && value !== null) {
          yaml += `${spaces}${key}:\n`
          toYaml(value, indent + 1)
        } else {
          yaml += `${spaces}${key}: ${value}\n`
        }
      }
    }
    
    toYaml(obj)
    return yaml
  } catch (error) {
    return '配置格式错误'
  }
}

const getConnectionTypeIcon = (type: string) => {
  const icons: Record<string, string> = {
    'TCP': 'Connection',
    'Serial': 'Link',
    'WebSocket': 'Connection',
    'HTTP': 'Link',
    'HTTPS': 'Setting'
  }
  return icons[type] || 'Connection'
}

const getConnectionTypeTagType = (type: string) => {
  const types: Record<string, string> = {
    'TCP': 'primary',
    'Serial': 'success',
    'WebSocket': 'warning',
    'HTTP': 'info',
    'HTTPS': 'danger'
  }
  return types[type] || 'info'
}

// 事件处理
const handleGoBack = () => {
  router.push('/connectors')
}

const handleSave = async () => {
  try {
    await basicFormRef.value?.validate()
  } catch (error) {
    ElMessage.error('请检查表单填写')
    return
  }
  
  saving.value = true
  
  try {
    await new Promise(resolve => setTimeout(resolve, 1500))
    
    if (isEditing.value) {
      ElMessage.success('连接配置已更新')
    } else {
      ElMessage.success('连接创建成功')
    }
    
    setTimeout(() => {
      router.push('/connectors')
    }, 1000)
    
  } catch (error) {
    ElMessage.error('保存失败，请重试')
  } finally {
    saving.value = false
  }
}

const handleTestConnection = async () => {
  testing.value = true
  
  try {
    await new Promise(resolve => setTimeout(resolve, 2500))
    
    const success = Math.random() > 0.25
    testResult.value = {
      success,
      message: success 
        ? '连接测试成功，网络通信正常' 
        : '连接测试失败，请检查网络和配置',
      timestamp: new Date(),
      responseTime: Math.floor(Math.random() * 300) + 50,
      latency: Math.floor(Math.random() * 100) + 10,
      bandwidth: success ? '10 Mbps' : undefined,
      logs: success 
        ? '初始化连接...\n建立TCP连接\n协议握手成功\n数据传输测试\n连接测试完成'
        : '初始化连接...\n尝试建立连接\n连接超时\n重试连接失败\n测试失败'
    }
    
    testDialogVisible.value = true
    
  } catch (error) {
    ElMessage.error('连接测试异常')
  } finally {
    testing.value = false
  }
}

const handleLoadTemplate = () => {
  templateDialogVisible.value = true
}

const handleApplyTemplate = () => {
  if (!selectedTemplate.value) return
  
  const template = selectedTemplate.value
  
  // 应用模板配置
  Object.assign(connectionForm.value.config, template.config)
  
  // 根据连接类型设置其他字段
  if (template.connectionType === 'TCP') {
    connectionForm.value.connectionType = 'tcp'
  } else if (template.connectionType === 'Serial') {
    connectionForm.value.connectionType = 'serial'
  } else if (template.connectionType === 'WebSocket') {
    connectionForm.value.connectionType = 'websocket'
  } else if (template.connectionType === 'HTTPS') {
    connectionForm.value.connectionType = 'https'
  }
  
  // 设置协议
  if (template.protocol === 'Modbus TCP') {
    connectionForm.value.protocol = 'modbus_tcp'
  } else if (template.protocol === 'Modbus RTU') {
    connectionForm.value.protocol = 'modbus_rtu'
  } else if (template.protocol === 'OPC UA') {
    connectionForm.value.protocol = 'opcua'
  }
  
  templateDialogVisible.value = false
  selectedTemplate.value = null
  
  ElMessage.success(`已应用模板：${template.name}`)
}

const handleConfigValidate = (valid: boolean) => {
  if (!valid) {
    ElMessage.warning('连接配置不完整，请检查必填项')
  }
}

const handleExportConfig = () => {
  const exportData = {
    connection: connectionForm.value,
    exportTime: new Date().toISOString(),
    version: '1.0'
  }
  
  const blob = new Blob([JSON.stringify(exportData, null, 2)], {
    type: 'application/json'
  })
  
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `connection_${connectionForm.value.name || 'config'}_${new Date().toISOString().split('T')[0]}.json`
  a.click()
  
  URL.revokeObjectURL(url)
  ElMessage.success('配置导出成功')
}

const handleImportConfig = () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = (e) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    
    const reader = new FileReader()
    reader.onload = (e) => {
      try {
        const data = JSON.parse(e.target?.result as string)
        if (data.connection) {
          Object.assign(connectionForm.value, data.connection)
          ElMessage.success('配置导入成功')
        } else {
          ElMessage.error('配置文件格式不正确')
        }
      } catch (error) {
        ElMessage.error('配置文件解析失败')
      }
    }
    reader.readAsText(file)
  }
  input.click()
}

const handleCopyConfig = () => {
  try {
    navigator.clipboard.writeText(formatConfigJson())
    ElMessage.success('配置已复制到剪贴板')
  } catch (error) {
    ElMessage.error('复制失败，请手动复制')
  }
}

const handleResetForm = () => {
  ElMessageBox.confirm(
    '确定要重置表单吗？所有未保存的更改将丢失。',
    '确认重置',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    }
  ).then(() => {
    // 重置表单到初始状态
    Object.assign(connectionForm.value, {
      id: '',
      name: '',
      description: '',
      connectionType: 'tcp',
      protocol: 'modbus_tcp',
      enabled: true,
      autoReconnect: true,
      config: {},
      security: {
        enableTls: false,
        verifyCertificate: true
      },
      pool: {
        maxConnections: 10,
        minConnections: 1,
        connectionTimeout: 5000,
        idleTimeout: 30000,
        maxIdleTime: 300000
      },
      retry: {
        maxRetries: 3,
        retryInterval: 1000,
        exponentialBackoff: true,
        maxRetryInterval: 10000
      },
      monitoring: {
        enableHealthCheck: true,
        healthCheckInterval: 30000,
        enableMetrics: true,
        metricsInterval: 5000
      }
    })
    
    ElMessage.success('表单已重置')
  })
}

const handleStartConnection = async () => {
  try {
    await new Promise(resolve => setTimeout(resolve, 1000))
    connectionStatus.value.status = 'connected'
    ElMessage.success('连接已启动')
  } catch (error) {
    ElMessage.error('连接启动失败')
  }
}

const handleStopConnection = async () => {
  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    connectionStatus.value.status = 'disconnected'
    ElMessage.success('连接已停止')
  } catch (error) {
    ElMessage.error('连接停止失败')
  }
}

const refreshPreview = () => {
  ElMessage.info('配置预览已刷新')
}

// 监听连接类型变化，更新配置预设
watch(() => connectionForm.value.connectionType, (newType) => {
  const defaultConfigs = {
    tcp: {
      host: '192.168.1.100',
      port: 502
    },
    serial: {
      serialPort: 'COM1',
      baudRate: 9600,
      dataBits: 8,
      parity: 'None',
      stopBits: 1
    },
    websocket: {
      url: 'ws://localhost:8080/ws'
    },
    http: {
      baseUrl: 'http://localhost:8080'
    },
    https: {
      baseUrl: 'https://localhost:8443'
    }
  }
  
  if (!Object.keys(connectionForm.value.config).length) {
    connectionForm.value.config = defaultConfigs[newType] || {}
  }
}, { immediate: true })

// 生命周期
onMounted(async () => {
  if (isEditing.value) {
    // 模拟加载现有连接配置
    const connectionId = route.params.id as string
    
    // 模拟连接数据
    const mockConnection = {
      id: connectionId,
      name: '主控制器连接',
      description: '生产线主控制器TCP连接',
      connectionType: 'tcp' as const,
      protocol: 'modbus_tcp' as const,
      enabled: true,
      autoReconnect: true,
      config: {
        host: '192.168.1.100',
        port: 502
      },
      security: {
        enableTls: false,
        verifyCertificate: true,
        username: 'admin'
      },
      pool: {
        maxConnections: 10,
        minConnections: 2,
        connectionTimeout: 5000,
        idleTimeout: 30000,
        maxIdleTime: 300000
      },
      retry: {
        maxRetries: 3,
        retryInterval: 1000,
        exponentialBackoff: true,
        maxRetryInterval: 10000
      },
      monitoring: {
        enableHealthCheck: true,
        healthCheckInterval: 30000,
        enableMetrics: true,
        metricsInterval: 5000
      }
    }
    
    Object.assign(connectionForm.value, mockConnection)
    
    // 模拟连接状态
    connectionStatus.value = {
      status: 'connected',
      establishedAt: new Date(Date.now() - 7200000), // 2 hours ago
      activeConnections: 3,
      bytesTransferred: 1024 * 1024 * 50, // 50MB
      lastActivity: new Date(),
      metrics: {
        packetsIn: 15670,
        packetsOut: 12450,
        errorsCount: 2,
        avgLatency: 25
      }
    }
  }
})
</script>

<style scoped lang="scss">
.connectors-form-page {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding: 16px 0;
  border-bottom: 1px solid var(--el-border-color-light);
  
  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
    
    .header-title {
      display: flex;
      align-items: center;
      gap: 12px;
      
      h1 {
        margin: 0;
        font-size: 24px;
        color: var(--el-text-color-primary);
      }
      
      .el-icon {
        color: var(--el-color-primary);
      }
    }
  }
  
  .header-actions {
    display: flex;
    gap: 12px;
  }
}

.form-container {
  .form-section {
    background: white;
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 20px;
    border: 1px solid var(--el-border-color-light);
    
    h3 {
      margin: 0 0 20px 0;
      font-size: 18px;
      color: var(--el-text-color-primary);
      display: flex;
      align-items: center;
      gap: 8px;
      
      .el-icon {
        color: var(--el-color-primary);
      }
    }
  }
  
  .advanced-config {
    background: white;
    border-radius: 8px;
    border: 1px solid var(--el-border-color-light);
    
    :deep(.el-collapse-item__header) {
      padding: 0 20px;
    }
    
    :deep(.el-collapse-item__content) {
      padding: 0 20px 20px;
    }
  }
}

.preview-section, .status-section, .quick-actions {
  background: white;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
  border: 1px solid var(--el-border-color-light);
  
  h3 {
    margin: 0 0 16px 0;
    font-size: 16px;
    color: var(--el-text-color-primary);
    display: flex;
    align-items: center;
    justify-content: space-between;
    
    .el-icon {
      color: var(--el-color-primary);
    }
  }
}

.config-preview {
  .json-preview, .yaml-preview {
    pre {
      background: var(--el-fill-color-light);
      padding: 12px;
      border-radius: 4px;
      font-size: 12px;
      max-height: 300px;
      overflow-y: auto;
      
      code {
        color: var(--el-text-color-primary);
      }
    }
  }
}

.status-details {
  margin-top: 16px;
  
  .error {
    color: var(--el-color-danger);
  }
}

.action-buttons {
  display: flex;
  flex-direction: column;
  gap: 12px;
  
  .el-button-group {
    display: flex;
    gap: 0;
  }
}

.test-result {
  .result-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
    
    .el-icon {
      &.success {
        color: var(--el-color-success);
      }
      
      &.error {
        color: var(--el-color-danger);
      }
    }
    
    .result-info {
      h3 {
        margin: 0 0 4px 0;
        color: var(--el-text-color-primary);
      }
      
      p {
        margin: 0;
        color: var(--el-text-color-regular);
        font-size: 14px;
      }
    }
  }
  
  .test-logs {
    margin-top: 16px;
    
    h4 {
      margin: 0 0 8px 0;
      font-size: 14px;
    }
    
    .log-content {
      background: var(--el-fill-color-dark);
      color: var(--el-text-color-primary);
      padding: 8px;
      font-size: 12px;
      line-height: 1.4;
      margin: 0;
    }
  }
}

.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 16px;
  max-height: 400px;
  overflow-y: auto;
  
  .template-card {
    border: 1px solid var(--el-border-color-light);
    border-radius: 8px;
    padding: 16px;
    cursor: pointer;
    transition: all 0.2s;
    
    &:hover {
      border-color: var(--el-color-primary);
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }
    
    &.active {
      border-color: var(--el-color-primary);
      background-color: var(--el-color-primary-light-9);
    }
    
    .template-header {
      display: flex;
      align-items: center;
      gap: 12px;
      margin-bottom: 12px;
      
      .template-icon {
        color: var(--el-color-primary);
        font-size: 20px;
      }
      
      .template-info {
        flex: 1;
        
        h4 {
          margin: 0 0 4px 0;
          font-size: 14px;
          color: var(--el-text-color-primary);
        }
        
        p {
          margin: 0;
          font-size: 12px;
          color: var(--el-text-color-regular);
          line-height: 1.4;
        }
      }
    }
    
    .config-items {
      display: flex;
      flex-wrap: wrap;
      gap: 6px;
      
      .config-item {
        background: var(--el-color-info-light-8);
        color: var(--el-color-info);
        padding: 2px 6px;
        border-radius: 3px;
        font-size: 11px;
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .form-container {
    .el-row {
      flex-direction: column;
      
      .el-col {
        width: 100% !important;
        max-width: none !important;
      }
    }
  }
}

@media (max-width: 768px) {
  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
    
    .header-actions {
      flex-wrap: wrap;
    }
  }
  
  .template-grid {
    grid-template-columns: 1fr;
  }
}
</style>