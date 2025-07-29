<template>
  <div class="protocol-config" :class="containerClass">
    <!-- 配置头部 -->
    <div class="config-header">
      <div class="header-title">
        <el-icon :size="18">
          <Setting />
        </el-icon>
        <span>{{ title }}</span>
        <el-tag v-if="protocolType" :type="getProtocolTagType(protocolType)">
          {{ getProtocolDisplayName(protocolType) }}
        </el-tag>
      </div>
      
      <div class="header-actions">
        <!-- 配置模板 -->
        <el-dropdown v-if="showTemplates" trigger="click" @command="handleTemplate">
          <el-button type="link" size="small">
            配置模板 <el-icon><ArrowDown /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="save">保存为模板</el-dropdown-item>
              <el-dropdown-item command="load">从模板加载</el-dropdown-item>
              <el-dropdown-item command="import">导入配置</el-dropdown-item>
              <el-dropdown-item command="export">导出配置</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        
        <!-- 帮助按钮 -->
        <el-button
          v-if="showHelp"
          type="link"
          size="small"
          :icon="QuestionFilled"
          @click="handleHelp"
        />
      </div>
    </div>
    
    <!-- 协议选择 -->
    <div class="protocol-selector">
      <el-form-item label="协议类型" required>
        <el-select
          v-model="currentProtocolType"
          placeholder="请选择协议类型"
          @change="handleProtocolChange"
          :disabled="protocolLocked"
        >
          <el-option
            v-for="protocol in supportedProtocols"
            :key="protocol.value"
            :label="protocol.label"
            :value="protocol.value"
          >
            <div class="protocol-option">
              <el-icon>
                <component :is="protocol.icon" />
              </el-icon>
              <span>{{ protocol.label }}</span>
              <el-tag size="small" :type="protocol.tagType">
                {{ protocol.category }}
              </el-tag>
            </div>
          </el-option>
        </el-select>
      </el-form-item>
      
      <div v-if="protocolDescription" class="protocol-description">
        <el-alert
          :title="protocolDescription.title"
          :description="protocolDescription.content"
          type="info"
          :show-icon="true"
          :closable="false"
          effect="light"
        />
      </div>
    </div>
    
    <!-- 动态配置表单 -->
    <div class="config-form" v-if="currentProtocolType">
      <el-tabs v-model="activeTab" type="border-card">
        <!-- 基本配置 -->
        <el-tab-pane label="基本配置" name="basic">
          <BaseForm
            ref="basicFormRef"
            v-model="config.basic"
            :fields="basicFields"
            :rules="basicRules"
            label-width="120px"
            @change="handleConfigChange"
          />
        </el-tab-pane>
        
        <!-- 连接配置 -->
        <el-tab-pane label="连接配置" name="connection">
          <BaseForm
            ref="connectionFormRef"
            v-model="config.connection"
            :fields="connectionFields"
            :rules="connectionRules"
            label-width="120px"
            @change="handleConfigChange"
          />
        </el-tab-pane>
        
        <!-- 安全配置 -->
        <el-tab-pane v-if="hasSecurityConfig" label="安全配置" name="security">
          <BaseForm
            ref="securityFormRef"
            v-model="config.security"
            :fields="securityFields"
            :rules="securityRules"
            label-width="120px"
            @change="handleConfigChange"
          />
        </el-tab-pane>
        
        <!-- 高级配置 -->
        <el-tab-pane label="高级配置" name="advanced">
          <BaseForm
            ref="advancedFormRef"
            v-model="config.advanced"
            :fields="advancedFields"
            :rules="advancedRules"
            label-width="120px"
            @change="handleConfigChange"
          />
        </el-tab-pane>
        
        <!-- 数据点映射 -->
        <el-tab-pane v-if="hasDataPointMapping" label="数据点映射" name="mapping">
          <div class="mapping-config">
            <div class="mapping-header">
              <el-button type="primary" size="small" @click="handleAddMapping">
                添加映射
              </el-button>
              <el-button size="small" @click="handleImportMapping">
                导入映射
              </el-button>
              <el-button size="small" @click="handleExportMapping">
                导出映射
              </el-button>
            </div>
            
            <BaseTable
              :data="dataPointMappings"
              :columns="mappingColumns"
              :pagination="false"
              @action="handleMappingAction"
            />
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
    
    <!-- 配置验证和测试 -->
    <div class="config-validation" v-if="showValidation && currentProtocolType">
      <el-card shadow="never" class="validation-card">
        <template #header>
          <div class="validation-header">
            <span>配置验证</span>
            <el-button
              type="primary"
              size="small"
              :loading="testing"
              @click="handleTest"
            >
              测试连接
            </el-button>
          </div>
        </template>
        
        <div class="validation-content">
          <!-- 验证结果 -->
          <div v-if="validationResult" class="validation-result">
            <el-alert
              :title="validationResult.title"
              :type="validationResult.type"
              :description="validationResult.message"
              :show-icon="true"
              :closable="false"
            />
          </div>
          
          <!-- 配置摘要 -->
          <div class="config-summary">
            <el-descriptions title="配置摘要" border column="2">
              <el-descriptions-item
                v-for="(value, key) in configSummary"
                :key="key"
                :label="key"
              >
                {{ value }}
              </el-descriptions-item>
            </el-descriptions>
          </div>
        </div>
      </el-card>
    </div>
    
    <!-- 操作按钮 -->
    <div class="config-actions" v-if="showActions">
      <el-button @click="handleReset">重置</el-button>
      <el-button @click="handleCancel">取消</el-button>
      <el-button
        type="primary"
        :loading="saving"
        @click="handleSave"
        :disabled="!isConfigValid"
      >
        保存配置
      </el-button>
    </div>
    
    <!-- 配置模板对话框 -->
    <el-dialog
      v-model="templateVisible"
      title="配置模板管理"
      width="600px"
      :before-close="handleTemplateClose"
    >
      <div class="template-manager">
        <el-tabs v-model="templateTab">
          <el-tab-pane label="保存模板" name="save">
            <el-form label-width="80px">
              <el-form-item label="模板名称">
                <el-input
                  v-model="templateName"
                  placeholder="输入模板名称"
                />
              </el-form-item>
              <el-form-item label="模板描述">
                <el-input
                  v-model="templateDescription"
                  type="textarea"
                  placeholder="输入模板描述"
                  :rows="3"
                />
              </el-form-item>
            </el-form>
          </el-tab-pane>
          
          <el-tab-pane label="加载模板" name="load">
            <BaseTable
              :data="configTemplates"
              :columns="templateColumns"
              @action="handleTemplateAction"
            />
          </el-tab-pane>
        </el-tabs>
      </div>
      
      <template #footer>
        <el-button @click="templateVisible = false">取消</el-button>
        <el-button
          v-if="templateTab === 'save'"
          type="primary"
          @click="handleSaveTemplate"
        >
          保存模板
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { 
  Setting, 
  ArrowDown, 
  QuestionFilled,
  Connection,
  Monitor,
  Link,
  Lock
} from '@element-plus/icons-vue'
import { BaseForm, BaseTable } from '../base'
import { ElMessage } from 'element-plus'

export type ProtocolType = 'modbus_tcp' | 'modbus_rtu' | 'opcua' | 'mqtt5' | 'ethernet_ip' | 'profinet'

export interface ProtocolConfig {
  basic: Record<string, any>
  connection: Record<string, any>
  security: Record<string, any>
  advanced: Record<string, any>
}

export interface DataPointMapping {
  id: string
  name: string
  address: string
  dataType: string
  scaling?: number
  offset?: number
  unit?: string
}

interface Props {
  modelValue?: ProtocolConfig
  protocolType?: ProtocolType
  title?: string
  
  // 功能控制
  showTemplates?: boolean
  showHelp?: boolean
  showValidation?: boolean
  showActions?: boolean
  protocolLocked?: boolean
  
  // 外观配置
  size?: 'small' | 'default' | 'large'
  
  // 自定义样式
  customClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: '协议配置',
  showTemplates: true,
  showHelp: true,
  showValidation: true,
  showActions: true,
  protocolLocked: false,
  size: 'default',
})

interface Emits {
  'update:modelValue': [config: ProtocolConfig]
  'protocol-change': [protocolType: ProtocolType]
  save: [config: ProtocolConfig]
  test: [config: ProtocolConfig]
  cancel: []
}

const emit = defineEmits<Emits>()

// 状态
const currentProtocolType = ref<ProtocolType | ''>('')
const activeTab = ref('basic')
const config = ref<ProtocolConfig>({
  basic: {},
  connection: {},
  security: {},
  advanced: {}
})

const testing = ref(false)
const saving = ref(false)
const validationResult = ref<any>(null)
const templateVisible = ref(false)
const templateTab = ref('save')
const templateName = ref('')
const templateDescription = ref('')

// 表单引用
const basicFormRef = ref()
const connectionFormRef = ref()
const securityFormRef = ref()
const advancedFormRef = ref()

// 数据点映射
const dataPointMappings = ref<DataPointMapping[]>([])

// 支持的协议
const supportedProtocols = [
  {
    value: 'modbus_tcp',
    label: 'Modbus TCP',
    category: '以太网',
    tagType: 'primary',
    icon: Link
  },
  {
    value: 'modbus_rtu',
    label: 'Modbus RTU',
    category: '串口',
    tagType: 'success',
    icon: Connection
  },
  {
    value: 'opcua',
    label: 'OPC UA',
    category: '工业标准',
    tagType: 'warning',
    icon: Monitor
  },
  {
    value: 'mqtt5',
    label: 'MQTT 5.0',
    category: '物联网',
    tagType: 'info',
    icon: Link
  },
  {
    value: 'ethernet_ip',
    label: 'Ethernet/IP',
    category: '以太网',
    tagType: 'primary',
    icon: Link
  },
  {
    value: 'profinet',
    label: 'PROFINET',
    category: '工业以太网',
    tagType: 'danger',
    icon: Link
  }
]

// 配置模板
const configTemplates = ref([
  {
    id: '1',
    name: 'Modbus TCP 默认配置',
    protocol: 'modbus_tcp',
    description: '标准 Modbus TCP 配置模板',
    createTime: new Date()
  }
])

// 计算属性
const containerClass = computed(() => {
  const classes = []
  
  classes.push(`protocol-config--${props.size}`)
  
  if (props.customClass) {
    classes.push(props.customClass)
  }
  
  return classes.join(' ')
})

const protocolDescription = computed(() => {
  if (!currentProtocolType.value) return null
  
  const descriptions = {
    modbus_tcp: {
      title: 'Modbus TCP',
      content: 'Modbus TCP是基于以太网的工业通信协议，支持标准的TCP/IP网络传输。'
    },
    modbus_rtu: {
      title: 'Modbus RTU',
      content: 'Modbus RTU是基于串口的工业通信协议，使用二进制传输，具有较高的传输效率。'
    },
    opcua: {
      title: 'OPC UA',
      content: 'OPC UA是现代工业通信标准，提供安全、可靠的数据交换和互操作性。'
    },
    mqtt5: {
      title: 'MQTT 5.0',
      content: 'MQTT 5.0是轻量级的物联网通信协议，支持发布/订阅模式的消息传递。'
    },
    ethernet_ip: {
      title: 'Ethernet/IP',
      content: 'Ethernet/IP是基于标准以太网的工业通信协议，广泛应用于自动化系统。'
    },
    profinet: {
      title: 'PROFINET',
      content: 'PROFINET是基于以太网的工业通信标准，提供实时数据交换能力。'
    }
  }
  
  return descriptions[currentProtocolType.value]
})

const hasSecurityConfig = computed(() => {
  return ['opcua', 'mqtt5'].includes(currentProtocolType.value)
})

const hasDataPointMapping = computed(() => {
  return ['modbus_tcp', 'modbus_rtu', 'opcua'].includes(currentProtocolType.value)
})

const basicFields = computed(() => {
  if (!currentProtocolType.value) return []
  
  const commonFields = [
    {
      key: 'name',
      label: '连接名称',
      type: 'text',
      required: true,
      placeholder: '输入连接名称'
    },
    {
      key: 'description',
      label: '描述',
      type: 'textarea',
      placeholder: '输入连接描述'
    }
  ]
  
  const protocolSpecificFields = {
    modbus_tcp: [
      {
        key: 'slaveId',
        label: '从站ID',
        type: 'number',
        required: true,
        min: 1,
        max: 255,
        defaultValue: 1
      }
    ],
    modbus_rtu: [
      {
        key: 'slaveId',
        label: '从站ID',
        type: 'number',
        required: true,
        min: 1,
        max: 255,
        defaultValue: 1
      },
      {
        key: 'baudRate',
        label: '波特率',
        type: 'select',
        required: true,
        options: [
          { label: '9600', value: 9600 },
          { label: '19200', value: 19200 },
          { label: '38400', value: 38400 },
          { label: '115200', value: 115200 }
        ],
        defaultValue: 9600
      },
      {
        key: 'parity',
        label: '校验位',
        type: 'select',
        required: true,
        options: [
          { label: '无校验', value: 'none' },
          { label: '奇校验', value: 'odd' },
          { label: '偶校验', value: 'even' }
        ],
        defaultValue: 'none'
      }
    ],
    opcua: [
      {
        key: 'endpointUrl',
        label: '端点URL',
        type: 'text',
        required: true,
        placeholder: 'opc.tcp://localhost:4840'
      }
    ],
    mqtt5: [
      {
        key: 'clientId',
        label: '客户端ID',
        type: 'text',
        required: true,
        placeholder: '输入客户端ID'
      },
      {
        key: 'keepAlive',
        label: '保活时间',
        type: 'number',
        unit: '秒',
        defaultValue: 60
      }
    ]
  }
  
  return [...commonFields, ...(protocolSpecificFields[currentProtocolType.value] || [])]
})

const connectionFields = computed(() => {
  if (!currentProtocolType.value) return []
  
  const tcpFields = [
    {
      key: 'host',
      label: 'IP地址',
      type: 'text',
      required: true,
      placeholder: '192.168.1.100'
    },
    {
      key: 'port',
      label: '端口',
      type: 'number',
      required: true,
      min: 1,
      max: 65535
    }
  ]
  
  const serialFields = [
    {
      key: 'port',
      label: '串口',
      type: 'select',
      required: true,
      options: [
        { label: 'COM1', value: 'COM1' },
        { label: 'COM2', value: 'COM2' },
        { label: 'COM3', value: 'COM3' },
        { label: '/dev/ttyUSB0', value: '/dev/ttyUSB0' },
        { label: '/dev/ttyS0', value: '/dev/ttyS0' }
      ]
    }
  ]
  
  const fieldMap = {
    modbus_tcp: tcpFields,
    modbus_rtu: serialFields,
    opcua: tcpFields,
    mqtt5: tcpFields,
    ethernet_ip: tcpFields,
    profinet: tcpFields
  }
  
  return fieldMap[currentProtocolType.value] || []
})

const securityFields = computed(() => {
  if (!hasSecurityConfig.value) return []
  
  const commonFields = [
    {
      key: 'username',
      label: '用户名',
      type: 'text',
      placeholder: '输入用户名'
    },
    {
      key: 'password',
      label: '密码',
      type: 'password',
      placeholder: '输入密码'
    }
  ]
  
  const protocolSpecificFields = {
    opcua: [
      {
        key: 'securityMode',
        label: '安全模式',
        type: 'select',
        options: [
          { label: '无安全', value: 'None' },
          { label: '签名', value: 'Sign' },
          { label: '签名并加密', value: 'SignAndEncrypt' }
        ],
        defaultValue: 'None'
      }
    ],
    mqtt5: [
      {
        key: 'useTLS',
        label: '启用TLS',
        type: 'switch',
        defaultValue: false
      }
    ]
  }
  
  return [...commonFields, ...(protocolSpecificFields[currentProtocolType.value] || [])]
})

const advancedFields = computed(() => [
  {
    key: 'timeout',
    label: '超时时间',
    type: 'number',
    unit: '毫秒',
    defaultValue: 5000,
    min: 1000,
    max: 30000
  },
  {
    key: 'retryCount',
    label: '重试次数',
    type: 'number',
    defaultValue: 3,
    min: 0,
    max: 10
  },
  {
    key: 'retryInterval',
    label: '重试间隔',
    type: 'number',
    unit: '毫秒',
    defaultValue: 1000,
    min: 100,
    max: 10000
  },
  {
    key: 'enableLogging',
    label: '启用日志',
    type: 'switch',
    defaultValue: true
  }
])

const mappingColumns = [
  { key: 'name', label: '数据点名称', width: 150 },
  { key: 'address', label: '地址', width: 120 },
  { key: 'dataType', label: '数据类型', width: 100 },
  { key: 'unit', label: '单位', width: 80 },
  { key: 'actions', label: '操作', width: 120, type: 'action' }
]

const templateColumns = [
  { key: 'name', label: '模板名称' },
  { key: 'protocol', label: '协议类型' },
  { key: 'description', label: '描述' },
  { key: 'createTime', label: '创建时间', type: 'datetime' },
  { key: 'actions', label: '操作', type: 'action' }
]

const basicRules = computed(() => ({
  name: [{ required: true, message: '请输入连接名称', trigger: 'blur' }]
}))

const connectionRules = computed(() => {
  const rules: any = {}
  
  if (['modbus_tcp', 'opcua', 'mqtt5'].includes(currentProtocolType.value)) {
    rules.host = [{ required: true, message: '请输入IP地址', trigger: 'blur' }]
    rules.port = [{ required: true, message: '请输入端口', trigger: 'blur' }]
  }
  
  return rules
})

const securityRules = computed(() => ({}))

const advancedRules = computed(() => ({}))

const configSummary = computed(() => {
  const summary: Record<string, any> = {}
  
  summary['协议类型'] = getProtocolDisplayName(currentProtocolType.value)
  
  if (config.value.basic.name) {
    summary['连接名称'] = config.value.basic.name
  }
  
  if (config.value.connection.host && config.value.connection.port) {
    summary['连接地址'] = `${config.value.connection.host}:${config.value.connection.port}`
  }
  
  if (config.value.advanced.timeout) {
    summary['超时时间'] = `${config.value.advanced.timeout}ms`
  }
  
  return summary
})

const isConfigValid = computed(() => {
  // 简单验证逻辑
  return currentProtocolType.value && config.value.basic.name
})

// 方法
const getProtocolDisplayName = (type: string) => {
  const protocol = supportedProtocols.find(p => p.value === type)
  return protocol?.label || type
}

const getProtocolTagType = (type: string) => {
  const protocol = supportedProtocols.find(p => p.value === type)
  return protocol?.tagType || 'info'
}

// 事件处理
const handleProtocolChange = (type: ProtocolType) => {
  currentProtocolType.value = type
  
  // 重置配置
  config.value = {
    basic: {},
    connection: {},
    security: {},
    advanced: {}
  }
  
  // 设置默认值
  setDefaultValues()
  
  emit('protocol-change', type)
}

const setDefaultValues = () => {
  // 设置基本配置默认值
  basicFields.value.forEach(field => {
    if (field.defaultValue !== undefined) {
      config.value.basic[field.key] = field.defaultValue
    }
  })
  
  // 设置连接配置默认端口
  const defaultPorts = {
    modbus_tcp: 502,
    opcua: 4840,
    mqtt5: 1883,
    ethernet_ip: 44818,
    profinet: 102
  }
  
  if (defaultPorts[currentProtocolType.value]) {
    config.value.connection.port = defaultPorts[currentProtocolType.value]
  }
  
  // 设置高级配置默认值
  advancedFields.value.forEach(field => {
    if (field.defaultValue !== undefined) {
      config.value.advanced[field.key] = field.defaultValue
    }
  })
}

const handleConfigChange = () => {
  emit('update:modelValue', config.value)
}

const handleTest = async () => {
  testing.value = true
  
  try {
    emit('test', config.value)
    
    // 模拟测试结果
    setTimeout(() => {
      validationResult.value = {
        type: 'success',
        title: '连接测试成功',
        message: '配置参数正确，连接正常'
      }
      testing.value = false
    }, 2000)
  } catch (error) {
    validationResult.value = {
      type: 'error',
      title: '连接测试失败',
      message: '请检查配置参数'
    }
    testing.value = false
  }
}

const handleSave = async () => {
  // 验证所有表单
  const validations = await Promise.all([
    basicFormRef.value?.validate(),
    connectionFormRef.value?.validate(),
    securityFormRef.value?.validate(),
    advancedFormRef.value?.validate()
  ])
  
  if (validations.every(v => v)) {
    saving.value = true
    
    try {
      emit('save', config.value)
      ElMessage.success('配置保存成功')
    } finally {
      saving.value = false
    }
  }
}

const handleReset = () => {
  config.value = {
    basic: {},
    connection: {},
    security: {},
    advanced: {}
  }
  setDefaultValues()
}

const handleCancel = () => {
  emit('cancel')
}

const handleTemplate = (command: string) => {
  switch (command) {
    case 'save':
    case 'load':
      templateVisible.value = true
      templateTab.value = command
      break
    case 'import':
      // 处理导入
      break
    case 'export':
      // 处理导出
      break
  }
}

const handleSaveTemplate = () => {
  // 保存模板逻辑
  ElMessage.success('模板保存成功')
  templateVisible.value = false
}

const handleTemplateClose = () => {
  templateVisible.value = false
  templateName.value = ''
  templateDescription.value = ''
}

const handleTemplateAction = (action: string, row: any) => {
  if (action === 'load') {
    // 加载模板配置
    ElMessage.success('模板加载成功')
    templateVisible.value = false
  }
}

const handleHelp = () => {
  // 显示帮助信息
}

const handleAddMapping = () => {
  // 添加数据点映射
}

const handleImportMapping = () => {
  // 导入映射
}

const handleExportMapping = () => {
  // 导出映射
}

const handleMappingAction = (action: string, row: DataPointMapping) => {
  // 处理映射操作
}

// 监听
watch(() => props.modelValue, (newValue) => {
  if (newValue) {
    config.value = { ...newValue }
  }
}, { immediate: true })

watch(() => props.protocolType, (newType) => {
  if (newType) {
    currentProtocolType.value = newType
    setDefaultValues()
  }
}, { immediate: true })

// 生命周期
onMounted(() => {
  if (props.protocolType) {
    currentProtocolType.value = props.protocolType
    setDefaultValues()
  }
})
</script>

<style scoped lang="scss">
.protocol-config {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 6px;
  
  &.protocol-config--small {
    .config-header {
      padding: 8px 12px;
    }
  }
  
  &.protocol-config--large {
    .config-header {
      padding: 16px 20px;
    }
  }
}

.config-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);
  
  .header-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    color: var(--el-text-color-primary);
  }
  
  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
}

.protocol-selector {
  padding: 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  
  .protocol-option {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .protocol-description {
    margin-top: 12px;
  }
}

.config-form {
  padding: 16px;
  
  :deep(.el-tabs__content) {
    padding-top: 16px;
  }
}

.mapping-config {
  .mapping-header {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }
}

.config-validation {
  margin: 16px;
  
  .validation-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .validation-content {
    .validation-result {
      margin-bottom: 16px;
    }
  }
}

.config-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);
}

.template-manager {
  min-height: 300px;
}

// 响应式设计
@media (max-width: 768px) {
  .protocol-config {
    .config-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }
    
    .config-actions {
      flex-direction: column;
      
      .el-button {
        width: 100%;
      }
    }
  }
}
</style>