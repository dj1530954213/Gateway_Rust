<template>
  <el-dialog
    v-model="dialogVisible"
    :title="isCreate ? '创建角色' : '编辑角色'"
    width="600px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @close="handleClose"
  >
    <div class="role-edit-dialog">
      <el-form
        ref="formRef"
        :model="roleForm"
        :rules="formRules"
        label-width="100px"
        label-position="top"
        @submit.prevent
      >
        <!-- 基本信息 -->
        <div class="form-section">
          <h3 class="section-title">基本信息</h3>
          <div class="form-grid">
            <el-form-item label="角色名称" prop="name" required>
              <el-input
                v-model="roleForm.name"
                placeholder="请输入角色名称"
                maxlength="50"
                show-word-limit
              />
            </el-form-item>

            <el-form-item label="角色标识" prop="code" required>
              <el-input
                v-model="roleForm.code"
                placeholder="请输入角色标识"
                :disabled="!isCreate"
                maxlength="50"
              />
              <div class="form-tip">
                角色标识用于系统内部识别，创建后不可修改
              </div>
            </el-form-item>

            <el-form-item label="角色描述" prop="description">
              <el-input
                v-model="roleForm.description"
                type="textarea"
                :rows="3"
                placeholder="请输入角色描述"
                maxlength="200"
                show-word-limit
              />
            </el-form-item>

            <el-form-item label="角色状态" prop="status">
              <el-radio-group v-model="roleForm.status">
                <el-radio label="active">启用</el-radio>
                <el-radio label="inactive">禁用</el-radio>
              </el-radio-group>
              <div class="form-tip">禁用的角色无法分配给用户</div>
            </el-form-item>
          </div>
        </div>

        <!-- 角色属性 -->
        <div class="form-section">
          <h3 class="section-title">角色属性</h3>
          <div class="form-grid">
            <el-form-item label="角色级别" prop="level">
              <el-select
                v-model="roleForm.level"
                placeholder="选择角色级别"
                style="width: 100%"
              >
                <el-option
                  v-for="level in roleLevels"
                  :key="level.value"
                  :label="level.label"
                  :value="level.value"
                >
                  <div class="level-option">
                    <span class="level-name">{{ level.label }}</span>
                    <span class="level-desc">{{ level.description }}</span>
                  </div>
                </el-option>
              </el-select>
              <div class="form-tip">角色级别决定了权限的范围和优先级</div>
            </el-form-item>

            <el-form-item label="数据范围" prop="dataScope">
              <el-select
                v-model="roleForm.dataScope"
                placeholder="选择数据范围"
                style="width: 100%"
              >
                <el-option label="全部数据" value="all" />
                <el-option label="部门数据" value="department" />
                <el-option label="个人数据" value="personal" />
                <el-option label="自定义数据" value="custom" />
              </el-select>
              <div class="form-tip">控制角色可以访问的数据范围</div>
            </el-form-item>

            <el-form-item label="排序顺序" prop="sortOrder">
              <el-input-number
                v-model="roleForm.sortOrder"
                :min="0"
                :max="9999"
                placeholder="排序顺序"
                style="width: 100%"
              />
              <div class="form-tip">数字越小排序越靠前</div>
            </el-form-item>

            <el-form-item label="有效期" prop="expireDate">
              <el-date-picker
                v-model="roleForm.expireDate"
                type="date"
                placeholder="选择有效期"
                style="width: 100%"
                :disabled-date="disabledDate"
              />
              <div class="form-tip">留空表示永不过期</div>
            </el-form-item>
          </div>
        </div>

        <!-- 高级设置 -->
        <div class="form-section">
          <h3 class="section-title">高级设置</h3>
          <div class="advanced-options">
            <el-form-item label="角色标签" prop="tags">
              <el-select
                v-model="roleForm.tags"
                multiple
                filterable
                allow-create
                placeholder="添加角色标签"
                style="width: 100%"
              >
                <el-option
                  v-for="tag in commonTags"
                  :key="tag"
                  :label="tag"
                  :value="tag"
                />
              </el-select>
              <div class="form-tip">标签用于角色分类和快速筛选</div>
            </el-form-item>

            <div class="checkbox-group">
              <el-checkbox v-model="roleForm.isDefault">
                设为默认角色
              </el-checkbox>
              <div class="checkbox-tip">新用户注册时自动分配此角色</div>

              <el-checkbox v-model="roleForm.allowMultiple">
                允许多重分配
              </el-checkbox>
              <div class="checkbox-tip">用户可以同时拥有多个此类角色</div>

              <el-checkbox v-model="roleForm.inheritParent">
                继承上级权限
              </el-checkbox>
              <div class="checkbox-tip">自动继承上级角色的权限</div>

              <el-checkbox v-model="roleForm.canDelegate">
                允许权限委托
              </el-checkbox>
              <div class="checkbox-tip">角色拥有者可以将权限委托给其他用户</div>
            </div>
          </div>
        </div>

        <!-- 关联设置 -->
        <div class="form-section">
          <h3 class="section-title">关联设置</h3>
          <div class="form-grid">
            <el-form-item label="上级角色" prop="parentRoleId">
              <el-select
                v-model="roleForm.parentRoleId"
                placeholder="选择上级角色"
                clearable
                style="width: 100%"
              >
                <el-option
                  v-for="role in availableParentRoles"
                  :key="role.id"
                  :label="role.name"
                  :value="role.id"
                />
              </el-select>
              <div class="form-tip">设置角色层级关系，用于权限继承</div>
            </el-form-item>

            <el-form-item label="关联部门" prop="departmentIds">
              <el-select
                v-model="roleForm.departmentIds"
                multiple
                placeholder="选择关联部门"
                style="width: 100%"
              >
                <el-option
                  v-for="dept in departments"
                  :key="dept.id"
                  :label="dept.name"
                  :value="dept.id"
                />
              </el-select>
              <div class="form-tip">限制角色只能在特定部门使用</div>
            </el-form-item>
          </div>
        </div>

        <!-- 权限模板 -->
        <div v-if="isCreate" class="form-section">
          <h3 class="section-title">权限模板</h3>
          <el-form-item label="基于模板创建" prop="templateId">
            <el-select
              v-model="roleForm.templateId"
              placeholder="选择权限模板（可选）"
              clearable
              style="width: 100%"
              @change="handleTemplateChange"
            >
              <el-option
                v-for="template in permissionTemplates"
                :key="template.id"
                :label="template.name"
                :value="template.id"
              >
                <div class="template-option">
                  <span class="template-name">{{ template.name }}</span>
                  <span class="template-desc">{{ template.description }}</span>
                </div>
              </el-option>
            </el-select>
            <div class="form-tip">
              选择模板后将自动配置相应的权限，后续可以进一步调整
            </div>
          </el-form-item>
        </div>

        <!-- 备注信息 -->
        <div class="form-section">
          <h3 class="section-title">备注信息</h3>
          <el-form-item label="备注" prop="remark">
            <el-input
              v-model="roleForm.remark"
              type="textarea"
              :rows="3"
              placeholder="请输入备注信息（可选）"
              maxlength="500"
              show-word-limit
            />
          </el-form-item>
        </div>
      </el-form>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button @click="resetForm">重置</el-button>
        <el-button
          type="primary"
          :loading="saving"
          :disabled="!canSave"
          @click="saveRole"
        >
          {{ isCreate ? '创建角色' : '保存更改' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * RoleEditDialog —— 角色编辑对话框组件
 *
 * 📝 Responsibilities:
 *  1. 角色信息的创建和编辑
 *  2. 角色属性和层级设置
 *  3. 权限模板应用
 *  4. 表单验证和数据校验
 *  5. 角色关联关系配置
 *
 * 📦 Dependencies:
 *  - Element Plus 组件库
 *  - 角色管理相关API
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { ref, computed, watch, nextTick } from 'vue'

// ===== Props & Emits =====
const props = defineProps<{
  visible: boolean
  roleData?: any
  isCreate: boolean
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  save: [roleData: any]
}>()

// ===== 响应式数据 =====
const dialogVisible = ref(props.visible)
const formRef = ref<FormInstance>()
const saving = ref(false)

// 表单数据
const roleForm = ref({
  id: '',
  name: '',
  code: '',
  description: '',
  status: 'active',
  level: 'normal',
  dataScope: 'department',
  sortOrder: 0,
  expireDate: '',
  tags: [] as string[],
  isDefault: false,
  allowMultiple: true,
  inheritParent: false,
  canDelegate: false,
  parentRoleId: '',
  departmentIds: [] as string[],
  templateId: '',
  remark: '',
})

// 选项数据
const roleLevels = ref([
  {
    value: 'super',
    label: '超级管理员',
    description: '系统最高级别，拥有所有权限',
  },
  {
    value: 'admin',
    label: '管理员',
    description: '管理级别，拥有大部分管理权限',
  },
  {
    value: 'manager',
    label: '经理',
    description: '部门级别，拥有部门管理权限',
  },
  {
    value: 'normal',
    label: '普通用户',
    description: '基础级别，拥有基本操作权限',
  },
  {
    value: 'guest',
    label: '访客',
    description: '最低级别，只有查看权限',
  },
])

const commonTags = ref([
  '生产',
  '运维',
  '管理',
  '临时',
  '外部',
  '系统',
  '业务',
  '技术',
])

const availableParentRoles = ref<any[]>([])
const departments = ref<any[]>([])
const permissionTemplates = ref<any[]>([])

// ===== 表单验证规则 =====
const formRules: FormRules = {
  name: [
    { required: true, message: '请输入角色名称', trigger: 'blur' },
    {
      min: 2,
      max: 50,
      message: '角色名称长度在 2 到 50 个字符',
      trigger: 'blur',
    },
  ],
  code: [
    { required: true, message: '请输入角色标识', trigger: 'blur' },
    {
      min: 2,
      max: 50,
      message: '角色标识长度在 2 到 50 个字符',
      trigger: 'blur',
    },
    {
      pattern: /^[a-zA-Z][a-zA-Z0-9_]*$/,
      message: '角色标识必须以字母开头，只能包含字母、数字和下划线',
      trigger: 'blur',
    },
  ],
  description: [
    { max: 200, message: '描述长度不能超过 200 个字符', trigger: 'blur' },
  ],
  level: [{ required: true, message: '请选择角色级别', trigger: 'change' }],
  sortOrder: [
    {
      type: 'number',
      min: 0,
      max: 9999,
      message: '排序顺序必须在 0 到 9999 之间',
      trigger: 'blur',
    },
  ],
}

// ===== 计算属性 =====
const isCreate = computed(() => props.isCreate)

const canSave = computed(() => {
  return (
    roleForm.value.name.trim() !== '' &&
    roleForm.value.code.trim() !== '' &&
    roleForm.value.level !== ''
  )
})

// ===== 方法 =====

/**
 * 初始化数据
 */
async function initializeData() {
  try {
    // 从API加载真实数据
    await Promise.all([
      loadAvailableParentRoles(),
      loadDepartments(),
      loadPermissionTemplates(),
    ])

    // 如果是编辑模式，填充表单数据
    if (!props.isCreate && props.roleData) {
      Object.assign(roleForm.value, props.roleData)
    } else {
      // 创建模式，重置表单
      resetFormData()
    }
  } catch (error) {
    console.error('初始化角色编辑对话框失败:', error)
    ElMessage.error('初始化失败')
  }
}

/**
 * 从API加载可用的上级角色
 */
async function loadAvailableParentRoles() {
  try {
    const response = await fetch('/api/v1/roles?filter=parent', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (response.ok) {
      availableParentRoles.value = await response.json()
    } else {
      availableParentRoles.value = []
      console.error('加载上级角色失败:', response.statusText)
    }
  } catch (error) {
    console.error('加载上级角色失败:', error)
    availableParentRoles.value = []
  }
}

/**
 * 从API加载部门数据
 */
async function loadDepartments() {
  try {
    const response = await fetch('/api/v1/users/departments', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (response.ok) {
      departments.value = await response.json()
    } else {
      departments.value = []
      console.error('加载部门列表失败:', response.statusText)
    }
  } catch (error) {
    console.error('加载部门列表失败:', error)
    departments.value = []
  }
}

/**
 * 从API加载权限模板
 */
async function loadPermissionTemplates() {
  try {
    const response = await fetch('/api/v1/permissions/templates', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (response.ok) {
      permissionTemplates.value = await response.json()
    } else {
      permissionTemplates.value = []
      console.error('加载权限模板失败:', response.statusText)
    }
  } catch (error) {
    console.error('加载权限模板失败:', error)
    permissionTemplates.value = []
  }
}

/**
 * 重置表单数据
 */
function resetFormData() {
  roleForm.value = {
    id: '',
    name: '',
    code: '',
    description: '',
    status: 'active',
    level: 'normal',
    dataScope: 'department',
    sortOrder: 0,
    expireDate: '',
    tags: [],
    isDefault: false,
    allowMultiple: true,
    inheritParent: false,
    canDelegate: false,
    parentRoleId: '',
    departmentIds: [],
    templateId: '',
    remark: '',
  }
}

/**
 * 处理模板变化
 */
function handleTemplateChange(templateId: string) {
  if (!templateId) return

  const template = permissionTemplates.value.find(t => t.id === templateId)
  if (template) {
    // 根据模板自动填充一些字段
    if (!roleForm.value.description) {
      roleForm.value.description = template.description
    }

    // 根据模板设置一些默认值
    switch (templateId) {
      case 'basic_operator':
        roleForm.value.level = 'normal'
        roleForm.value.dataScope = 'department'
        break
      case 'advanced_operator':
        roleForm.value.level = 'normal'
        roleForm.value.dataScope = 'department'
        break
      case 'system_admin':
        roleForm.value.level = 'admin'
        roleForm.value.dataScope = 'all'
        break
      case 'data_analyst':
        roleForm.value.level = 'normal'
        roleForm.value.dataScope = 'all'
        break
    }

    ElMessage.success('已应用权限模板配置')
  }
}

/**
 * 禁用日期
 */
function disabledDate(time: Date): boolean {
  return time.getTime() < Date.now() - 8.64e7 // 不能选择今天之前的日期
}

/**
 * 重置表单
 */
function resetForm() {
  formRef.value?.resetFields()
  if (props.isCreate) {
    resetFormData()
  } else if (props.roleData) {
    Object.assign(roleForm.value, props.roleData)
  }
}

/**
 * 保存角色
 */
async function saveRole() {
  try {
    // 验证表单
    await formRef.value?.validate()

    saving.value = true

    // 调用真实API保存角色
    const apiUrl = props.isCreate
      ? '/api/v1/roles'
      : `/api/v1/roles/${roleForm.value.id}`
    const method = props.isCreate ? 'POST' : 'PUT'

    const response = await fetch(apiUrl, {
      method,
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(roleForm.value),
    })

    if (response.ok) {
      const roleData = await response.json()
      emit('save', roleData)
      dialogVisible.value = false
      ElMessage.success(props.isCreate ? '角色创建成功' : '角色更新成功')
    } else {
      const errorData = await response.json()
      throw new Error(errorData.message || '保存失败')
    }
  } catch (error) {
    console.error('保存角色失败:', error)
    ElMessage.error(`保存角色失败: ${error.message}`)
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
    ElMessageBox.confirm('你有未保存的更改，确定要关闭吗？', '确认关闭', {
      type: 'warning',
    })
      .then(() => {
        dialogVisible.value = false
      })
      .catch(() => {
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
  return (
    roleForm.value.name.trim() !== '' ||
    roleForm.value.code.trim() !== '' ||
    roleForm.value.description.trim() !== ''
  )
}

// ===== 监听器 =====
watch(
  () => props.visible,
  visible => {
    dialogVisible.value = visible
    if (visible) {
      nextTick(() => {
        initializeData()
      })
    }
  }
)

watch(dialogVisible, visible => {
  emit('update:visible', visible)
  if (!visible) {
    // 重置表单
    formRef.value?.resetFields()
  }
})

// 监听角色名称变化，自动生成角色标识
watch(
  () => roleForm.value.name,
  newName => {
    if (props.isCreate && newName && !roleForm.value.code) {
      // 自动生成角色标识
      const code = newName
        .toLowerCase()
        .replace(/[\u4e00-\u9fa5]/g, '') // 移除中文字符
        .replace(/\s+/g, '_') // 空格替换为下划线
        .replace(/[^a-z0-9_]/g, '') // 移除非字母数字下划线字符
        .substring(0, 20) // 限制长度

      if (code) {
        roleForm.value.code = code
      }
    }
  }
)
</script>

<style scoped lang="scss">
.role-edit-dialog {
  .form-section {
    margin-bottom: 32px;

    &:last-child {
      margin-bottom: 0;
    }

    .section-title {
      font-size: 16px;
      font-weight: 600;
      color: #303133;
      margin: 0 0 16px 0;
      padding-bottom: 8px;
      border-bottom: 1px solid #ebeef5;
    }

    .form-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
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

    .advanced-options {
      .checkbox-group {
        display: flex;
        flex-direction: column;
        gap: 16px;

        .el-checkbox {
          margin-bottom: 4px;
        }

        .checkbox-tip {
          font-size: 12px;
          color: #909399;
          margin-left: 24px;
          margin-bottom: 8px;
        }
      }
    }
  }

  .level-option {
    display: flex;
    flex-direction: column;

    .level-name {
      font-weight: 500;
      color: #303133;
    }

    .level-desc {
      font-size: 12px;
      color: #909399;
      margin-top: 2px;
    }
  }

  .template-option {
    display: flex;
    flex-direction: column;

    .template-name {
      font-weight: 500;
      color: #303133;
    }

    .template-desc {
      font-size: 12px;
      color: #909399;
      margin-top: 2px;
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

// 响应式设计
@media (max-width: 768px) {
  .role-edit-dialog {
    .form-section .form-grid {
      grid-template-columns: 1fr;
      gap: 12px;
    }
  }
}
</style>
