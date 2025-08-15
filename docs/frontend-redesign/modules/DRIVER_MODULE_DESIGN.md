# 驱动管理模块设计文档

## 📋 模块概述

驱动管理模块是系统的核心模块，负责管理各种协议驱动的生命周期，包括驱动的安装、配置、启动、停止和删除等操作。

## 🎯 设计目标

1. **简化操作流程**: 一步创建驱动和配置，避免重复输入
2. **智能表单**: 根据协议类型动态显示配置项
3. **实时状态**: 实时显示驱动运行状态和统计信息
4. **批量操作**: 支持批量启动、停止、删除等操作
5. **配置模板**: 提供常用配置模板，快速创建

## 🏗️ 模块结构

```
driver/
├── views/
│   ├── DriverList.vue        # 驱动列表页面
│   ├── DriverCreate.vue      # 创建驱动页面
│   ├── DriverDetail.vue      # 驱动详情页面
│   └── DriverConfig.vue      # 驱动配置页面
├── components/
│   ├── DriverTable.vue       # 驱动列表表格
│   ├── DriverForm.vue        # 驱动表单组件
│   ├── ConfigForm.vue        # 配置表单组件
│   ├── DriverStatus.vue      # 驱动状态显示
│   └── ProtocolSelector.vue  # 协议选择器
├── composables/
│   ├── useDriverList.ts      # 驱动列表逻辑
│   ├── useDriverForm.ts      # 驱动表单逻辑
│   └── useDriverConfig.ts    # 配置逻辑
└── types/
    └── driver.ts             # 类型定义
```

## 📝 数据模型

### 驱动创建流程数据结构

```typescript
// types/driver.ts

// 驱动创建向导数据
export interface DriverWizardData {
  step: 1 | 2 | 3;  // 当前步骤
  driver: DriverBasicInfo;
  config: DriverConfigInfo;
  devices?: DeviceInfo[];  // 可选：同时创建设备
}

// 步骤1: 基本信息
export interface DriverBasicInfo {
  name: string;
  protocol: ProtocolType;
  version?: string;
  description?: string;
}

// 步骤2: 配置信息
export interface DriverConfigInfo {
  name: string;
  template?: string;  // 使用的模板ID
  parameters: Record<string, any>;  // 动态配置参数
  enabled: boolean;
}

// 协议类型枚举
export enum ProtocolType {
  ModbusTCP = 'ModbusTCP',
  ModbusRTU = 'ModbusRTU',
  OPCUA = 'OPC-UA',
  MQTT = 'MQTT',
  BACnet = 'BACnet',
  S7 = 'S7'
}

// 协议配置模板
export interface ProtocolTemplate {
  id: string;
  protocol: ProtocolType;
  name: string;
  description: string;
  schema: ConfigSchema;  // 配置结构定义
  defaults: Record<string, any>;  // 默认值
}

// 配置结构定义
export interface ConfigSchema {
  fields: ConfigField[];
  validation?: ValidationRule[];
}

export interface ConfigField {
  key: string;
  label: string;
  type: 'text' | 'number' | 'select' | 'checkbox' | 'ip' | 'port';
  required?: boolean;
  placeholder?: string;
  options?: Array<{label: string; value: any}>;
  description?: string;
  depends?: string;  // 依赖其他字段
  visible?: (data: any) => boolean;  // 动态显示条件
}
```

## 🎨 UI设计

### 1. 驱动列表页面

```vue
<!-- views/DriverList.vue -->
<template>
  <div class="driver-list-page">
    <!-- 页面头部 -->
    <PageHeader
      title="驱动管理"
      description="管理系统中的协议驱动和配置"
    >
      <template #actions>
        <el-button type="primary" @click="handleCreate">
          <Icon name="plus" /> 创建驱动
        </el-button>
        <el-button @click="handleImport">
          <Icon name="upload" /> 导入驱动
        </el-button>
      </template>
    </PageHeader>

    <!-- 筛选器 -->
    <FilterBar v-model="filters" @search="handleSearch">
      <el-select v-model="filters.protocol" placeholder="协议类型">
        <el-option label="全部" value="" />
        <el-option
          v-for="proto in protocols"
          :key="proto"
          :label="proto"
          :value="proto"
        />
      </el-select>
      <el-select v-model="filters.status" placeholder="状态">
        <el-option label="全部" value="" />
        <el-option label="运行中" value="running" />
        <el-option label="已停止" value="stopped" />
        <el-option label="错误" value="error" />
      </el-select>
    </FilterBar>

    <!-- 驱动表格 -->
    <DriverTable
      :data="driverList"
      :loading="loading"
      @edit="handleEdit"
      @delete="handleDelete"
      @toggle="handleToggle"
      @config="handleConfig"
    />
  </div>
</template>
```

### 2. 创建驱动向导

```vue
<!-- views/DriverCreate.vue -->
<template>
  <div class="driver-create-page">
    <el-card>
      <template #header>
        <h2>创建驱动</h2>
      </template>

      <!-- 步骤条 -->
      <el-steps :active="currentStep" align-center>
        <el-step title="选择协议" />
        <el-step title="基本信息" />
        <el-step title="连接配置" />
        <el-step title="完成" />
      </el-steps>

      <!-- 步骤内容 -->
      <div class="step-content">
        <!-- 步骤1: 选择协议 -->
        <ProtocolSelector
          v-if="currentStep === 0"
          v-model="wizardData.protocol"
          @next="nextStep"
        />

        <!-- 步骤2: 基本信息 -->
        <BasicInfoForm
          v-if="currentStep === 1"
          v-model="wizardData.driver"
          :protocol="wizardData.protocol"
          @prev="prevStep"
          @next="nextStep"
        />

        <!-- 步骤3: 连接配置 -->
        <DynamicConfigForm
          v-if="currentStep === 2"
          v-model="wizardData.config"
          :protocol="wizardData.protocol"
          :templates="configTemplates"
          @prev="prevStep"
          @next="nextStep"
        />

        <!-- 步骤4: 确认创建 -->
        <CreateSummary
          v-if="currentStep === 3"
          :data="wizardData"
          @prev="prevStep"
          @create="handleCreate"
        />
      </div>
    </el-card>
  </div>
</template>
```

### 3. 动态配置表单

```vue
<!-- components/DynamicConfigForm.vue -->
<template>
  <el-form
    ref="formRef"
    :model="formData"
    :rules="formRules"
    label-width="120px"
  >
    <!-- 配置模板选择 -->
    <el-form-item label="配置模板">
      <el-select
        v-model="selectedTemplate"
        placeholder="选择模板或自定义配置"
        @change="applyTemplate"
      >
        <el-option label="自定义配置" value="" />
        <el-option
          v-for="template in templates"
          :key="template.id"
          :label="template.name"
          :value="template.id"
        >
          <span>{{ template.name }}</span>
          <span class="template-desc">{{ template.description }}</span>
        </el-option>
      </el-select>
    </el-form-item>

    <!-- 动态表单字段 -->
    <template v-for="field in visibleFields" :key="field.key">
      <!-- 文本输入 -->
      <el-form-item
        v-if="field.type === 'text'"
        :label="field.label"
        :prop="field.key"
        :required="field.required"
      >
        <el-input
          v-model="formData[field.key]"
          :placeholder="field.placeholder"
        />
        <div class="field-desc">{{ field.description }}</div>
      </el-form-item>

      <!-- IP地址输入 -->
      <el-form-item
        v-else-if="field.type === 'ip'"
        :label="field.label"
        :prop="field.key"
        :required="field.required"
      >
        <IpAddressInput
          v-model="formData[field.key]"
          :placeholder="field.placeholder || '192.168.1.1'"
        />
      </el-form-item>

      <!-- 端口输入 -->
      <el-form-item
        v-else-if="field.type === 'port'"
        :label="field.label"
        :prop="field.key"
        :required="field.required"
      >
        <el-input-number
          v-model="formData[field.key]"
          :min="1"
          :max="65535"
          :placeholder="field.placeholder || '502'"
        />
      </el-form-item>

      <!-- 下拉选择 -->
      <el-form-item
        v-else-if="field.type === 'select'"
        :label="field.label"
        :prop="field.key"
        :required="field.required"
      >
        <el-select v-model="formData[field.key]">
          <el-option
            v-for="opt in field.options"
            :key="opt.value"
            :label="opt.label"
            :value="opt.value"
          />
        </el-select>
      </el-form-item>
    </template>

    <!-- 高级选项（折叠） -->
    <el-collapse v-model="showAdvanced">
      <el-collapse-item title="高级选项" name="advanced">
        <!-- 高级配置字段 -->
      </el-collapse-item>
    </el-collapse>
  </el-form>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useDriverConfig } from '../composables/useDriverConfig'

const props = defineProps<{
  protocol: ProtocolType
  templates: ProtocolTemplate[]
}>()

const { formData, formRules, getFieldsForProtocol } = useDriverConfig(props.protocol)

// 计算可见字段
const visibleFields = computed(() => {
  const fields = getFieldsForProtocol(props.protocol)
  return fields.filter(field => {
    if (field.visible) {
      return field.visible(formData.value)
    }
    return true
  })
})

// 应用模板
function applyTemplate(templateId: string) {
  const template = props.templates.find(t => t.id === templateId)
  if (template) {
    Object.assign(formData.value, template.defaults)
  }
}
</script>
```

## 🔧 业务逻辑

### 1. 驱动创建逻辑

```typescript
// composables/useDriverCreate.ts
export function useDriverCreate() {
  const driverStore = useDriverStore()
  const router = useRouter()
  const { showSuccess, showError } = useNotification()
  
  const wizardData = reactive<DriverWizardData>({
    step: 1,
    driver: {
      name: '',
      protocol: ProtocolType.ModbusTCP,
      version: '1.0.0'
    },
    config: {
      name: '',
      parameters: {},
      enabled: true
    }
  })
  
  // 创建驱动和配置
  async function createDriver() {
    try {
      // 验证数据
      await validateWizardData(wizardData)
      
      // 1. 创建驱动
      const driver = await driverStore.createDriver({
        name: wizardData.driver.name,
        protocol: wizardData.driver.protocol,
        version: wizardData.driver.version
      })
      
      // 2. 创建配置
      const config = await driverStore.createConfig({
        driver_id: driver.id,
        name: wizardData.config.name || `${driver.name} 配置`,
        config: wizardData.config.parameters,
        enabled: wizardData.config.enabled
      })
      
      // 3. 如果选择了创建设备，创建设备
      if (wizardData.devices?.length) {
        for (const device of wizardData.devices) {
          await deviceStore.createDevice({
            ...device,
            driver_id: driver.id,
            config_id: config.id
          })
        }
      }
      
      showSuccess('驱动创建成功')
      router.push(`/drivers/${driver.id}`)
      
    } catch (error) {
      showError('创建失败: ' + error.message)
      throw error
    }
  }
  
  return {
    wizardData,
    createDriver
  }
}
```

### 2. 配置模板管理

```typescript
// utils/configTemplates.ts

// ModbusTCP配置模板
export const modbusTcpTemplates: ProtocolTemplate[] = [
  {
    id: 'modbus-tcp-basic',
    protocol: ProtocolType.ModbusTCP,
    name: '基础ModbusTCP',
    description: '适用于标准ModbusTCP设备',
    schema: {
      fields: [
        {
          key: 'host',
          label: 'IP地址',
          type: 'ip',
          required: true,
          placeholder: '192.168.1.100'
        },
        {
          key: 'port',
          label: '端口',
          type: 'port',
          required: true,
          placeholder: '502'
        },
        {
          key: 'unitId',
          label: '单元ID',
          type: 'number',
          required: true,
          placeholder: '1'
        },
        {
          key: 'timeout',
          label: '超时时间(ms)',
          type: 'number',
          required: false,
          placeholder: '5000'
        },
        {
          key: 'retryCount',
          label: '重试次数',
          type: 'number',
          required: false,
          placeholder: '3'
        }
      ]
    },
    defaults: {
      port: 502,
      unitId: 1,
      timeout: 5000,
      retryCount: 3
    }
  },
  {
    id: 'modbus-tcp-gateway',
    protocol: ProtocolType.ModbusTCP,
    name: 'Modbus网关',
    description: '适用于Modbus网关设备，支持多从站',
    schema: {
      fields: [
        // ... 网关特定配置
      ]
    },
    defaults: {
      // ... 默认值
    }
  }
]

// 获取协议的配置模板
export function getTemplatesForProtocol(protocol: ProtocolType): ProtocolTemplate[] {
  switch (protocol) {
    case ProtocolType.ModbusTCP:
      return modbusTcpTemplates
    case ProtocolType.OPCUA:
      return opcuaTemplates
    // ... 其他协议
    default:
      return []
  }
}
```

## 🔍 验证规则

```typescript
// utils/validators/driver.ts

export const driverValidationRules = {
  name: [
    { required: true, message: '请输入驱动名称', trigger: 'blur' },
    { min: 2, max: 50, message: '长度在 2 到 50 个字符', trigger: 'blur' },
    { pattern: /^[a-zA-Z0-9_\u4e00-\u9fa5]+$/, message: '只能包含字母、数字、下划线和中文', trigger: 'blur' }
  ],
  
  protocol: [
    { required: true, message: '请选择协议类型', trigger: 'change' }
  ],
  
  // IP地址验证
  ip: [
    { required: true, message: '请输入IP地址', trigger: 'blur' },
    { 
      validator: (rule, value, callback) => {
        const ipRegex = /^(\d{1,3}\.){3}\d{1,3}$/
        if (!ipRegex.test(value)) {
          callback(new Error('请输入有效的IP地址'))
        } else {
          const parts = value.split('.')
          const valid = parts.every(part => {
            const num = parseInt(part)
            return num >= 0 && num <= 255
          })
          if (!valid) {
            callback(new Error('IP地址格式不正确'))
          } else {
            callback()
          }
        }
      },
      trigger: 'blur'
    }
  ],
  
  // 端口验证
  port: [
    { required: true, message: '请输入端口号', trigger: 'blur' },
    { type: 'number', min: 1, max: 65535, message: '端口号必须在1-65535之间', trigger: 'blur' }
  ]
}
```

## ⚡ 性能优化

1. **表格虚拟滚动**: 驱动列表超过100条时启用虚拟滚动
2. **配置缓存**: 缓存常用配置模板，减少重复请求
3. **防抖处理**: 表单输入防抖，减少验证频率
4. **懒加载**: 详情页面按需加载设备和标签数据

## 🧪 测试用例

### 单元测试
```typescript
// tests/unit/driver/useDriverCreate.spec.ts
describe('useDriverCreate', () => {
  it('should create driver with default config', async () => {
    const { wizardData, createDriver } = useDriverCreate()
    
    wizardData.driver.name = 'Test Driver'
    wizardData.driver.protocol = ProtocolType.ModbusTCP
    wizardData.config.parameters = {
      host: '192.168.1.100',
      port: 502
    }
    
    await createDriver()
    
    expect(driverStore.drivers).toHaveLength(1)
    expect(driverStore.configs).toHaveLength(1)
  })
})
```

### E2E测试
```typescript
// tests/e2e/driver/create-driver.spec.ts
test('create ModbusTCP driver', async ({ page }) => {
  await page.goto('/drivers')
  await page.click('button:has-text("创建驱动")')
  
  // 选择协议
  await page.click('.protocol-card[data-protocol="ModbusTCP"]')
  await page.click('button:has-text("下一步")')
  
  // 填写基本信息
  await page.fill('input[name="name"]', 'Test ModbusTCP Driver')
  await page.click('button:has-text("下一步")')
  
  // 配置连接
  await page.fill('input[name="host"]', '192.168.1.100')
  await page.fill('input[name="port"]', '502')
  await page.click('button:has-text("创建")')
  
  // 验证创建成功
  await expect(page).toHaveURL(/\/drivers\/[\w-]+/)
  await expect(page.locator('h1')).toContainText('Test ModbusTCP Driver')
})