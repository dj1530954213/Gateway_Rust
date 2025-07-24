<template>
  <div class="user-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="title-section">
          <h1>用户管理</h1>
          <p class="description">管理系统用户账户、权限和访问控制</p>
        </div>
        <div class="header-actions">
          <el-button type="primary" @click="openCreateUserDialog">
            <el-icon><Plus /></el-icon>
            添加用户
          </el-button>
          <el-button @click="openBatchOperationDialog">
            <el-icon><Operation /></el-icon>
            批量操作
          </el-button>
        </div>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ userStats.totalUsers }}</div>
          <div class="stat-label">总用户数</div>
          <div class="stat-icon users">
            <el-icon><User /></el-icon>
          </div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ userStats.activeUsers }}</div>
          <div class="stat-label">活跃用户</div>
          <div class="stat-icon active">
            <el-icon><UserFilled /></el-icon>
          </div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ userStats.onlineUsers }}</div>
          <div class="stat-label">在线用户</div>
          <div class="stat-icon online">
            <el-icon><Connection /></el-icon>
          </div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ userStats.adminUsers }}</div>
          <div class="stat-label">管理员</div>
          <div class="stat-icon admin">
            <el-icon><User /></el-icon>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 主要内容区域 -->
    <div class="main-content">
      <el-card>
        <!-- 搜索和筛选 -->
        <div class="toolbar">
          <div class="search-section">
            <el-input
              v-model="searchQuery"
              placeholder="搜索用户名、邮箱..."
              size="default"
              style="width: 300px"
              clearable
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
            <el-select
              v-model="selectedRole"
              placeholder="角色筛选"
              size="default"
              style="width: 150px"
              clearable
            >
              <el-option label="全部角色" value="" />
              <el-option label="管理员" value="admin" />
              <el-option label="操作员" value="operator" />
              <el-option label="观察者" value="viewer" />
              <el-option label="访客" value="guest" />
            </el-select>
            <el-select
              v-model="selectedStatus"
              placeholder="状态筛选"
              size="default"
              style="width: 120px"
              clearable
            >
              <el-option label="全部状态" value="" />
              <el-option label="活跃" value="active" />
              <el-option label="禁用" value="disabled" />
              <el-option label="锁定" value="locked" />
            </el-select>
            <el-button @click="resetFilters">
              <el-icon><RefreshRight /></el-icon>
              重置
            </el-button>
          </div>
          <div class="view-options">
            <el-radio-group v-model="viewMode" size="small">
              <el-radio-button label="table">表格</el-radio-button>
              <el-radio-button label="card">卡片</el-radio-button>
            </el-radio-group>
          </div>
        </div>

        <!-- 表格视图 -->
        <div v-if="viewMode === 'table'" class="table-view">
          <el-table
            :data="filteredUsers"
            v-loading="loading"
            @selection-change="handleSelectionChange"
            stripe
            style="width: 100%"
          >
            <el-table-column type="selection" width="55" />
            <el-table-column prop="avatar" label="头像" width="80">
              <template #default="{ row }">
                <el-avatar :size="40" :src="row.avatar">
                  {{ row.username.charAt(0).toUpperCase() }}
                </el-avatar>
              </template>
            </el-table-column>
            <el-table-column prop="username" label="用户名" min-width="120" />
            <el-table-column prop="email" label="邮箱" min-width="180" />
            <el-table-column prop="role" label="角色" width="100">
              <template #default="{ row }">
                <el-tag :type="getRoleType(row.role)" size="small">
                  {{ getRoleLabel(row.role) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="department" label="部门" width="120" />
            <el-table-column prop="status" label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="getStatusType(row.status)" size="small">
                  {{ getStatusLabel(row.status) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="lastLogin" label="最后登录" width="150">
              <template #default="{ row }">
                <span v-if="row.lastLogin">{{ formatDate(row.lastLogin) }}</span>
                <span v-else class="text-muted">从未登录</span>
              </template>
            </el-table-column>
            <el-table-column prop="isOnline" label="在线状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.isOnline ? 'success' : 'info'" size="small">
                  {{ row.isOnline ? '在线' : '离线' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="createdAt" label="创建时间" width="150">
              <template #default="{ row }">
                {{ formatDate(row.createdAt) }}
              </template>
            </el-table-column>
            <el-table-column label="操作" width="200" fixed="right">
              <template #default="{ row }">
                <el-button
                  type="primary"
                  text
                  size="small"
                  @click="viewUser(row)"
                >
                  查看
                </el-button>
                <el-button
                  type="primary"
                  text
                  size="small"
                  @click="editUser(row)"
                >
                  编辑
                </el-button>
                <el-button
                  :type="row.status === 'active' ? 'warning' : 'success'"
                  text
                  size="small"
                  @click="toggleUserStatus(row)"
                >
                  {{ row.status === 'active' ? '禁用' : '启用' }}
                </el-button>
                <el-dropdown @command="(cmd: string) => handleUserAction(cmd, row)">
                  <el-button type="primary" text size="small">
                    更多<el-icon class="el-icon--right"><ArrowDown /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item command="resetPassword">重置密码</el-dropdown-item>
                      <el-dropdown-item command="permissions">权限设置</el-dropdown-item>
                      <el-dropdown-item command="sessions">会话管理</el-dropdown-item>
                      <el-dropdown-item divided command="delete">删除用户</el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <!-- 卡片视图 -->
        <div v-if="viewMode === 'card'" class="card-view">
          <div class="user-cards">
            <div 
              v-for="user in filteredUsers" 
              :key="user.id"
              class="user-card"
              @click="viewUser(user)"
            >
              <div class="card-header">
                <el-avatar :size="60" :src="user.avatar">
                  {{ user.username.charAt(0).toUpperCase() }}
                </el-avatar>
                <div class="user-info">
                  <div class="username">{{ user.username }}</div>
                  <div class="email">{{ user.email }}</div>
                </div>
                <div class="user-status">
                  <el-tag :type="user.isOnline ? 'success' : 'info'" size="small">
                    {{ user.isOnline ? '在线' : '离线' }}
                  </el-tag>
                </div>
              </div>
              <div class="card-body">
                <div class="info-item">
                  <label>角色:</label>
                  <el-tag :type="getRoleType(user.role)" size="small">
                    {{ getRoleLabel(user.role) }}
                  </el-tag>
                </div>
                <div class="info-item">
                  <label>部门:</label>
                  <span>{{ user.department }}</span>
                </div>
                <div class="info-item">
                  <label>状态:</label>
                  <el-tag :type="getStatusType(user.status)" size="small">
                    {{ getStatusLabel(user.status) }}
                  </el-tag>
                </div>
                <div class="info-item">
                  <label>最后登录:</label>
                  <span v-if="user.lastLogin">{{ formatDate(user.lastLogin) }}</span>
                  <span v-else class="text-muted">从未登录</span>
                </div>
              </div>
              <div class="card-actions">
                <el-button size="small" @click.stop="editUser(user)">编辑</el-button>
                <el-button
                  size="small"
                  :type="user.status === 'active' ? 'warning' : 'success'"
                  @click.stop="toggleUserStatus(user)"
                >
                  {{ user.status === 'active' ? '禁用' : '启用' }}
                </el-button>
              </div>
            </div>
          </div>
        </div>

        <!-- 分页 -->
        <div class="pagination-wrapper">
          <el-pagination
            v-model:current-page="currentPage"
            v-model:page-size="pageSize"
            :page-sizes="[10, 20, 50, 100]"
            :total="filteredUsers.length"
            layout="total, sizes, prev, pager, next, jumper"
          />
        </div>
      </el-card>
    </div>

    <!-- 创建/编辑用户对话框 -->
    <el-dialog
      v-model="userDialogVisible"
      :title="isEditMode ? '编辑用户' : '添加用户'"
      width="600px"
      :before-close="closeUserDialog"
    >
      <el-form
        ref="userFormRef"
        :model="userForm"
        :rules="userFormRules"
        label-width="100px"
      >
        <div class="form-grid">
          <div class="form-left">
            <el-form-item label="用户名" prop="username">
              <el-input v-model="userForm.username" placeholder="请输入用户名" />
            </el-form-item>
            <el-form-item label="邮箱" prop="email">
              <el-input v-model="userForm.email" placeholder="请输入邮箱" />
            </el-form-item>
            <el-form-item v-if="!isEditMode" label="密码" prop="password">
              <el-input
                v-model="userForm.password"
                type="password"
                placeholder="请输入密码"
                show-password
              />
            </el-form-item>
            <el-form-item label="真实姓名" prop="realName">
              <el-input v-model="userForm.realName" placeholder="请输入真实姓名" />
            </el-form-item>
            <el-form-item label="手机号码" prop="phone">
              <el-input v-model="userForm.phone" placeholder="请输入手机号码" />
            </el-form-item>
          </div>
          <div class="form-right">
            <el-form-item label="用户角色" prop="role">
              <el-select v-model="userForm.role" placeholder="选择用户角色">
                <el-option label="管理员" value="admin" />
                <el-option label="操作员" value="operator" />
                <el-option label="观察者" value="viewer" />
                <el-option label="访客" value="guest" />
              </el-select>
            </el-form-item>
            <el-form-item label="所属部门" prop="department">
              <el-input v-model="userForm.department" placeholder="请输入所属部门" />
            </el-form-item>
            <el-form-item label="用户状态" prop="status">
              <el-radio-group v-model="userForm.status">
                <el-radio label="active">活跃</el-radio>
                <el-radio label="disabled">禁用</el-radio>
              </el-radio-group>
            </el-form-item>
            <el-form-item label="备注">
              <el-input
                v-model="userForm.note"
                type="textarea"
                :rows="3"
                placeholder="备注信息（可选）"
              />
            </el-form-item>
          </div>
        </div>
      </el-form>

      <template #footer>
        <el-button @click="closeUserDialog">取消</el-button>
        <el-button type="primary" @click="saveUser" :loading="submitting">
          {{ isEditMode ? '保存' : '创建' }}
        </el-button>
      </template>
    </el-dialog>

    <!-- 用户详情对话框 -->
    <el-dialog
      v-model="userDetailVisible"
      title="用户详情"
      width="800px"
    >
      <div v-if="selectedUser" class="user-detail">
        <!-- 用户基本信息 -->
        <div class="detail-section">
          <h3>基本信息</h3>
          <div class="detail-content">
            <div class="user-avatar-section">
              <el-avatar :size="80" :src="selectedUser.avatar">
                {{ selectedUser.username.charAt(0).toUpperCase() }}
              </el-avatar>
              <div class="user-basic-info">
                <div class="info-row">
                  <strong>{{ selectedUser.username }}</strong>
                  <el-tag :type="getRoleType(selectedUser.role)" size="small">
                    {{ getRoleLabel(selectedUser.role) }}
                  </el-tag>
                </div>
                <div class="info-row">{{ selectedUser.email }}</div>
                <div class="info-row">{{ selectedUser.department }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- 详细信息 -->
        <div class="detail-section">
          <h3>详细信息</h3>
          <div class="info-grid">
            <div class="info-item">
              <label>真实姓名:</label>
              <span>{{ selectedUser.realName || '-' }}</span>
            </div>
            <div class="info-item">
              <label>手机号码:</label>
              <span>{{ selectedUser.phone || '-' }}</span>
            </div>
            <div class="info-item">
              <label>用户状态:</label>
              <el-tag :type="getStatusType(selectedUser.status)">
                {{ getStatusLabel(selectedUser.status) }}
              </el-tag>
            </div>
            <div class="info-item">
              <label>在线状态:</label>
              <el-tag :type="selectedUser.isOnline ? 'success' : 'info'">
                {{ selectedUser.isOnline ? '在线' : '离线' }}
              </el-tag>
            </div>
            <div class="info-item">
              <label>创建时间:</label>
              <span>{{ formatDate(selectedUser.createdAt) }}</span>
            </div>
            <div class="info-item">
              <label>最后登录:</label>
              <span v-if="selectedUser.lastLogin">{{ formatDate(selectedUser.lastLogin) }}</span>
              <span v-else class="text-muted">从未登录</span>
            </div>
            <div class="info-item">
              <label>登录次数:</label>
              <span>{{ selectedUser.loginCount || 0 }} 次</span>
            </div>
            <div class="info-item">
              <label>备注:</label>
              <span>{{ selectedUser.note || '-' }}</span>
            </div>
          </div>
        </div>

        <!-- 权限信息 -->
        <div class="detail-section">
          <h3>权限信息</h3>
          <div class="permissions-list">
            <el-tag 
              v-for="permission in selectedUser.permissions"
              :key="permission"
              type="info"
              size="small"
              style="margin: 4px"
            >
              {{ getPermissionLabel(permission) }}
            </el-tag>
          </div>
        </div>

        <!-- 活动记录 -->
        <div class="detail-section">
          <h3>最近活动</h3>
          <div class="activity-list">
            <div 
              v-for="activity in selectedUser.recentActivities"
              :key="activity.id"
              class="activity-item"
            >
              <div class="activity-icon">
                <el-icon><Operation /></el-icon>
              </div>
              <div class="activity-content">
                <div class="activity-title">{{ activity.action }}</div>
                <div class="activity-time">{{ formatDate(activity.timestamp) }}</div>
              </div>
              <div class="activity-result">
                <el-tag 
                  :type="activity.result === 'success' ? 'success' : 'danger'"
                  size="small"
                >
                  {{ activity.result === 'success' ? '成功' : '失败' }}
                </el-tag>
              </div>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <el-button @click="userDetailVisible = false">关闭</el-button>
        <el-button type="primary" @click="editUser(selectedUser)">编辑用户</el-button>
      </template>
    </el-dialog>

    <!-- 批量操作对话框 -->
    <el-dialog
      v-model="batchOperationVisible"
      title="批量操作"
      width="500px"
    >
      <div class="batch-operation">
        <div class="selected-users">
          <p>已选择 {{ selectedUsers.length }} 个用户</p>
          <div class="user-tags">
            <el-tag
              v-for="user in selectedUsers"
              :key="user.id"
              closable
              @close="removeFromSelection(user)"
            >
              {{ user.username }}
            </el-tag>
          </div>
        </div>

        <el-form label-width="100px">
          <el-form-item label="操作类型">
            <el-radio-group v-model="batchOperation">
              <el-radio label="enable">批量启用</el-radio>
              <el-radio label="disable">批量禁用</el-radio>
              <el-radio label="delete">批量删除</el-radio>
              <el-radio label="changeRole">批量修改角色</el-radio>
            </el-radio-group>
          </el-form-item>
          
          <el-form-item v-if="batchOperation === 'changeRole'" label="新角色">
            <el-select v-model="newRole" placeholder="选择新角色">
              <el-option label="管理员" value="admin" />
              <el-option label="操作员" value="operator" />
              <el-option label="观察者" value="viewer" />
              <el-option label="访客" value="guest" />
            </el-select>
          </el-form-item>
        </el-form>
      </div>

      <template #footer>
        <el-button @click="batchOperationVisible = false">取消</el-button>
        <el-button 
          type="primary" 
          @click="executeBatchOperation"
          :loading="batchProcessing"
        >
          执行操作
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import {
  Plus, Operation, Search, RefreshRight, User, UserFilled,
  Connection, ArrowDown
} from '@element-plus/icons-vue'

interface UserItem {
  id: string
  username: string
  email: string
  role: 'admin' | 'operator' | 'viewer' | 'guest'
  department: string
  status: 'active' | 'disabled' | 'locked'
  avatar?: string
  realName?: string
  phone?: string
  note?: string
  isOnline: boolean
  lastLogin?: string
  createdAt: string
  loginCount: number
  permissions: string[]
  recentActivities: Array<{
    id: string
    action: string
    timestamp: string
    result: 'success' | 'failed'
  }>
}

interface UserStats {
  totalUsers: number
  activeUsers: number
  onlineUsers: number
  adminUsers: number
}

// 响应式数据
const loading = ref(false)
const searchQuery = ref('')
const selectedRole = ref('')
const selectedStatus = ref('')
const viewMode = ref('table')
const currentPage = ref(1)
const pageSize = ref(20)
const userDialogVisible = ref(false)
const userDetailVisible = ref(false)
const batchOperationVisible = ref(false)
const isEditMode = ref(false)
const submitting = ref(false)
const batchProcessing = ref(false)

// 表单数据
const userFormRef = ref<FormInstance>()
const userForm = reactive({
  username: '',
  email: '',
  password: '',
  role: 'viewer',
  department: '',
  status: 'active',
  realName: '',
  phone: '',
  note: ''
})

// 用户数据
const users = ref<UserItem[]>([])
const selectedUser = ref<UserItem | null>(null)
const selectedUsers = ref<UserItem[]>([])
const batchOperation = ref('enable')
const newRole = ref('')

// 统计数据
const userStats = ref<UserStats>({
  totalUsers: 0,
  activeUsers: 0,
  onlineUsers: 0,
  adminUsers: 0
})

// 表单验证规则
const userFormRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 3, max: 20, message: '用户名长度为3-20个字符', trigger: 'blur' }
  ],
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入正确的邮箱格式', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码长度不能少于6位', trigger: 'blur' }
  ],
  role: [
    { required: true, message: '请选择用户角色', trigger: 'change' }
  ]
}

// 计算属性
const filteredUsers = computed(() => {
  let filtered = users.value

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(user => 
      user.username.toLowerCase().includes(query) ||
      user.email.toLowerCase().includes(query) ||
      user.realName?.toLowerCase().includes(query)
    )
  }

  if (selectedRole.value) {
    filtered = filtered.filter(user => user.role === selectedRole.value)
  }

  if (selectedStatus.value) {
    filtered = filtered.filter(user => user.status === selectedStatus.value)
  }

  return filtered
})

// 工具函数
const formatDate = (dateString: string): string => {
  return new Date(dateString).toLocaleString('zh-CN')
}

const getRoleType = (role: string) => {
  const types = {
    'admin': 'danger',
    'operator': 'warning',
    'viewer': 'success',
    'guest': 'info'
  }
  return types[role as keyof typeof types] || 'info'
}

const getRoleLabel = (role: string): string => {
  const labels = {
    'admin': '管理员',
    'operator': '操作员',
    'viewer': '观察者',
    'guest': '访客'
  }
  return labels[role as keyof typeof labels] || role
}

const getStatusType = (status: string) => {
  const types = {
    'active': 'success',
    'disabled': 'warning',
    'locked': 'danger'
  }
  return types[status as keyof typeof types] || 'info'
}

const getStatusLabel = (status: string): string => {
  const labels = {
    'active': '活跃',
    'disabled': '禁用',
    'locked': '锁定'
  }
  return labels[status as keyof typeof labels] || status
}

const getPermissionLabel = (permission: string): string => {
  const labels = {
    'user.view': '查看用户',
    'user.create': '创建用户',
    'user.edit': '编辑用户',
    'user.delete': '删除用户',
    'driver.view': '查看驱动',
    'driver.manage': '管理驱动',
    'system.config': '系统配置',
    'system.monitor': '系统监控'
  }
  return labels[permission as keyof typeof labels] || permission
}

// 用户操作
const openCreateUserDialog = () => {
  isEditMode.value = false
  resetUserForm()
  userDialogVisible.value = true
}

const editUser = (user: UserItem) => {
  isEditMode.value = true
  Object.assign(userForm, {
    username: user.username,
    email: user.email,
    password: '',
    role: user.role,
    department: user.department,
    status: user.status,
    realName: user.realName || '',
    phone: user.phone || '',
    note: user.note || ''
  })
  selectedUser.value = user
  userDialogVisible.value = true
}

const viewUser = (user: UserItem) => {
  selectedUser.value = user
  userDetailVisible.value = true
}

const closeUserDialog = () => {
  userDialogVisible.value = false
  resetUserForm()
  selectedUser.value = null
}

const resetUserForm = () => {
  Object.assign(userForm, {
    username: '',
    email: '',
    password: '',
    role: 'viewer',
    department: '',
    status: 'active',
    realName: '',
    phone: '',
    note: ''
  })
  userFormRef.value?.resetFields()
}

const saveUser = async () => {
  if (!userFormRef.value) return

  try {
    await userFormRef.value.validate()
    submitting.value = true

    // 模拟保存用户
    await new Promise(resolve => setTimeout(resolve, 1000))

    if (isEditMode.value && selectedUser.value) {
      // 更新用户
      const index = users.value.findIndex(u => u.id === selectedUser.value!.id)
      if (index > -1) {
        Object.assign(users.value[index], {
          username: userForm.username,
          email: userForm.email,
          role: userForm.role,
          department: userForm.department,
          status: userForm.status,
          realName: userForm.realName,
          phone: userForm.phone,
          note: userForm.note
        })
      }
      ElMessage.success('用户更新成功')
    } else {
      // 创建新用户
      const newUser: UserItem = {
        id: `user_${Date.now()}`,
        username: userForm.username,
        email: userForm.email,
        role: userForm.role as any,
        department: userForm.department,
        status: userForm.status as any,
        realName: userForm.realName,
        phone: userForm.phone,
        note: userForm.note,
        isOnline: false,
        createdAt: new Date().toISOString(),
        loginCount: 0,
        permissions: getDefaultPermissions(userForm.role),
        recentActivities: []
      }
      users.value.unshift(newUser)
      ElMessage.success('用户创建成功')
    }

    updateStats()
    closeUserDialog()
  } catch (error) {
    console.error('保存用户失败:', error)
  } finally {
    submitting.value = false
  }
}

const toggleUserStatus = async (user: UserItem) => {
  try {
    const newStatus = user.status === 'active' ? 'disabled' : 'active'
    const action = newStatus === 'active' ? '启用' : '禁用'
    
    await ElMessageBox.confirm(
      `确定要${action}用户 "${user.username}" 吗？`,
      `${action}用户`,
      {
        type: 'warning'
      }
    )

    user.status = newStatus
    updateStats()
    ElMessage.success(`用户已${action}`)
  } catch (error) {
    // 用户取消
  }
}

const handleUserAction = async (command: string, user: UserItem) => {
  switch (command) {
    case 'resetPassword':
      await resetPassword(user)
      break
    case 'permissions':
      ElMessage.info('权限设置功能开发中...')
      break
    case 'sessions':
      ElMessage.info('会话管理功能开发中...')
      break
    case 'delete':
      await deleteUser(user)
      break
  }
}

const resetPassword = async (user: UserItem) => {
  try {
    await ElMessageBox.confirm(
      `确定要重置用户 "${user.username}" 的密码吗？新密码将通过邮件发送给用户。`,
      '重置密码',
      {
        type: 'warning'
      }
    )

    // 模拟重置密码
    await new Promise(resolve => setTimeout(resolve, 1000))
    ElMessage.success('密码已重置，新密码已发送到用户邮箱')
  } catch (error) {
    // 用户取消
  }
}

const deleteUser = async (user: UserItem) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除用户 "${user.username}" 吗？此操作不可撤销。`,
      '删除用户',
      {
        type: 'warning',
        confirmButtonText: '删除',
        cancelButtonText: '取消'
      }
    )

    const index = users.value.findIndex(u => u.id === user.id)
    if (index > -1) {
      users.value.splice(index, 1)
      updateStats()
      ElMessage.success('用户删除成功')
    }
  } catch (error) {
    // 用户取消
  }
}

const handleSelectionChange = (selection: UserItem[]) => {
  selectedUsers.value = selection
}

const openBatchOperationDialog = () => {
  if (selectedUsers.value.length === 0) {
    ElMessage.warning('请先选择要操作的用户')
    return
  }
  batchOperationVisible.value = true
}

const removeFromSelection = (user: UserItem) => {
  const index = selectedUsers.value.findIndex(u => u.id === user.id)
  if (index > -1) {
    selectedUsers.value.splice(index, 1)
  }
}

const executeBatchOperation = async () => {
  if (selectedUsers.value.length === 0) {
    ElMessage.warning('没有选择任何用户')
    return
  }

  try {
    batchProcessing.value = true
    
    // 模拟批量操作
    await new Promise(resolve => setTimeout(resolve, 2000))

    switch (batchOperation.value) {
      case 'enable':
        selectedUsers.value.forEach(user => {
          user.status = 'active'
        })
        ElMessage.success(`已启用 ${selectedUsers.value.length} 个用户`)
        break
      case 'disable':
        selectedUsers.value.forEach(user => {
          user.status = 'disabled'
        })
        ElMessage.success(`已禁用 ${selectedUsers.value.length} 个用户`)
        break
      case 'changeRole':
        if (!newRole.value) {
          ElMessage.warning('请选择新角色')
          return
        }
        selectedUsers.value.forEach(user => {
          user.role = newRole.value as any
          user.permissions = getDefaultPermissions(newRole.value)
        })
        ElMessage.success(`已修改 ${selectedUsers.value.length} 个用户的角色`)
        break
      case 'delete':
        await ElMessageBox.confirm(
          `确定要删除选中的 ${selectedUsers.value.length} 个用户吗？此操作不可撤销。`,
          '批量删除用户',
          { type: 'warning' }
        )
        const idsToDelete = selectedUsers.value.map(u => u.id)
        users.value = users.value.filter(u => !idsToDelete.includes(u.id))
        ElMessage.success(`已删除 ${selectedUsers.value.length} 个用户`)
        break
    }

    updateStats()
    selectedUsers.value = []
    batchOperationVisible.value = false
  } catch (error) {
    console.error('批量操作失败:', error)
  } finally {
    batchProcessing.value = false
  }
}

const resetFilters = () => {
  searchQuery.value = ''
  selectedRole.value = ''
  selectedStatus.value = ''
}

const getDefaultPermissions = (role: string): string[] => {
  const permissionSets = {
    'admin': [
      'user.view', 'user.create', 'user.edit', 'user.delete',
      'driver.view', 'driver.manage',
      'system.config', 'system.monitor'
    ],
    'operator': [
      'user.view', 'driver.view', 'driver.manage', 'system.monitor'
    ],
    'viewer': ['user.view', 'driver.view', 'system.monitor'],
    'guest': ['user.view', 'driver.view']
  }
  return permissionSets[role as keyof typeof permissionSets] || []
}

const updateStats = () => {
  userStats.value.totalUsers = users.value.length
  userStats.value.activeUsers = users.value.filter(u => u.status === 'active').length
  userStats.value.onlineUsers = users.value.filter(u => u.isOnline).length
  userStats.value.adminUsers = users.value.filter(u => u.role === 'admin').length
}

// 模拟数据加载
const loadMockData = () => {
  const mockUsers: UserItem[] = [
    {
      id: 'user_1',
      username: 'admin',
      email: 'admin@gateway.com',
      role: 'admin',
      department: 'IT部门',
      status: 'active',
      realName: '系统管理员',
      phone: '13800138001',
      note: '系统默认管理员账户',
      isOnline: true,
      lastLogin: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
      createdAt: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
      loginCount: 256,
      permissions: [
        'user.view', 'user.create', 'user.edit', 'user.delete',
        'driver.view', 'driver.manage',
        'system.config', 'system.monitor'
      ],
      recentActivities: [
        {
          id: 'act_1',
          action: '修改系统配置',
          timestamp: new Date(Date.now() - 10 * 60 * 1000).toISOString(),
          result: 'success'
        },
        {
          id: 'act_2',
          action: '创建新用户',
          timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
          result: 'success'
        }
      ]
    },
    {
      id: 'user_2',
      username: 'operator1',
      email: 'operator1@gateway.com',
      role: 'operator',
      department: '运维部门',
      status: 'active',
      realName: '张三',
      phone: '13800138002',
      isOnline: false,
      lastLogin: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
      createdAt: new Date(Date.now() - 20 * 24 * 60 * 60 * 1000).toISOString(),
      loginCount: 89,
      permissions: [
        'user.view', 'driver.view', 'driver.manage', 'system.monitor'
      ],
      recentActivities: [
        {
          id: 'act_3',
          action: '重启驱动服务',
          timestamp: new Date(Date.now() - 3 * 60 * 60 * 1000).toISOString(),
          result: 'success'
        }
      ]
    },
    {
      id: 'user_3',
      username: 'viewer1',
      email: 'viewer1@gateway.com',
      role: 'viewer',
      department: '业务部门',
      status: 'active',
      realName: '李四',
      phone: '13800138003',
      isOnline: true,
      lastLogin: new Date(Date.now() - 1 * 60 * 60 * 1000).toISOString(),
      createdAt: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000).toISOString(),
      loginCount: 45,
      permissions: ['user.view', 'driver.view', 'system.monitor'],
      recentActivities: [
        {
          id: 'act_4',
          action: '查看系统监控',
          timestamp: new Date(Date.now() - 1 * 60 * 60 * 1000).toISOString(),
          result: 'success'
        }
      ]
    },
    {
      id: 'user_4',
      username: 'guest1',
      email: 'guest1@gateway.com',
      role: 'guest',
      department: '访客',
      status: 'disabled',
      realName: '王五',
      isOnline: false,
      lastLogin: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
      createdAt: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
      loginCount: 3,
      permissions: ['user.view', 'driver.view'],
      recentActivities: []
    }
  ]

  users.value = mockUsers
  updateStats()
}

// 生命周期
onMounted(() => {
  loadMockData()
})
</script>

<style scoped lang="scss">
.user-management {
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
  
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    padding: 20px 24px;
    
    .stat-card {
      .stat-content {
        display: flex;
        align-items: center;
        position: relative;
        
        .stat-value {
          font-size: 32px;
          font-weight: bold;
          color: #303133;
          margin-bottom: 4px;
        }
        
        .stat-label {
          color: #909399;
          font-size: 14px;
        }
        
        .stat-icon {
          position: absolute;
          right: 12px;
          top: 50%;
          transform: translateY(-50%);
          font-size: 32px;
          opacity: 0.3;
          
          &.users { color: #409eff; }
          &.active { color: #67c23a; }
          &.online { color: #67c23a; }
          &.admin { color: #f56c6c; }
        }
      }
    }
  }
  
  .main-content {
    flex: 1;
    padding: 0 24px 24px;
    
    .toolbar {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;
      
      .search-section {
        display: flex;
        gap: 12px;
        align-items: center;
      }
    }
    
    .table-view {
      .text-muted {
        color: #909399;
      }
    }
    
    .card-view {
      .user-cards {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: 16px;
        
        .user-card {
          border: 1px solid #e4e7ed;
          border-radius: 8px;
          padding: 16px;
          background: white;
          cursor: pointer;
          transition: all 0.3s;
          
          &:hover {
            border-color: #409eff;
            box-shadow: 0 2px 8px rgba(64, 158, 255, 0.1);
          }
          
          .card-header {
            display: flex;
            align-items: center;
            margin-bottom: 12px;
            
            .user-info {
              flex: 1;
              margin-left: 12px;
              
              .username {
                font-weight: 500;
                color: #303133;
              }
              
              .email {
                font-size: 12px;
                color: #909399;
              }
            }
          }
          
          .card-body {
            .info-item {
              display: flex;
              align-items: center;
              margin-bottom: 8px;
              
              label {
                min-width: 60px;
                font-weight: 500;
                color: #606266;
                font-size: 12px;
              }
              
              span {
                font-size: 12px;
                color: #303133;
                
                &.text-muted {
                  color: #909399;
                }
              }
            }
          }
          
          .card-actions {
            display: flex;
            gap: 8px;
            margin-top: 12px;
            padding-top: 12px;
            border-top: 1px solid #f5f7fa;
          }
        }
      }
    }
    
    .pagination-wrapper {
      display: flex;
      justify-content: center;
      margin-top: 20px;
    }
  }
  
  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }
  
  .user-detail {
    .detail-section {
      margin-bottom: 24px;
      
      h3 {
        margin: 0 0 16px 0;
        font-size: 16px;
        font-weight: 500;
        color: #303133;
        padding-bottom: 8px;
        border-bottom: 1px solid #e4e7ed;
      }
      
      .detail-content {
        .user-avatar-section {
          display: flex;
          align-items: center;
          gap: 16px;
          
          .user-basic-info {
            .info-row {
              margin-bottom: 8px;
              
              &:first-child {
                font-size: 18px;
                font-weight: 500;
                color: #303133;
                display: flex;
                align-items: center;
                gap: 8px;
              }
              
              &:not(:first-child) {
                color: #606266;
              }
            }
          }
        }
      }
      
      .info-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 16px;
        
        .info-item {
          display: flex;
          align-items: center;
          
          label {
            min-width: 80px;
            font-weight: 500;
            color: #606266;
          }
          
          span {
            color: #303133;
            
            &.text-muted {
              color: #909399;
            }
          }
        }
      }
      
      .permissions-list {
        padding: 12px;
        background: #f8f9fa;
        border-radius: 4px;
        min-height: 60px;
      }
      
      .activity-list {
        .activity-item {
          display: flex;
          align-items: center;
          padding: 12px 0;
          border-bottom: 1px solid #f5f7fa;
          
          &:last-child {
            border-bottom: none;
          }
          
          .activity-icon {
            width: 32px;
            height: 32px;
            border-radius: 50%;
            background: #f0f9ff;
            display: flex;
            align-items: center;
            justify-content: center;
            color: #409eff;
          }
          
          .activity-content {
            flex: 1;
            margin-left: 12px;
            
            .activity-title {
              font-weight: 500;
              color: #303133;
            }
            
            .activity-time {
              font-size: 12px;
              color: #909399;
              margin-top: 2px;
            }
          }
        }
      }
    }
  }
  
  .batch-operation {
    .selected-users {
      margin-bottom: 20px;
      
      p {
        margin: 0 0 8px 0;
        font-weight: 500;
      }
      
      .user-tags {
        .el-tag {
          margin: 4px 4px 4px 0;
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .user-management {
    .stats-grid {
      grid-template-columns: 1fr 1fr;
      padding: 16px;
    }
    
    .page-header {
      padding: 16px;
      
      .header-content {
        flex-direction: column;
        align-items: flex-start;
        gap: 16px;
      }
    }
    
    .main-content {
      padding: 0 16px 16px;
      
      .toolbar {
        flex-direction: column;
        align-items: stretch;
        gap: 12px;
        
        .search-section {
          flex-wrap: wrap;
        }
      }
    }
    
    .form-grid {
      grid-template-columns: 1fr;
    }
    
    .user-detail .info-grid {
      grid-template-columns: 1fr;
    }
  }
}
</style>