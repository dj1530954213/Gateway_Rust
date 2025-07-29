<template>
  <div class="data-points-form">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="title-section">
          <h1>{{ isEditMode ? '编辑数据点' : '新建数据点' }}</h1>
          <p class="description">配置工业数据采集点，设置数据源连接和采集参数</p>
        </div>
        <div class="header-actions">
          <el-button @click="handleCancel">取消</el-button>
          <el-button type="primary" @click="handleSave" :loading="saving">
            {{ isEditMode ? '保存修改' : '创建数据点' }}
          </el-button>
        </div>
      </div>
    </div>

    <!-- 表单内容 -->
    <div class="form-content">
      <el-form
        ref="formRef"
        :model="formData"
        :rules="formRules"
        label-width="120px"
        size="default"
      >
        <!-- 基本信息 -->
        <el-card class="form-section">
          <template #header>
            <div class="section-header">
              <el-icon><Setting /></el-icon>
              <span>基本信息</span>
            </div>
          </template>

          <div class="form-grid">
            <div class="form-left">
              <el-form-item label="数据点名称" prop="name">
                <el-input
                  v-model="formData.name"
                  placeholder="请输入数据点名称"
                  clearable
                />
              </el-form-item>

              <el-form-item label="数据点标识" prop="identifier">
                <el-input
                  v-model="formData.identifier"
                  placeholder="请输入唯一标识符"
                  clearable
                />
                <div class="form-hint">用于系统内部识别的唯一标识符</div>
              </el-form-item>

              <el-form-item label="数据类型" prop="dataType">
                <el-select
                  v-model="formData.dataType"
                  placeholder="选择数据类型"
                  @change="handleDataTypeChange"
                >
                  <el-option label="整数 (Integer)" value="integer" />
                  <el-option label="浮点数 (Float)" value="float" />
                  <el-option label="布尔值 (Boolean)" value="boolean" />
                  <el-option label="字符串 (String)" value="string" />
                  <el-option label="位 (Bit)" value="bit" />
                  <el-option label="字节 (Byte)" value="byte" />
                  <el-option label="双字 (DWord)" value="dword" />
                </el-select>
              </el-form-item>

              <el-form-item label="单位" prop="unit">
                <el-input
                  v-model="formData.unit"
                  placeholder="请输入单位（如：℃、V、A）"
                  clearable
                />
              </el-form-item>

              <el-form-item label="描述">
                <el-input
                  v-model="formData.description"
                  type="textarea"
                  :rows="3"
                  placeholder="请输入数据点描述"
                />
              </el-form-item>
            </div>

            <div class="form-right">
              <el-form-item label="采集频率" prop="scanRate">
                <el-input-number
                  v-model="formData.scanRate"
                  :min="100"
                  :max="60000"
                  :step="100"
                  placeholder="毫秒"
                />
                <div class="form-hint">采集间隔时间（毫秒）</div>
              </el-form-item>

              <el-form-item label="数据点组" prop="group">
                <el-select
                  v-model="formData.group"
                  placeholder="选择数据点组"
                  filterable
                  allow-create
                >
                  <el-option
                    v-for="group in dataPointGroups"
                    :key="group"
                    :label="group"
                    :value="group"
                  />
                </el-select>
              </el-form-item>

              <el-form-item label="访问权限" prop="accessLevel">
                <el-select v-model="formData.accessLevel" placeholder="选择访问权限">
                  <el-option label="只读" value="read" />
                  <el-option label="读写" value="readWrite" />
                  <el-option label="只写" value="write" />
                </el-select>
              </el-form-item>

              <el-form-item label="状态">
                <el-switch
                  v-model="formData.enabled"
                  active-text="启用"
                  inactive-text="禁用"
                />
              </el-form-item>

              <el-form-item label="高优先级">
                <el-switch
                  v-model="formData.highPriority"
                  active-text="是"
                  inactive-text="否"
                />
                <div class="form-hint">高优先级数据点将优先处理</div>
              </el-form-item>
            </div>
          </div>
        </el-card>

        <!-- 数据源配置 -->
        <el-card class="form-section">
          <template #header>
            <div class="section-header">
              <el-icon><Connection /></el-icon>
              <span>数据源配置</span>
            </div>
          </template>

          <div class="form-grid">
            <div class="form-left">
              <el-form-item label="驱动连接器" prop="driverId">
                <el-select
                  v-model="formData.driverId"
                  placeholder="选择驱动连接器"
                  @change="handleDriverChange"
                >
                  <el-option
                    v-for="driver in drivers"
                    :key="driver.id"
                    :label="`${driver.name} (${driver.type})`"
                    :value="driver.id"
                  />
                </el-select>
              </el-form-item>

              <el-form-item label="设备地址" prop="deviceAddress">
                <el-input
                  v-model="formData.deviceAddress"
                  placeholder="请输入设备地址"
                  clearable
                />
                <div class="form-hint">设备在网络中的地址标识</div>
              </el-form-item>

              <el-form-item label="寄存器地址" prop="registerAddress">
                <el-input-number
                  v-model="formData.registerAddress"
                  :min="0"
                  :max="65535"
                  placeholder="寄存器地址"
                />
              </el-form-item>

              <el-form-item label="功能码" prop="functionCode">
                <el-select
                  v-model="formData.functionCode"
                  placeholder="选择功能码"
                >
                  <el-option label="01 - 读线圈状态" value="01" />
                  <el-option label="02 - 读离散输入" value="02" />
                  <el-option label="03 - 读保持寄存器" value="03" />
                  <el-option label="04 - 读输入寄存器" value="04" />
                  <el-option label="05 - 写单个线圈" value="05" />
                  <el-option label="06 - 写单个寄存器" value="06" />
                  <el-option label="15 - 写多个线圈" value="15" />
                  <el-option label="16 - 写多个寄存器" value="16" />
                </el-select>
              </el-form-item>
            </div>

            <div class="form-right">
              <el-form-item label="数据长度" prop="dataLength">
                <el-input-number
                  v-model="formData.dataLength"
                  :min="1"
                  :max="255"
                  placeholder="数据长度"
                />
                <div class="form-hint">读取的数据长度（字节数）</div>
              </el-form-item>

              <el-form-item label="字节序" prop="byteOrder">
                <el-select v-model="formData.byteOrder" placeholder="选择字节序">
                  <el-option label="大端序 (Big Endian)" value="bigEndian" />
                  <el-option label="小端序 (Little Endian)" value="littleEndian" />
                </el-select>
              </el-form-item>

              <el-form-item label="偏移量" prop="offset">
                <el-input-number
                  v-model="formData.offset"
                  :min="0"
                  placeholder="字节偏移量"
                />
              </el-form-item>

              <el-form-item label="超时时间" prop="timeout">
                <el-input-number
                  v-model="formData.timeout"
                  :min="1000"
                  :max="30000"
                  :step="1000"
                  placeholder="毫秒"
                />
                <div class="form-hint">读取操作超时时间（毫秒）</div>
              </el-form-item>
            </div>
          </div>
        </el-card>

        <!-- 数值转换 -->
        <el-card class="form-section">
          <template #header>
            <div class="section-header">
              <el-icon><Operation /></el-icon>
              <span>数值转换</span>
            </div>
          </template>

          <div class="form-grid">
            <div class="form-left">
              <el-form-item label="缩放因子" prop="scaleFactor">
                <el-input-number
                  v-model="formData.scaleFactor"
                  :precision="6"
                  :step="0.1"
                  placeholder="缩放因子"
                />
                <div class="form-hint">原始值乘以此因子得到实际值</div>
              </el-form-item>

              <el-form-item label="偏移值" prop="offsetValue">
                <el-input-number
                  v-model="formData.offsetValue"
                  :precision="6"
                  :step="0.1"
                  placeholder="偏移值"
                />
                <div class="form-hint">转换后添加的偏移值</div>
              </el-form-item>

              <el-form-item label="最小值" prop="minValue">
                <el-input-number
                  v-model="formData.minValue"
                  :precision="6"
                  placeholder="最小值"
                />
              </el-form-item>

              <el-form-item label="最大值" prop="maxValue">
                <el-input-number
                  v-model="formData.maxValue"
                  :precision="6"
                  placeholder="最大值"
                />
              </el-form-item>
            </div>

            <div class="form-right">
              <el-form-item label="小数位数" prop="decimalPlaces">
                <el-input-number
                  v-model="formData.decimalPlaces"
                  :min="0"
                  :max="6"
                  placeholder="小数位数"
                />
              </el-form-item>

              <el-form-item label="数据映射">
                <el-button @click="openMappingDialog" type="primary" text>
                  <el-icon><Edit /></el-icon>
                  配置映射规则
                </el-button>
                <div class="form-hint">
                  {{ formData.mappingRules.length > 0 ? `已配置 ${formData.mappingRules.length} 条映射规则` : '未配置映射规则' }}
                </div>
              </el-form-item>

              <el-form-item label="数值验证">
                <el-switch
                  v-model="formData.enableValidation"
                  active-text="启用"
                  inactive-text="禁用"
                />
                <div class="form-hint">启用数值范围验证</div>
              </el-form-item>
            </div>
          </div>
        </el-card>

        <!-- 报警配置 -->
        <el-card class="form-section">
          <template #header>
            <div class="section-header">
              <el-icon><Warning /></el-icon>
              <span>报警配置</span>
            </div>
          </template>

          <div class="alarm-config">
            <el-form-item label="启用报警">
              <el-switch
                v-model="formData.enableAlarm"
                active-text="启用"
                inactive-text="禁用"
              />
            </el-form-item>

            <div v-if="formData.enableAlarm" class="alarm-rules">
              <div class="alarm-rule-grid">
                <el-form-item label="高高报警">
                  <el-input-number
                    v-model="formData.alarmConfig.highHigh"
                    :precision="2"
                    placeholder="高高报警值"
                  />
                </el-form-item>

                <el-form-item label="高报警">
                  <el-input-number
                    v-model="formData.alarmConfig.high"
                    :precision="2"
                    placeholder="高报警值"
                  />
                </el-form-item>

                <el-form-item label="低报警">
                  <el-input-number
                    v-model="formData.alarmConfig.low"
                    :precision="2"
                    placeholder="低报警值"
                  />
                </el-form-item>

                <el-form-item label="低低报警">
                  <el-input-number
                    v-model="formData.alarmConfig.lowLow"
                    :precision="2"
                    placeholder="低低报警值"
                  />
                </el-form-item>
              </div>

              <el-form-item label="延时确认">
                <el-input-number
                  v-model="formData.alarmConfig.delay"
                  :min="0"
                  :max="300"
                  placeholder="秒"
                />
                <div class="form-hint">报警触发后的延时确认时间（秒）</div>
              </el-form-item>
            </div>
          </div>
        </el-card>

        <!-- 存储配置 -->
        <el-card class="form-section">
          <template #header>
            <div class="section-header">
              <el-icon><Monitor /></el-icon>
              <span>存储配置</span>
            </div>
          </template>

          <div class="form-grid">
            <div class="form-left">
              <el-form-item label="启用历史存储">
                <el-switch
                  v-model="formData.enableHistory"
                  active-text="启用"
                  inactive-text="禁用"
                />
              </el-form-item>

              <el-form-item v-if="formData.enableHistory" label="存储频率">
                <el-select v-model="formData.historyConfig.frequency" placeholder="选择存储频率">
                  <el-option label="每次变化" value="onChange" />
                  <el-option label="每秒" value="1s" />
                  <el-option label="每10秒" value="10s" />
                  <el-option label="每分钟" value="1m" />
                  <el-option label="每5分钟" value="5m" />
                  <el-option label="每小时" value="1h" />
                </el-select>
              </el-form-item>

              <el-form-item v-if="formData.enableHistory" label="变化阈值">
                <el-input-number
                  v-model="formData.historyConfig.changeThreshold"
                  :precision="2"
                  :min="0"
                  placeholder="变化阈值"
                />
                <div class="form-hint">数值变化超过此值才存储</div>
              </el-form-item>
            </div>

            <div class="form-right">
              <el-form-item v-if="formData.enableHistory" label="保留期限">
                <el-input-number
                  v-model="formData.historyConfig.retentionDays"
                  :min="1"
                  :max="3650"
                  placeholder="天数"
                />
                <div class="form-hint">历史数据保留天数</div>
              </el-form-item>

              <el-form-item v-if="formData.enableHistory" label="压缩存储">
                <el-switch
                  v-model="formData.historyConfig.compression"
                  active-text="启用"
                  inactive-text="禁用"
                />
              </el-form-item>

              <el-form-item label="实时缓存">
                <el-switch
                  v-model="formData.enableCache"
                  active-text="启用"
                  inactive-text="禁用"
                />
                <div class="form-hint">启用实时数据缓存以提高性能</div>
              </el-form-item>
            </div>
          </div>
        </el-card>

        <!-- 测试区域 -->
        <el-card class="form-section">
          <template #header>
            <div class="section-header">
              <el-icon><Monitor /></el-icon>
              <span>连接测试</span>
            </div>
          </template>

          <div class="test-section">
            <div class="test-controls">
              <el-button type="primary" @click="testConnection" :loading="testing">
                <el-icon><Connection /></el-icon>
                测试连接
              </el-button>
              <el-button @click="readValue" :loading="reading">
                <el-icon><View /></el-icon>
                读取数值
              </el-button>
              <el-button v-if="formData.accessLevel !== 'read'" @click="openWriteDialog">
                <el-icon><Edit /></el-icon>
                写入测试
              </el-button>
            </div>

            <div v-if="testResult" class="test-result">
              <div class="result-item">
                <label>连接状态:</label>
                <el-tag :type="testResult.connected ? 'success' : 'danger'">
                  {{ testResult.connected ? '连接成功' : '连接失败' }}
                </el-tag>
              </div>
              <div v-if="testResult.value !== undefined" class="result-item">
                <label>读取数值:</label>
                <span class="value">{{ testResult.value }} {{ formData.unit }}</span>
              </div>
              <div v-if="testResult.timestamp" class="result-item">
                <label>读取时间:</label>
                <span>{{ formatDate(testResult.timestamp) }}</span>
              </div>
              <div v-if="testResult.quality" class="result-item">
                <label>数据质量:</label>
                <el-tag :type="getQualityType(testResult.quality)">
                  {{ getQualityLabel(testResult.quality) }}
                </el-tag>
              </div>
              <div v-if="testResult.error" class="result-item error">
                <label>错误信息:</label>
                <span>{{ testResult.error }}</span>
              </div>
            </div>
          </div>
        </el-card>
      </el-form>
    </div>

    <!-- 映射规则对话框 -->
    <el-dialog
      v-model="mappingDialogVisible"
      title="配置数据映射"
      width="600px"
    >
      <div class="mapping-config">
        <div class="mapping-toolbar">
          <el-button type="primary" size="small" @click="addMappingRule">
            <el-icon><Plus /></el-icon>
            添加映射
          </el-button>
        </div>

        <el-table :data="formData.mappingRules" style="width: 100%">
          <el-table-column prop="input" label="输入值" width="120">
            <template #default="{ row, $index }">
              <el-input-number
                v-model="row.input"
                size="small"
                :precision="2"
              />
            </template>
          </el-table-column>
          <el-table-column prop="output" label="输出值" width="120">
            <template #default="{ row, $index }">
              <el-input-number
                v-model="row.output"
                size="small"
                :precision="2"
              />
            </template>
          </el-table-column>
          <el-table-column prop="description" label="描述">
            <template #default="{ row, $index }">
              <el-input
                v-model="row.description"
                size="small"
                placeholder="映射描述"
              />
            </template>
          </el-table-column>
          <el-table-column label="操作" width="80">
            <template #default="{ row, $index }">
              <el-button
                type="danger"
                size="small"
                text
                @click="removeMappingRule($index)"
              >
                删除
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <template #footer>
        <el-button @click="mappingDialogVisible = false">关闭</el-button>
      </template>
    </el-dialog>

    <!-- 写入测试对话框 -->
    <el-dialog
      v-model="writeDialogVisible"
      title="写入测试"
      width="400px"
    >
      <el-form label-width="80px">
        <el-form-item label="写入值">
          <el-input-number
            v-model="writeValue"
            :precision="formData.decimalPlaces"
            placeholder="请输入要写入的值"
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="writeDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="writeTestValue" :loading="writing">
          写入
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import {
  Setting, Connection, Operation, Warning, Monitor,
  Plus, Edit, View
} from '@element-plus/icons-vue'

interface DataPoint {
  id?: string
  name: string
  identifier: string
  dataType: string
  unit: string
  description: string
  scanRate: number
  group: string
  accessLevel: string
  enabled: boolean
  highPriority: boolean
  driverId: string
  deviceAddress: string
  registerAddress: number
  functionCode: string
  dataLength: number
  byteOrder: string
  offset: number
  timeout: number
  scaleFactor: number
  offsetValue: number
  minValue?: number
  maxValue?: number
  decimalPlaces: number
  enableValidation: boolean
  mappingRules: Array<{
    input: number
    output: number
    description: string
  }>
  enableAlarm: boolean
  alarmConfig: {
    highHigh?: number
    high?: number
    low?: number
    lowLow?: number
    delay: number
  }
  enableHistory: boolean
  historyConfig: {
    frequency: string
    changeThreshold: number
    retentionDays: number
    compression: boolean
  }
  enableCache: boolean
}

interface Driver {
  id: string
  name: string
  type: string
  status: string
}

interface TestResult {
  connected: boolean
  value?: any
  timestamp?: string
  quality?: string
  error?: string
}

// 响应式数据
const router = useRouter()
const route = useRoute()
const formRef = ref<FormInstance>()
const saving = ref(false)
const testing = ref(false)
const reading = ref(false)
const writing = ref(false)
const mappingDialogVisible = ref(false)
const writeDialogVisible = ref(false)
const writeValue = ref(0)

// 判断是否为编辑模式
const isEditMode = computed(() => !!route.params.id)

// 表单数据
const formData = reactive<DataPoint>({
  name: '',
  identifier: '',
  dataType: 'float',
  unit: '',
  description: '',
  scanRate: 1000,
  group: '',
  accessLevel: 'read',
  enabled: true,
  highPriority: false,
  driverId: '',
  deviceAddress: '',
  registerAddress: 0,
  functionCode: '03',
  dataLength: 2,
  byteOrder: 'bigEndian',
  offset: 0,
  timeout: 5000,
  scaleFactor: 1,
  offsetValue: 0,
  decimalPlaces: 2,
  enableValidation: false,
  mappingRules: [],
  enableAlarm: false,
  alarmConfig: {
    delay: 5
  },
  enableHistory: true,
  historyConfig: {
    frequency: 'onChange',
    changeThreshold: 0.1,
    retentionDays: 30,
    compression: true
  },
  enableCache: true
})

// 驱动列表
const drivers = ref<Driver[]>([])

// 数据点组列表
const dataPointGroups = ref<string[]>([
  '温度传感器',
  '压力传感器',
  '流量计',
  '电量表',
  '控制阀',
  '泵站',
  '风机',
  '其他'
])

// 测试结果
const testResult = ref<TestResult | null>(null)

// 表单验证规则
const formRules: FormRules = {
  name: [
    { required: true, message: '请输入数据点名称', trigger: 'blur' }
  ],
  identifier: [
    { required: true, message: '请输入数据点标识', trigger: 'blur' },
    { pattern: /^[a-zA-Z][a-zA-Z0-9_]*$/, message: '标识符必须以字母开头，只能包含字母、数字和下划线', trigger: 'blur' }
  ],
  dataType: [
    { required: true, message: '请选择数据类型', trigger: 'change' }
  ],
  scanRate: [
    { required: true, message: '请设置采集频率', trigger: 'blur' },
    { type: 'number', min: 100, max: 60000, message: '采集频率必须在100-60000毫秒之间', trigger: 'blur' }
  ],
  driverId: [
    { required: true, message: '请选择驱动连接器', trigger: 'change' }
  ],
  registerAddress: [
    { required: true, message: '请输入寄存器地址', trigger: 'blur' },
    { type: 'number', min: 0, max: 65535, message: '寄存器地址必须在0-65535之间', trigger: 'blur' }
  ],
  functionCode: [
    { required: true, message: '请选择功能码', trigger: 'change' }
  ]
}

// 工具函数
const formatDate = (dateString: string): string => {
  return new Date(dateString).toLocaleString('zh-CN')
}

const getQualityType = (quality: string) => {
  const types = {
    'good': 'success',
    'uncertain': 'warning',
    'bad': 'danger'
  }
  return types[quality as keyof typeof types] || 'info'
}

const getQualityLabel = (quality: string): string => {
  const labels = {
    'good': '良好',
    'uncertain': '不确定',
    'bad': '错误'
  }
  return labels[quality as keyof typeof labels] || quality
}

// 事件处理
const handleCancel = () => {
  router.back()
}

const handleSave = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
    saving.value = true

    // 模拟保存数据点
    await new Promise(resolve => setTimeout(resolve, 1500))

    const action = isEditMode.value ? '更新' : '创建'
    ElMessage.success(`数据点${action}成功`)
    
    // 返回列表页
    router.push('/data-points')
  } catch (error) {
    console.error('保存失败:', error)
    ElMessage.error('保存失败，请检查表单数据')
  } finally {
    saving.value = false
  }
}

const handleDataTypeChange = (dataType: string) => {
  // 根据数据类型调整默认参数
  switch (dataType) {
    case 'integer':
      formData.dataLength = 2
      formData.decimalPlaces = 0
      break
    case 'float':
      formData.dataLength = 4
      formData.decimalPlaces = 2
      break
    case 'boolean':
    case 'bit':
      formData.dataLength = 1
      formData.decimalPlaces = 0
      break
    case 'byte':
      formData.dataLength = 1
      formData.decimalPlaces = 0
      break
    case 'dword':
      formData.dataLength = 4
      formData.decimalPlaces = 0
      break
    case 'string':
      formData.dataLength = 32
      formData.decimalPlaces = 0
      break
  }
}

const handleDriverChange = (driverId: string) => {
  const driver = drivers.value.find(d => d.id === driverId)
  if (driver) {
    // 根据驱动类型设置默认参数
    if (driver.type === 'ModbusTCP' || driver.type === 'ModbusRTU') {
      formData.functionCode = '03'
      formData.timeout = 5000
    }
  }
}

const testConnection = async () => {
  if (!formData.driverId) {
    ElMessage.warning('请先选择驱动连接器')
    return
  }

  try {
    testing.value = true
    testResult.value = null

    // 模拟连接测试
    await new Promise(resolve => setTimeout(resolve, 2000))

    const success = Math.random() > 0.2 // 80% 成功率
    testResult.value = {
      connected: success,
      timestamp: new Date().toISOString(),
      error: success ? undefined : '连接超时或设备无响应'
    }

    if (success) {
      ElMessage.success('连接测试成功')
    } else {
      ElMessage.error('连接测试失败')
    }
  } catch (error) {
    console.error('测试连接失败:', error)
    testResult.value = {
      connected: false,
      error: '测试连接时发生错误'
    }
  } finally {
    testing.value = false
  }
}

const readValue = async () => {
  if (!formData.driverId || !formData.registerAddress) {
    ElMessage.warning('请先配置驱动连接器和寄存器地址')
    return
  }

  try {
    reading.value = true

    // 模拟读取数值
    await new Promise(resolve => setTimeout(resolve, 1500))

    const success = Math.random() > 0.15 // 85% 成功率
    if (success) {
      let value: any
      switch (formData.dataType) {
        case 'boolean':
        case 'bit':
          value = Math.random() > 0.5
          break
        case 'integer':
          value = Math.floor(Math.random() * 1000)
          break
        case 'float':
          value = (Math.random() * 100).toFixed(formData.decimalPlaces)
          break
        case 'string':
          value = 'Sample String'
          break
        default:
          value = (Math.random() * 100).toFixed(2)
      }

      testResult.value = {
        connected: true,
        value,
        timestamp: new Date().toISOString(),
        quality: Math.random() > 0.1 ? 'good' : 'uncertain'
      }

      ElMessage.success('数值读取成功')
    } else {
      testResult.value = {
        connected: false,
        error: '读取失败：设备无响应或地址错误'
      }
      ElMessage.error('数值读取失败')
    }
  } catch (error) {
    console.error('读取数值失败:', error)
    ElMessage.error('读取数值时发生错误')
  } finally {
    reading.value = false
  }
}

const openWriteDialog = () => {
  writeValue.value = 0
  writeDialogVisible.value = true
}

const writeTestValue = async () => {
  try {
    writing.value = true

    // 模拟写入操作
    await new Promise(resolve => setTimeout(resolve, 1000))

    const success = Math.random() > 0.2 // 80% 成功率
    if (success) {
      ElMessage.success(`成功写入值: ${writeValue.value}`)
    } else {
      ElMessage.error('写入失败：设备无响应或权限不足')
    }

    writeDialogVisible.value = false
  } catch (error) {
    console.error('写入失败:', error)
    ElMessage.error('写入操作时发生错误')
  } finally {
    writing.value = false
  }
}

const openMappingDialog = () => {
  mappingDialogVisible.value = true
}

const addMappingRule = () => {
  formData.mappingRules.push({
    input: 0,
    output: 0,
    description: ''
  })
}

const removeMappingRule = (index: number) => {
  formData.mappingRules.splice(index, 1)
}

// 模拟数据加载
const loadMockData = () => {
  // 模拟驱动列表
  drivers.value = [
    {
      id: 'driver_1',
      name: 'PLC_01',
      type: 'ModbusTCP',
      status: 'connected'
    },
    {
      id: 'driver_2',
      name: 'RTU_Device',
      type: 'ModbusRTU',
      status: 'connected'
    },
    {
      id: 'driver_3',
      name: 'OPC_Server',
      type: 'OPC_UA',
      status: 'connected'
    }
  ]

  // 如果是编辑模式，加载现有数据
  if (isEditMode.value) {
    // 模拟加载现有数据点数据
    Object.assign(formData, {
      name: '温度传感器01',
      identifier: 'TEMP_01',
      dataType: 'float',
      unit: '℃',
      description: '锅炉进水温度传感器',
      scanRate: 2000,
      group: '温度传感器',
      accessLevel: 'read',
      enabled: true,
      highPriority: false,
      driverId: 'driver_1',
      deviceAddress: '192.168.1.100',
      registerAddress: 40001,
      functionCode: '03',
      dataLength: 2,
      byteOrder: 'bigEndian',
      offset: 0,
      timeout: 5000,
      scaleFactor: 0.1,
      offsetValue: 0,
      minValue: -50,
      maxValue: 200,
      decimalPlaces: 1,
      enableValidation: true,
      mappingRules: [],
      enableAlarm: true,
      alarmConfig: {
        highHigh: 180,
        high: 160,
        low: 10,
        lowLow: 0,
        delay: 5
      },
      enableHistory: true,
      historyConfig: {
        frequency: 'onChange',
        changeThreshold: 0.5,
        retentionDays: 90,
        compression: true
      },
      enableCache: true
    })
  }
}

// 生命周期
onMounted(() => {
  loadMockData()
})
</script>

<style scoped lang="scss">
.data-points-form {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #f5f7fa;
  
  .page-header {
    background: white;
    padding: 24px;
    border-bottom: 1px solid #e4e7ed;
    
    .header-content {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      .title-section {
        h1 {
          margin: 0;
          font-size: 24px;
          font-weight: 600;
          color: #303133;
        }
        
        .description {
          margin: 8px 0 0 0;
          color: #909399;
          font-size: 14px;
        }
      }
      
      .header-actions {
        display: flex;
        gap: 12px;
      }
    }
  }
  
  .form-content {
    flex: 1;
    padding: 20px;
    overflow-y: auto;
    
    .form-section {
      margin-bottom: 20px;
      
      .section-header {
        display: flex;
        align-items: center;
        gap: 8px;
        font-weight: 500;
        color: #303133;
      }
    }
    
    .form-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 30px;
      
      @media (max-width: 1200px) {
        grid-template-columns: 1fr;
        gap: 0;
      }
    }
    
    .form-hint {
      font-size: 12px;
      color: #909399;
      margin-top: 4px;
      line-height: 1.4;
    }
    
    .alarm-config {
      .alarm-rules {
        margin-top: 16px;
        
        .alarm-rule-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 20px;
          margin-bottom: 16px;
          
          @media (max-width: 768px) {
            grid-template-columns: 1fr;
          }
        }
      }
    }
    
    .test-section {
      .test-controls {
        display: flex;
        gap: 12px;
        margin-bottom: 16px;
        flex-wrap: wrap;
      }
      
      .test-result {
        background: #f8f9fa;
        border: 1px solid #e9ecef;
        border-radius: 4px;
        padding: 16px;
        
        .result-item {
          display: flex;
          align-items: center;
          margin-bottom: 8px;
          
          &:last-child {
            margin-bottom: 0;
          }
          
          label {
            min-width: 80px;
            font-weight: 500;
            color: #606266;
          }
          
          .value {
            font-family: monospace;
            font-size: 16px;
            font-weight: bold;
            color: #409eff;
          }
          
          &.error {
            color: #f56c6c;
          }
        }
      }
    }
  }
  
  .mapping-config {
    .mapping-toolbar {
      margin-bottom: 16px;
    }
  }
}

// Element Plus 样式调整
.el-form-item {
  margin-bottom: 20px;
}

.el-card {
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  
  :deep(.el-card__header) {
    background: #f8f9fa;
    border-bottom: 1px solid #e9ecef;
  }
}

.el-input-number {
  width: 100%;
}

.el-select {
  width: 100%;
}

// 响应式设计
@media (max-width: 768px) {
  .data-points-form {
    .page-header {
      padding: 16px;
      
      .header-content {
        flex-direction: column;
        align-items: flex-start;
        gap: 16px;
      }
    }
    
    .form-content {
      padding: 16px;
    }
    
    .test-section .test-controls {
      flex-direction: column;
      
      .el-button {
        width: 100%;
      }
    }
  }
}
</style>