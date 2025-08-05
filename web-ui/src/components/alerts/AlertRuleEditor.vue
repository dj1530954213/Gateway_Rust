<template>
  <el-dialog
    v-model="dialogVisible"
    :title="isEditMode ? '编辑报警规则' : '新建报警规则'"
    width="900px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @close="handleClose"
  >
    <div class="alert-rule-editor">
      <el-form
        ref="formRef"
        :model="ruleForm"
        :rules="formRules"
        label-width="120px"
        label-position="top"
        @submit.prevent
      >
        <!-- 基本信息 -->
        <div class="form-section">
          <h3 class="section-title">基本信息</h3>
          <div class="form-grid">
            <el-form-item label="规则名称" prop="name" required>
              <el-input
                v-model="ruleForm.name"
                placeholder="请输入规则名称"
                maxlength="100"
                show-word-limit
              />
            </el-form-item>

            <el-form-item label="规则描述" prop="description">
              <el-input
                v-model="ruleForm.description"
                type="textarea"
                :rows="3"
                placeholder="请输入规则描述（可选）"
                maxlength="500"
                show-word-limit
              />
            </el-form-item>

            <el-form-item label="报警级别" prop="severity" required>
              <el-select v-model="ruleForm.severity" placeholder="选择报警级别">
                <el-option
                  v-for="level in severityOptions"
                  :key="level.value"
                  :label="level.label"
                  :value="level.value"
                >
                  <div class="severity-option">
                    <el-tag :type="level.type" size="small">{{ level.label }}</el-tag>
                    <span class="severity-desc">{{ level.description }}</span>
                  </div>
                </el-option>
              </el-select>
            </el-form-item>

            <el-form-item label="规则状态" prop="status">
              <el-radio-group v-model="ruleForm.status">
                <el-radio label="enabled">启用</el-radio>
                <el-radio label="disabled">禁用</el-radio>
              </el-radio-group>
              <div class="form-tip">
                禁用的规则不会触发报警，但会保留配置
              </div>
            </el-form-item>
          </div>
        </div>

        <!-- 监控目标 -->
        <div class="form-section">
          <h3 class="section-title">监控目标</h3>
          <div class="form-grid">
            <el-form-item label="监控范围" prop="scope" required>
              <el-radio-group v-model="ruleForm.scope" @change="handleScopeChange">
                <el-radio label="device">指定设备</el-radio>
                <el-radio label="tag">指定标签</el-radio>
                <el-radio label="group">设备分组</el-radio>
                <el-radio label="all">全部设备</el-radio>
              </el-radio-group>
            </el-form-item>

            <!-- 设备选择 -->
            <el-form-item
              v-if="ruleForm.scope === 'device'"
              label="选择设备"
              prop="deviceIds"
              required
            >
              <el-select
                v-model="ruleForm.deviceIds"
                multiple
                placeholder="请选择要监控的设备"
                style="width: 100%"
                @change="handleDeviceChange"
              >
                <el-option
                  v-for="device in availableDevices"
                  :key="device.id"
                  :label="device.name"
                  :value="device.id"
                >
                  <div class="device-option">
                    <span class="device-name">{{ device.name }}</span>
                    <span class="device-protocol">{{ device.protocol }}</span>
                  </div>
                </el-option>
              </el-select>
            </el-form-item>

            <!-- 标签选择 -->
            <el-form-item
              v-if="ruleForm.scope === 'tag'"
              label="选择标签"
              prop="tagIds"
              required
            >
              <el-select
                v-model="ruleForm.tagIds"
                multiple
                placeholder="请选择要监控的数据点位"
                style="width: 100%"
                filterable
              >
                <el-option
                  v-for="tag in availableTags"
                  :key="tag.id"
                  :label="`${tag.deviceName} - ${tag.name}`"
                  :value="tag.id"
                >
                  <div class="tag-option">
                    <span class="tag-name">{{ tag.name }}</span>
                    <span class="tag-device">{{ tag.deviceName }}</span>
                    <span class="tag-address">{{ tag.address }}</span>
                  </div>
                </el-option>
              </el-select>
            </el-form-item>

            <!-- 设备分组选择 -->
            <el-form-item
              v-if="ruleForm.scope === 'group'"
              label="设备分组"
              prop="groupIds"
              required
            >
              <el-select
                v-model="ruleForm.groupIds"
                multiple
                placeholder="请选择设备分组"
                style="width: 100%"
              >
                <el-option
                  v-for="group in deviceGroups"
                  :key="group.id"
                  :label="group.name"
                  :value="group.id"
                />
              </el-select>
            </el-form-item>
          </div>
        </div>

        <!-- 触发条件 -->
        <div class="form-section">
          <h3 class="section-title">触发条件</h3>
          <AlertConditionConfig
            v-model="ruleForm.conditions"
            :available-tags="availableTags"
            :scope="ruleForm.scope"
            :selected-devices="ruleForm.deviceIds"
            :selected-tags="ruleForm.tagIds"
          />
        </div>

        <!-- 报警设置 -->
        <div class="form-section">
          <h3 class="section-title">报警设置</h3>
          <div class="form-grid">
            <el-form-item label="触发方式" prop="triggerMode">
              <el-radio-group v-model="ruleForm.triggerMode">
                <el-radio label="immediate">立即触发</el-radio>
                <el-radio label="duration">持续时间</el-radio>
                <el-radio label="count">次数统计</el-radio>
              </el-radio-group>
            </el-form-item>

            <el-form-item
              v-if="ruleForm.triggerMode === 'duration'"
              label="持续时间"
              prop="duration"
            >
              <div class="duration-input">
                <el-input-number
                  v-model="ruleForm.duration.value"
                  :min="1"
                  :max="86400"
                  placeholder="时间值"
                />
                <el-select v-model="ruleForm.duration.unit" style="width: 100px">
                  <el-option label="秒" value="seconds" />
                  <el-option label="分钟" value="minutes" />
                  <el-option label="小时" value="hours" />
                </el-select>
              </div>
              <div class="form-tip">
                条件满足且持续指定时间后才触发报警
              </div>
            </el-form-item>

            <el-form-item
              v-if="ruleForm.triggerMode === 'count'"
              label="触发次数"
              prop="triggerCount"
            >
              <div class="count-input">
                <span>在</span>
                <el-input-number
                  v-model="ruleForm.triggerCount.timeWindow"
                  :min="1"
                  :max="1440"
                  placeholder="时间窗口"
                />
                <span>分钟内触发</span>
                <el-input-number
                  v-model="ruleForm.triggerCount.count"
                  :min="1"
                  :max="1000"
                  placeholder="次数"
                />
                <span>次</span>
              </div>
            </el-form-item>

            <el-form-item label="恢复条件" prop="recoveryCondition">
              <el-radio-group v-model="ruleForm.recoveryCondition">
                <el-radio label="auto">自动恢复</el-radio>
                <el-radio label="manual">手动恢复</el-radio>
                <el-radio label="timeout">超时恢复</el-radio>
              </el-radio-group>
            </el-form-item>

            <el-form-item
              v-if="ruleForm.recoveryCondition === 'timeout'"
              label="恢复超时"
              prop="recoveryTimeout"
            >
              <div class="timeout-input">
                <el-input-number
                  v-model="ruleForm.recoveryTimeout.value"
                  :min="1"
                  :max="168"
                  placeholder="超时值"
                />
                <el-select v-model="ruleForm.recoveryTimeout.unit" style="width: 100px">
                  <el-option label="小时" value="hours" />
                  <el-option label="天" value="days" />
                </el-select>
              </div>
            </el-form-item>
          </div>
        </div>

        <!-- 通知设置 -->
        <div class="form-section">
          <h3 class="section-title">通知设置</h3>
          <AlertNotificationSettings
            v-model="ruleForm.notifications"
            :severity="ruleForm.severity"
          />
        </div>

        <!-- 高级选项 -->
        <div class="form-section">
          <h3 class="section-title">
            <span>高级选项</span>
            <el-button type="link" @click="showAdvanced = !showAdvanced">
              {{ showAdvanced ? '收起' : '展开' }}
              <el-icon>
                <ArrowDown v-if="!showAdvanced" />
                <ArrowUp v-else />
              </el-icon>
            </el-button>
          </h3>
          
          <div v-show="showAdvanced" class="advanced-options">
            <div class="form-grid">
              <el-form-item label="报警抑制" prop="suppression">
                <el-checkbox v-model="ruleForm.suppression.enabled">
                  启用报警抑制
                </el-checkbox>
                <div v-if="ruleForm.suppression.enabled" class="suppression-config">
                  <div class="suppression-item">
                    <span>抑制时间:</span>
                    <el-input-number
                      v-model="ruleForm.suppression.duration"
                      :min="1"
                      :max="1440"
                    />
                    <span>分钟</span>
                  </div>
                  <div class="form-tip">
                    在指定时间内，相同条件不会重复发送报警
                  </div>
                </div>
              </el-form-item>

              <el-form-item label="报警分组" prop="grouping">
                <el-checkbox v-model="ruleForm.grouping.enabled">
                  启用报警分组
                </el-checkbox>
                <div v-if="ruleForm.grouping.enabled" class="grouping-config">
                  <el-select
                    v-model="ruleForm.grouping.fields"
                    multiple
                    placeholder="选择分组字段"
                  >
                    <el-option label="设备" value="device" />
                    <el-option label="标签" value="tag" />
                    <el-option label="级别" value="severity" />
                  </el-select>
                </div>
              </el-form-item>

              <el-form-item label="标签过滤" prop="tagFilters">
                <div class="tag-filters">
                  <div
                    v-for="(filter, index) in ruleForm.tagFilters"
                    :key="index"
                    class="filter-item"
                  >
                    <el-input
                      v-model="filter.key"
                      placeholder="标签键"
                      style="width: 120px"
                    />
                    <el-select v-model="filter.operator" style="width: 80px">
                      <el-option label="=" value="eq" />
                      <el-option label="!=" value="ne" />
                      <el-option label="~=" value="regex" />
                    </el-select>
                    <el-input
                      v-model="filter.value"
                      placeholder="标签值"
                      style="width: 120px"
                    />
                    <el-button
                      type="link"
                      @click="removeTagFilter(index)"
                      :disabled="ruleForm.tagFilters.length <= 1"
                    >
                      <el-icon><Delete /></el-icon>
                    </el-button>
                  </div>
                  <el-button type="link" @click="addTagFilter">
                    <el-icon><Plus /></el-icon>
                    添加过滤条件
                  </el-button>
                </div>
              </el-form-item>

              <el-form-item label="自定义属性" prop="customAttributes">
                <div class="custom-attributes">
                  <div
                    v-for="(attr, index) in ruleForm.customAttributes"
                    :key="index"
                    class="attribute-item"
                  >
                    <el-input
                      v-model="attr.key"
                      placeholder="属性名"
                      style="width: 150px"
                    />
                    <el-input
                      v-model="attr.value"
                      placeholder="属性值"
                      style="width: 200px"
                    />
                    <el-button
                      type="link"
                      @click="removeCustomAttribute(index)"
                      :disabled="ruleForm.customAttributes.length <= 1"
                    >
                      <el-icon><Delete /></el-icon>
                    </el-button>
                  </div>
                  <el-button type="link" @click="addCustomAttribute">
                    <el-icon><Plus /></el-icon>
                    添加自定义属性
                  </el-button>
                </div>
              </el-form-item>
            </div>
          </div>
        </div>
      </el-form>

      <!-- 规则预览 -->
      <div class="rule-preview">
        <h3 class="section-title">规则预览</h3>
        <div class="preview-content">
          <div class="preview-item">
            <span class="preview-label">规则名称:</span>
            <span class="preview-value">{{ ruleForm.name || '未设置' }}</span>
          </div>
          <div class="preview-item">
            <span class="preview-label">监控范围:</span>
            <span class="preview-value">{{ getScopeDescription() }}</span>
          </div>
          <div class="preview-item">
            <span class="preview-label">触发条件:</span>
            <span class="preview-value">{{ getConditionsDescription() }}</span>
          </div>
          <div class="preview-item">
            <span class="preview-label">通知方式:</span>
            <span class="preview-value">{{ getNotificationsDescription() }}</span>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <div class="footer-left">
          <el-button @click="testRule" :disabled="!canTest">
            <el-icon><VideoPlay /></el-icon>
            测试规则
          </el-button>
        </div>
        
        <div class="footer-right">
          <el-button @click="handleClose">取消</el-button>
          <el-button @click="saveAsDraft">保存草稿</el-button>
          <el-button
            type="primary"
            @click="saveRule"
            :loading="saving"
            :disabled="!canSave"
          >
            {{ isEditMode ? '更新规则' : '创建规则' }}
          </el-button>
        </div>
      </div>
    </template>

    <!-- 规则测试对话框 -->
    <AlertRuleTestDialog
      v-model:visible="showTestDialog"
      :rule-data="ruleForm"
      @test-complete="handleTestComplete"
    />
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * AlertRuleEditor —— 报警规则编辑器组件
 *
 * 📝 Responsibilities:
 *  1. 报警规则的创建和编辑
 *  2. 监控目标配置
 *  3. 触发条件设置
 *  4. 通知配置管理
 *  5. 规则验证和预览
 *
 * 📦 Dependencies:
 *  - AlertConditionConfig 条件配置组件
 *  - AlertNotificationSettings 通知设置组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, watch, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import {
  ArrowDown,
  ArrowUp,
  Plus,
  Delete,
  VideoPlay
} from '@element-plus/icons-vue'

// API导入
import { devicesApi, tagsApi } from '@/api'

// 组件导入
import AlertConditionConfig from './AlertConditionConfig.vue'
import AlertNotificationSettings from './AlertNotificationSettings.vue'
import AlertRuleTestDialog from './AlertRuleTestDialog.vue'

// ===== Props & Emits =====
const props = defineProps<{
  visible: boolean
  ruleData?: any
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  'save': [ruleData: any]
}>()

// ===== 响应式数据 =====
const dialogVisible = ref(props.visible)
const formRef = ref<FormInstance>()
const saving = ref(false)
const showAdvanced = ref(false)
const showTestDialog = ref(false)

// 表单数据
const ruleForm = ref({
  id: '',
  name: '',
  description: '',
  severity: 'warning',
  status: 'enabled',
  scope: 'device',
  deviceIds: [] as string[],
  tagIds: [] as string[],
  groupIds: [] as string[],
  conditions: [],
  triggerMode: 'immediate',
  duration: {
    value: 5,
    unit: 'minutes'
  },
  triggerCount: {
    timeWindow: 10,
    count: 3
  },
  recoveryCondition: 'auto',
  recoveryTimeout: {
    value: 24,
    unit: 'hours'
  },
  notifications: [],
  suppression: {
    enabled: false,
    duration: 30
  },
  grouping: {
    enabled: false,
    fields: []
  },
  tagFilters: [
    { key: '', operator: 'eq', value: '' }
  ],
  customAttributes: [
    { key: '', value: '' }
  ]
})

// 可用数据
const availableDevices = ref([])
const availableTags = ref([])
const deviceGroups = ref([])

// 严重程度选项
const severityOptions = [
  {
    value: 'critical',
    label: '严重',
    type: 'danger',
    description: '需要立即处理的严重问题'
  },
  {
    value: 'warning',
    label: '警告',
    type: 'warning',
    description: '需要关注的警告信息'
  },
  {
    value: 'info',
    label: '信息',
    type: 'info',
    description: '一般性的通知信息'
  }
]

// ===== 表单验证规则 =====
const formRules: FormRules = {
  name: [
    { required: true, message: '请输入规则名称', trigger: 'blur' },
    { min: 1, max: 100, message: '长度在 1 到 100 个字符', trigger: 'blur' }
  ],
  severity: [
    { required: true, message: '请选择报警级别', trigger: 'change' }
  ],
  scope: [
    { required: true, message: '请选择监控范围', trigger: 'change' }
  ],
  deviceIds: [
    {
      validator: (rule, value, callback) => {
        if (ruleForm.value.scope === 'device' && (!value || value.length === 0)) {
          callback(new Error('请选择要监控的设备'))
        } else {
          callback()
        }
      },
      trigger: 'change'
    }
  ],
  tagIds: [
    {
      validator: (rule, value, callback) => {
        if (ruleForm.value.scope === 'tag' && (!value || value.length === 0)) {
          callback(new Error('请选择要监控的标签'))
        } else {
          callback()
        }
      },
      trigger: 'change'
    }
  ],
  groupIds: [
    {
      validator: (rule, value, callback) => {
        if (ruleForm.value.scope === 'group' && (!value || value.length === 0)) {
          callback(new Error('请选择设备分组'))
        } else {
          callback()
        }
      },
      trigger: 'change'
    }
  ]
}

// ===== 计算属性 =====
const isEditMode = computed(() => {
  return !!(props.ruleData && props.ruleData.id)
})

const canSave = computed(() => {
  return ruleForm.value.name.trim() !== '' &&
         ruleForm.value.severity !== '' &&
         hasValidScope.value &&
         ruleForm.value.conditions.length > 0
})

const canTest = computed(() => {
  return canSave.value
})

const hasValidScope = computed(() => {
  switch (ruleForm.value.scope) {
    case 'device':
      return ruleForm.value.deviceIds.length > 0
    case 'tag':
      return ruleForm.value.tagIds.length > 0
    case 'group':
      return ruleForm.value.groupIds.length > 0
    case 'all':
      return true
    default:
      return false
  }
})

// ===== 方法 =====

/**
 * 初始化数据
 */
async function initializeData() {
  try {
    // 加载可用设备、标签和分组数据
    await Promise.all([
      fetchAvailableDevices(),
      fetchAvailableTags(),
      fetchDeviceGroups()
    ])

    // 如果是编辑模式，填充表单数据
    if (isEditMode.value && props.ruleData) {
      Object.assign(ruleForm.value, props.ruleData)
    }

  } catch (error) {
    console.error('初始化规则编辑器失败:', error)
    ElMessage.error('初始化失败')
  }
}

/**
 * 获取可用设备列表
 */
async function fetchAvailableDevices() {
  try {
    const response = await devicesApi.list({ enabled: true, size: 1000 })
    availableDevices.value = response.items.map(device => ({
      id: device.id,
      name: device.device_name,
      protocol: device.protocol,
      status: device.enabled ? 'online' : 'offline'
    }))
  } catch (error) {
    console.error('获取设备列表失败:', error)
    ElMessage.error('获取设备列表失败')
  }
}

/**
 * 获取可用标签列表
 */
async function fetchAvailableTags() {
  try {
    const response = await tagsApi.list({ size: 1000 })
    availableTags.value = response.items.map(tag => ({
      id: tag.id,
      name: tag.tag_name,
      address: tag.address,
      deviceId: tag.device_id,
      deviceName: tag.device_name,
      dataType: tag.data_type,
      unit: tag.unit
    }))
  } catch (error) {
    console.error('获取标签列表失败:', error)
    ElMessage.error('获取标签列表失败')
  }
}

/**
 * 获取设备分组列表
 */
async function fetchDeviceGroups() {
  try {
    const response = await devicesApi.getGroups()
    deviceGroups.value = response.items.map(group => ({
      id: group.id,
      name: group.group_name,
      deviceCount: group.device_count
    }))
  } catch (error) {
    console.error('获取设备分组失败:', error)
    ElMessage.error('获取设备分组失败')
  }
}

/**
 * 处理监控范围变化
 */
function handleScopeChange() {
  // 清空相关的选择
  ruleForm.value.deviceIds = []
  ruleForm.value.tagIds = []
  ruleForm.value.groupIds = []
  ruleForm.value.conditions = []
}

/**
 * 处理设备选择变化
 */
async function handleDeviceChange() {
  if (ruleForm.value.deviceIds.length > 0) {
    // 加载选中设备的标签
    const deviceTags = availableTags.value.filter(tag =>
      ruleForm.value.deviceIds.includes(tag.deviceId)
    )
    // 更新可用标签列表
    availableTags.value = availableTags.value
  }
}

/**
 * 添加标签过滤条件
 */
function addTagFilter() {
  ruleForm.value.tagFilters.push({
    key: '',
    operator: 'eq',
    value: ''
  })
}

/**
 * 移除标签过滤条件
 */
function removeTagFilter(index: number) {
  ruleForm.value.tagFilters.splice(index, 1)
}

/**
 * 添加自定义属性
 */
function addCustomAttribute() {
  ruleForm.value.customAttributes.push({
    key: '',
    value: ''
  })
}

/**
 * 移除自定义属性
 */
function removeCustomAttribute(index: number) {
  ruleForm.value.customAttributes.splice(index, 1)
}

/**
 * 获取监控范围描述
 */
function getScopeDescription(): string {
  switch (ruleForm.value.scope) {
    case 'device':
      return ruleForm.value.deviceIds.length > 0
        ? `${ruleForm.value.deviceIds.length} 个设备`
        : '未选择设备'
    case 'tag':
      return ruleForm.value.tagIds.length > 0
        ? `${ruleForm.value.tagIds.length} 个标签`
        : '未选择标签'
    case 'group':
      return ruleForm.value.groupIds.length > 0
        ? `${ruleForm.value.groupIds.length} 个分组`
        : '未选择分组'
    case 'all':
      return '全部设备'
    default:
      return '未设置'
  }
}

/**
 * 获取条件描述
 */
function getConditionsDescription(): string {
  if (ruleForm.value.conditions.length === 0) {
    return '未设置触发条件'
  }
  
  return `${ruleForm.value.conditions.length} 个条件`
}

/**
 * 获取通知描述
 */
function getNotificationsDescription(): string {
  if (ruleForm.value.notifications.length === 0) {
    return '未设置通知方式'
  }
  
  return `${ruleForm.value.notifications.length} 种通知方式`
}

/**
 * 测试规则
 */
async function testRule() {
  try {
    // 验证表单
    await formRef.value?.validate()
    
    // 打开测试对话框
    showTestDialog.value = true
    
  } catch (error) {
    console.error('规则验证失败:', error)
    ElMessage.error('请先完成规则配置')
  }
}

/**
 * 处理测试完成
 */
function handleTestComplete(result: any) {
  ElMessage.success(`测试完成，触发率: ${result.summary.successRate}%`)
}

/**
 * 保存为草稿
 */
async function saveAsDraft() {
  try {
    const draftData = {
      ...ruleForm.value,
      status: 'disabled',
      isDraft: true
    }
    
    emit('save', draftData)
    ElMessage.success('草稿已保存')
  } catch (error) {
    console.error('保存草稿失败:', error)
    ElMessage.error('保存草稿失败')
  }
}

/**
 * 保存规则
 */
async function saveRule() {
  try {
    // 验证表单
    await formRef.value?.validate()
    
    saving.value = true
    
    const ruleData = {
      ...ruleForm.value,
      updatedAt: new Date().toISOString()
    }
    
    if (!isEditMode.value) {
      ruleData.createdAt = new Date().toISOString()
    }
    
    emit('save', ruleData)
    
  } catch (error) {
    console.error('保存规则失败:', error)
    ElMessage.error('保存规则失败')
  } finally {
    saving.value = false
  }
}

/**
 * 处理对话框关闭
 */
function handleClose() {
  // 检查是否有未保存的更改
  if (hasUnsavedChanges()) {
    ElMessageBox.confirm(
      '你有未保存的更改，确定要关闭吗？',
      '确认关闭',
      {
        type: 'warning'
      }
    ).then(() => {
      dialogVisible.value = false
    }).catch(() => {
      // 用户取消关闭
    })
  } else {
    dialogVisible.value = false
  }
}

/**
 * 检查是否有未保存的更改
 */
function hasUnsavedChanges(): boolean {
  // 简单的检查逻辑，实际应该比较表单数据和原始数据
  return ruleForm.value.name.trim() !== '' || 
         ruleForm.value.description.trim() !== ''
}

// ===== 监听器 =====
watch(() => props.visible, (visible) => {
  dialogVisible.value = visible
  if (visible) {
    nextTick(() => {
      initializeData()
    })
  }
})

watch(dialogVisible, (visible) => {
  emit('update:visible', visible)
  if (!visible) {
    // 重置表单
    formRef.value?.resetFields()
    showAdvanced.value = false
  }
})
</script>

<style scoped lang="scss">
.alert-rule-editor {
  .form-section {
    margin-bottom: 32px;
    
    &:last-child {
      margin-bottom: 0;
    }
    
    .section-title {
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-size: 16px;
      font-weight: 600;
      color: #303133;
      margin: 0 0 16px 0;
      padding-bottom: 8px;
      border-bottom: 1px solid #ebeef5;
    }
    
    .form-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
      gap: 16px;
      
      .el-form-item {
        margin-bottom: 16px;
      }
    }
    
    .form-tip {
      font-size: 12px;
      color: #909399;
      margin-top: 4px;
    }
  }
  
  .severity-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    
    .severity-desc {
      font-size: 12px;
      color: #909399;
    }
  }
  
  .device-option,
  .tag-option {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    
    .device-name,
    .tag-name {
      font-weight: 500;
    }
    
    .device-protocol,
    .tag-device,
    .tag-address {
      font-size: 12px;
      color: #909399;
      font-family: monospace;
    }
  }
  
  .duration-input,
  .timeout-input {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .count-input {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    
    span {
      font-size: 14px;
      color: #606266;
    }
  }
  
  .advanced-options {
    .suppression-config,
    .grouping-config {
      margin-top: 8px;
      padding: 12px;
      background: #f8f9fa;
      border-radius: 4px;
      
      .suppression-item {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 8px;
        
        span {
          font-size: 13px;
          color: #606266;
        }
      }
    }
    
    .tag-filters,
    .custom-attributes {
      .filter-item,
      .attribute-item {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 8px;
      }
    }
  }
  
  .rule-preview {
    margin-top: 24px;
    padding: 16px;
    background: #f8f9fa;
    border-radius: 6px;
    
    .section-title {
      font-size: 14px;
      font-weight: 600;
      color: #303133;
      margin: 0 0 12px 0;
      border: none;
      padding: 0;
    }
    
    .preview-content {
      .preview-item {
        display: flex;
        margin-bottom: 8px;
        
        &:last-child {
          margin-bottom: 0;
        }
        
        .preview-label {
          width: 100px;
          font-size: 13px;
          color: #909399;
          flex-shrink: 0;
        }
        
        .preview-value {
          font-size: 13px;
          color: #303133;
          flex: 1;
        }
      }
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  
  .footer-left,
  .footer-right {
    display: flex;
    gap: 12px;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .alert-rule-editor {
    .form-section .form-grid {
      grid-template-columns: 1fr;
      gap: 12px;
    }
    
    .count-input {
      flex-direction: column;
      align-items: stretch;
      gap: 4px;
    }
    
    .dialog-footer {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
      
      .footer-left,
      .footer-right {
        justify-content: center;
      }
    }
  }
}
</style>