<template>
  <el-dialog
    v-model="dialogVisible"
    :title="isCreate ? '创建用户' : '编辑用户'"
    width="800px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @close="handleClose"
  >
    <div class="user-edit-dialog">
      <el-form
        ref="formRef"
        :model="userForm"
        :rules="formRules"
        label-width="100px"
        label-position="top"
        @submit.prevent
      >
        <!-- 基本信息 -->
        <div class="form-section">
          <h3 class="section-title">基本信息</h3>
          <div class="form-grid">
            <el-form-item label="用户名" prop="username" required>
              <el-input
                v-model="userForm.username"
                placeholder="请输入用户名"
                :disabled="!isCreate"
                maxlength="50"
                show-word-limit
              />
              <div class="form-tip">用户名用于登录，创建后不可修改</div>
            </el-form-item>

            <el-form-item label="真实姓名" prop="name" required>
              <el-input
                v-model="userForm.name"
                placeholder="请输入真实姓名"
                maxlength="50"
                show-word-limit
              />
            </el-form-item>

            <el-form-item label="邮箱地址" prop="email" required>
              <el-input
                v-model="userForm.email"
                type="email"
                placeholder="请输入邮箱地址"
                maxlength="100"
              />
            </el-form-item>

            <el-form-item label="手机号码" prop="phone">
              <el-input
                v-model="userForm.phone"
                placeholder="请输入手机号码"
                maxlength="20"
              />
            </el-form-item>
          </div>
        </div>

        <!-- 密码设置 -->
        <div v-if="isCreate" class="form-section">
          <h3 class="section-title">密码设置</h3>
          <div class="form-grid">
            <el-form-item label="初始密码" prop="password" required>
              <el-input
                v-model="userForm.password"
                type="password"
                placeholder="请输入初始密码"
                show-password
                maxlength="50"
              />
              <div class="form-tip">
                密码长度至少8位，包含字母、数字和特殊字符
              </div>
            </el-form-item>

            <el-form-item label="确认密码" prop="confirmPassword" required>
              <el-input
                v-model="userForm.confirmPassword"
                type="password"
                placeholder="请再次输入密码"
                show-password
                maxlength="50"
              />
            </el-form-item>
          </div>
        </div>

        <!-- 角色和权限 -->
        <div class="form-section">
          <h3 class="section-title">角色和权限</h3>
          <div class="form-grid">
            <el-form-item label="用户角色" prop="role" required>
              <el-select
                v-model="userForm.role"
                placeholder="选择用户角色"
                style="width: 100%"
                @change="handleRoleChange"
              >
                <el-option
                  v-for="role in availableRoles"
                  :key="role.id"
                  :label="role.name"
                  :value="role.id"
                >
                  <div class="role-option">
                    <span class="role-name">{{ role.name }}</span>
                    <span class="role-desc">{{ role.description }}</span>
                  </div>
                </el-option>
              </el-select>
            </el-form-item>

            <el-form-item label="所属部门" prop="departmentId">
              <el-select
                v-model="userForm.departmentId"
                placeholder="选择所属部门"
                style="width: 100%"
              >
                <el-option
                  v-for="dept in departments"
                  :key="dept.id"
                  :label="dept.name"
                  :value="dept.id"
                />
              </el-select>
            </el-form-item>

            <el-form-item label="用户状态" prop="status">
              <el-radio-group v-model="userForm.status">
                <el-radio label="active">启用</el-radio>
                <el-radio label="inactive">禁用</el-radio>
              </el-radio-group>
              <div class="form-tip">禁用的用户将无法登录系统</div>
            </el-form-item>

            <el-form-item label="账户有效期" prop="expireDate">
              <el-date-picker
                v-model="userForm.expireDate"
                type="date"
                placeholder="选择账户有效期"
                style="width: 100%"
                :disabled-date="disabledDate"
              />
              <div class="form-tip">留空表示永不过期</div>
            </el-form-item>
          </div>
        </div>

        <!-- 个人信息 -->
        <div class="form-section">
          <h3 class="section-title">个人信息</h3>
          <div class="form-grid">
            <el-form-item label="工号" prop="employeeId">
              <el-input
                v-model="userForm.employeeId"
                placeholder="请输入工号"
                maxlength="20"
              />
            </el-form-item>

            <el-form-item label="职位" prop="position">
              <el-input
                v-model="userForm.position"
                placeholder="请输入职位"
                maxlength="50"
              />
            </el-form-item>

            <el-form-item label="直属上级" prop="managerId">
              <el-select
                v-model="userForm.managerId"
                placeholder="选择直属上级"
                filterable
                clearable
                style="width: 100%"
              >
                <el-option
                  v-for="manager in potentialManagers"
                  :key="manager.id"
                  :label="manager.name"
                  :value="manager.id"
                >
                  <div class="manager-option">
                    <span class="manager-name">{{ manager.name }}</span>
                    <span class="manager-dept">{{
                      manager.departmentName
                    }}</span>
                  </div>
                </el-option>
              </el-select>
            </el-form-item>

            <el-form-item label="入职时间" prop="joinDate">
              <el-date-picker
                v-model="userForm.joinDate"
                type="date"
                placeholder="选择入职时间"
                style="width: 100%"
                :disabled-date="disabledJoinDate"
              />
            </el-form-item>
          </div>
        </div>

        <!-- 头像设置 -->
        <div class="form-section">
          <h3 class="section-title">头像设置</h3>
          <div class="avatar-section">
            <div class="avatar-preview">
              <el-avatar :size="80" :src="userForm.avatar">
                <el-icon><UserFilled /></el-icon>
              </el-avatar>
            </div>
            <div class="avatar-actions">
              <el-upload
                :show-file-list="false"
                :before-upload="beforeAvatarUpload"
                :on-success="handleAvatarSuccess"
                action="#"
                accept="image/*"
              >
                <el-button>上传头像</el-button>
              </el-upload>
              <el-button @click="generateRandomAvatar">随机生成</el-button>
              <div class="avatar-tip">
                支持 JPG、PNG 格式，文件大小不超过 2MB
              </div>
            </div>
          </div>
        </div>

        <!-- 权限预览 -->
        <div v-if="selectedRolePermissions.length > 0" class="form-section">
          <h3 class="section-title">权限预览</h3>
          <div class="permissions-preview">
            <div class="permission-categories">
              <div
                v-for="category in permissionCategories"
                :key="category.name"
                class="permission-category"
              >
                <div class="category-header">
                  <el-icon class="category-icon">
                    <component :is="category.icon" />
                  </el-icon>
                  <span class="category-name">{{ category.name }}</span>
                </div>
                <div class="category-permissions">
                  <el-tag
                    v-for="permission in category.permissions"
                    :key="permission.id"
                    :type="permission.granted ? 'success' : 'info'"
                    size="small"
                    class="permission-tag"
                  >
                    {{ permission.name }}
                  </el-tag>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 备注信息 -->
        <div class="form-section">
          <h3 class="section-title">备注信息</h3>
          <el-form-item label="备注" prop="remark">
            <el-input
              v-model="userForm.remark"
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
          @click="saveUser"
        >
          {{ isCreate ? '创建用户' : '保存更改' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * UserEditDialog —— 用户编辑对话框组件
 *
 * 📝 Responsibilities:
 *  1. 用户信息的创建和编辑
 *  2. 角色权限预览和分配
 *  3. 表单验证和数据校验
 *  4. 头像上传和管理
 *  5. 用户状态和有效期管理
 *
 * 📦 Dependencies:
 *  - Element Plus 组件库
 *  - 用户管理相关API
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  UserFilled,
  Monitor,
  Setting,
  Document,
  Connection,
  Bell,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { ref, computed, watch, nextTick } from 'vue'

// ===== Props & Emits =====
const props = defineProps<{
  visible: boolean
  userData?: any
  isCreate: boolean
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  save: [userData: any]
}>()

// ===== 响应式数据 =====
const dialogVisible = ref(props.visible)
const formRef = ref<FormInstance>()
const saving = ref(false)

// 表单数据
const userForm = ref({
  id: '',
  username: '',
  name: '',
  email: '',
  phone: '',
  password: '',
  confirmPassword: '',
  role: '',
  departmentId: '',
  status: 'active',
  expireDate: '',
  employeeId: '',
  position: '',
  managerId: '',
  joinDate: '',
  avatar: '',
  remark: '',
})

// 可用选项数据
const availableRoles = ref<any[]>([])
const departments = ref<any[]>([])
const potentialManagers = ref<any[]>([])

// ===== 表单验证规则 =====
const formRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    {
      min: 3,
      max: 50,
      message: '用户名长度在 3 到 50 个字符',
      trigger: 'blur',
    },
    {
      pattern: /^[a-zA-Z0-9_]+$/,
      message: '用户名只能包含字母、数字和下划线',
      trigger: 'blur',
    },
  ],
  name: [
    { required: true, message: '请输入真实姓名', trigger: 'blur' },
    { min: 2, max: 50, message: '姓名长度在 2 到 50 个字符', trigger: 'blur' },
  ],
  email: [
    { required: true, message: '请输入邮箱地址', trigger: 'blur' },
    { type: 'email', message: '请输入正确的邮箱地址', trigger: 'blur' },
  ],
  phone: [
    {
      pattern: /^1[3-9]\d{9}$/,
      message: '请输入正确的手机号码',
      trigger: 'blur',
    },
  ],
  password: [
    {
      validator: (rule, value, callback) => {
        if (props.isCreate) {
          if (!value) {
            callback(new Error('请输入初始密码'))
          } else if (value.length < 8) {
            callback(new Error('密码长度至少8位'))
          } else if (
            !/^(?=.*[a-zA-Z])(?=.*\d)(?=.*[!@#$%^&*])[A-Za-z\d!@#$%^&*]/.test(
              value
            )
          ) {
            callback(new Error('密码必须包含字母、数字和特殊字符'))
          } else {
            callback()
          }
        } else {
          callback()
        }
      },
      trigger: 'blur',
    },
  ],
  confirmPassword: [
    {
      validator: (rule, value, callback) => {
        if (props.isCreate) {
          if (!value) {
            callback(new Error('请确认密码'))
          } else if (value !== userForm.value.password) {
            callback(new Error('两次输入的密码不一致'))
          } else {
            callback()
          }
        } else {
          callback()
        }
      },
      trigger: 'blur',
    },
  ],
  role: [{ required: true, message: '请选择用户角色', trigger: 'change' }],
}

// ===== 计算属性 =====
const isCreate = computed(() => props.isCreate)

const canSave = computed(() => {
  return (
    userForm.value.username.trim() !== '' &&
    userForm.value.name.trim() !== '' &&
    userForm.value.email.trim() !== '' &&
    userForm.value.role !== ''
  )
})

const selectedRolePermissions = computed(() => {
  const role = availableRoles.value.find(r => r.id === userForm.value.role)
  return role ? role.permissions || [] : []
})

const permissionCategories = computed(() => {
  const permissions = selectedRolePermissions.value

  return [
    {
      name: '设备管理',
      icon: 'Monitor',
      permissions: permissions.filter((p: any) => p.category === 'device'),
    },
    {
      name: '系统设置',
      icon: 'Setting',
      permissions: permissions.filter((p: any) => p.category === 'system'),
    },
    {
      name: '数据管理',
      icon: 'Document',
      permissions: permissions.filter((p: any) => p.category === 'data'),
    },
    {
      name: '网络连接',
      icon: 'Connection',
      permissions: permissions.filter((p: any) => p.category === 'network'),
    },
    {
      name: '报警管理',
      icon: 'Bell',
      permissions: permissions.filter((p: any) => p.category === 'alert'),
    },
  ].filter(category => category.permissions.length > 0)
})

// ===== 方法 =====

/**
 * 初始化数据
 */
async function initializeData() {
  try {
    // 从API加载真实数据
    await loadAvailableRoles()
    await loadDepartments()
    await loadPotentialManagers()

    // 如果是编辑模式，填充表单数据
    if (!props.isCreate && props.userData) {
      Object.assign(userForm.value, props.userData)
    } else {
      // 创建模式，重置表单
      resetFormData()
    }
  } catch (error) {
    console.error('初始化用户编辑对话框失败:', error)
    ElMessage.error('初始化失败')
  }
}

/**
 * 加载可用角色
 */
async function loadAvailableRoles() {
  try {
    const response = await fetch('/api/v1/users/roles')
    if (response.ok) {
      availableRoles.value = await response.json()
    } else {
      availableRoles.value = []
    }
  } catch (error) {
    console.error('加载角色列表失败:', error)
    availableRoles.value = []
  }
}

/**
 * 加载部门列表
 */
async function loadDepartments() {
  try {
    const response = await fetch('/api/v1/users/departments')
    if (response.ok) {
      departments.value = await response.json()
    } else {
      departments.value = []
    }
  } catch (error) {
    console.error('加载部门列表失败:', error)
    departments.value = []
  }
}

/**
 * 加载潜在管理员
 */
async function loadPotentialManagers() {
  try {
    const response = await fetch('/api/v1/users/managers')
    if (response.ok) {
      potentialManagers.value = await response.json()
    } else {
      potentialManagers.value = []
    }
  } catch (error) {
    console.error('加载管理员列表失败:', error)
    potentialManagers.value = []
  }
}

/**
 * 重置表单数据
 */
function resetFormData() {
  userForm.value = {
    id: '',
    username: '',
    name: '',
    email: '',
    phone: '',
    password: '',
    confirmPassword: '',
    role: '',
    departmentId: '',
    status: 'active',
    expireDate: '',
    employeeId: '',
    position: '',
    managerId: '',
    joinDate: '',
    avatar: '',
    remark: '',
  }
}

/**
 * 处理角色变化
 */
function handleRoleChange() {
  // 角色变化时可以做一些额外处理
  console.log('角色已更改为:', userForm.value.role)
}

/**
 * 禁用日期（账户有效期）
 */
function disabledDate(time: Date): boolean {
  return time.getTime() < Date.now() - 8.64e7 // 不能选择今天之前的日期
}

/**
 * 禁用日期（入职时间）
 */
function disabledJoinDate(time: Date): boolean {
  return time.getTime() > Date.now() // 不能选择未来的日期
}

/**
 * 头像上传前验证
 */
function beforeAvatarUpload(file: File): boolean {
  const isValidType = ['image/jpeg', 'image/png', 'image/gif'].includes(
    file.type
  )
  const isLt2M = file.size / 1024 / 1024 < 2

  if (!isValidType) {
    ElMessage.error('头像只能是 JPG、PNG、GIF 格式!')
    return false
  }
  if (!isLt2M) {
    ElMessage.error('头像大小不能超过 2MB!')
    return false
  }

  // 模拟上传
  const reader = new FileReader()
  reader.onload = e => {
    userForm.value.avatar = e.target?.result as string
  }
  reader.readAsDataURL(file)

  return false // 阻止自动上传
}

/**
 * 头像上传成功
 */
function handleAvatarSuccess(response: any) {
  userForm.value.avatar = response.url
  ElMessage.success('头像上传成功')
}

/**
 * 生成随机头像
 */
function generateRandomAvatar() {
  const seed = Date.now().toString(36)
  userForm.value.avatar = `https://api.dicebear.com/7.x/avataaars/svg?seed=${seed}`
  ElMessage.success('头像已生成')
}

/**
 * 重置表单
 */
function resetForm() {
  formRef.value?.resetFields()
  if (props.isCreate) {
    resetFormData()
  } else if (props.userData) {
    Object.assign(userForm.value, props.userData)
  }
}

/**
 * 保存用户
 */
async function saveUser() {
  try {
    // 验证表单
    await formRef.value?.validate()

    saving.value = true

    // 调用API保存用户数据
    const apiUrl = props.isCreate
      ? '/api/v1/users'
      : `/api/v1/users/${userForm.value.id}`
    const method = props.isCreate ? 'POST' : 'PUT'

    const response = await fetch(apiUrl, {
      method,
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(userForm.value),
    })

    if (!response.ok) {
      throw new Error('保存用户信息失败')
    }

    const userData = await response.json()

    if (props.isCreate) {
      userData.createdAt = new Date().toISOString()
    }

    emit('save', userData)
    dialogVisible.value = false
  } catch (error) {
    console.error('保存用户失败:', error)
    ElMessage.error('保存用户失败')
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
    userForm.value.username.trim() !== '' ||
    userForm.value.name.trim() !== '' ||
    userForm.value.email.trim() !== ''
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
</script>

<style scoped lang="scss">
.user-edit-dialog {
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
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
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

  .role-option {
    display: flex;
    flex-direction: column;

    .role-name {
      font-weight: 500;
      color: #303133;
    }

    .role-desc {
      font-size: 12px;
      color: #909399;
      margin-top: 2px;
    }
  }

  .manager-option {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .manager-name {
      font-weight: 500;
      color: #303133;
    }

    .manager-dept {
      font-size: 12px;
      color: #909399;
    }
  }

  .avatar-section {
    display: flex;
    align-items: flex-start;
    gap: 20px;

    .avatar-preview {
      flex-shrink: 0;
    }

    .avatar-actions {
      flex: 1;

      .el-button {
        margin-right: 12px;
        margin-bottom: 8px;
      }

      .avatar-tip {
        font-size: 12px;
        color: #909399;
        margin-top: 8px;
      }
    }
  }

  .permissions-preview {
    .permission-categories {
      .permission-category {
        margin-bottom: 20px;

        &:last-child {
          margin-bottom: 0;
        }

        .category-header {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 12px;

          .category-icon {
            font-size: 16px;
            color: #409eff;
          }

          .category-name {
            font-weight: 500;
            color: #303133;
          }
        }

        .category-permissions {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;

          .permission-tag {
            margin: 0;
          }
        }
      }
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
  .user-edit-dialog {
    .form-section .form-grid {
      grid-template-columns: 1fr;
      gap: 12px;
    }

    .avatar-section {
      flex-direction: column;
      gap: 16px;
      align-items: center;
    }

    .permissions-preview
      .permission-categories
      .permission-category
      .category-permissions {
      gap: 6px;
    }
  }
}
</style>
