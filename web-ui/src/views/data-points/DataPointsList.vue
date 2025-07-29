<template>
  <div class="datapoints-list-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-title">
        <el-icon :size="24">
          <DataLine />
        </el-icon>
        <h1>数据点管理</h1>
        <el-tag v-if="datapointsStats" type="info" size="large">
          {{ datapointsStats.total }} 个数据点
        </el-tag>
      </div>
      
      <div class="header-actions">
        <el-button
          type="primary"
          :icon="Plus"
          @click="handleAddDatapoint"
        >
          新增数据点
        </el-button>
        
        <el-dropdown trigger="click" @command="handleHeaderAction">
          <el-button :icon="MoreFilled" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="import" :icon="Upload">
                导入数据点
              </el-dropdown-item>
              <el-dropdown-item command="export" :icon="Download">
                导出数据点
              </el-dropdown-item>
              <el-dropdown-item command="batch" :icon="DocumentCopy">
                批量配置
              </el-dropdown-item>
              <el-dropdown-item command="refresh" :icon="Refresh">
                刷新数据
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
    
    <!-- 数据点状态概览 -->
    <div class="datapoints-overview">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--total">
            <div class="stat-content">
              <div class="stat-number">{{ datapointsStats.total }}</div>
              <div class="stat-label">总数据点</div>
            </div>
            <el-icon class="stat-icon">
              <DataLine />
            </el-icon>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--active">
            <div class="stat-content">
              <div class="stat-number">{{ datapointsStats.active }}</div>
              <div class="stat-label">激活</div>
            </div>
            <el-icon class="stat-icon">
              <CircleCheckFilled />
            </el-icon>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--good">
            <div class="stat-content">
              <div class="stat-number">{{ datapointsStats.goodQuality }}</div>
              <div class="stat-label">质量良好</div>
            </div>
            <el-icon class="stat-icon">
              <SuccessFilled />
            </el-icon>
          </el-card>
        </el-col>
        
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--alarm">
            <div class="stat-content">
              <div class="stat-number">{{ datapointsStats.alarming }}</div>
              <div class="stat-label">报警中</div>
            </div>
            <el-icon class="stat-icon">
              <WarningFilled />
            </el-icon>
          </el-card>
        </el-col>
      </el-row>
    </div>
    
    <!-- 主要内容区域 -->
    <div class="datapoints-content">
      <el-row :gutter="16">
        <!-- 左侧：数据点树形结构 -->
        <el-col :span="6">
          <el-card shadow="never" class="tree-card">
            <template #header>
              <div class="tree-header">
                <span>数据点分组</span>
                <div class="tree-actions">
                  <el-button 
                    type="text" 
                    size="small" 
                    :icon="FolderAdd"
                    @click="handleAddGroup"
                  >
                    新增分组
                  </el-button>
                </div>
              </div>
            </template>
            
            <DataPointSelector
              ref="datapointSelectorRef"
              v-model="selectedDatapoints"
              :data-points="datapointTree"
              :show-tree="true"
              :show-values="true"
              multiple
              @node-click="handleTreeNodeClick"
              @selection-change="handleTreeSelectionChange"
              style="height: 600px"
            />
          </el-card>
        </el-col>
        
        <!-- 右侧：数据点列表和详情 -->
        <el-col :span="18">
          <!-- 筛选和搜索 -->
          <div class="datapoints-filters">
            <el-card shadow="never">
              <div class="filter-content">
                <div class="filter-left">
                  <SearchBox
                    v-model="searchKeyword"
                    placeholder="搜索数据点名称、地址或描述"
                    :suggestions="searchSuggestions"
                    style="width: 300px"
                    @search="handleSearch"
                  />
                  
                  <FilterPanel
                    v-model="filterConditions"
                    :filters="filterOptions"
                    :show-quick-filters="true"
                    :quick-filters="quickFilters"
                    :show-apply-button="false"
                    style="margin-left: 16px"
                    @filter="handleFilter"
                  />
                </div>
                
                <div class="filter-right">
                  <ActionButtons
                    :actions="batchActions"
                    :disabled="selectedDatapoints.length === 0"
                    @actionClick="handleBatchAction"
                  />
                </div>
              </div>
            </el-card>
          </div>
          
          <!-- 数据点列表表格 -->
          <div class="datapoints-table">
            <LoadingCard
              v-if="loading && datapoints.length === 0"
              status="loading"
              loading-text="正在加载数据点..."
              :show-progress="true"
              :progress="loadingProgress"
              min-height="400px"
            />
            
            <BaseTable
              v-else
              v-model:selection="selectedDatapoints"
              :data="filteredDatapoints"
              :columns="tableColumns"
              :loading="loading"
              :pagination="paginationConfig"
              show-selection
              @action="handleTableAction"
              @refresh="handleRefresh"
            />
          </div>
        </el-col>
      </el-row>
    </div>
    
    <!-- 数据点详情抽屉 -->
    <el-drawer
      v-model="detailDrawerVisible"
      :title="`数据点详情 - ${selectedDatapoint?.name}`"
      size="700px"
      :before-close="handleDetailDrawerClose"
    >
      <div v-if="selectedDatapoint" class="datapoint-detail">
        <el-tabs v-model="detailActiveTab">
          <!-- 基本信息 -->
          <el-tab-pane label="基本信息" name="basic">
            <el-descriptions border column="2">
              <el-descriptions-item label="数据点名称">
                {{ selectedDatapoint.name }}
              </el-descriptions-item>
              <el-descriptions-item label="数据类型">
                <el-tag :type="getDataTypeTagType(selectedDatapoint.dataType)">
                  {{ selectedDatapoint.dataType }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="地址">
                {{ selectedDatapoint.address }}
              </el-descriptions-item>
              <el-descriptions-item label="驱动来源">
                {{ getDriverName(selectedDatapoint.driverId) }}
              </el-descriptions-item>
              <el-descriptions-item label="数据质量">
                <StatusTag :status="selectedDatapoint.quality" />
              </el-descriptions-item>
              <el-descriptions-item label="单位">
                {{ selectedDatapoint.unit || '-' }}
              </el-descriptions-item>
              <el-descriptions-item label="创建时间">
                {{ formatTime(selectedDatapoint.createTime) }}
              </el-descriptions-item>
              <el-descriptions-item label="最后更新">
                {{ formatTime(selectedDatapoint.updateTime) }}
              </el-descriptions-item>
            </el-descriptions>
            
            <!-- 实时数值 -->
            <div class="current-value-section" style="margin-top: 20px">
              <el-divider content-position="left">实时数值</el-divider>
              <div class="value-display">
                <div class="value-item">
                  <div class="value-label">当前值</div>
                  <div class="value-number" :class="getValueClass(selectedDatapoint)">
                    {{ formatValue(selectedDatapoint.currentValue, selectedDatapoint.dataType) }}
                    <span v-if="selectedDatapoint.unit" class="value-unit">{{ selectedDatapoint.unit }}</span>
                  </div>
                </div>
                <div class="value-item">
                  <div class="value-label">最后读取时间</div>
                  <div class="value-time">{{ formatTime(selectedDatapoint.lastReadTime) }}</div>
                </div>
              </div>
            </div>
          </el-tab-pane>
          
          <!-- 历史趋势 -->
          <el-tab-pane label="历史趋势" name="history">
            <div class="datapoint-history">
              <div class="history-controls">
                <TimeRangePicker
                  v-model="historyTimeRange"
                  @change="handleTimeRangeChange"
                />
                <el-button type="primary" @click="handleRefreshHistory">
                  刷新数据
                </el-button>
              </div>
              
              <ChartContainer
                title="数据点历史趋势"
                chart-type="line"
                :data="getDatapointChartData(selectedDatapoint)"
                :height="'400px'"
                support-realtime
              />
            </div>
          </el-tab-pane>
          
          <!-- 报警配置 -->
          <el-tab-pane label="报警配置" name="alarms">
            <div class="datapoint-alarms">
              <div class="alarm-rules">
                <div class="rule-header">
                  <h4>报警规则</h4>
                  <el-button type="primary" size="small" @click="handleAddAlarmRule">
                    新增规则
                  </el-button>
                </div>
                
                <BaseTable
                  :data="getDatapointAlarmRules(selectedDatapoint)"
                  :columns="alarmRuleColumns"
                  :pagination="false"
                  @action="handleAlarmRuleAction"
                />
              </div>
            </div>
          </el-tab-pane>
          
          <!-- 配置参数 -->
          <el-tab-pane label="配置参数" name="config">
            <div class="datapoint-config">
              <BaseForm
                v-model="datapointConfigData"
                :fields="datapointConfigFields"
                :rules="datapointConfigRules"
                label-width="120px"
                @submit="handleConfigSave"
              />
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>
      
      <template #footer>
        <div class="drawer-footer">
          <el-button @click="detailDrawerVisible = false">关闭</el-button>
          <el-button
            v-if="selectedDatapoint?.status === 'inactive'"
            type="success"
            @click="handleActivateDatapoint(selectedDatapoint)"
          >
            激活数据点
          </el-button>
          <el-button
            v-if="selectedDatapoint?.status === 'active'"
            type="warning"
            @click="handleDeactivateDatapoint(selectedDatapoint)"
          >
            停用数据点
          </el-button>
          <el-button
            type="primary"
            @click="handleEditDatapoint(selectedDatapoint)"
          >
            编辑数据点
          </el-button>
        </div>
      </template>
    </el-drawer>
    
    <!-- 数据点配置对话框 -->
    <el-dialog
      v-model="configDialogVisible"
      :title="configDialogTitle"
      width="900px"
      :before-close="handleConfigDialogClose"
    >
      <BaseForm
        v-if="editingDatapoint"
        v-model="datapointFormData"
        :fields="datapointFormFields"
        :rules="datapointFormRules"
        label-width="120px"
        @submit="handleFormSubmit"
        @cancel="handleFormCancel"
      />
    </el-dialog>
    
    <!-- 批量配置对话框 -->
    <el-dialog
      v-model="batchConfigDialogVisible"
      title="批量数据点配置"
      width="800px"
      :before-close="handleBatchConfigDialogClose"
    >
      <div class="batch-config">
        <div class="batch-info">
          <el-alert
            title="批量配置"
            :description="`将对选中的 ${selectedDatapoints.length} 个数据点应用相同配置`"
            type="info"
            show-icon
          />
        </div>
        
        <BaseForm
          v-model="batchConfigData"
          :fields="batchConfigFields"
          label-width="120px"
          style="margin-top: 20px"
        />
      </div>
      
      <template #footer>
        <el-button @click="batchConfigDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleBatchConfigSave">
          应用配置
        </el-button>
      </template>
    </el-dialog>
    
    <!-- 文件上传对话框 -->
    <el-dialog
      v-model="uploadDialogVisible"
      title="导入数据点配置"
      width="600px"
      :before-close="handleUploadDialogClose"
    >
      <FileUploader
        title="选择数据点配置文件"
        :upload-types="datapointUploadTypes"
        @success="handleDatapointUploadSuccess"
      />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, onUnmounted, h } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  DataLine,
  Plus,
  MoreFilled,
  Upload,
  Download,
  DocumentCopy,
  Refresh,
  CircleCheckFilled,
  SuccessFilled,
  WarningFilled,
  FolderAdd
} from '@element-plus/icons-vue'

// 导入基础组件
import { BaseTable, SearchBox, FilterPanel, ActionButtons, LoadingCard, StatusTag, BaseForm } from '../../components/base'

// 导入业务组件
import { DataPointSelector, TimeRangePicker, ChartContainer, FileUploader } from '../../components/business'

// 类型定义
export interface DataPoint {
  id: string
  name: string
  description?: string
  address: string
  dataType: 'boolean' | 'integer' | 'float' | 'string' | 'datetime'
  driverId: string
  groupId?: string
  status: 'active' | 'inactive' | 'error'
  quality: 'good' | 'uncertain' | 'bad' | 'invalid'
  currentValue: any
  lastReadTime: Date
  unit?: string
  scaling?: {
    factor: number
    offset: number
  }
  limits?: {
    min?: number
    max?: number
  }
  createTime: Date
  updateTime: Date
  config?: any
}

export interface DataPointGroup {
  id: string
  name: string
  parentId?: string
  description?: string
  children?: DataPoint[]
  groups?: DataPointGroup[]
  createTime: Date
}

export interface DatapointsStats {
  total: number
  active: number
  goodQuality: number
  alarming: number
}

export interface AlarmRule {
  id: string
  datapointId: string
  name: string
  condition: 'gt' | 'lt' | 'eq' | 'ne' | 'range'
  value: number | string
  highValue?: number
  priority: 'low' | 'medium' | 'high' | 'critical'
  enabled: boolean
  createTime: Date
}

// 状态管理
const datapoints = ref<DataPoint[]>([
  {
    id: '1',
    name: '反应器温度',
    description: '主反应器内部温度传感器',
    address: '40001',
    dataType: 'float',
    driverId: '1',
    groupId: 'temp_group',
    status: 'active',
    quality: 'good',
    currentValue: 75.6,
    lastReadTime: new Date(),
    unit: '°C',
    scaling: { factor: 0.1, offset: 0 },
    limits: { min: 0, max: 200 },
    createTime: new Date('2024-01-15'),
    updateTime: new Date()
  },
  {
    id: '2',
    name: '进料流量',
    description: '主管道进料流量计',
    address: '40002',
    dataType: 'float',
    driverId: '1',
    groupId: 'flow_group',
    status: 'active',
    quality: 'good',
    currentValue: 150.2,
    lastReadTime: new Date(Date.now() - 5000),
    unit: 'L/min',
    scaling: { factor: 1, offset: 0 },
    limits: { min: 0, max: 500 },
    createTime: new Date('2024-01-16'),
    updateTime: new Date(Date.now() - 5000)
  },
  {
    id: '3',
    name: '压力开关',
    description: '高压保护开关状态',
    address: '00001',
    dataType: 'boolean',
    driverId: '2',
    groupId: 'safety_group',
    status: 'active',
    quality: 'good',
    currentValue: false,
    lastReadTime: new Date(Date.now() - 2000),
    unit: '',
    createTime: new Date('2024-01-10'),
    updateTime: new Date(Date.now() - 2000)
  },
  {
    id: '4',
    name: '设备状态',
    description: '主设备运行状态代码',
    address: '40010',
    dataType: 'integer',
    driverId: '3',
    groupId: 'status_group',
    status: 'active',
    quality: 'uncertain',
    currentValue: 2,
    lastReadTime: new Date(Date.now() - 30000),
    unit: '',
    createTime: new Date('2024-01-12'),
    updateTime: new Date(Date.now() - 30000)
  },
  {
    id: '5',
    name: '批次编号',
    description: '当前生产批次编号',
    address: '40020',
    dataType: 'string',
    driverId: '4',
    groupId: 'info_group',
    status: 'active',
    quality: 'good',
    currentValue: 'BATCH_20240121_001',
    lastReadTime: new Date(Date.now() - 10000),
    unit: '',
    createTime: new Date('2024-01-20'),
    updateTime: new Date(Date.now() - 10000)
  }
])

// 数据点分组
const datapointGroups = ref<DataPointGroup[]>([
  {
    id: 'temp_group',
    name: '温度监控',
    description: '各部位温度传感器',
    createTime: new Date('2024-01-01')
  },
  {
    id: 'flow_group', 
    name: '流量监控',
    description: '流量和液位传感器',
    createTime: new Date('2024-01-01')
  },
  {
    id: 'safety_group',
    name: '安全监控',
    description: '安全相关开关和传感器',
    createTime: new Date('2024-01-01')
  },
  {
    id: 'status_group',
    name: '状态监控',
    description: '设备状态和控制信息',
    createTime: new Date('2024-01-01')
  },
  {
    id: 'info_group',
    name: '信息监控',
    description: '生产信息和标识',
    createTime: new Date('2024-01-01')
  }
])

const loading = ref(false)
const loadingProgress = ref(0)
const selectedDatapoints = ref<DataPoint[]>([])
const searchKeyword = ref('')
const filterConditions = ref<any>({})

// 对话框状态
const detailDrawerVisible = ref(false)
const configDialogVisible = ref(false)
const batchConfigDialogVisible = ref(false)
const uploadDialogVisible = ref(false)

// 选中的数据点和配置
const selectedDatapoint = ref<DataPoint | null>(null)
const editingDatapoint = ref<DataPoint | null>(null)
const datapointFormData = ref<any>({})
const datapointConfigData = ref<any>({})
const batchConfigData = ref<any>({})
const detailActiveTab = ref('basic')

// 历史数据查询
const historyTimeRange = ref({
  startTime: new Date(Date.now() - 24 * 60 * 60 * 1000),
  endTime: new Date()
})

// WebSocket连接（模拟实时数据）
const wsConnection: WebSocket | null = null

// 计算属性
const datapointsStats = computed<DatapointsStats>(() => {
  const stats = {
    total: datapoints.value.length,
    active: 0,
    goodQuality: 0,
    alarming: 0
  }
  
  datapoints.value.forEach(dp => {
    if (dp.status === 'active') {
      stats.active++
    }
    if (dp.quality === 'good') {
      stats.goodQuality++
    }
    if (hasAlarm(dp)) {
      stats.alarming++
    }
  })
  
  return stats
})

const filteredDatapoints = computed(() => {
  let result = datapoints.value
  
  // 搜索过滤
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase()
    result = result.filter(dp => 
      dp.name.toLowerCase().includes(keyword) ||
      dp.address.toLowerCase().includes(keyword) ||
      (dp.description?.toLowerCase() || '').includes(keyword)
    )
  }
  
  // 状态过滤
  if (filterConditions.value.status) {
    result = result.filter(dp => dp.status === filterConditions.value.status)
  }
  
  // 数据类型过滤
  if (filterConditions.value.dataType) {
    result = result.filter(dp => dp.dataType === filterConditions.value.dataType)
  }
  
  // 质量过滤
  if (filterConditions.value.quality) {
    result = result.filter(dp => dp.quality === filterConditions.value.quality)
  }
  
  // 驱动过滤
  if (filterConditions.value.driverId) {
    result = result.filter(dp => dp.driverId === filterConditions.value.driverId)
  }
  
  return result
})

const searchSuggestions = computed(() => {
  const suggestions = []
  datapoints.value.forEach(dp => {
    suggestions.push(dp.name)
    suggestions.push(dp.address)
    if (dp.description) suggestions.push(dp.description)
  })
  return [...new Set(suggestions)]
})

const datapointTree = computed(() => {
  // 构建树形结构数据
  const tree = []
  
  datapointGroups.value.forEach(group => {
    const groupNode = {
      id: group.id,
      name: group.name,
      type: 'group',
      children: datapoints.value
        .filter(dp => dp.groupId === group.id)
        .map(dp => ({
          id: dp.id,
          name: dp.name,
          type: 'datapoint',
          dataType: dp.dataType,
          address: dp.address,
          status: dp.status,
          quality: dp.quality,
          currentValue: dp.currentValue,
          unit: dp.unit,
          lastUpdate: dp.lastReadTime
        }))
    }
    tree.push(groupNode)
  })
  
  // 未分组的数据点
  const ungrouped = datapoints.value.filter(dp => !dp.groupId)
  if (ungrouped.length > 0) {
    tree.push({
      id: 'ungrouped',
      name: '未分组',
      type: 'group',
      children: ungrouped.map(dp => ({
        id: dp.id,
        name: dp.name,
        type: 'datapoint',
        dataType: dp.dataType,
        address: dp.address,
        status: dp.status,
        quality: dp.quality,
        currentValue: dp.currentValue,
        unit: dp.unit,
        lastUpdate: dp.lastReadTime
      }))
    })
  }
  
  return tree
})

// 筛选器配置
const filterOptions = [
  {
    key: 'status',
    label: '状态',
    type: 'select',
    options: [
      { label: '激活', value: 'active' },
      { label: '未激活', value: 'inactive' },
      { label: '错误', value: 'error' }
    ]
  },
  {
    key: 'quality',
    label: '数据质量',
    type: 'select',
    options: [
      { label: '良好', value: 'good' },
      { label: '不确定', value: 'uncertain' },
      { label: '错误', value: 'bad' },
      { label: '无效', value: 'invalid' }
    ]
  },
  {
    key: 'dataType',
    label: '数据类型',
    type: 'select',
    options: [
      { label: '布尔', value: 'boolean' },
      { label: '整数', value: 'integer' },
      { label: '浮点', value: 'float' },
      { label: '字符串', value: 'string' },
      { label: '时间', value: 'datetime' }
    ]
  },
  {
    key: 'driverId',
    label: '驱动来源',
    type: 'select',
    options: [
      { label: 'PLC主控制器', value: '1' },
      { label: '温度传感器组', value: '2' },
      { label: 'SCADA系统', value: '3' },
      { label: 'MQTT网关', value: '4' }
    ]
  }
]

const quickFilters = [
  { label: '激活', key: 'status', value: 'active' },
  { label: '质量良好', key: 'quality', value: 'good' },
  { label: '浮点数', key: 'dataType', value: 'float' },
  { label: '布尔值', key: 'dataType', value: 'boolean' }
]

// 批量操作
const batchActions = [
  {
    key: 'activate',
    label: '批量激活',
    type: 'success',
    icon: 'CircleCheckFilled',
    confirm: {
      title: '确定要激活选中的数据点吗？',
      confirmText: '激活',
      cancelText: '取消'
    }
  },
  {
    key: 'deactivate',
    label: '批量停用',
    type: 'warning',
    icon: 'CircleCloseFilled',
    confirm: {
      title: '确定要停用选中的数据点吗？',
      confirmText: '停用',
      cancelText: '取消'
    }
  },
  {
    key: 'config',
    label: '批量配置',
    type: 'primary',
    icon: 'Setting',
    action: () => batchConfigDialogVisible.value = true
  },
  {
    key: 'delete',
    label: '批量删除',
    type: 'danger',
    icon: 'Delete',
    confirm: {
      title: '确定要删除选中的数据点吗？此操作不可恢复！',
      confirmText: '删除',
      cancelText: '取消',
      type: 'warning'
    }
  }
]

// 表格列配置
const tableColumns = [
  {
    key: 'name',
    label: '数据点名称',
    width: 200,
    sortable: true
  },
  {
    key: 'address',
    label: '地址',
    width: 120,
    sortable: true
  },
  {
    key: 'dataType',
    label: '数据类型',
    width: 100,
    type: 'tag',
    formatter: (row: DataPoint) => {
      const typeMap: Record<string, string> = {
        boolean: 'Bool',
        integer: 'Int',
        float: 'Float',
        string: 'String',
        datetime: 'DateTime'
      }
      return typeMap[row.dataType] || row.dataType
    }
  },
  {
    key: 'currentValue',
    label: '当前值',
    width: 150,
    formatter: (row: DataPoint) => {
      return formatValue(row.currentValue, row.dataType) + (row.unit ? ` ${row.unit}` : '')
    }
  },
  {
    key: 'quality',
    label: '质量',
    width: 100,
    type: 'custom',
    render: (row: DataPoint) => {
      return h(StatusTag, { status: row.quality })
    }
  },
  {
    key: 'status',
    label: '状态',
    width: 100,
    type: 'custom',
    render: (row: DataPoint) => {
      return h(StatusTag, { status: row.status })
    }
  },
  {
    key: 'driverId',
    label: '驱动来源',
    width: 150,
    formatter: (row: DataPoint) => getDriverName(row.driverId)
  },
  {
    key: 'lastReadTime',
    label: '最后读取',
    width: 160,
    type: 'datetime'
  },
  {
    key: 'actions',
    label: '操作',
    width: 200,
    type: 'action',
    actions: [
      {
        key: 'view',
        label: '详情',
        type: 'primary',
        icon: 'View'
      },
      {
        key: 'edit',
        label: '编辑',
        type: 'default',
        icon: 'Edit'
      },
      {
        key: 'activate',
        label: '激活',
        type: 'success',
        icon: 'CircleCheckFilled',
        show: (row: DataPoint) => row.status !== 'active'
      },
      {
        key: 'deactivate',
        label: '停用',
        type: 'warning',
        icon: 'CircleCloseFilled',
        show: (row: DataPoint) => row.status === 'active'
      },
      {
        key: 'delete',
        label: '删除',
        type: 'danger',
        icon: 'Delete',
        confirm: {
          title: '确定要删除这个数据点吗？',
          confirmText: '删除',
          cancelText: '取消'
        }
      }
    ]
  }
]

// 分页配置
const paginationConfig = {
  pageSize: 15,
  showSizeChanger: true,
  showQuickJumper: true,
  showTotal: true
}

// 报警规则表格列
const alarmRuleColumns = [
  { key: 'name', label: '规则名称', width: 150 },
  { key: 'condition', label: '条件', width: 80, formatter: (row: any) => getConditionText(row.condition) },
  { key: 'value', label: '阈值', width: 100 },
  { key: 'priority', label: '优先级', width: 80, type: 'tag' },
  { key: 'enabled', label: '状态', width: 80, type: 'tag', formatter: (row: any) => row.enabled ? '启用' : '禁用' },
  { key: 'actions', label: '操作', width: 120, type: 'action', actions: [
    { key: 'edit', label: '编辑', type: 'primary', icon: 'Edit' },
    { key: 'delete', label: '删除', type: 'danger', icon: 'Delete' }
  ]}
]

// 数据点表单字段
const datapointFormFields = [
  {
    key: 'name',
    label: '数据点名称',
    type: 'text',
    required: true,
    placeholder: '请输入数据点名称'
  },
  {
    key: 'description',
    label: '描述',
    type: 'textarea',
    placeholder: '请输入数据点描述'
  },
  {
    key: 'address',
    label: '地址',
    type: 'text',
    required: true,
    placeholder: '如：40001'
  },
  {
    key: 'dataType',
    label: '数据类型',
    type: 'select',
    required: true,
    options: [
      { label: '布尔值', value: 'boolean' },
      { label: '整数', value: 'integer' },
      { label: '浮点数', value: 'float' },
      { label: '字符串', value: 'string' },
      { label: '日期时间', value: 'datetime' }
    ]
  },
  {
    key: 'driverId',
    label: '驱动来源',
    type: 'select',
    required: true,
    options: [
      { label: 'PLC主控制器', value: '1' },
      { label: '温度传感器组', value: '2' },
      { label: 'SCADA系统', value: '3' },
      { label: 'MQTT网关', value: '4' }
    ]
  },
  {
    key: 'groupId',
    label: '所属分组',
    type: 'select',
    options: [
      { label: '温度监控', value: 'temp_group' },
      { label: '流量监控', value: 'flow_group' },
      { label: '安全监控', value: 'safety_group' },
      { label: '状态监控', value: 'status_group' },
      { label: '信息监控', value: 'info_group' }
    ]
  },
  {
    key: 'unit',
    label: '单位',
    type: 'text',
    placeholder: '如：°C, L/min, bar'
  }
]

const datapointFormRules = {
  name: [{ required: true, message: '请输入数据点名称' }],
  address: [{ required: true, message: '请输入数据点地址' }],
  dataType: [{ required: true, message: '请选择数据类型' }],
  driverId: [{ required: true, message: '请选择驱动来源' }]
}

// 数据点配置字段
const datapointConfigFields = [
  {
    key: 'scaling.factor',
    label: '缩放系数',
    type: 'number',
    placeholder: '默认为1',
    step: 0.01
  },
  {
    key: 'scaling.offset',
    label: '偏移量',
    type: 'number',
    placeholder: '默认为0',
    step: 0.01
  },
  {
    key: 'limits.min',
    label: '最小值',
    type: 'number',
    placeholder: '数值最小值限制'
  },
  {
    key: 'limits.max',
    label: '最大值',
    type: 'number',
    placeholder: '数值最大值限制'
  }
]

const datapointConfigRules = {}

// 批量配置字段
const batchConfigFields = [
  {
    key: 'status',
    label: '状态',
    type: 'select',
    options: [
      { label: '激活', value: 'active' },
      { label: '未激活', value: 'inactive' }
    ]
  },
  {
    key: 'scaling.factor',
    label: '缩放系数',
    type: 'number',
    step: 0.01
  },
  {
    key: 'scaling.offset',
    label: '偏移量',
    type: 'number',
    step: 0.01
  }
]

// 上传类型配置
const datapointUploadTypes = [
  {
    value: 'csv',
    label: 'CSV文件',
    accept: '.csv',
    maxSize: 5 * 1024 * 1024,
    description: '支持CSV格式数据点配置，最大5MB'
  },
  {
    value: 'excel',
    label: 'Excel文件',
    accept: '.xlsx,.xls',
    maxSize: 10 * 1024 * 1024,
    description: '支持Excel格式数据点配置，最大10MB'
  }
]

// 事件处理方法
const handleSearch = (keyword: string) => {
  searchKeyword.value = keyword
}

const handleFilter = (conditions: any) => {
  filterConditions.value = conditions
}

const handleTableAction = (action: string, row: DataPoint) => {
  switch (action) {
    case 'view':
      handleViewDatapoint(row)
      break
    case 'edit':
      handleEditDatapoint(row)
      break
    case 'activate':
      handleActivateDatapoint(row)
      break
    case 'deactivate':
      handleDeactivateDatapoint(row)
      break
    case 'delete':
      handleDeleteDatapoint(row)
      break
  }
}

const handleBatchAction = (action: string) => {
  if (selectedDatapoints.value.length === 0) {
    ElMessage.warning('请先选择要操作的数据点')
    return
  }
  
  switch (action) {
    case 'activate':
      handleBatchActivate()
      break
    case 'deactivate':
      handleBatchDeactivate()
      break
    case 'config':
      batchConfigDialogVisible.value = true
      break
    case 'delete':
      handleBatchDelete()
      break
  }
}

const handleTreeNodeClick = (data: any) => {
  if (data.type === 'datapoint') {
    const datapoint = datapoints.value.find(dp => dp.id === data.id)
    if (datapoint) {
      handleViewDatapoint(datapoint)
    }
  }
}

const handleTreeSelectionChange = (selectedNodes: any[]) => {
  const selectedIds = selectedNodes
    .filter(node => node.type === 'datapoint')
    .map(node => node.id)
  
  selectedDatapoints.value = datapoints.value.filter(dp => selectedIds.includes(dp.id))
}

const handleViewDatapoint = (datapoint: DataPoint) => {
  selectedDatapoint.value = datapoint
  detailDrawerVisible.value = true
  detailActiveTab.value = 'basic'
}

const handleEditDatapoint = (datapoint: DataPoint) => {
  editingDatapoint.value = datapoint
  datapointFormData.value = {
    name: datapoint.name,
    description: datapoint.description,
    address: datapoint.address,
    dataType: datapoint.dataType,
    driverId: datapoint.driverId,
    groupId: datapoint.groupId,
    unit: datapoint.unit
  }
  configDialogVisible.value = true
}

const handleActivateDatapoint = async (datapoint: DataPoint) => {
  try {
    ElMessage.loading('正在激活数据点...', 0)
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    const index = datapoints.value.findIndex(dp => dp.id === datapoint.id)
    if (index > -1) {
      datapoints.value[index].status = 'active'
      datapoints.value[index].updateTime = new Date()
    }
    
    ElMessage.closeAll()
    ElMessage.success(`数据点 ${datapoint.name} 激活成功`)
  } catch (error) {
    ElMessage.closeAll()
    ElMessage.error('数据点激活失败')
  }
}

const handleDeactivateDatapoint = async (datapoint: DataPoint) => {
  try {
    ElMessage.loading('正在停用数据点...', 0)
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    const index = datapoints.value.findIndex(dp => dp.id === datapoint.id)
    if (index > -1) {
      datapoints.value[index].status = 'inactive'
      datapoints.value[index].updateTime = new Date()
    }
    
    ElMessage.closeAll()
    ElMessage.success(`数据点 ${datapoint.name} 停用成功`)
  } catch (error) {
    ElMessage.closeAll()
    ElMessage.error('数据点停用失败')
  }
}

const handleDeleteDatapoint = (datapoint: DataPoint) => {
  const index = datapoints.value.findIndex(dp => dp.id === datapoint.id)
  if (index > -1) {
    datapoints.value.splice(index, 1)
    ElMessage.success(`数据点 ${datapoint.name} 删除成功`)
  }
}

const handleBatchActivate = async () => {
  const inactiveDatapoints = selectedDatapoints.value.filter(dp => dp.status !== 'active')
  if (inactiveDatapoints.length === 0) {
    ElMessage.warning('选中的数据点都已激活')
    return
  }
  
  ElMessage.loading(`正在激活 ${inactiveDatapoints.length} 个数据点...`, 0)
  
  for (const dp of inactiveDatapoints) {
    await handleActivateDatapoint(dp)
  }
  
  ElMessage.closeAll()
  ElMessage.success(`成功激活 ${inactiveDatapoints.length} 个数据点`)
  selectedDatapoints.value = []
}

const handleBatchDeactivate = async () => {
  const activeDatapoints = selectedDatapoints.value.filter(dp => dp.status === 'active')
  if (activeDatapoints.length === 0) {
    ElMessage.warning('选中的数据点都已停用')
    return
  }
  
  ElMessage.loading(`正在停用 ${activeDatapoints.length} 个数据点...`, 0)
  
  for (const dp of activeDatapoints) {
    await handleDeactivateDatapoint(dp)
  }
  
  ElMessage.closeAll()
  ElMessage.success(`成功停用 ${activeDatapoints.length} 个数据点`)
  selectedDatapoints.value = []
}

const handleBatchDelete = () => {
  selectedDatapoints.value.forEach(datapoint => {
    const index = datapoints.value.findIndex(dp => dp.id === datapoint.id)
    if (index > -1) {
      datapoints.value.splice(index, 1)
    }
  })
  
  ElMessage.success(`成功删除 ${selectedDatapoints.value.length} 个数据点`)
  selectedDatapoints.value = []
}

const handleAddDatapoint = () => {
  editingDatapoint.value = null
  datapointFormData.value = {}
  configDialogVisible.value = true
}

const handleAddGroup = () => {
  ElMessage.info('新增数据点分组功能')
}

const handleHeaderAction = (command: string) => {
  switch (command) {
    case 'import':
      uploadDialogVisible.value = true
      break
    case 'export':
      handleExportDatapoints()
      break
    case 'batch':
      if (selectedDatapoints.value.length === 0) {
        ElMessage.warning('请先选择要配置的数据点')
        return
      }
      batchConfigDialogVisible.value = true
      break
    case 'refresh':
      handleRefresh()
      break
  }
}

const handleExportDatapoints = () => {
  const config = {
    datapoints: datapoints.value.map(dp => ({
      name: dp.name,
      description: dp.description,
      address: dp.address,
      dataType: dp.dataType,
      driverId: dp.driverId,
      groupId: dp.groupId,
      unit: dp.unit,
      scaling: dp.scaling,
      limits: dp.limits
    })),
    exportTime: new Date().toISOString()
  }
  
  const blob = new Blob([JSON.stringify(config, null, 2)], {
    type: 'application/json'
  })
  
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `datapoints_config_${new Date().toISOString().split('T')[0]}.json`
  a.click()
  
  URL.revokeObjectURL(url)
  ElMessage.success('数据点配置导出成功')
}

const handleRefresh = () => {
  loading.value = true
  loadingProgress.value = 0
  
  const interval = setInterval(() => {
    loadingProgress.value += 10
    if (loadingProgress.value >= 100) {
      clearInterval(interval)
      loading.value = false
      loadingProgress.value = 0
      ElMessage.success('数据刷新成功')
    }
  }, 100)
}

// 对话框处理
const handleDetailDrawerClose = () => {
  detailDrawerVisible.value = false
  selectedDatapoint.value = null
}

const handleConfigDialogClose = () => {
  configDialogVisible.value = false
  editingDatapoint.value = null
  datapointFormData.value = {}
}

const handleBatchConfigDialogClose = () => {
  batchConfigDialogVisible.value = false
  batchConfigData.value = {}
}

const handleFormSubmit = (data: any) => {
  if (editingDatapoint.value) {
    // 编辑数据点
    const index = datapoints.value.findIndex(dp => dp.id === editingDatapoint.value!.id)
    if (index > -1) {
      datapoints.value[index] = {
        ...datapoints.value[index],
        ...data,
        updateTime: new Date()
      }
    }
    ElMessage.success('数据点更新成功')
  } else {
    // 新增数据点
    const newDatapoint: DataPoint = {
      id: Date.now().toString(),
      ...data,
      status: 'active',
      quality: 'good',
      currentValue: null,
      lastReadTime: new Date(),
      createTime: new Date(),
      updateTime: new Date()
    }
    datapoints.value.push(newDatapoint)
    ElMessage.success('数据点创建成功')
  }
  
  handleConfigDialogClose()
}

const handleFormCancel = () => {
  handleConfigDialogClose()
}

const handleConfigSave = (config: any) => {
  if (selectedDatapoint.value) {
    const index = datapoints.value.findIndex(dp => dp.id === selectedDatapoint.value!.id)
    if (index > -1) {
      datapoints.value[index] = {
        ...datapoints.value[index],
        ...config,
        updateTime: new Date()
      }
    }
    ElMessage.success('配置保存成功')
  }
}

const handleBatchConfigSave = () => {
  selectedDatapoints.value.forEach(dp => {
    const index = datapoints.value.findIndex(d => d.id === dp.id)
    if (index > -1) {
      datapoints.value[index] = {
        ...datapoints.value[index],
        ...batchConfigData.value,
        updateTime: new Date()
      }
    }
  })
  
  ElMessage.success(`成功配置 ${selectedDatapoints.value.length} 个数据点`)
  handleBatchConfigDialogClose()
}

const handleUploadDialogClose = () => {
  uploadDialogVisible.value = false
}

const handleDatapointUploadSuccess = (file: any, response: any) => {
  ElMessage.success(`数据点配置文件 ${file.name} 上传成功`)
  handleUploadDialogClose()
  // 这里可以解析上传的配置文件并更新数据点列表
}

const handleTimeRangeChange = (range: any) => {
  historyTimeRange.value = range
}

const handleRefreshHistory = () => {
  ElMessage.success('历史数据刷新成功')
}

const handleAddAlarmRule = () => {
  ElMessage.info('新增报警规则功能')
}

const handleAlarmRuleAction = (action: string, row: any) => {
  ElMessage.info(`${action} 报警规则: ${row.name}`)
}

// 辅助方法
const formatValue = (value: any, dataType: string) => {
  if (value === null || value === undefined) return '-'
  
  switch (dataType) {
    case 'boolean':
      return value ? '是' : '否'
    case 'integer':
      return value.toString()
    case 'float':
      return Number(value).toFixed(2)
    case 'string':
      return value.toString()
    case 'datetime':
      return new Date(value).toLocaleString('zh-CN')
    default:
      return value.toString()
  }
}

const formatTime = (time: Date) => {
  return time.toLocaleString('zh-CN')
}

const getDataTypeTagType = (dataType: string) => {
  const typeMap: Record<string, string> = {
    boolean: 'info',
    integer: 'primary',
    float: 'success',
    string: 'warning',
    datetime: 'danger'
  }
  return typeMap[dataType] || 'default'
}

const getValueClass = (datapoint: DataPoint) => {
  const classes = ['value-number']
  
  if (datapoint.quality !== 'good') {
    classes.push('value-uncertain')
  }
  
  if (datapoint.limits) {
    const value = Number(datapoint.currentValue)
    if (value < (datapoint.limits.min || -Infinity) || value > (datapoint.limits.max || Infinity)) {
      classes.push('value-alarm')
    }
  }
  
  return classes.join(' ')
}

const getDriverName = (driverId: string) => {
  const driverMap: Record<string, string> = {
    '1': 'PLC主控制器',
    '2': '温度传感器组',
    '3': 'SCADA系统',
    '4': 'MQTT网关'
  }
  return driverMap[driverId] || '未知驱动'
}

const getConditionText = (condition: string) => {
  const conditionMap: Record<string, string> = {
    gt: '>',
    lt: '<',
    eq: '=',
    ne: '≠',
    range: '范围'
  }
  return conditionMap[condition] || condition
}

const hasAlarm = (datapoint: DataPoint) => {
  // 模拟报警判断逻辑
  return datapoint.quality === 'bad' || 
         datapoint.status === 'error' ||
         (datapoint.limits && datapoint.currentValue && 
          (Number(datapoint.currentValue) < (datapoint.limits.min || -Infinity) ||
           Number(datapoint.currentValue) > (datapoint.limits.max || Infinity)))
}

const getDatapointChartData = (datapoint: DataPoint) => {
  // 生成模拟的历史数据
  const now = Date.now()
  const data = []
  
  for (let i = 119; i >= 0; i--) {
    const time = now - i * 60000 // 2小时的数据，每分钟一个点
    let value = Number(datapoint.currentValue) || 0
    value += (Math.random() - 0.5) * value * 0.1 // 添加10%的随机变化
    
    data.push({
      timestamp: new Date(time),
      value: Number(value.toFixed(2))
    })
  }
  
  return {
    series: [
      {
        name: datapoint.name,
        data: data.map(d => ({ x: d.timestamp, y: d.value })),
        unit: datapoint.unit
      }
    ]
  }
}

const getDatapointAlarmRules = (datapoint: DataPoint) => {
  // 返回模拟的报警规则数据
  return [
    {
      id: '1',
      datapointId: datapoint.id,
      name: '高温报警',
      condition: 'gt',
      value: 80,
      priority: 'high',
      enabled: true,
      createTime: new Date('2024-01-15')
    },
    {
      id: '2',
      datapointId: datapoint.id,
      name: '异常值报警',
      condition: 'range',
      value: 0,
      highValue: 200,
      priority: 'medium',
      enabled: true,
      createTime: new Date('2024-01-16')
    }
  ]
}

const configDialogTitle = computed(() => {
  return editingDatapoint.value ? `编辑数据点 - ${editingDatapoint.value.name}` : '新增数据点'
})

// 实时数据更新
const startRealtimeUpdate = () => {
  const updateInterval = setInterval(() => {
    datapoints.value.forEach(dp => {
      if (dp.status === 'active') {
        // 模拟数据值变化
        const baseValue = Number(dp.currentValue) || 0
        const variation = (Math.random() - 0.5) * baseValue * 0.05 // 5%变化
        
        switch (dp.dataType) {
          case 'float':
            dp.currentValue = Number((baseValue + variation).toFixed(2))
            break
          case 'integer':
            dp.currentValue = Math.round(baseValue + variation)
            break
          case 'boolean':
            if (Math.random() < 0.1) { // 10%概率切换
              dp.currentValue = !dp.currentValue
            }
            break
        }
        
        dp.lastReadTime = new Date()
        dp.updateTime = new Date()
        
        // 随机改变数据质量
        if (Math.random() < 0.05) { // 5%概率质量变化
          dp.quality = ['good', 'uncertain', 'bad'][Math.floor(Math.random() * 3)] as any
        } else {
          dp.quality = 'good'
        }
      }
    })
  }, 3000) // 每3秒更新一次
  
  return updateInterval
}

// 生命周期钩子
onMounted(() => {
  // 启动实时更新
  const updateInterval = startRealtimeUpdate()
  
  // 组件卸载时清理
  onUnmounted(() => {
    if (updateInterval) {
      clearInterval(updateInterval)
    }
  })
})
</script>

<style scoped lang="scss">
.datapoints-list-page {
  padding: 16px;
  background: var(--el-bg-color-page);
  min-height: 100vh;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  
  .header-title {
    display: flex;
    align-items: center;
    gap: 12px;
    
    h1 {
      margin: 0;
      font-size: 24px;
      font-weight: 600;
      color: var(--el-text-color-primary);
    }
  }
  
  .header-actions {
    display: flex;
    gap: 12px;
  }
}

.datapoints-overview {
  margin-bottom: 20px;
  
  .stat-card {
    position: relative;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.3s;
    
    &:hover {
      transform: translateY(-2px);
      box-shadow: var(--el-box-shadow);
    }
    
    .stat-content {
      position: relative;
      z-index: 2;
      
      .stat-number {
        font-size: 32px;
        font-weight: bold;
        margin-bottom: 4px;
      }
      
      .stat-label {
        font-size: 14px;
        color: var(--el-text-color-secondary);
      }
    }
    
    .stat-icon {
      position: absolute;
      right: 16px;
      top: 50%;
      transform: translateY(-50%);
      font-size: 48px;
      opacity: 0.1;
    }
    
    &.stat-card--total {
      .stat-number { color: var(--el-color-primary); }
      .stat-icon { color: var(--el-color-primary); }
    }
    
    &.stat-card--active {
      .stat-number { color: var(--el-color-success); }
      .stat-icon { color: var(--el-color-success); }
    }
    
    &.stat-card--good {
      .stat-number { color: var(--el-color-info); }
      .stat-icon { color: var(--el-color-info); }
    }
    
    &.stat-card--alarm {
      .stat-number { color: var(--el-color-danger); }
      .stat-icon { color: var(--el-color-danger); }
    }
  }
}

.datapoints-content {
  .tree-card {
    height: 680px;
    
    .tree-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      .tree-actions {
        display: flex;
        gap: 8px;
      }
    }
  }
  
  .datapoints-filters {
    margin-bottom: 16px;
    
    .filter-content {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 16px;
      
      .filter-left {
        display: flex;
        align-items: center;
      }
    }
  }
  
  .datapoints-table {
    background: var(--el-bg-color);
    border-radius: 8px;
    overflow: hidden;
  }
}

.datapoint-detail {
  .current-value-section {
    .value-display {
      display: flex;
      gap: 40px;
      
      .value-item {
        flex: 1;
        
        .value-label {
          font-size: 14px;
          color: var(--el-text-color-secondary);
          margin-bottom: 8px;
        }
        
        .value-number {
          font-size: 28px;
          font-weight: bold;
          color: var(--el-color-success);
          
          &.value-uncertain {
            color: var(--el-color-warning);
          }
          
          &.value-alarm {
            color: var(--el-color-danger);
          }
          
          .value-unit {
            font-size: 16px;
            color: var(--el-text-color-secondary);
            margin-left: 8px;
          }
        }
        
        .value-time {
          font-size: 14px;
          color: var(--el-text-color-regular);
        }
      }
    }
  }
  
  .datapoint-history {
    .history-controls {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;
    }
  }
  
  .datapoint-alarms {
    .rule-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;
      
      h4 {
        margin: 0;
        font-size: 16px;
        color: var(--el-text-color-primary);
      }
    }
  }
}

.drawer-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.batch-config {
  .batch-info {
    margin-bottom: 20px;
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .datapoints-content {
    .el-row > .el-col:first-child {
      span: 8;
    }
    
    .el-row > .el-col:last-child {
      span: 16;
    }
  }
}

@media (max-width: 768px) {
  .datapoints-list-page {
    padding: 8px;
  }
  
  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }
  
  .datapoints-overview {
    .el-col {
      margin-bottom: 12px;
    }
  }
  
  .datapoints-content {
    .el-row {
      flex-direction: column;
    }
    
    .tree-card {
      height: 300px;
      margin-bottom: 16px;
    }
    
    .filter-content {
      flex-direction: column;
      align-items: flex-start;
      gap: 16px;
      
      .filter-left {
        width: 100%;
      }
    }
  }
}
</style>