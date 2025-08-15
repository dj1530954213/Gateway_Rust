<template>
  <el-dialog
    v-model="dialogVisible"
    :title="dialogTitle"
    width="800px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @closed="handleDialogClosed"
  >
    <el-form
      ref="formRef"
      :model="formData"
      :rules="formRules"
      label-width="120px"
      size="default"
    >
      <!-- 基本信息 -->
      <el-card class="form-section" shadow="never">
        <template #header>
          <span class="section-title">基本信息</span>
        </template>

        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="设备名称" prop="name">
              <el-input
                v-model="formData.name"
                placeholder="请输入设备名称"
                clearable
              />
            </el-form-item>
          </el-col>

          <el-col :span="12">
            <el-form-item label="协议类型" prop="protocol">
              <el-select
                v-model="formData.protocol"
                placeholder="请选择协议类型"
                style="width: 100%"
                @change="handleProtocolChange"
              >
                <el-option label="Modbus TCP" value="ModbusTcp" />
                <el-option label="Modbus RTU" value="ModbusRtu" />
                <el-option label="OPC UA" value="OpcUa" />
                <el-option label="MQTT" value="Mqtt" />
              </el-select>
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item label="设备位置" prop="location">
          <el-input
            v-model="formData.location"
            type="textarea"
            :rows="3"
            placeholder="请输入设备位置信息（可选）"
            maxlength="500"
            show-word-limit
          />
        </el-form-item>

        <el-form-item label="启用状态" prop="enabled">
          <el-switch
            v-model="formData.enabled"
            active-text="启用"
            inactive-text="禁用"
          />
        </el-form-item>
      </el-card>

      <!-- 连接配置 -->
      <el-card class="form-section" shadow="never">
        <template #header>
          <span class="section-title">连接配置</span>
        </template>

        <!-- Modbus TCP 配置 -->
        <template v-if="formData.protocol === 'ModbusTcp'">
          <el-row :gutter="20">
            <el-col :span="12">
              <el-form-item label="主机地址" prop="config.host">
                <el-input
                  v-model="formData.config.host"
                  placeholder="请输入IP地址或主机名"
                />
              </el-form-item>
            </el-col>

            <el-col :span="12">
              <el-form-item label="端口号" prop="config.port">
                <el-input-number
                  v-model="formData.config.port"
                  :min="1"
                  :max="65535"
                  placeholder="502"
                  style="width: 100%"
                />
              </el-form-item>
            </el-col>
          </el-row>

          <el-row :gutter="20">
            <el-col :span="12">
              <el-form-item label="从站ID" prop="config.slave_id">
                <el-input-number
                  v-model="formData.config.slave_id"
                  :min="1"
                  :max="255"
                  placeholder="1"
                  style="width: 100%"
                />
              </el-form-item>
            </el-col>

            <el-col :span="12">
              <el-form-item label="连接超时" prop="config.timeout">
                <el-input-number
                  v-model="formData.config.timeout"
                  :min="1000"
                  :max="30000"
                  :step="1000"
                  placeholder="5000"
                  style="width: 100%"
                >
                  <template #append>ms</template>
                </el-input-number>
              </el-form-item>
            </el-col>
          </el-row>
        </template>

        <!-- Modbus RTU 配置 -->
        <template v-else-if="formData.protocol === 'ModbusRtu'">
          <el-row :gutter="20">
            <el-col :span="12">
              <el-form-item label="串口端口" prop="config.port_name">
                <el-input
                  v-model="formData.config.port_name"
                  placeholder="如: COM1, /dev/ttyUSB0"
                />
              </el-form-item>
            </el-col>

            <el-col :span="12">
              <el-form-item label="波特率" prop="config.baud_rate">
                <el-select
                  v-model="formData.config.baud_rate"
                  placeholder="请选择波特率"
                  style="width: 100%"
                >
                  <el-option label="9600" :value="9600" />
                  <el-option label="19200" :value="19200" />
                  <el-option label="38400" :value="38400" />
                  <el-option label="57600" :value="57600" />
                  <el-option label="115200" :value="115200" />
                </el-select>
              </el-form-item>
            </el-col>
          </el-row>

          <el-row :gutter="20">
            <el-col :span="8">
              <el-form-item label="数据位" prop="config.data_bits">
                <el-select
                  v-model="formData.config.data_bits"
                  style="width: 100%"
                >
                  <el-option label="7" :value="7" />
                  <el-option label="8" :value="8" />
                </el-select>
              </el-form-item>
            </el-col>

            <el-col :span="8">
              <el-form-item label="停止位" prop="config.stop_bits">
                <el-select
                  v-model="formData.config.stop_bits"
                  style="width: 100%"
                >
                  <el-option label="1" :value="1" />
                  <el-option label="2" :value="2" />
                </el-select>
              </el-form-item>
            </el-col>

            <el-col :span="8">
              <el-form-item label="校验位" prop="config.parity">
                <el-select v-model="formData.config.parity" style="width: 100%">
                  <el-option label="无" value="None" />
                  <el-option label="奇校验" value="Odd" />
                  <el-option label="偶校验" value="Even" />
                </el-select>
              </el-form-item>
            </el-col>
          </el-row>

          <el-form-item label="从站ID" prop="config.slave_id">
            <el-input-number
              v-model="formData.config.slave_id"
              :min="1"
              :max="255"
              placeholder="1"
              style="width: 200px"
            />
          </el-form-item>
        </template>

        <!-- OPC UA 配置 -->
        <template v-else-if="formData.protocol === 'OpcUa'">
          <el-form-item label="端点URL" prop="config.endpoint">
            <el-input
              v-model="formData.config.endpoint"
              placeholder="opc.tcp://localhost:4840"
            />
          </el-form-item>

          <el-row :gutter="20">
            <el-col :span="12">
              <el-form-item label="用户名" prop="config.username">
                <el-input
                  v-model="formData.config.username"
                  placeholder="用户名（可选）"
                />
              </el-form-item>
            </el-col>

            <el-col :span="12">
              <el-form-item label="密码" prop="config.password">
                <el-input
                  v-model="formData.config.password"
                  type="password"
                  placeholder="密码（可选）"
                  show-password
                />
              </el-form-item>
            </el-col>
          </el-row>
        </template>

        <!-- MQTT 配置 -->
        <template v-else-if="formData.protocol === 'Mqtt'">
          <el-row :gutter="20">
            <el-col :span="16">
              <el-form-item label="Broker地址" prop="config.broker_url">
                <el-input
                  v-model="formData.config.broker_url"
                  placeholder="mqtt://localhost:1883"
                />
              </el-form-item>
            </el-col>

            <el-col :span="8">
              <el-form-item label="客户端ID" prop="config.client_id">
                <el-input
                  v-model="formData.config.client_id"
                  placeholder="自动生成"
                />
              </el-form-item>
            </el-col>
          </el-row>

          <el-row :gutter="20">
            <el-col :span="12">
              <el-form-item label="用户名" prop="config.username">
                <el-input
                  v-model="formData.config.username"
                  placeholder="用户名（可选）"
                />
              </el-form-item>
            </el-col>

            <el-col :span="12">
              <el-form-item label="密码" prop="config.password">
                <el-input
                  v-model="formData.config.password"
                  type="password"
                  placeholder="密码（可选）"
                  show-password
                />
              </el-form-item>
            </el-col>
          </el-row>
        </template>
      </el-card>
    </el-form>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleCancel">取消</el-button>
        <el-button
          v-if="mode === 'create'"
          type="info"
          :loading="testing"
          @click="handleTestConnection"
        >
          测试连接
        </el-button>
        <el-button type="primary" :loading="submitting" @click="handleSubmit">
          {{ mode === 'create' ? '创建' : '保存' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * DeviceFormDialog —— 设备表单对话框
 *
 * 📝 Responsibilities:
 *  1. 新增/编辑设备表单
 *  2. 动态协议配置
 *  3. 表单验证
 *  4. 连接测试
 *
 * 📦 Dependencies:
 *  - Element Plus 表单组件
 *  - 设备 API 和 Store
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { ref, computed, watch, nextTick } from 'vue'

import type { DeviceVO, DeviceCreateReq } from '@/api/devices'
import { useDevicesStore } from '@/stores'

// ===== Props =====
const props = defineProps<{
  visible: boolean
  device?: DeviceVO | null
  mode: 'create' | 'edit'
}>()

// ===== Emits =====
const emit = defineEmits<{
  'update:visible': [visible: boolean]
  success: []
}>()

// ===== Store =====
const devicesStore = useDevicesStore()

// ===== 响应式数据 =====
const formRef = ref<FormInstance>()
const submitting = ref(false)
const testing = ref(false)

// 对话框显示状态
const dialogVisible = computed({
  get: () => props.visible,
  set: value => emit('update:visible', value),
})

// 对话框标题
const dialogTitle = computed(() => {
  return props.mode === 'create' ? '新增设备' : '编辑设备'
})

// 表单数据
const formData = ref<DeviceCreateReq>({
  name: '',
  protocol: 'ModbusTcp',
  config: {},
  location: '',
  enabled: true,
})

// 表单验证规则
const formRules: FormRules = {
  name: [
    { required: true, message: '请输入设备名称', trigger: 'blur' },
    {
      min: 2,
      max: 50,
      message: '设备名称长度应在 2-50 字符之间',
      trigger: 'blur',
    },
  ],
  protocol: [{ required: true, message: '请选择协议类型', trigger: 'change' }],
  'config.host': [
    { required: true, message: '请输入主机地址', trigger: 'blur' },
  ],
  'config.port': [
    {
      type: 'number',
      min: 1,
      max: 65535,
      message: '端口号应在 1-65535 之间',
      trigger: 'blur',
    },
  ],
  'config.port_name': [
    { required: true, message: '请输入串口端口', trigger: 'blur' },
  ],
  'config.baud_rate': [
    { required: true, message: '请选择波特率', trigger: 'change' },
  ],
  'config.endpoint': [
    { required: true, message: '请输入端点URL', trigger: 'blur' },
  ],
  'config.broker_url': [
    { required: true, message: '请输入Broker地址', trigger: 'blur' },
  ],
}

// ===== 方法 =====

/**
 * 处理协议变更
 */
function handleProtocolChange(protocol: string) {
  // 清空连接配置
  formData.value.config = {}

  // 设置默认值
  switch (protocol) {
    case 'ModbusTcp':
      formData.value.config = {
        host: '',
        port: 502,
        slave_id: 1,
        timeout: 5000,
      }
      break
    case 'ModbusRtu':
      formData.value.config = {
        port_name: '',
        baud_rate: 9600,
        data_bits: 8,
        stop_bits: 1,
        parity: 'None',
        slave_id: 1,
      }
      break
    case 'OpcUa':
      formData.value.config = {
        endpoint: 'opc.tcp://localhost:4840',
        username: '',
        password: '',
      }
      break
    case 'Mqtt':
      formData.value.config = {
        broker_url: 'mqtt://localhost:1883',
        client_id: '',
        username: '',
        password: '',
      }
      break
  }
}

/**
 * 测试连接
 */
async function handleTestConnection() {
  // 先验证表单
  if (!formRef.value) return

  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) {
    ElMessage.warning('请先完善表单信息')
    return
  }

  testing.value = true
  try {
    const success = await devicesStore.testDeviceConnection(formData.value)
    if (success) {
      ElMessage.success('连接测试成功')
    }
  } finally {
    testing.value = false
  }
}

/**
 * 提交表单
 */
async function handleSubmit() {
  if (!formRef.value) return

  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) return

  submitting.value = true

  try {
    let success = false

    if (props.mode === 'create') {
      const result = await devicesStore.createDevice(formData.value)
      success = !!result
    } else if (props.device) {
      const result = await devicesStore.updateDevice(
        props.device.id,
        formData.value
      )
      success = !!result
    }

    if (success) {
      emit('success')
      ElMessage.success(`设备${props.mode === 'create' ? '创建' : '更新'}成功`)
    }
  } finally {
    submitting.value = false
  }
}

/**
 * 取消操作
 */
function handleCancel() {
  dialogVisible.value = false
}

/**
 * 对话框关闭处理
 */
function handleDialogClosed() {
  // 重置表单
  formRef.value?.resetFields()

  // 重置状态
  submitting.value = false
  testing.value = false
}

/**
 * 初始化表单数据
 */
function initFormData() {
  if (props.mode === 'edit' && props.device) {
    // 编辑模式：填充现有数据
    formData.value = {
      name: props.device.name,
      protocol: props.device.protocol,
      config: { ...props.device.config },
      location: props.device.location || '',
      enabled: props.device.enabled,
    }
  } else {
    // 创建模式：重置为默认值
    formData.value = {
      name: '',
      protocol: 'ModbusTcp',
      config: {
        host: '',
        port: 502,
        slave_id: 1,
        timeout: 5000,
      },
      location: '',
      enabled: true,
    }
  }
}

// ===== 监听器 =====
watch(
  () => props.visible,
  visible => {
    if (visible) {
      nextTick(() => {
        initFormData()
      })
    }
  }
)

watch(
  () => props.device,
  () => {
    if (props.visible) {
      initFormData()
    }
  }
)
</script>

<style scoped lang="scss">
.form-section {
  margin-bottom: 20px;

  &:last-child {
    margin-bottom: 0;
  }

  .section-title {
    font-weight: 600;
    color: #303133;
  }

  :deep(.el-card__header) {
    padding: 12px 20px;
    background-color: #f8f9fa;
    border-bottom: 1px solid #ebeef5;
  }

  :deep(.el-card__body) {
    padding: 20px;
  }
}

.dialog-footer {
  text-align: right;

  .el-button {
    margin-left: 12px;

    &:first-child {
      margin-left: 0;
    }
  }
}

// 表单项优化
:deep(.el-form-item) {
  margin-bottom: 18px;

  .el-form-item__label {
    font-weight: 500;
    color: #606266;
  }

  .el-form-item__content {
    .el-input__wrapper,
    .el-select .el-input__wrapper,
    .el-textarea__inner {
      border-radius: 6px;
    }
  }
}

// 输入框组样式
:deep(.el-input-group__append) {
  background-color: #f5f7fa;
  border-color: #dcdfe6;
  color: #909399;
}

// 响应式设计
@media (max-width: 768px) {
  :deep(.el-dialog) {
    width: 95% !important;
    margin: 5vh auto;
  }

  .form-section {
    :deep(.el-row) {
      .el-col {
        margin-bottom: 10px;
      }
    }
  }
}
</style>
