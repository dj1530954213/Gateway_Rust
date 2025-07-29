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
            <el-form-item label="点位名称" prop="name">
              <el-input
                v-model="formData.name"
                placeholder="请输入点位名称"
                clearable
              />
            </el-form-item>
          </el-col>
          
          <el-col :span="12">
            <el-form-item label="所属设备" prop="device_id">
              <el-select
                v-model="formData.device_id"
                placeholder="请选择设备"
                style="width: 100%"
                filterable
              >
                <el-option
                  v-for="device in availableDevices"
                  :key="device.id"
                  :label="device.name"
                  :value="device.id"
                >
                  <div class="device-option">
                    <span class="device-name">{{ device.name }}</span>
                    <el-tag :type="getProtocolTagType(device.protocol)" size="small">
                      {{ device.protocol }}
                    </el-tag>
                  </div>
                </el-option>
              </el-select>
            </el-form-item>
          </el-col>
        </el-row>
        
        <el-form-item label="点位描述" prop="description">
          <el-input
            v-model="formData.description"
            type="textarea"
            :rows="3"
            placeholder="请输入点位描述信息（可选）"
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

      <!-- 数据配置 -->
      <el-card class="form-section" shadow="never">
        <template #header>
          <span class="section-title">数据配置</span>
        </template>
        
        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="数据类型" prop="data_type">
              <el-select
                v-model="formData.data_type"
                placeholder="请选择数据类型"
                style="width: 100%"
              >
                <el-option label="布尔型 (Bool)" value="Bool" />
                <el-option label="整数型 (Int32)" value="Int32" />
                <el-option label="浮点型 (Float32)" value="Float32" />
                <el-option label="字符串 (String)" value="String" />
              </el-select>
            </el-form-item>
          </el-col>
          
          <el-col :span="12">
            <el-form-item label="访问模式" prop="access_mode">
              <el-select
                v-model="formData.access_mode"
                placeholder="请选择访问模式"
                style="width: 100%"
              >
                <el-option label="只读" value="ReadOnly" />
                <el-option label="只写" value="WriteOnly" />
                <el-option label="读写" value="ReadWrite" />
              </el-select>
            </el-form-item>
          </el-col>
        </el-row>
      </el-card>

      <!-- 地址配置 -->
      <el-card class="form-section" shadow="never">
        <template #header>
          <span class="section-title">地址配置</span>
        </template>
        
        <el-row :gutter="20">
          <el-col :span="16">
            <el-form-item label="寄存器地址" prop="address">
              <el-input
                v-model="formData.address"
                placeholder="请输入寄存器地址，如: 40001, 0x1000"
              >
                <template #prepend>
                  <span>地址</span>
                </template>
              </el-input>
            </el-form-item>
          </el-col>
          
          <el-col :span="8">
            <el-form-item label="寄存器类型" prop="register_type">
              <el-select
                v-model="formData.register_type"
                placeholder="寄存器类型"
                style="width: 100%"
              >
                <el-option label="线圈 (Coil)" value="Coil" />
                <el-option label="离散输入 (Discrete)" value="DiscreteInput" />
                <el-option label="输入寄存器" value="InputRegister" />
                <el-option label="保持寄存器" value="HoldingRegister" />
              </el-select>
            </el-form-item>
          </el-col>
        </el-row>
        
        <!-- 高级配置 -->
        <el-collapse>
          <el-collapse-item title="高级配置" name="advanced">
            <el-row :gutter="20">
              <el-col :span="8">
                <el-form-item label="缩放因子" prop="scaling_factor">
                  <el-input-number
                    v-model="formData.scaling_factor"
                    :precision="3"
                    :step="0.1"
                    :min="0.001"
                    :max="1000"
                    placeholder="1.0"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
              
              <el-col :span="8">
                <el-form-item label="偏移量" prop="offset">
                  <el-input-number
                    v-model="formData.offset"
                    :precision="3"
                    :step="0.1"
                    :min="-1000"
                    :max="1000"
                    placeholder="0.0"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
              
              <el-col :span="8">
                <el-form-item label="单位" prop="unit">
                  <el-input
                    v-model="formData.unit"
                    placeholder="如: ℃, bar, %"
                    clearable
                  />
                </el-form-item>
              </el-col>
            </el-row>
            
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="最小值" prop="min_value">
                  <el-input-number
                    v-model="formData.min_value"
                    :precision="3"
                    style="width: 100%"
                    placeholder="最小值限制"
                  />
                </el-form-item>
              </el-col>
              
              <el-col :span="12">
                <el-form-item label="最大值" prop="max_value">
                  <el-input-number
                    v-model="formData.max_value"
                    :precision="3"
                    style="width: 100%"
                    placeholder="最大值限制"
                  />
                </el-form-item>
              </el-col>
            </el-row>
          </el-collapse-item>
        </el-collapse>
      </el-card>
    </el-form>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleCancel">取消</el-button>
        <el-button
          type="info"
          :loading="testing"
          @click="handleTestTag"
          v-if="mode === 'create' && canTestRead"
        >
          测试读取
        </el-button>
        <el-button
          type="primary"
          :loading="submitting"
          @click="handleSubmit"
        >
          {{ mode === 'create' ? '创建' : '保存' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * TagFormDialog —— 点位表单对话框
 *
 * 📝 Responsibilities:
 *  1. 新增/编辑点位表单
 *  2. 数据类型和访问模式配置
 *  3. 地址配置和高级参数
 *  4. 表单验证和数据处理
 *
 * 📦 Dependencies:
 *  - Element Plus 表单组件
 *  - 点位 API 和 Store
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, watch, nextTick } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { useTagsStore } from '@/stores'
import type { TagVO, TagCreateReq, TagDataType } from '@/api/tags'
import type { DeviceVO } from '@/api/devices'

// ===== Props =====
const props = defineProps<{
  visible: boolean
  tag?: TagVO | null
  mode: 'create' | 'edit'
  availableDevices: DeviceVO[]
}>()

// ===== Emits =====
const emit = defineEmits<{
  'update:visible': [visible: boolean]
  'success': []
}>()

// ===== Store =====
const tagsStore = useTagsStore()

// ===== 响应式数据 =====
const formRef = ref<FormInstance>()
const submitting = ref(false)
const testing = ref(false)

// 对话框显示状态
const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

// 对话框标题
const dialogTitle = computed(() => {
  return props.mode === 'create' ? '新增点位' : '编辑点位'
})

// 表单数据
const formData = ref<TagCreateReq>({
  name: '',
  device_id: '',
  data_type: 'Bool' as TagDataType,
  access_mode: 'ReadWrite',
  address: '',
  register_type: 'HoldingRegister',
  description: '',
  enabled: true,
  scaling_factor: 1.0,
  offset: 0.0,
  unit: '',
  min_value: null,
  max_value: null,
})

// 检查是否可以测试读取
const canTestRead = computed(() => {
  return formData.value.device_id && 
         formData.value.address && 
         (formData.value.access_mode === 'ReadOnly' || formData.value.access_mode === 'ReadWrite')
})

// 表单验证规则
const formRules: FormRules = {
  name: [
    { required: true, message: '请输入点位名称', trigger: 'blur' },
    { min: 2, max: 100, message: '点位名称长度应在 2-100 字符之间', trigger: 'blur' },
  ],
  device_id: [
    { required: true, message: '请选择所属设备', trigger: 'change' },
  ],
  data_type: [
    { required: true, message: '请选择数据类型', trigger: 'change' },
  ],
  access_mode: [
    { required: true, message: '请选择访问模式', trigger: 'change' },
  ],
  address: [
    { required: true, message: '请输入寄存器地址', trigger: 'blur' },
  ],
  register_type: [
    { required: true, message: '请选择寄存器类型', trigger: 'change' },
  ],
}

// ===== 方法 =====

/**
 * 获取协议标签类型
 */
function getProtocolTagType(protocol: string): string {
  const typeMap: Record<string, string> = {
    'ModbusTcp': 'primary',
    'ModbusRtu': 'success',
    'OpcUa': 'warning',
    'Mqtt': 'info',
  }
  return typeMap[protocol] || 'default'
}

/**
 * 测试点位
 */
async function handleTestTag() {
  // 先验证表单
  if (!formRef.value) return
  
  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) {
    ElMessage.warning('请先完善表单信息')
    return
  }
  
  testing.value = true
  try {
    // TODO: 实现点位测试功能
    ElMessage.info('点位测试功能开发中...')
  } catch (error) {
    ElMessage.error('点位测试失败')
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
      const result = await tagsStore.createTag(formData.value)
      success = !!result
    } else if (props.tag) {
      const result = await tagsStore.updateTag(props.tag.id, formData.value)
      success = !!result
    }
    
    if (success) {
      emit('success')
      ElMessage.success(`点位${props.mode === 'create' ? '创建' : '更新'}成功`)
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
  if (props.mode === 'edit' && props.tag) {
    // 编辑模式：填充现有数据
    formData.value = {
      name: props.tag.name,
      device_id: props.tag.device_id,
      data_type: props.tag.data_type,
      access_mode: props.tag.access_mode,
      address: props.tag.address,
      register_type: props.tag.register_type || 'HoldingRegister',
      description: props.tag.description || '',
      enabled: props.tag.enabled,
      scaling_factor: props.tag.scaling_factor || 1.0,
      offset: props.tag.offset || 0.0,
      unit: props.tag.unit || '',
      min_value: props.tag.min_value,
      max_value: props.tag.max_value,
    }
  } else {
    // 创建模式：重置为默认值
    formData.value = {
      name: '',
      device_id: '',
      data_type: 'Bool' as TagDataType,
      access_mode: 'ReadWrite',
      address: '',
      register_type: 'HoldingRegister',
      description: '',
      enabled: true,
      scaling_factor: 1.0,
      offset: 0.0,
      unit: '',
      min_value: null,
      max_value: null,
    }
  }
}

// ===== 监听器 =====
watch(() => props.visible, (visible) => {
  if (visible) {
    nextTick(() => {
      initFormData()
    })
  }
})

watch(() => props.tag, () => {
  if (props.visible) {
    initFormData()
  }
})
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

.device-option {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  
  .device-name {
    flex: 1;
    margin-right: 8px;
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
:deep(.el-input-group__prepend) {
  background-color: #f5f7fa;
  border-color: #dcdfe6;
  color: #909399;
}

// 折叠面板样式
:deep(.el-collapse) {
  border: none;
  
  .el-collapse-item__header {
    background-color: #fafafa;
    border: 1px solid #ebeef5;
    border-radius: 6px;
    padding: 0 16px;
    margin-bottom: 8px;
    font-weight: 500;
    
    &.is-active {
      border-bottom-left-radius: 0;
      border-bottom-right-radius: 0;
      margin-bottom: 0;
    }
  }
  
  .el-collapse-item__wrap {
    border: 1px solid #ebeef5;
    border-top: none;
    border-radius: 0 0 6px 6px;
  }
  
  .el-collapse-item__content {
    padding: 16px;
    background-color: #fafafa;
  }
}

// 数字输入框样式
:deep(.el-input-number) {
  .el-input__wrapper {
    border-radius: 6px;
  }
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
  
  .device-option {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
}
</style>