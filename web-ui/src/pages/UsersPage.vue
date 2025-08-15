<template>
  <div class="users-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="title-section">
          <h1 class="page-title">用户管理</h1>
          <p class="page-description">管理系统用户、角色权限和访问控制</p>
        </div>

        <div class="header-actions">
          <el-button
            v-permission="'users:create'"
            type="primary"
            @click="showCreateUser = true"
          >
            <el-icon><Plus /></el-icon>
            创建用户
          </el-button>
          <el-button :loading="loading" @click="refreshUsers">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
          <el-dropdown @command="handleBatchAction">
            <el-button :disabled="selectedUsers.length === 0">
              批量操作
              <el-icon><ArrowDown /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item
                  v-permission="'users:update'"
                  command="enable"
                >
                  启用用户
                </el-dropdown-item>
                <el-dropdown-item
                  v-permission="'users:update'"
                  command="disable"
                >
                  禁用用户
                </el-dropdown-item>
                <el-dropdown-item
                  v-permission="'users:update'"
                  command="resetPassword"
                >
                  重置密码
                </el-dropdown-item>
                <el-dropdown-item
                  v-permission="'users:delete'"
                  command="delete"
                  divided
                >
                  删除用户
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
    </div>

    <!-- 筛选和搜索面板 -->
    <el-card class="filter-panel" shadow="never">
      <div class="filter-form">
        <div class="filter-row">
          <div class="filter-item">
            <label class="filter-label">用户状态</label>
            <el-select
              v-model="filterForm.status"
              placeholder="选择状态"
              clearable
              style="width: 150px"
              @change="handleFilterChange"
            >
              <el-option label="全部" value="" />
              <el-option label="启用" value="active" />
              <el-option label="禁用" value="inactive" />
              <el-option label="锁定" value="locked" />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">用户角色</label>
            <el-select
              v-model="filterForm.role"
              placeholder="选择角色"
              clearable
              style="width: 150px"
              @change="handleFilterChange"
            >
              <el-option label="全部" value="" />
              <el-option
                v-for="role in availableRoles"
                :key="role.id"
                :label="role.name"
                :value="role.id"
              />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">部门</label>
            <el-select
              v-model="filterForm.department"
              placeholder="选择部门"
              clearable
              style="width: 150px"
              @change="handleFilterChange"
            >
              <el-option label="全部" value="" />
              <el-option
                v-for="dept in departments"
                :key="dept.id"
                :label="dept.name"
                :value="dept.id"
              />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">搜索</label>
            <el-input
              v-model="filterForm.keyword"
              placeholder="搜索用户名、邮箱或姓名"
              clearable
              style="width: 300px"
              @keyup.enter="searchUsers"
              @clear="searchUsers"
            >
              <template #append>
                <el-button @click="searchUsers">
                  <el-icon><Search /></el-icon>
                </el-button>
              </template>
            </el-input>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 用户统计卡片 -->
    <div class="stats-cards">
      <el-card shadow="never" class="stat-card">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon><User /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-number">{{ userStats.total }}</div>
            <div class="stat-label">总用户数</div>
          </div>
        </div>
      </el-card>

      <el-card shadow="never" class="stat-card active">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon><Select /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-number">{{ userStats.active }}</div>
            <div class="stat-label">活跃用户</div>
          </div>
        </div>
      </el-card>

      <el-card shadow="never" class="stat-card online">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon><Connection /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-number">{{ userStats.online }}</div>
            <div class="stat-label">在线用户</div>
          </div>
        </div>
      </el-card>

      <el-card shadow="never" class="stat-card locked">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon><Lock /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-number">{{ userStats.locked }}</div>
            <div class="stat-label">锁定用户</div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 用户列表 -->
    <el-card class="users-table" shadow="never">
      <template #header>
        <div class="table-header">
          <span class="table-title">用户列表</span>
          <div class="table-actions">
            <el-button-group>
              <el-button
                :type="viewMode === 'table' ? 'primary' : 'default'"
                @click="viewMode = 'table'"
              >
                <el-icon><List /></el-icon>
                表格
              </el-button>
              <el-button
                :type="viewMode === 'card' ? 'primary' : 'default'"
                @click="viewMode = 'card'"
              >
                <el-icon><Grid /></el-icon>
                卡片
              </el-button>
            </el-button-group>
          </div>
        </div>
      </template>

      <!-- 表格视图 -->
      <div v-if="viewMode === 'table'">
        <el-table
          :data="userList"
          :loading="loading"
          row-key="id"
          @selection-change="handleSelectionChange"
          @sort-change="handleSortChange"
        >
          <el-table-column type="selection" width="55" />

          <el-table-column label="用户信息" min-width="250">
            <template #default="{ row }">
              <div class="user-info">
                <el-avatar :size="40" :src="row.avatar">
                  <el-icon><UserFilled /></el-icon>
                </el-avatar>
                <div class="user-details">
                  <div class="user-name">{{ row.name }}</div>
                  <div class="user-username">@{{ row.username }}</div>
                  <div class="user-email">{{ row.email }}</div>
                </div>
              </div>
            </template>
          </el-table-column>

          <el-table-column label="角色" width="120">
            <template #default="{ row }">
              <el-tag :type="getRoleType(row.role)" size="small">
                {{ getRoleName(row.role) }}
              </el-tag>
            </template>
          </el-table-column>

          <el-table-column label="部门" width="120">
            <template #default="{ row }">
              <span>{{ getDepartmentName(row.departmentId) }}</span>
            </template>
          </el-table-column>

          <el-table-column label="状态" width="100" align="center">
            <template #default="{ row }">
              <el-tag :type="getStatusType(row.status)" size="small">
                {{ getStatusLabel(row.status) }}
              </el-tag>
            </template>
          </el-table-column>

          <el-table-column
            label="最后登录"
            width="180"
            sortable="custom"
            prop="lastLoginTime"
          >
            <template #default="{ row }">
              <div v-if="row.lastLoginTime" class="login-info">
                <div class="login-time">
                  {{ formatDateTime(row.lastLoginTime) }}
                </div>
                <div class="login-ip">{{ row.lastLoginIp }}</div>
              </div>
              <span v-else class="text-muted">从未登录</span>
            </template>
          </el-table-column>

          <el-table-column
            label="创建时间"
            width="150"
            sortable="custom"
            prop="createdAt"
          >
            <template #default="{ row }">
              {{ formatDate(row.createdAt) }}
            </template>
          </el-table-column>

          <el-table-column label="操作" width="200" fixed="right">
            <template #default="{ row }">
              <div class="table-actions">
                <el-button
                  type="link"
                  size="small"
                  @click="viewUserDetail(row)"
                >
                  详情
                </el-button>
                <el-button
                  v-permission="'users:update'"
                  type="link"
                  size="small"
                  @click="editUser(row)"
                >
                  编辑
                </el-button>
                <el-dropdown @command="cmd => handleUserAction(cmd, row)">
                  <el-button type="link" size="small">
                    更多 <el-icon><ArrowDown /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item
                        v-permission="'users:update'"
                        :command="`${row.status === 'active' ? 'disable' : 'enable'}`"
                      >
                        {{ row.status === 'active' ? '禁用' : '启用' }}
                      </el-dropdown-item>
                      <el-dropdown-item
                        v-permission="'users:update'"
                        command="resetPassword"
                      >
                        重置密码
                      </el-dropdown-item>
                      <el-dropdown-item
                        v-permission="'permissions:assign'"
                        command="permissions"
                      >
                        权限设置
                      </el-dropdown-item>
                      <el-dropdown-item
                        v-permission.any="['users:update', 'system:logs']"
                        command="sessions"
                        divided
                      >
                        会话管理
                      </el-dropdown-item>
                      <el-dropdown-item
                        v-permission="'system:logs'"
                        command="logs"
                      >
                        操作日志
                      </el-dropdown-item>
                      <el-dropdown-item
                        v-permission="'users:delete'"
                        command="delete"
                        divided
                      >
                        删除用户
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </div>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- 卡片视图 -->
      <div v-else class="card-view">
        <div class="user-cards">
          <div
            v-for="user in userList"
            :key="user.id"
            class="user-card"
            @click="viewUserDetail(user)"
          >
            <div class="card-header">
              <el-avatar :size="60" :src="user.avatar">
                <el-icon><UserFilled /></el-icon>
              </el-avatar>
              <div class="user-status">
                <el-tag :type="getStatusType(user.status)" size="small">
                  {{ getStatusLabel(user.status) }}
                </el-tag>
              </div>
            </div>

            <div class="card-content">
              <div class="user-name">{{ user.name }}</div>
              <div class="user-username">@{{ user.username }}</div>
              <div class="user-email">{{ user.email }}</div>

              <div class="user-meta">
                <div class="meta-item">
                  <span class="meta-label">角色：</span>
                  <el-tag :type="getRoleType(user.role)" size="small">
                    {{ getRoleName(user.role) }}
                  </el-tag>
                </div>
                <div class="meta-item">
                  <span class="meta-label">部门：</span>
                  <span>{{ getDepartmentName(user.departmentId) }}</span>
                </div>
                <div v-if="user.lastLoginTime" class="meta-item">
                  <span class="meta-label">最后登录：</span>
                  <span>{{ formatDateTime(user.lastLoginTime) }}</span>
                </div>
              </div>
            </div>

            <div class="card-actions">
              <el-button type="link" size="small" @click.stop="editUser(user)">
                编辑
              </el-button>
              <el-button
                type="link"
                size="small"
                @click.stop="
                  user.status === 'active'
                    ? disableUser(user)
                    : enableUser(user)
                "
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
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.size"
          :total="pagination.total"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- 用户编辑对话框 -->
    <UserEditDialog
      v-model:visible="showEditUser"
      :user-data="selectedUser"
      :is-create="isCreateMode"
      @save="handleUserSave"
    />

    <!-- 用户详情对话框 -->
    <UserDetailDialog
      v-model:visible="showUserDetail"
      :user-data="selectedUser"
    />

    <!-- 权限分配对话框 -->
    <PermissionAssignDialog
      v-model:visible="showPermissionDialog"
      :user-data="selectedUser"
      @save="handlePermissionSave"
    />

    <!-- 会话管理对话框 -->
    <UserSessionDialog
      v-model:visible="showSessionDialog"
      :user-data="selectedUser"
    />

    <!-- 用户活动日志对话框 -->
    <UserActivityDialog
      v-model:visible="showActivityDialog"
      :user-data="selectedUser"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * UsersPage —— 用户管理页面
 *
 * 📝 Responsibilities:
 *  1. 用户列表展示和管理
 *  2. 用户信息CRUD操作
 *  3. 用户状态管理和批量操作
 *  4. 角色权限分配
 *  5. 用户活动监控
 *
 * 📦 Dependencies:
 *  - UserEditDialog 用户编辑对话框
 *  - UserDetailDialog 用户详情对话框
 *  - PermissionAssignDialog 权限分配对话框
 *  - UserSessionDialog 会话管理对话框
 *  - UserActivityDialog 活动日志对话框
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Plus,
  Refresh,
  ArrowDown,
  Search,
  User,
  Select,
  Connection,
  Lock,
  List,
  Grid,
  UserFilled,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'

// 组件导入（这些组件将在后续步骤中创建）
import PermissionAssignDialog from '@/components/users/PermissionAssignDialog.vue'
import UserActivityDialog from '@/components/users/UserActivityDialog.vue'
import UserDetailDialog from '@/components/users/UserDetailDialog.vue'
import UserEditDialog from '@/components/users/UserEditDialog.vue'
import UserSessionDialog from '@/components/users/UserSessionDialog.vue'
import { usePermission } from '@/composables/usePermission'

// ===== 路由 =====
const router = useRouter()
const { hasPermission, canPerformAction, logUserActivity, permissionState } =
  usePermission()

// ===== 响应式数据 =====
const loading = ref(false)
const viewMode = ref<'table' | 'card'>('table')

// 对话框状态
const showCreateUser = ref(false)
const showEditUser = ref(false)
const showUserDetail = ref(false)
const showPermissionDialog = ref(false)
const showSessionDialog = ref(false)
const showActivityDialog = ref(false)

// 选中的用户和模式
const selectedUser = ref<any>(null)
const selectedUsers = ref<any[]>([])
const isCreateMode = ref(false)

// 筛选表单
const filterForm = ref({
  status: '',
  role: '',
  department: '',
  keyword: '',
})

// 分页
const pagination = ref({
  page: 1,
  size: 20,
  total: 0,
})

// 排序
const sortField = ref('createdAt')
const sortOrder = ref('desc')

// 数据
const userList = ref<any[]>([])
const availableRoles = ref<any[]>([])
const departments = ref<any[]>([])

// 用户统计
const userStats = ref({
  total: 0,
  active: 0,
  online: 0,
  locked: 0,
})

// ===== 计算属性 =====
const hasSelection = computed(() => {
  return selectedUsers.value.length > 0
})

// ===== 方法 =====

/**
 * 初始化数据
 */
async function initializeData() {
  try {
    // 生成模拟数据
    availableRoles.value = generateMockRoles()
    departments.value = generateMockDepartments()

    // 加载用户数据
    await loadUsers()
  } catch (error) {
    console.error('初始化用户管理页面失败:', error)
    ElMessage.error('页面初始化失败')
  }
}

/**
 * 生成模拟角色数据
 */
function generateMockRoles() {
  return [
    { id: 'admin', name: '系统管理员', description: '系统最高权限' },
    { id: 'operator', name: '操作员', description: '设备操作权限' },
    { id: 'viewer', name: '观察员', description: '只读权限' },
    { id: 'engineer', name: '工程师', description: '工程配置权限' },
    { id: 'manager', name: '管理员', description: '部门管理权限' },
  ]
}

/**
 * 生成模拟部门数据
 */
function generateMockDepartments() {
  return [
    { id: 'it', name: 'IT部门' },
    { id: 'production', name: '生产部门' },
    { id: 'maintenance', name: '维护部门' },
    { id: 'quality', name: '质量部门' },
    { id: 'management', name: '管理部门' },
  ]
}

/**
 * 生成模拟用户数据
 */
function generateMockUsers() {
  const users = []
  const roles = ['admin', 'operator', 'viewer', 'engineer', 'manager']
  const statuses = ['active', 'inactive', 'locked']
  const depts = ['it', 'production', 'maintenance', 'quality', 'management']

  for (let i = 1; i <= 50; i++) {
    const status = statuses[i % statuses.length]
    const lastLoginTime =
      status === 'active' && Math.random() > 0.3
        ? new Date(Date.now() - Math.random() * 30 * 24 * 60 * 60 * 1000)
        : null

    users.push({
      id: `user_${i}`,
      username: `user${i}`,
      name: `用户${i}`,
      email: `user${i}@company.com`,
      phone: `1380013800${String(i).padStart(2, '0')}`,
      avatar: `https://api.dicebear.com/7.x/avataaars/svg?seed=user${i}`,
      role: roles[i % roles.length],
      departmentId: depts[i % depts.length],
      status,
      lastLoginTime: lastLoginTime?.toISOString() || null,
      lastLoginIp: lastLoginTime ? `192.168.1.${100 + (i % 50)}` : null,
      createdAt: new Date(
        Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000
      ).toISOString(),
      updatedAt: new Date(
        Date.now() - Math.random() * 30 * 24 * 60 * 60 * 1000
      ).toISOString(),
      loginCount: Math.floor(Math.random() * 1000),
      isOnline: status === 'active' && Math.random() > 0.7,
    })
  }

  return users
}

/**
 * 加载用户数据
 */
async function loadUsers() {
  try {
    loading.value = true

    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000))

    let mockUsers = generateMockUsers()

    // 应用筛选条件
    if (filterForm.value.status) {
      mockUsers = mockUsers.filter(
        user => user.status === filterForm.value.status
      )
    }

    if (filterForm.value.role) {
      mockUsers = mockUsers.filter(user => user.role === filterForm.value.role)
    }

    if (filterForm.value.department) {
      mockUsers = mockUsers.filter(
        user => user.departmentId === filterForm.value.department
      )
    }

    if (filterForm.value.keyword) {
      const keyword = filterForm.value.keyword.toLowerCase()
      mockUsers = mockUsers.filter(
        user =>
          user.username.toLowerCase().includes(keyword) ||
          user.name.toLowerCase().includes(keyword) ||
          user.email.toLowerCase().includes(keyword)
      )
    }

    // 排序
    mockUsers.sort((a, b) => {
      const aValue = a[sortField.value]
      const bValue = b[sortField.value]

      if (sortOrder.value === 'desc') {
        return bValue > aValue ? 1 : -1
      } else {
        return aValue > bValue ? 1 : -1
      }
    })

    // 分页
    const start = (pagination.value.page - 1) * pagination.value.size
    const end = start + pagination.value.size
    userList.value = mockUsers.slice(start, end)
    pagination.value.total = mockUsers.length

    // 更新统计
    updateUserStats(mockUsers)
  } catch (error) {
    console.error('加载用户数据失败:', error)
    ElMessage.error('加载用户数据失败')
  } finally {
    loading.value = false
  }
}

/**
 * 更新用户统计
 */
function updateUserStats(users: any[]) {
  userStats.value = {
    total: users.length,
    active: users.filter(u => u.status === 'active').length,
    online: users.filter(u => u.isOnline).length,
    locked: users.filter(u => u.status === 'locked').length,
  }
}

/**
 * 搜索用户
 */
function searchUsers() {
  pagination.value.page = 1
  loadUsers()
}

/**
 * 刷新用户列表
 */
function refreshUsers() {
  loadUsers()
}

/**
 * 处理筛选变化
 */
function handleFilterChange() {
  pagination.value.page = 1
  loadUsers()
}

/**
 * 处理选择变化
 */
function handleSelectionChange(selection: any[]) {
  selectedUsers.value = selection
}

/**
 * 处理排序变化
 */
function handleSortChange({
  prop,
  order,
}: {
  prop: string
  order: string | null
}) {
  if (prop && order) {
    sortField.value = prop
    sortOrder.value = order === 'ascending' ? 'asc' : 'desc'
    pagination.value.page = 1
    loadUsers()
  }
}

/**
 * 处理页面变化
 */
function handlePageChange(page: number) {
  pagination.value.page = page
  loadUsers()
}

/**
 * 处理页面大小变化
 */
function handleSizeChange(size: number) {
  pagination.value.size = size
  pagination.value.page = 1
  loadUsers()
}

/**
 * 创建用户
 */
function createUser() {
  if (!canPerformAction('users', 'create').value) {
    ElMessage.error('权限不足：无法创建用户')
    return
  }

  selectedUser.value = null
  isCreateMode.value = true
  showEditUser.value = true

  // 记录活动日志
  logUserActivity({
    type: 'user_management',
    action: 'create_user_dialog_open',
    target: 'new_user',
  })
}

/**
 * 编辑用户
 */
function editUser(user: any) {
  if (!canPerformAction('users', 'update').value) {
    ElMessage.error('权限不足：无法编辑用户')
    return
  }

  selectedUser.value = user
  isCreateMode.value = false
  showEditUser.value = true

  // 记录活动日志
  logUserActivity({
    type: 'user_management',
    action: 'edit_user_dialog_open',
    target: user.id,
    details: { username: user.username, name: user.name },
  })
}

/**
 * 查看用户详情
 */
function viewUserDetail(user: any) {
  selectedUser.value = user
  showUserDetail.value = true
}

/**
 * 处理批量操作
 */
async function handleBatchAction(command: string) {
  if (selectedUsers.value.length === 0) {
    ElMessage.warning('请先选择要操作的用户')
    return
  }

  const actionMap = {
    enable: '启用',
    disable: '禁用',
    resetPassword: '重置密码',
    delete: '删除',
  }

  const actionName = actionMap[command as keyof typeof actionMap]

  try {
    await ElMessageBox.confirm(
      `确定要${actionName}选中的 ${selectedUsers.value.length} 个用户吗？`,
      '批量操作确认',
      {
        type: 'warning',
      }
    )

    // 模拟批量操作
    await new Promise(resolve => setTimeout(resolve, 1000))

    ElMessage.success(`已${actionName} ${selectedUsers.value.length} 个用户`)
    selectedUsers.value = []
    await loadUsers()
  } catch (error) {
    // 用户取消操作
  }
}

/**
 * 处理用户操作
 */
async function handleUserAction(command: string, user: any) {
  selectedUser.value = user

  // 权限检查映射
  const permissionMap: Record<string, string> = {
    enable: 'users:update',
    disable: 'users:update',
    resetPassword: 'users:update',
    permissions: 'permissions:assign',
    sessions: 'users:update',
    logs: 'system:logs',
    delete: 'users:delete',
  }

  const requiredPermission = permissionMap[command]
  if (requiredPermission && !hasPermission(requiredPermission).value) {
    ElMessage.error('权限不足：无法执行此操作')
    return
  }

  // 记录活动日志
  logUserActivity({
    type: 'user_management',
    action: `user_action_${command}`,
    target: user.id,
    details: { username: user.username, command },
  })

  switch (command) {
    case 'enable':
      await enableUser(user)
      break
    case 'disable':
      await disableUser(user)
      break
    case 'resetPassword':
      await resetUserPassword(user)
      break
    case 'permissions':
      showPermissionDialog.value = true
      break
    case 'sessions':
      showSessionDialog.value = true
      break
    case 'logs':
      showActivityDialog.value = true
      break
    case 'delete':
      await deleteUser(user)
      break
  }
}

/**
 * 启用用户
 */
async function enableUser(user: any) {
  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    ElMessage.success('用户已启用')
    await loadUsers()
  } catch (error) {
    ElMessage.error('启用用户失败')
  }
}

/**
 * 禁用用户
 */
async function disableUser(user: any) {
  try {
    await ElMessageBox.confirm(`确定要禁用用户 ${user.name} 吗？`, '确认禁用', {
      type: 'warning',
    })

    await new Promise(resolve => setTimeout(resolve, 500))
    ElMessage.success('用户已禁用')
    await loadUsers()
  } catch (error) {
    // 用户取消或操作失败
  }
}

/**
 * 重置用户密码
 */
async function resetUserPassword(user: any) {
  try {
    await ElMessageBox.confirm(
      `确定要重置用户 ${user.name} 的密码吗？`,
      '确认重置密码',
      {
        type: 'warning',
      }
    )

    await new Promise(resolve => setTimeout(resolve, 500))
    ElMessage.success('密码重置成功，新密码已发送到用户邮箱')
  } catch (error) {
    // 用户取消或操作失败
  }
}

/**
 * 删除用户
 */
async function deleteUser(user: any) {
  try {
    await ElMessageBox.confirm(
      `确定要删除用户 ${user.name} 吗？此操作不可恢复。`,
      '确认删除',
      {
        type: 'error',
      }
    )

    await new Promise(resolve => setTimeout(resolve, 500))
    ElMessage.success('用户已删除')
    await loadUsers()
  } catch (error) {
    // 用户取消或操作失败
  }
}

/**
 * 处理用户保存
 */
function handleUserSave(userData: any) {
  if (isCreateMode.value) {
    ElMessage.success('用户创建成功')
  } else {
    ElMessage.success('用户信息更新成功')
  }
  loadUsers()
}

/**
 * 处理权限保存
 */
function handlePermissionSave(permissionData: any) {
  ElMessage.success('用户权限更新成功')
}

/**
 * 获取角色类型
 */
function getRoleType(role: string): string {
  const typeMap: { [key: string]: string } = {
    admin: 'danger',
    manager: 'warning',
    engineer: 'primary',
    operator: 'success',
    viewer: 'info',
  }
  return typeMap[role] || 'info'
}

/**
 * 获取角色名称
 */
function getRoleName(roleId: string): string {
  const role = availableRoles.value.find(r => r.id === roleId)
  return role ? role.name : roleId
}

/**
 * 获取部门名称
 */
function getDepartmentName(deptId: string): string {
  const dept = departments.value.find(d => d.id === deptId)
  return dept ? dept.name : deptId
}

/**
 * 获取状态类型
 */
function getStatusType(status: string): string {
  const typeMap: { [key: string]: string } = {
    active: 'success',
    inactive: 'warning',
    locked: 'danger',
  }
  return typeMap[status] || 'info'
}

/**
 * 获取状态标签
 */
function getStatusLabel(status: string): string {
  const labelMap: { [key: string]: string } = {
    active: '启用',
    inactive: '禁用',
    locked: '锁定',
  }
  return labelMap[status] || status
}

/**
 * 格式化日期时间
 */
function formatDateTime(dateStr: string): string {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN')
}

/**
 * 格式化日期
 */
function formatDate(dateStr: string): string {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN')
}

// ===== 生命周期 =====
onMounted(async () => {
  await initializeData()
})

// ===== 监听器 =====
watch(
  () => showCreateUser.value,
  show => {
    if (show) {
      createUser()
      showCreateUser.value = false
    }
  }
)
</script>

<style scoped lang="scss">
.users-page {
  padding: 24px;
  background: #f5f5f5;
  min-height: 100vh;

  .page-header {
    margin-bottom: 16px;

    .header-content {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;

      .title-section {
        .page-title {
          font-size: 28px;
          font-weight: 600;
          color: #303133;
          margin: 0 0 8px 0;
        }

        .page-description {
          font-size: 14px;
          color: #606266;
          margin: 0;
        }
      }

      .header-actions {
        display: flex;
        gap: 12px;
      }
    }
  }

  .filter-panel {
    margin-bottom: 16px;

    .filter-form {
      .filter-row {
        display: flex;
        align-items: flex-end;
        gap: 20px;
        flex-wrap: wrap;

        .filter-item {
          display: flex;
          flex-direction: column;

          .filter-label {
            font-size: 13px;
            color: #606266;
            margin-bottom: 6px;
            font-weight: 500;
          }
        }
      }
    }
  }

  .stats-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    margin-bottom: 16px;

    .stat-card {
      border: none;

      &.active {
        border-left: 4px solid #67c23a;
      }

      &.online {
        border-left: 4px solid #409eff;
      }

      &.locked {
        border-left: 4px solid #f56c6c;
      }

      .stat-content {
        display: flex;
        align-items: center;
        gap: 16px;

        .stat-icon {
          font-size: 32px;
          color: #409eff;
          background: #e1f3fe;
          width: 60px;
          height: 60px;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
        }

        .stat-info {
          .stat-number {
            font-size: 24px;
            font-weight: 600;
            color: #303133;
            margin-bottom: 4px;
          }

          .stat-label {
            font-size: 14px;
            color: #606266;
          }
        }
      }
    }
  }

  .users-table {
    .table-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .table-title {
        font-size: 16px;
        font-weight: 600;
        color: #303133;
      }

      .table-actions {
        display: flex;
        gap: 12px;
      }
    }

    .user-info {
      display: flex;
      align-items: center;
      gap: 12px;

      .user-details {
        .user-name {
          font-weight: 500;
          color: #303133;
          margin-bottom: 2px;
        }

        .user-username {
          font-size: 12px;
          color: #909399;
          margin-bottom: 2px;
        }

        .user-email {
          font-size: 12px;
          color: #606266;
        }
      }
    }

    .login-info {
      .login-time {
        color: #303133;
        margin-bottom: 2px;
      }

      .login-ip {
        font-size: 12px;
        color: #909399;
        font-family: monospace;
      }
    }

    .text-muted {
      color: #c0c4cc;
    }

    .table-actions {
      display: flex;
      gap: 8px;
    }

    .card-view {
      .user-cards {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
        gap: 16px;

        .user-card {
          border: 1px solid #e4e7ed;
          border-radius: 8px;
          padding: 20px;
          background: #fff;
          cursor: pointer;
          transition: all 0.3s;

          &:hover {
            border-color: #409eff;
            box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
          }

          .card-header {
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            margin-bottom: 16px;
          }

          .card-content {
            .user-name {
              font-size: 16px;
              font-weight: 600;
              color: #303133;
              margin-bottom: 4px;
            }

            .user-username {
              font-size: 13px;
              color: #909399;
              margin-bottom: 4px;
            }

            .user-email {
              font-size: 13px;
              color: #606266;
              margin-bottom: 16px;
            }

            .user-meta {
              .meta-item {
                display: flex;
                align-items: center;
                margin-bottom: 8px;
                font-size: 13px;

                &:last-child {
                  margin-bottom: 0;
                }

                .meta-label {
                  color: #606266;
                  margin-right: 8px;
                  min-width: 60px;
                }
              }
            }
          }

          .card-actions {
            margin-top: 16px;
            padding-top: 16px;
            border-top: 1px solid #f0f2f5;
            display: flex;
            gap: 12px;
          }
        }
      }
    }

    .pagination-wrapper {
      margin-top: 20px;
      text-align: center;
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .users-page {
    .filter-panel .filter-form .filter-row {
      flex-direction: column;
      align-items: stretch;
      gap: 16px;

      .filter-item {
        width: 100%;

        :deep(.el-select),
        :deep(.el-input) {
          width: 100% !important;
        }
      }
    }

    .stats-cards {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}

@media (max-width: 768px) {
  .users-page {
    padding: 16px;

    .page-header .header-content {
      flex-direction: column;
      gap: 16px;
      align-items: stretch;
    }

    .stats-cards {
      grid-template-columns: 1fr;
    }

    .users-table {
      .table-header {
        flex-direction: column;
        gap: 12px;
        align-items: stretch;
      }

      .card-view .user-cards {
        grid-template-columns: 1fr;
      }
    }
  }
}
</style>
