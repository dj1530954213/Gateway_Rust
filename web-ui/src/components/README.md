# Gateway Components 使用文档

本文档介绍工业网关前端组件库的使用方法和最佳实践。

## 📁 组件结构

```
components/
├── base/           # 基础组件层
│   ├── BaseTable.vue
│   ├── BaseForm.vue
│   ├── StatusTag.vue
│   ├── ActionButtons.vue
│   ├── SearchBox.vue
│   ├── FilterPanel.vue
│   ├── ConfirmDialog.vue
│   ├── LoadingCard.vue
│   └── index.ts
├── business/       # 业务组件层
│   ├── ConnectionStatus.vue
│   ├── DataPointSelector.vue
│   ├── ProtocolConfig.vue
│   ├── TimeRangePicker.vue
│   ├── ChartContainer.vue
│   ├── ConfigValidation.vue
│   ├── LogViewer.vue
│   ├── FileUploader.vue
│   └── index.ts
└── layout/        # 布局组件层
    └── ...
```

## 🚀 快速开始

### 安装依赖

组件库基于以下技术栈：
- Vue 3 + TypeScript
- Element Plus UI 库
- Pinia 状态管理
- Vue Router 路由

### 导入组件

```typescript
// 导入基础组件
import { BaseTable, BaseForm, StatusTag } from '@/components/base'

// 导入业务组件
import { ConnectionStatus, DataPointSelector } from '@/components/business'

// 导入类型定义
import type { TableColumn, ConnectionInfo } from '@/components/base'
```

## 📋 基础组件层

### BaseTable 数据表格

通用数据表格组件，支持分页、排序、筛选等功能。

```vue
<template>
  <BaseTable
    :data="tableData"
    :columns="columns"
    :pagination="paginationConfig"
    @action="handleAction"
  />
</template>

<script setup lang="ts">
import { BaseTable } from '@/components/base'
import type { TableColumn } from '@/components/base'

const tableData = ref([
  { id: 1, name: '设备1', status: 'online' },
  { id: 2, name: '设备2', status: 'offline' }
])

const columns: TableColumn[] = [
  { key: 'id', label: 'ID', width: 80 },
  { key: 'name', label: '设备名称' },
  { key: 'status', label: '状态', type: 'tag' },
  { key: 'actions', label: '操作', type: 'action' }
]

const paginationConfig = {
  pageSize: 10,
  showSizeChanger: true,
  showQuickJumper: true
}

const handleAction = (action: string, row: any) => {
  console.log('Action:', action, 'Row:', row)
}
</script>
```

### BaseForm 动态表单

支持多种字段类型的动态表单组件。

```vue
<template>
  <BaseForm
    v-model="formData"
    :fields="formFields"
    :rules="formRules"
    @submit="handleSubmit"
  />
</template>

<script setup lang="ts">
import { BaseForm } from '@/components/base'
import type { FormField } from '@/components/base'

const formData = ref({})

const formFields: FormField[] = [
  {
    key: 'name',
    label: '设备名称',
    type: 'text',
    required: true,
    placeholder: '请输入设备名称'
  },
  {
    key: 'type',
    label: '设备类型',
    type: 'select',
    options: [
      { label: 'Modbus', value: 'modbus' },
      { label: 'OPC UA', value: 'opcua' }
    ]
  }
]

const formRules = {
  name: [{ required: true, message: '请输入设备名称' }]
}

const handleSubmit = (data: any) => {
  console.log('Form data:', data)
}
</script>
```

### StatusTag 状态标签

用于显示各种状态的标签组件。

```vue
<template>
  <StatusTag :status="deviceStatus" />
</template>

<script setup lang="ts">
import { StatusTag } from '@/components/base'

const deviceStatus = ref('connected')
</script>
```

### ActionButtons 操作按钮组

支持确认对话框和下拉菜单的按钮组件。

```vue
<template>
  <ActionButtons
    :actions="actions"
    @actionClick="handleActionClick"
  />
</template>

<script setup lang="ts">
import { ActionButtons } from '@/components/base'
import type { ActionButton } from '@/components/base'

const actions: ActionButton[] = [
  {
    key: 'edit',
    label: '编辑',
    type: 'primary',
    icon: 'Edit'
  },
  {
    key: 'delete',
    label: '删除',
    type: 'danger',
    confirm: {
      title: '确定要删除吗？',
      confirmText: '删除',
      cancelText: '取消'
    }
  }
]

const handleActionClick = (action: ActionButton) => {
  console.log('Action clicked:', action.key)
}
</script>
```

## 🏢 业务组件层

### ConnectionStatus 连接状态

显示设备连接状态和详细信息。

```vue
<template>
  <ConnectionStatus
    :connection="connectionInfo"
    @reconnect="handleReconnect"
    @disconnect="handleDisconnect"
  />
</template>

<script setup lang="ts">
import { ConnectionStatus } from '@/components/business'
import type { ConnectionInfo } from '@/components/business'

const connectionInfo: ConnectionInfo = {
  id: 'device_001',
  name: 'Modbus设备1',
  status: 'connected',
  protocol: 'Modbus TCP',
  host: '',
  port: 502,
  lastUpdate: new Date(),
  metrics: {
    latency: 25,
    packetLoss: 0.1,
    throughput: 1024
  }
}

const handleReconnect = (connection: ConnectionInfo) => {
  // 重连逻辑
}

const handleDisconnect = (connection: ConnectionInfo) => {
  // 断开连接逻辑
}
</script>
```

### DataPointSelector 数据点选择器

用于选择和管理数据点的组件。

```vue
<template>
  <DataPointSelector
    v-model="selectedPoints"
    :data-points="dataPoints"
    multiple
    @change="handleSelectionChange"
  />
</template>

<script setup lang="ts">
import { DataPointSelector } from '@/components/business'
import type { DataPoint } from '@/components/business'

const selectedPoints = ref<string[]>([])

const dataPoints: DataPoint[] = [
  {
    id: '1',
    name: '温度传感器1',
    type: 'datapoint',
    dataType: 'number',
    address: '40001',
    unit: '°C',
    quality: 'good'
  }
]

const handleSelectionChange = (selectedDataPoints: DataPoint[]) => {
  console.log('Selected data points:', selectedDataPoints)
}
</script>
```

### ProtocolConfig 协议配置

用于配置各种工业协议的组件。

```vue
<template>
  <ProtocolConfig
    v-model="protocolConfig"
    protocol-type="modbus_tcp"
    @save="handleSave"
    @test="handleTest"
  />
</template>

<script setup lang="ts">
import { ProtocolConfig } from '@/components/business'
import type { ProtocolConfig as ConfigType } from '@/components/business'

const protocolConfig = ref<ConfigType>({
  basic: {},
  connection: {},
  security: {},
  advanced: {}
})

const handleSave = (config: ConfigType) => {
  console.log('Protocol config saved:', config)
}

const handleTest = (config: ConfigType) => {
  console.log('Testing protocol config:', config)
}
</script>
```

### ChartContainer 图表容器

支持多种图表类型的数据可视化组件。

```vue
<template>
  <ChartContainer
    title="设备监控"
    chart-type="line"
    :data="chartData"
    :config="chartConfig"
    support-realtime
    @refresh="handleRefresh"
  />
</template>

<script setup lang="ts">
import { ChartContainer } from '@/components/business'
import type { ChartData, ChartConfig } from '@/components/business'

const chartData: ChartData = {
  series: [
    {
      name: '温度',
      data: [20, 22, 21, 25, 23, 24, 26]
    }
  ]
}

const chartConfig: ChartConfig = {
  basic: {
    title: '温度趋势',
    type: 'line'
  },
  style: {
    height: '400px'
  },
  data: {
    refreshInterval: 5000
  }
}

const handleRefresh = () => {
  // 刷新图表数据
}
</script>
```

### LogViewer 日志查看器

用于查看和管理系统日志的组件。

```vue
<template>
  <LogViewer
    title="系统日志"
    :logs="logEntries"
    support-realtime
    @export="handleExport"
  />
</template>

<script setup lang="ts">
import { LogViewer } from '@/components/business'
import type { LogEntry } from '@/components/business'

const logEntries: LogEntry[] = [
  {
    id: '1',
    timestamp: new Date(),
    level: 'info',
    source: 'Gateway',
    message: 'System started successfully'
  }
]

const handleExport = (logs: LogEntry[]) => {
  console.log('Exporting logs:', logs)
}
</script>
```

### FileUploader 文件上传

支持多种文件类型的上传组件。

```vue
<template>
  <FileUploader
    title="配置文件上传"
    :upload-types="uploadTypes"
    multiple
    @success="handleUploadSuccess"
  />
</template>

<script setup lang="ts">
import { FileUploader } from '@/components/business'
import type { UploadType, FileItem } from '@/components/business'

const uploadTypes: UploadType[] = [
  {
    value: 'config',
    label: '配置文件',
    accept: '.json,.xml,.yaml',
    maxSize: 10 * 1024 * 1024,
    description: '支持JSON、XML、YAML格式'
  }
]

const handleUploadSuccess = (file: FileItem, response: any) => {
  console.log('File uploaded:', file, response)
}
</script>
```

## 🎨 样式定制

### 主题变量

组件库使用 CSS 变量进行主题定制：

```css
:root {
  --el-color-primary: #409eff;
  --el-color-success: #67c23a;
  --el-color-warning: #e6a23c;
  --el-color-danger: #f56c6c;
  --el-color-info: #909399;
}
```

### 自定义样式类

所有组件都支持 `customClass` 属性：

```vue
<template>
  <BaseTable
    :data="data"
    :columns="columns"
    custom-class="my-custom-table"
  />
</template>

<style scoped>
.my-custom-table {
  /* 自定义样式 */
}
</style>
```

## 📱 响应式设计

组件库内置响应式支持，自动适配不同屏幕尺寸：

- **桌面端**: > 1200px
- **平板端**: 768px - 1200px  
- **移动端**: < 768px

## 🔧 最佳实践

### 1. 组件组合

优先使用基础组件组合实现复杂功能：

```vue
<template>
  <el-card>
    <template #header>
      <SearchBox v-model="searchKeyword" @search="handleSearch" />
    </template>
    
    <FilterPanel :filters="filters" @filter="handleFilter" />
    
    <BaseTable
      :data="filteredData"
      :columns="columns"
      :loading="loading"
    />
  </el-card>
</template>
```

### 2. 类型安全

充分利用 TypeScript 类型检查：

```typescript
import type { TableColumn, FormField } from '@/components/base'

// 定义明确的类型
const columns: TableColumn[] = [...]
const fields: FormField[] = [...]
```

### 3. 事件处理

使用统一的事件处理模式：

```vue
<script setup lang="ts">
// 事件处理函数命名规范
const handleTableAction = (action: string, row: any) => {
  switch (action) {
    case 'edit':
      handleEdit(row)
      break
    case 'delete':
      handleDelete(row)
      break
  }
}

const handleEdit = (row: any) => {
  // 编辑逻辑
}

const handleDelete = (row: any) => {
  // 删除逻辑
}
</script>
```

### 4. 状态管理

使用 Pinia 进行状态管理：

```typescript
// stores/devices.ts
import { defineStore } from 'pinia'
import type { ConnectionInfo } from '@/components/business'

export const useDevicesStore = defineStore('devices', () => {
  const devices = ref<ConnectionInfo[]>([])
  
  const addDevice = (device: ConnectionInfo) => {
    devices.value.push(device)
  }
  
  return { devices, addDevice }
})
```

## 🧪 测试指南

### 单元测试

使用 Vitest 进行单元测试：

```typescript
// tests/components/BaseTable.test.ts
import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import BaseTable from '@/components/base/BaseTable.vue'

describe('BaseTable', () => {
  it('renders correctly', () => {
    const wrapper = mount(BaseTable, {
      props: {
        data: [{ id: 1, name: 'Test' }],
        columns: [
          { key: 'id', label: 'ID' },
          { key: 'name', label: 'Name' }
        ]
      }
    })
    
    expect(wrapper.find('.base-table').exists()).toBe(true)
  })
})
```

### E2E 测试

使用 Cypress 进行端到端测试：

```typescript
// cypress/e2e/device-management.cy.ts
describe('Device Management', () => {
  it('should add a new device', () => {
    cy.visit('/devices')
    cy.get('[data-cy=add-device-btn]').click()
    cy.get('[data-cy=device-name-input]').type('Test Device')
    cy.get('[data-cy=save-btn]').click()
    cy.contains('Test Device').should('be.visible')
  })
})
```

## 📄 许可证

MIT License

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 📞 支持

如有问题，请提交 Issue 或联系开发团队。