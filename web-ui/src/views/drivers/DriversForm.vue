<template>
  <div class="drivers-form-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <el-button :icon="ArrowLeft" @click="handleGoBack">
          返回列表
        </el-button>
        
        <div class="header-title">
          <el-icon :size="24">
            <Setting />
          </el-icon>
          <h1>{{ isEditing ? '编辑驱动' : '创建驱动' }}</h1>
          <el-tag v-if="driverForm.id" type="info">ID: {{ driverForm.id }}</el-tag>
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
          {{ isEditing ? '更新配置' : '创建驱动' }}
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
              v-model="driverForm"
              :fields="basicFields"
              :rules="formRules"
              label-width="140px"
              ref="basicFormRef"
            />
          </div>

          <!-- 协议特定配置 -->
          <div class="form-section">
            <h3>
              <el-icon><Tools /></el-icon>
              {{ protocolConfig.title }}
              <el-tag size="small" type="info">{{ protocolConfig.protocol }}</el-tag>
            </h3>
            
            <ProtocolConfig
              v-model="driverForm.config"
              :protocol="driverForm.protocol"
              :connection-type="driverForm.connectionType"
              @validate="handleProtocolValidate"
            />
          </div>

          <!-- 高级设置 -->
          <el-collapse v-model="expandedSections" class="advanced-config">
            <el-collapse-item title="性能设置" name="performance">
              <BaseForm
                v-model="driverForm"
                :fields="performanceFields"
                :rules="performanceRules"
                label-width="140px"
              />
            </el-collapse-item>

            <el-collapse-item title="重连策略" name="retry">
              <BaseForm
                v-model="driverForm"
                :fields="retryFields"
                :rules="retryRules" 
                label-width="140px"
              />
            </el-collapse-item>

            <el-collapse-item title="日志设置" name="logging">
              <BaseForm
                v-model="driverForm"
                :fields="loggingFields"
                label-width="140px"
              />
            </el-collapse-item>

            <el-collapse-item title="安全配置" name="security">
              <BaseForm
                v-model="driverForm"
                :fields="securityFields"
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
                
                <el-tab-pane label="摘要" name="summary">
                  <div class="config-summary">
                    <ConfigValidation 
                      :config="driverForm"
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
              <el-icon><Connection /></el-icon>
              连接状态
            </h3>
            
            <ConnectionStatus
              :status="driverStatus.status"
              :name="driverForm.name"
              :type="driverForm.protocol"
              :show-metrics="true"
              :metrics="driverStatus.metrics"
            />
            
            <div class="status-details">
              <el-descriptions :column="1" size="small" border>
                <el-descriptions-item label="运行时间">
                  {{ formatUptime(driverStatus.uptime) }}
                </el-descriptions-item>
                <el-descriptions-item label="消息计数">
                  {{ formatNumber(driverStatus.messageCount) }}
                </el-descriptions-item>
                <el-descriptions-item label="错误计数">
                  <span :class="{ error: driverStatus.errorCount > 0 }">
                    {{ driverStatus.errorCount }}
                  </span>
                </el-descriptions-item>
                <el-descriptions-item label="最后更新">
                  {{ formatTime(driverStatus.lastUpdate) }}
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
            <el-descriptions-item label="协议版本">
              {{ testResult.protocolVersion || 'N/A' }}
            </el-descriptions-item>
            <el-descriptions-item label="设备信息">
              {{ testResult.deviceInfo || 'N/A' }}
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
      title="选择驱动模板"
      width="800px"
    >
      <div class="template-grid">
        <div 
          v-for="template in driverTemplates"
          :key="template.id"
          class="template-card"
          :class="{ active: selectedTemplate?.id === template.id }"
          @click="selectedTemplate = template"
        >
          <div class="template-header">
            <el-icon class="template-icon">
              <component :is="getProtocolIcon(template.protocol)" />
            </el-icon>
            <div class="template-info">
              <h4>{{ template.name }}</h4>
              <p>{{ template.description }}</p>
            </div>
            <el-tag size="small" :type="getProtocolTagType(template.protocol)">
              {{ template.protocol }}
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
  Setting,
  Check,
  Link,
  DocumentCopy,
  Tools,
  View,
  Refresh,
  Connection,
  Operation,
  Document,
  Upload,
  CopyDocument,
  DeleteFilled,
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

// 导入API
import { driverConfigsApi } from '../../api/drivers'

// 类型定义
interface DriverForm {
  id: string
  name: string
  description: string
  protocol: 'modbus_tcp' | 'modbus_rtu' | 'opcua' | 'mqtt' | 'ethernet_ip' | 'bacnet'
  connectionType: 'ethernet' | 'serial'
  enabled: boolean
  config: Record<string, any>
  performance: {
    scanInterval: number
    timeout: number
    maxConcurrentRequests: number
    batchSize: number
  }
  retry: {
    maxRetries: number
    retryInterval: number
    exponentialBackoff: boolean
    maxRetryInterval: number
  }
  logging: {
    level: 'error' | 'warn' | 'info' | 'debug' | 'trace'
    enableRequestLog: boolean
    enableResponseLog: boolean
    maxLogSize: number
  }
  security: {
    enableSsl: boolean
    verifyCertificate: boolean
    clientCertPath?: string
    clientKeyPath?: string
  }
}

interface TestResult {
  success: boolean
  message: string
  timestamp: Date
  responseTime: number
  protocolVersion?: string
  deviceInfo?: string
  logs?: string
}

interface DriverStatus {
  status: 'connected' | 'disconnected' | 'connecting' | 'error'
  uptime: number
  messageCount: number
  errorCount: number
  lastUpdate: Date
  metrics: Record<string, any>
}

interface DriverTemplate {
  id: string
  name: string
  description: string
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
const expandedSections = ref(['performance'])

const basicFormRef = ref()
const selectedTemplate = ref<DriverTemplate | null>(null)

// 表单数据
const driverForm = ref<DriverForm>({
  id: '',
  name: '',
  description: '',
  protocol: 'modbus_tcp',
  connectionType: 'ethernet',
  enabled: true,
  config: {},
  performance: {
    scanInterval: 1000,
    timeout: 5000,
    maxConcurrentRequests: 10,
    batchSize: 100
  },
  retry: {
    maxRetries: 3,
    retryInterval: 1000,
    exponentialBackoff: true,
    maxRetryInterval: 10000
  },
  logging: {
    level: 'info',
    enableRequestLog: false,
    enableResponseLog: false,
    maxLogSize: 10
  },
  security: {
    enableSsl: false,
    verifyCertificate: true
  }
})

const testResult = ref<TestResult>({
  success: false,
  message: '',
  timestamp: new Date(),
  responseTime: 0
})

const driverStatus = ref<DriverStatus>({
  status: 'disconnected',
  uptime: 0,
  messageCount: 0,
  errorCount: 0,
  lastUpdate: new Date(),
  metrics: {}
})

// 模拟模板数据
const driverTemplates = ref<DriverTemplate[]>([
  {
    id: '1',
    name: 'Modbus TCP 标准模板',
    description: '标准的Modbus TCP连接模板，适用于大多数PLC设备',
    protocol: 'Modbus TCP',
    config: {
      host: '',
      port: 502,
      unitId: 1,
      timeout: 5000
    },
    configItems: ['主机地址', '端口号', '单元ID', '超时时间']
  },
  {
    id: '2',
    name: 'OPC UA 安全连接',
    description: '带安全认证的OPC UA连接模板',
    protocol: 'OPC UA',
    config: {
      endpoint: 'opc.tcp://localhost:4840',
      securityPolicy: 'Basic256Sha256',
      messageSecurityMode: 'SignAndEncrypt'
    },
    configItems: ['端点URL', '安全策略', '消息安全模式', '用户认证']
  },
  {
    id: '3',
    name: 'Modbus RTU 串口',
    description: 'Modbus RTU串口连接模板',
    protocol: 'Modbus RTU',
    config: {
      serialPort: 'COM1',
      baudRate: 9600,
      dataBits: 8,
      parity: 'None',
      stopBits: 1
    },
    configItems: ['串口号', '波特率', '数据位', '校验位', '停止位']
  }
])

// 计算属性
const isEditing = computed(() => !!route.params.id && route.params.id !== 'new')

const protocolConfig = computed(() => {
  const configs = {
    modbus_tcp: { title: 'Modbus TCP 配置', protocol: 'Modbus TCP' },
    modbus_rtu: { title: 'Modbus RTU 配置', protocol: 'Modbus RTU' },
    opcua: { title: 'OPC UA 配置', protocol: 'OPC UA' },
    mqtt: { title: 'MQTT 配置', protocol: 'MQTT' },
    ethernet_ip: { title: 'EtherNet/IP 配置', protocol: 'EtherNet/IP' },
    bacnet: { title: 'BACnet 配置', protocol: 'BACnet' }
  }
  return configs[driverForm.value.protocol] || configs.modbus_tcp
})

// 表单字段配置
const basicFields = computed(() => [
  {
    name: 'name',
    label: '驱动名称',
    type: 'input',
    required: true,
    placeholder: '请输入驱动名称'
  },
  {
    name: 'description',
    label: '驱动描述',
    type: 'textarea',
    placeholder: '请输入驱动描述（可选）'
  },
  {
    name: 'protocol',
    label: '通信协议',
    type: 'select',
    required: true,
    options: [
      { label: 'Modbus TCP', value: 'modbus_tcp' },
      { label: 'Modbus RTU', value: 'modbus_rtu' },
      { label: 'OPC UA', value: 'opcua' },
      { label: 'MQTT', value: 'mqtt' },
      { label: 'EtherNet/IP', value: 'ethernet_ip' },
      { label: 'BACnet', value: 'bacnet' }
    ]
  },
  {
    name: 'connectionType',
    label: '连接方式',
    type: 'select',
    required: true,
    options: [
      { label: '以太网', value: 'ethernet' },
      { label: '串口', value: 'serial' }
    ]
  },
  {
    name: 'enabled',
    label: '启用状态',
    type: 'switch',
    activeText: '启用',
    inactiveText: '禁用'
  }
])

const performanceFields = [
  {
    name: 'performance.scanInterval',
    label: '扫描间隔(ms)',
    type: 'number',
    min: 100,
    max: 60000,
    placeholder: '数据扫描间隔'
  },
  {
    name: 'performance.timeout',
    label: '请求超时(ms)',
    type: 'number',
    min: 1000,
    max: 30000,
    placeholder: '单次请求超时时间'
  },
  {
    name: 'performance.maxConcurrentRequests',
    label: '最大并发数',
    type: 'number',
    min: 1,
    max: 50,
    placeholder: '最大同时请求数量'
  },
  {
    name: 'performance.batchSize',
    label: '批次大小',
    type: 'number',
    min: 1,
    max: 1000,
    placeholder: '批量读取数据点数量'
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

const loggingFields = [
  {
    name: 'logging.level',
    label: '日志级别',
    type: 'select',
    options: [
      { label: '错误 (Error)', value: 'error' },
      { label: '警告 (Warn)', value: 'warn' },
      { label: '信息 (Info)', value: 'info' },
      { label: '调试 (Debug)', value: 'debug' },
      { label: '跟踪 (Trace)', value: 'trace' }
    ]
  },
  {
    name: 'logging.enableRequestLog',
    label: '记录请求日志',
    type: 'switch'
  },
  {
    name: 'logging.enableResponseLog',
    label: '记录响应日志',
    type: 'switch'
  },
  {
    name: 'logging.maxLogSize',
    label: '最大日志大小(MB)',
    type: 'number',
    min: 1,
    max: 100
  }
]

const securityFields = [
  {
    name: 'security.enableSsl',
    label: '启用SSL/TLS',
    type: 'switch'
  },
  {
    name: 'security.verifyCertificate',
    label: '验证证书',
    type: 'switch'
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
  }
]

// 验证规则
const formRules = {
  name: [
    { required: true, message: '请输入驱动名称', trigger: 'blur' },
    { min: 2, max: 50, message: '名称长度应在2-50字符之间', trigger: 'blur' }
  ],
  protocol: [
    { required: true, message: '请选择通信协议', trigger: 'change' }
  ],
  connectionType: [
    { required: true, message: '请选择连接方式', trigger: 'change' }
  ]
}

const performanceRules = {
  'performance.scanInterval': [
    { required: true, message: '请输入扫描间隔', trigger: 'blur' }
  ],
  'performance.timeout': [
    { required: true, message: '请输入请求超时时间', trigger: 'blur' }
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
  // 配置验证规则
  required: ['name', 'protocol', 'config.host', 'config.port'],
  ranges: {
    'config.port': { min: 1, max: 65535 },
    'performance.scanInterval': { min: 100, max: 60000 },
    'performance.timeout': { min: 1000, max: 30000 }
  }
}))

// 方法
const formatNumber = (num: number) => {
  return num.toLocaleString()
}

const formatTime = (time: Date) => {
  return time.toLocaleString('zh-CN')
}

const formatUptime = (seconds: number) => {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  return `${hours}h ${minutes}m`
}

const formatConfigJson = () => {
  try {
    return JSON.stringify(driverForm.value, null, 2)
  } catch (error) {
    return '配置格式错误'
  }
}

const formatConfigYaml = () => {
  // 简化的YAML格式化
  try {
    const obj = driverForm.value
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

const getProtocolIcon = (protocol: string) => {
  const icons: Record<string, string> = {
    'Modbus TCP': 'Connection',
    'Modbus RTU': 'Link',
    'OPC UA': 'Setting',
    'MQTT': 'Connection',
    'EtherNet/IP': 'Link',
    'BACnet': 'Setting'
  }
  return icons[protocol] || 'Setting'
}

const getProtocolTagType = (protocol: string) => {
  const types: Record<string, string> = {
    'Modbus TCP': 'primary',
    'Modbus RTU': 'success',
    'OPC UA': 'warning',
    'MQTT': 'info',
    'EtherNet/IP': 'danger',
    'BACnet': 'primary'
  }
  return types[protocol] || 'info'
}

// 事件处理
const handleGoBack = () => {
  router.push('/drivers')
}

const handleSave = async () => {
  // 验证表单
  try {
    await basicFormRef.value?.validate()
  } catch (error) {
    ElMessage.error('请检查表单填写')
    return
  }
  
  saving.value = true
  
  try {
    if (isEditing.value) {
      // 更新驱动配置
      const updateReq = {
        name: driverForm.value.name,
        description: driverForm.value.description,
        protocol: driverForm.value.protocol,
        connection_type: driverForm.value.connectionType,
        enabled: driverForm.value.enabled,
        config: driverForm.value.config,
        scan_interval: driverForm.value.performance.scanInterval,
        timeout: driverForm.value.performance.timeout,
        max_concurrent: driverForm.value.performance.maxConcurrentRequests,
        batch_size: driverForm.value.performance.batchSize,
        max_retries: driverForm.value.retry.maxRetries,
        retry_interval: driverForm.value.retry.retryInterval,
        exponential_backoff: driverForm.value.retry.exponentialBackoff,
        max_retry_interval: driverForm.value.retry.maxRetryInterval,
        log_level: driverForm.value.logging.level,
        enable_request_log: driverForm.value.logging.enableRequestLog,
        enable_response_log: driverForm.value.logging.enableResponseLog,
        max_log_size: driverForm.value.logging.maxLogSize,
        enable_ssl: driverForm.value.security.enableSsl,
        verify_certificate: driverForm.value.security.verifyCertificate,
        client_cert_path: driverForm.value.security.clientCertPath,
        client_key_path: driverForm.value.security.clientKeyPath,
      }
      
      await driverConfigsApi.update(driverForm.value.id, updateReq)
      ElMessage.success('驱动配置已更新')
    } else {
      // 创建驱动配置
      const createReq = {
        name: driverForm.value.name,
        description: driverForm.value.description,
        protocol: driverForm.value.protocol,
        connection_type: driverForm.value.connectionType,
        enabled: driverForm.value.enabled,
        config: driverForm.value.config,
        scan_interval: driverForm.value.performance.scanInterval,
        timeout: driverForm.value.performance.timeout,
        max_concurrent: driverForm.value.performance.maxConcurrentRequests,
        batch_size: driverForm.value.performance.batchSize,
        max_retries: driverForm.value.retry.maxRetries,
        retry_interval: driverForm.value.retry.retryInterval,
        exponential_backoff: driverForm.value.retry.exponentialBackoff,
        max_retry_interval: driverForm.value.retry.maxRetryInterval,
        log_level: driverForm.value.logging.level,
        enable_request_log: driverForm.value.logging.enableRequestLog,
        enable_response_log: driverForm.value.logging.enableResponseLog,
        max_log_size: driverForm.value.logging.maxLogSize,
        enable_ssl: driverForm.value.security.enableSsl,
        verify_certificate: driverForm.value.security.verifyCertificate,
        client_cert_path: driverForm.value.security.clientCertPath,
        client_key_path: driverForm.value.security.clientKeyPath,
      }
      
      const response = await driverConfigsApi.create(createReq)
      driverForm.value.id = response.driver_config.id
      ElMessage.success('驱动创建成功')
    }
    
    // 返回列表
    setTimeout(() => {
      router.push('/drivers')
    }, 1000)
    
  } catch (error: any) {
    console.error('保存驱动配置失败:', error)
    const errorMessage = error?.response?.data?.message || error?.message || '保存失败，请重试'
    ElMessage.error(errorMessage)
  } finally {
    saving.value = false
  }
}

const handleTestConnection = async () => {
  testing.value = true
  
  try {
    // 模拟连接测试
    await new Promise(resolve => setTimeout(resolve, 3000))
    
    const success = Math.random() > 0.3
    testResult.value = {
      success,
      message: success 
        ? '连接测试成功，设备响应正常' 
        : '连接测试失败，请检查网络和配置',
      timestamp: new Date(),
      responseTime: Math.floor(Math.random() * 500) + 100,
      protocolVersion: success ? 'v1.0' : undefined,
      deviceInfo: success ? 'Device Model: ABC-123' : undefined,
      logs: success 
        ? '连接建立...\n握手成功\n读取设备信息\n测试完成'
        : '连接尝试...\n连接超时\n无法建立连接\n测试失败'
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
  Object.assign(driverForm.value.config, template.config)
  
  // 根据协议设置其他字段
  if (template.protocol === 'Modbus TCP') {
    driverForm.value.protocol = 'modbus_tcp'
    driverForm.value.connectionType = 'ethernet'
  } else if (template.protocol === 'Modbus RTU') {
    driverForm.value.protocol = 'modbus_rtu'
    driverForm.value.connectionType = 'serial'
  } else if (template.protocol === 'OPC UA') {
    driverForm.value.protocol = 'opcua'
    driverForm.value.connectionType = 'ethernet'
  }
  
  templateDialogVisible.value = false
  selectedTemplate.value = null
  
  ElMessage.success(`已应用模板：${template.name}`)
}

const handleProtocolValidate = (valid: boolean) => {
  // 协议配置验证回调
  if (!valid) {
    ElMessage.warning('协议配置不完整，请检查必填项')
  }
}

const handleExportConfig = () => {
  const exportData = {
    driver: driverForm.value,
    exportTime: new Date().toISOString(),
    version: '1.0'
  }
  
  const blob = new Blob([JSON.stringify(exportData, null, 2)], {
    type: 'application/json'
  })
  
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `driver_${driverForm.value.name || 'config'}_${new Date().toISOString().split('T')[0]}.json`
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
        if (data.driver) {
          Object.assign(driverForm.value, data.driver)
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
    Object.assign(driverForm.value, {
      id: '',
      name: '',
      description: '',
      protocol: 'modbus_tcp',
      connectionType: 'ethernet',
      enabled: true,
      config: {},
      performance: {
        scanInterval: 1000,
        timeout: 5000,
        maxConcurrentRequests: 10,
        batchSize: 100
      },
      retry: {
        maxRetries: 3,
        retryInterval: 1000,
        exponentialBackoff: true,
        maxRetryInterval: 10000
      },
      logging: {
        level: 'info',
        enableRequestLog: false,
        enableResponseLog: false,
        maxLogSize: 10
      },
      security: {
        enableSsl: false,
        verifyCertificate: true
      }
    })
    
    ElMessage.success('表单已重置')
  })
}

const refreshPreview = () => {
  ElMessage.info('配置预览已刷新')
}

// 监听协议变化，更新配置预设
watch(() => driverForm.value.protocol, (newProtocol) => {
  // 根据协议类型设置默认配置
  const defaultConfigs = {
    modbus_tcp: {
      host: '',
      port: 502,
      unitId: 1
    },
    modbus_rtu: {
      serialPort: 'COM1',
      baudRate: 9600,
      unitId: 1
    },
    opcua: {
      endpoint: 'opc.tcp://localhost:4840',
      securityPolicy: 'None'
    },
    mqtt: {
      broker: 'mqtt://localhost:1883',
      topic: 'data/#'
    },
    ethernet_ip: {
      host: '',
      port: 44818
    },
    bacnet: {
      host: '',
      port: 47808,
      deviceId: 1001
    }
  }
  
  if (!Object.keys(driverForm.value.config).length) {
    driverForm.value.config = defaultConfigs[newProtocol] || {}
  }
}, { immediate: true })

// 生命周期
onMounted(async () => {
  if (isEditing.value) {
    // 模拟加载现有驱动配置
    const driverId = route.params.id as string
    
    // 模拟驱动数据
    const mockDriver = {
      id: driverId,
      name: 'PLC主控制器',
      description: '生产线主PLC控制器',
      protocol: 'modbus_tcp' as const,
      connectionType: 'ethernet' as const,
      enabled: true,
      config: {
        host: '',
        port: 502,
        unitId: 1
      },
      performance: {
        scanInterval: 1000,
        timeout: 5000,
        maxConcurrentRequests: 10,
        batchSize: 100
      },
      retry: {
        maxRetries: 3,
        retryInterval: 1000,
        exponentialBackoff: true,
        maxRetryInterval: 10000
      },
      logging: {
        level: 'info' as const,
        enableRequestLog: true,
        enableResponseLog: false,
        maxLogSize: 10
      },
      security: {
        enableSsl: false,
        verifyCertificate: true
      }
    }
    
    Object.assign(driverForm.value, mockDriver)
    
    // 模拟驱动状态
    driverStatus.value = {
      status: 'connected',
      uptime: 86400, // 1 day
      messageCount: 125670,
      errorCount: 3,
      lastUpdate: new Date(),
      metrics: {
        readRate: 850,
        writeRate: 120,
        avgLatency: 45
      }
    }
  }
})
</script>