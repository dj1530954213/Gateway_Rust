<template>
  <div class="role-permission-manager">
    <!-- 页面头部 -->
    <div class="manager-header">
      <div class="header-content">
        <div class="title-section">
          <h2 class="manager-title">角色权限管理</h2>
          <p class="manager-description">管理系统角色和权限分配</p>
        </div>
        
        <div class="header-actions">
          <el-button type="primary" @click="showCreateRole = true">
            <el-icon><Plus /></el-icon>
            创建角色
          </el-button>
          <el-button @click="refreshRoles" :loading="loading">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </div>
      </div>
    </div>

    <!-- 主要内容区域 -->
    <div class="manager-content">
      <!-- 左侧角色列表 -->
      <div class="roles-panel">
        <el-card shadow="never" class="roles-card">
          <template #header>
            <div class="card-header">
              <span class="card-title">系统角色</span>
              <el-tag size="small">{{ roleList.length }} 个角色</el-tag>
            </div>
          </template>

          <div class="roles-list">
            <div
              v-for="role in roleList"
              :key="role.id"
              class="role-item"
              :class="{ active: selectedRole?.id === role.id }"
              @click="selectRole(role)"
            >
              <div class="role-info">
                <div class="role-header">
                  <div class="role-name">{{ role.name }}</div>
                  <el-dropdown @command="(cmd) => handleRoleAction(cmd, role)">
                    <el-button type="link" size="small" @click.stop>
                      <el-icon><MoreFilled /></el-icon>
                    </el-button>
                    <template #dropdown>
                      <el-dropdown-menu>
                        <el-dropdown-item command="edit">编辑角色</el-dropdown-item>
                        <el-dropdown-item command="copy">复制角色</el-dropdown-item>
                        <el-dropdown-item command="users">查看用户</el-dropdown-item>
                        <el-dropdown-item command="delete" divided>删除角色</el-dropdown-item>
                      </el-dropdown-menu>
                    </template>
                  </el-dropdown>
                </div>
                
                <div class="role-description">{{ role.description }}</div>
                
                <div class="role-meta">
                  <span class="meta-item">
                    <el-icon><User /></el-icon>
                    {{ role.userCount || 0 }} 个用户
                  </span>
                  <span class="meta-item">
                    <el-icon><Key /></el-icon>
                    {{ getPermissionCount(role) }} 个权限
                  </span>
                </div>
                
                <div class="role-status">
                  <el-tag :type="role.isBuiltIn ? 'warning' : 'success'" size="small">
                    {{ role.isBuiltIn ? '系统内置' : '自定义' }}
                  </el-tag>
                  <el-tag :type="role.status === 'active' ? 'success' : 'info'" size="small">
                    {{ role.status === 'active' ? '启用' : '禁用' }}
                  </el-tag>
                </div>
              </div>
            </div>
          </div>
        </el-card>
      </div>

      <!-- 右侧权限配置 -->
      <div class="permissions-panel">
        <el-card v-if="selectedRole" shadow="never" class="permissions-card">
          <template #header>
            <div class="card-header">
              <div class="permission-title">
                <span class="card-title">{{ selectedRole.name }} - 权限配置</span>
                <el-button type="link" @click="togglePermissionMode">
                  <el-icon><Switch /></el-icon>
                  {{ permissionMode === 'category' ? '切换到列表视图' : '切换到分类视图' }}
                </el-button>
              </div>
              <div class="permission-actions">
                <el-button @click="selectAllPermissions">全选</el-button>
                <el-button @click="clearAllPermissions">清空</el-button>
                <el-button type="primary" @click="savePermissions" :loading="savingPermissions">
                  保存权限
                </el-button>
              </div>
            </div>
          </template>

          <!-- 权限统计 -->
          <div class="permission-stats">
            <div class="stat-item">
              <span class="stat-label">已分配权限：</span>
              <span class="stat-value">{{ getGrantedPermissionCount() }}/{{ getAllPermissionCount() }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">权限覆盖率：</span>
              <span class="stat-value">{{ getPermissionCoverageRate() }}%</span>
            </div>
          </div>

          <!-- 分类视图 -->
          <div v-if="permissionMode === 'category'" class="permission-categories">
            <div
              v-for="category in permissionCategories"
              :key="category.id"
              class="permission-category"
            >
              <div class="category-header">
                <div class="category-info">
                  <el-icon class="category-icon">
                    <component :is="category.icon" />
                  </el-icon>
                  <span class="category-name">{{ category.name }}</span>
                  <el-tag size="small">{{ category.permissions.length }} 个权限</el-tag>
                </div>
                
                <div class="category-actions">
                  <el-checkbox
                    :model-value="isCategoryAllSelected(category)"
                    :indeterminate="isCategoryIndeterminate(category)"
                    @change="toggleCategoryPermissions(category, $event)"
                  >
                    全选
                  </el-checkbox>
                </div>
              </div>
              
              <div class="category-description">{{ category.description }}</div>
              
              <div class="category-permissions">
                <div
                  v-for="permission in category.permissions"
                  :key="permission.id"
                  class="permission-item"
                >
                  <el-checkbox
                    :model-value="isPermissionGranted(permission.id)"
                    @change="togglePermission(permission.id, $event)"
                  >
                    <div class="permission-info">
                      <div class="permission-name">{{ permission.name }}</div>
                      <div class="permission-description">{{ permission.description }}</div>
                    </div>
                  </el-checkbox>
                </div>
              </div>
            </div>
          </div>

          <!-- 列表视图 -->
          <div v-else class="permission-list">
            <div class="list-search">
              <el-input
                v-model="permissionSearchKeyword"
                placeholder="搜索权限..."
                clearable
                style="width: 300px"
              >
                <template #prefix>
                  <el-icon><Search /></el-icon>
                </template>
              </el-input>
            </div>

            <el-table
              :data="filteredPermissions"
              :loading="loadingPermissions"
              row-key="id"
              @selection-change="handlePermissionSelectionChange"
            >
              <el-table-column type="selection" width="55" />
              
              <el-table-column label="权限名称" min-width="200">
                <template #default="{ row }">
                  <div class="permission-info">
                    <div class="permission-name">{{ row.name }}</div>
                    <div class="permission-code">{{ row.code }}</div>
                  </div>
                </template>
              </el-table-column>

              <el-table-column label="权限描述" min-width="300">
                <template #default="{ row }">
                  {{ row.description }}
                </template>
              </el-table-column>

              <el-table-column label="分类" width="120">
                <template #default="{ row }">
                  <el-tag size="small">{{ getCategoryName(row.categoryId) }}</el-tag>
                </template>
              </el-table-column>

              <el-table-column label="风险级别" width="100">
                <template #default="{ row }">
                  <el-tag :type="getRiskLevelType(row.riskLevel)" size="small">
                    {{ getRiskLevelLabel(row.riskLevel) }}
                  </el-tag>
                </template>
              </el-table-column>

              <el-table-column label="状态" width="100">
                <template #default="{ row }">
                  <el-switch
                    :model-value="isPermissionGranted(row.id)"
                    @change="togglePermission(row.id, $event)"
                  />
                </template>
              </el-table-column>
            </el-table>
          </div>
        </el-card>

        <!-- 未选择角色时的占位符 -->
        <el-card v-else shadow="never" class="placeholder-card">
          <el-empty
            description="请从左侧选择一个角色来配置权限"
            :image-size="120"
          >
            <el-button type="primary" @click="showCreateRole = true">
              创建新角色
            </el-button>
          </el-empty>
        </el-card>
      </div>
    </div>

    <!-- 角色编辑对话框 -->
    <RoleEditDialog
      v-model:visible="showEditRole"
      :role-data="editingRole"
      :is-create="isCreateMode"
      @save="handleRoleSave"
    />

    <!-- 用户分配对话框 -->
    <RoleUsersDialog
      v-model:visible="showRoleUsers"
      :role-data="selectedRole"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * RolePermissionManager —— 角色权限管理组件
 *
 * 📝 Responsibilities:
 *  1. 角色列表展示和管理
 *  2. 权限分类和配置界面
 *  3. 权限分配和撤销操作
 *  4. 角色权限批量操作
 *  5. 权限覆盖率统计
 *
 * 📦 Dependencies:
 *  - RoleEditDialog 角色编辑对话框
 *  - RoleUsersDialog 角色用户对话框
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, onMounted, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Plus,
  Refresh,
  MoreFilled,
  User,
  Key,
  Switch,
  Search,
  Monitor,
  Setting,
  Document,
  Connection,
  Bell,
  Shield,
  Lock,
  View,
  Edit,
  Delete
} from '@element-plus/icons-vue'

// 组件导入（这些组件将在后续步骤中创建）
import RoleEditDialog from './RoleEditDialog.vue'
import RoleUsersDialog from './RoleUsersDialog.vue'

// ===== 响应式数据 =====
const loading = ref(false)
const loadingPermissions = ref(false)
const savingPermissions = ref(false)
const permissionMode = ref<'category' | 'list'>('category')
const permissionSearchKeyword = ref('')

// 对话框状态
const showCreateRole = ref(false)
const showEditRole = ref(false)
const showRoleUsers = ref(false)

// 选中的角色和编辑状态
const selectedRole = ref<any>(null)
const editingRole = ref<any>(null)
const isCreateMode = ref(false)

// 数据
const roleList = ref<any[]>([])
const permissionCategories = ref<any[]>([])
const allPermissions = ref<any[]>([])
const rolePermissions = ref<string[]>([]) // 当前角色的权限ID列表

// ===== 计算属性 =====
const filteredPermissions = computed(() => {
  if (!permissionSearchKeyword.value) {
    return allPermissions.value
  }
  
  const keyword = permissionSearchKeyword.value.toLowerCase()
  return allPermissions.value.filter(permission => 
    permission.name.toLowerCase().includes(keyword) ||
    permission.description.toLowerCase().includes(keyword) ||
    permission.code.toLowerCase().includes(keyword)
  )
})

// ===== 方法 =====

/**
 * 初始化数据
 */
async function initializeData() {
  try {
    loading.value = true
    
    // 生成模拟数据
    roleList.value = generateMockRoles()
    permissionCategories.value = generateMockPermissionCategories()
    allPermissions.value = generateMockPermissions()
    
    // 选择第一个角色
    if (roleList.value.length > 0) {
      selectRole(roleList.value[0])
    }

  } catch (error) {
    console.error('初始化角色权限管理失败:', error)
    ElMessage.error('初始化失败')
  } finally {
    loading.value = false
  }
}

/**
 * 生成模拟角色数据
 */
function generateMockRoles() {
  return [
    {
      id: 'admin',
      name: '系统管理员',
      description: '拥有系统的最高权限，可以管理所有功能和用户',
      status: 'active',
      isBuiltIn: true,
      userCount: 2,
      createdAt: '2025-01-01T00:00:00Z',
      updatedAt: '2025-07-27T10:00:00Z'
    },
    {
      id: 'operator',
      name: '操作员',
      description: '负责日常设备操作和监控，具有设备控制权限',
      status: 'active',
      isBuiltIn: true,
      userCount: 8,
      createdAt: '2025-01-01T00:00:00Z',
      updatedAt: '2025-07-27T10:00:00Z'
    },
    {
      id: 'viewer',
      name: '观察员',
      description: '只能查看数据和报表，无法进行修改操作',
      status: 'active',
      isBuiltIn: true,
      userCount: 15,
      createdAt: '2025-01-01T00:00:00Z',
      updatedAt: '2025-07-27T10:00:00Z'
    },
    {
      id: 'engineer',
      name: '工程师',
      description: '负责系统配置和工程项目管理',
      status: 'active',
      isBuiltIn: false,
      userCount: 5,
      createdAt: '2025-02-15T00:00:00Z',
      updatedAt: '2025-07-27T10:00:00Z'
    },
    {
      id: 'manager',
      name: '部门经理',
      description: '负责部门用户管理和审批流程',
      status: 'active',
      isBuiltIn: false,
      userCount: 3,
      createdAt: '2025-03-01T00:00:00Z',
      updatedAt: '2025-07-27T10:00:00Z'
    }
  ]
}

/**
 * 生成模拟权限分类
 */
function generateMockPermissionCategories() {
  return [
    {
      id: 'device',
      name: '设备管理',
      description: '设备相关的所有操作权限',
      icon: 'Monitor',
      permissions: []
    },
    {
      id: 'system',
      name: '系统管理',
      description: '系统配置和用户管理权限',
      icon: 'Setting',
      permissions: []
    },
    {
      id: 'data',
      name: '数据管理',
      description: '数据查询、导出和分析权限',
      icon: 'Document',
      permissions: []
    },
    {
      id: 'network',
      name: '网络管理',
      description: '网络配置和连接管理权限',
      icon: 'Connection',
      permissions: []
    },
    {
      id: 'alert',
      name: '报警管理',
      description: '报警规则和通知管理权限',
      icon: 'Bell',
      permissions: []
    },
    {
      id: 'security',
      name: '安全管理',
      description: '安全策略和访问控制权限',
      icon: 'Shield',
      permissions: []
    }
  ]
}

/**
 * 生成模拟权限数据
 */
function generateMockPermissions() {
  const permissions = [
    // 设备管理权限
    { id: 'device_view', name: '查看设备', code: 'device:view', description: '查看设备列表和详细信息', categoryId: 'device', riskLevel: 'low' },
    { id: 'device_create', name: '创建设备', code: 'device:create', description: '添加新设备到系统', categoryId: 'device', riskLevel: 'medium' },
    { id: 'device_update', name: '编辑设备', code: 'device:update', description: '修改设备配置和参数', categoryId: 'device', riskLevel: 'medium' },
    { id: 'device_delete', name: '删除设备', code: 'device:delete', description: '从系统中删除设备', categoryId: 'device', riskLevel: 'high' },
    { id: 'device_control', name: '控制设备', code: 'device:control', description: '远程控制设备运行状态', categoryId: 'device', riskLevel: 'high' },

    // 系统管理权限
    { id: 'system_config', name: '系统配置', code: 'system:config', description: '修改系统配置参数', categoryId: 'system', riskLevel: 'high' },
    { id: 'user_view', name: '查看用户', code: 'user:view', description: '查看用户列表和信息', categoryId: 'system', riskLevel: 'low' },
    { id: 'user_create', name: '创建用户', code: 'user:create', description: '创建新用户账户', categoryId: 'system', riskLevel: 'medium' },
    { id: 'user_update', name: '编辑用户', code: 'user:update', description: '修改用户信息和权限', categoryId: 'system', riskLevel: 'medium' },
    { id: 'user_delete', name: '删除用户', code: 'user:delete', description: '删除用户账户', categoryId: 'system', riskLevel: 'high' },
    { id: 'role_manage', name: '角色管理', code: 'role:manage', description: '管理系统角色和权限', categoryId: 'system', riskLevel: 'high' },

    // 数据管理权限
    { id: 'data_view', name: '查看数据', code: 'data:view', description: '查看历史数据和报表', categoryId: 'data', riskLevel: 'low' },
    { id: 'data_export', name: '导出数据', code: 'data:export', description: '导出数据到文件', categoryId: 'data', riskLevel: 'medium' },
    { id: 'data_import', name: '导入数据', code: 'data:import', description: '从文件导入数据', categoryId: 'data', riskLevel: 'medium' },
    { id: 'data_delete', name: '删除数据', code: 'data:delete', description: '删除历史数据记录', categoryId: 'data', riskLevel: 'high' },

    // 网络管理权限
    { id: 'network_view', name: '查看网络', code: 'network:view', description: '查看网络连接状态', categoryId: 'network', riskLevel: 'low' },
    { id: 'network_config', name: '网络配置', code: 'network:config', description: '配置网络连接参数', categoryId: 'network', riskLevel: 'high' },
    { id: 'protocol_manage', name: '协议管理', code: 'protocol:manage', description: '管理通信协议配置', categoryId: 'network', riskLevel: 'medium' },

    // 报警管理权限
    { id: 'alert_view', name: '查看报警', code: 'alert:view', description: '查看报警记录和状态', categoryId: 'alert', riskLevel: 'low' },
    { id: 'alert_config', name: '配置报警', code: 'alert:config', description: '配置报警规则和阈值', categoryId: 'alert', riskLevel: 'medium' },
    { id: 'alert_handle', name: '处理报警', code: 'alert:handle', description: '确认和处理报警事件', categoryId: 'alert', riskLevel: 'medium' },
    { id: 'notification_manage', name: '通知管理', code: 'notification:manage', description: '管理报警通知设置', categoryId: 'alert', riskLevel: 'medium' },

    // 安全管理权限
    { id: 'security_view', name: '查看安全日志', code: 'security:view', description: '查看系统安全日志', categoryId: 'security', riskLevel: 'medium' },
    { id: 'security_config', name: '安全配置', code: 'security:config', description: '配置安全策略和规则', categoryId: 'security', riskLevel: 'high' },
    { id: 'audit_manage', name: '审计管理', code: 'audit:manage', description: '管理系统审计功能', categoryId: 'security', riskLevel: 'high' },
    { id: 'backup_manage', name: '备份管理', code: 'backup:manage', description: '管理系统备份和恢复', categoryId: 'security', riskLevel: 'high' }
  ]

  // 将权限分配到对应的分类中
  permissionCategories.value.forEach(category => {
    category.permissions = permissions.filter(p => p.categoryId === category.id)
  })

  return permissions
}

/**
 * 选择角色
 */
async function selectRole(role: any) {
  selectedRole.value = role
  await loadRolePermissions(role.id)
}

/**
 * 加载角色权限
 */
async function loadRolePermissions(roleId: string) {
  try {
    loadingPermissions.value = true
    
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 500))
    
    // 根据角色生成对应的权限
    const rolePermissionMap: { [key: string]: string[] } = {
      admin: allPermissions.value.map(p => p.id), // 管理员拥有所有权限
      operator: [
        'device_view', 'device_create', 'device_update', 'device_control',
        'data_view', 'data_export',
        'alert_view', 'alert_handle',
        'network_view'
      ],
      viewer: [
        'device_view',
        'data_view',
        'alert_view',
        'network_view'
      ],
      engineer: [
        'device_view', 'device_create', 'device_update',
        'data_view', 'data_export', 'data_import',
        'network_view', 'network_config', 'protocol_manage',
        'alert_view', 'alert_config'
      ],
      manager: [
        'device_view',
        'user_view', 'user_create', 'user_update',
        'data_view', 'data_export',
        'alert_view', 'alert_config', 'notification_manage'
      ]
    }
    
    rolePermissions.value = rolePermissionMap[roleId] || []
    
  } catch (error) {
    console.error('加载角色权限失败:', error)
    ElMessage.error('加载角色权限失败')
  } finally {
    loadingPermissions.value = false
  }
}

/**
 * 刷新角色列表
 */
function refreshRoles() {
  initializeData()
}

/**
 * 获取权限数量
 */
function getPermissionCount(role: any): number {
  // 这里应该从实际的权限分配中计算
  const permissionCounts: { [key: string]: number } = {
    admin: allPermissions.value.length,
    operator: 9,
    viewer: 4,
    engineer: 10,
    manager: 9
  }
  return permissionCounts[role.id] || 0
}

/**
 * 获取已分配权限数量
 */
function getGrantedPermissionCount(): number {
  return rolePermissions.value.length
}

/**
 * 获取所有权限数量
 */
function getAllPermissionCount(): number {
  return allPermissions.value.length
}

/**
 * 获取权限覆盖率
 */
function getPermissionCoverageRate(): number {
  if (allPermissions.value.length === 0) return 0
  return Math.round((rolePermissions.value.length / allPermissions.value.length) * 100)
}

/**
 * 检查权限是否已分配
 */
function isPermissionGranted(permissionId: string): boolean {
  return rolePermissions.value.includes(permissionId)
}

/**
 * 切换单个权限
 */
function togglePermission(permissionId: string, granted: boolean) {
  if (granted) {
    if (!rolePermissions.value.includes(permissionId)) {
      rolePermissions.value.push(permissionId)
    }
  } else {
    const index = rolePermissions.value.indexOf(permissionId)
    if (index > -1) {
      rolePermissions.value.splice(index, 1)
    }
  }
}

/**
 * 检查分类是否全选
 */
function isCategoryAllSelected(category: any): boolean {
  return category.permissions.every((p: any) => isPermissionGranted(p.id))
}

/**
 * 检查分类是否部分选中
 */
function isCategoryIndeterminate(category: any): boolean {
  const granted = category.permissions.filter((p: any) => isPermissionGranted(p.id))
  return granted.length > 0 && granted.length < category.permissions.length
}

/**
 * 切换分类权限
 */
function toggleCategoryPermissions(category: any, selected: boolean) {
  category.permissions.forEach((permission: any) => {
    togglePermission(permission.id, selected)
  })
}

/**
 * 全选权限
 */
function selectAllPermissions() {
  rolePermissions.value = allPermissions.value.map(p => p.id)
}

/**
 * 清空权限
 */
function clearAllPermissions() {
  rolePermissions.value = []
}

/**
 * 切换权限模式
 */
function togglePermissionMode() {
  permissionMode.value = permissionMode.value === 'category' ? 'list' : 'category'
}

/**
 * 处理权限选择变化（列表模式）
 */
function handlePermissionSelectionChange(selection: any[]) {
  rolePermissions.value = selection.map(p => p.id)
}

/**
 * 保存权限
 */
async function savePermissions() {
  if (!selectedRole.value) return
  
  try {
    savingPermissions.value = true
    
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    ElMessage.success('权限保存成功')
    
  } catch (error) {
    console.error('保存权限失败:', error)
    ElMessage.error('保存权限失败')
  } finally {
    savingPermissions.value = false
  }
}

/**
 * 处理角色操作
 */
async function handleRoleAction(command: string, role: any) {
  switch (command) {
    case 'edit':
      editRole(role)
      break
    case 'copy':
      await copyRole(role)
      break
    case 'users':
      showRoleUsers.value = true
      break
    case 'delete':
      await deleteRole(role)
      break
  }
}

/**
 * 编辑角色
 */
function editRole(role: any) {
  editingRole.value = role
  isCreateMode.value = false
  showEditRole.value = true
}

/**
 * 复制角色
 */
async function copyRole(role: any) {
  try {
    const copiedRole = {
      ...role,
      id: `${role.id}_copy_${Date.now()}`,
      name: `${role.name} - 副本`,
      isBuiltIn: false
    }
    
    editingRole.value = copiedRole
    isCreateMode.value = true
    showEditRole.value = true
    
  } catch (error) {
    ElMessage.error('复制角色失败')
  }
}

/**
 * 删除角色
 */
async function deleteRole(role: any) {
  if (role.isBuiltIn) {
    ElMessage.warning('系统内置角色不能删除')
    return
  }
  
  try {
    await ElMessageBox.confirm(
      `确定要删除角色 "${role.name}" 吗？此操作不可恢复。`,
      '确认删除',
      {
        type: 'error'
      }
    )
    
    // 模拟删除操作
    await new Promise(resolve => setTimeout(resolve, 500))
    
    ElMessage.success('角色删除成功')
    await refreshRoles()
    
  } catch (error) {
    // 用户取消删除
  }
}

/**
 * 处理角色保存
 */
function handleRoleSave(roleData: any) {
  if (isCreateMode.value) {
    ElMessage.success('角色创建成功')
  } else {
    ElMessage.success('角色更新成功')
  }
  refreshRoles()
}

/**
 * 获取分类名称
 */
function getCategoryName(categoryId: string): string {
  const category = permissionCategories.value.find(c => c.id === categoryId)
  return category ? category.name : categoryId
}

/**
 * 获取风险级别类型
 */
function getRiskLevelType(riskLevel: string): string {
  const typeMap: { [key: string]: string } = {
    low: 'success',
    medium: 'warning',
    high: 'danger'
  }
  return typeMap[riskLevel] || 'info'
}

/**
 * 获取风险级别标签
 */
function getRiskLevelLabel(riskLevel: string): string {
  const labelMap: { [key: string]: string } = {
    low: '低风险',
    medium: '中风险',
    high: '高风险'
  }
  return labelMap[riskLevel] || riskLevel
}

// ===== 生命周期 =====
onMounted(async () => {
  await initializeData()
})

// ===== 监听器 =====
watch(() => showCreateRole.value, (show) => {
  if (show) {
    editingRole.value = null
    isCreateMode.value = true
    showEditRole.value = true
    showCreateRole.value = false
  }
})
</script>

<style scoped lang="scss">
.role-permission-manager {
  .manager-header {
    margin-bottom: 16px;
    
    .header-content {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      
      .title-section {
        .manager-title {
          font-size: 24px;
          font-weight: 600;
          color: #303133;
          margin: 0 0 8px 0;
        }
        
        .manager-description {
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

  .manager-content {
    display: grid;
    grid-template-columns: 400px 1fr;
    gap: 16px;
    min-height: 600px;

    .roles-panel {
      .roles-card {
        height: fit-content;
        
        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          
          .card-title {
            font-size: 16px;
            font-weight: 600;
            color: #303133;
          }
        }
        
        .roles-list {
          .role-item {
            padding: 16px;
            border: 1px solid #e4e7ed;
            border-radius: 8px;
            margin-bottom: 12px;
            cursor: pointer;
            transition: all 0.3s;
            
            &:last-child {
              margin-bottom: 0;
            }
            
            &:hover {
              border-color: #409eff;
              background: #f0f9ff;
            }
            
            &.active {
              border-color: #409eff;
              background: #e1f3fe;
            }
            
            .role-info {
              .role-header {
                display: flex;
                justify-content: space-between;
                align-items: flex-start;
                margin-bottom: 8px;
                
                .role-name {
                  font-size: 16px;
                  font-weight: 600;
                  color: #303133;
                }
              }
              
              .role-description {
                font-size: 13px;
                color: #606266;
                line-height: 1.5;
                margin-bottom: 12px;
              }
              
              .role-meta {
                display: flex;
                gap: 16px;
                margin-bottom: 12px;
                
                .meta-item {
                  display: flex;
                  align-items: center;
                  gap: 4px;
                  font-size: 12px;
                  color: #909399;
                  
                  .el-icon {
                    font-size: 14px;
                  }
                }
              }
              
              .role-status {
                display: flex;
                gap: 8px;
              }
            }
          }
        }
      }
    }

    .permissions-panel {
      .permissions-card {
        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          
          .permission-title {
            display: flex;
            align-items: center;
            gap: 12px;
            
            .card-title {
              font-size: 16px;
              font-weight: 600;
              color: #303133;
            }
          }
          
          .permission-actions {
            display: flex;
            gap: 12px;
          }
        }
        
        .permission-stats {
          display: flex;
          gap: 24px;
          margin-bottom: 24px;
          padding: 16px;
          background: #f8f9fa;
          border-radius: 6px;
          
          .stat-item {
            .stat-label {
              font-size: 13px;
              color: #606266;
            }
            
            .stat-value {
              font-size: 16px;
              font-weight: 600;
              color: #303133;
              margin-left: 8px;
            }
          }
        }
        
        .permission-categories {
          .permission-category {
            margin-bottom: 32px;
            
            &:last-child {
              margin-bottom: 0;
            }
            
            .category-header {
              display: flex;
              justify-content: space-between;
              align-items: center;
              margin-bottom: 12px;
              
              .category-info {
                display: flex;
                align-items: center;
                gap: 12px;
                
                .category-icon {
                  font-size: 18px;
                  color: #409eff;
                }
                
                .category-name {
                  font-size: 16px;
                  font-weight: 600;
                  color: #303133;
                }
              }
            }
            
            .category-description {
              font-size: 13px;
              color: #606266;
              margin-bottom: 16px;
              line-height: 1.5;
            }
            
            .category-permissions {
              display: grid;
              grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
              gap: 12px;
              
              .permission-item {
                padding: 12px;
                border: 1px solid #e4e7ed;
                border-radius: 6px;
                background: #fff;
                
                .permission-info {
                  .permission-name {
                    font-size: 14px;
                    font-weight: 500;
                    color: #303133;
                    margin-bottom: 4px;
                  }
                  
                  .permission-description {
                    font-size: 12px;
                    color: #606266;
                    line-height: 1.4;
                  }
                }
              }
            }
          }
        }
        
        .permission-list {
          .list-search {
            margin-bottom: 16px;
          }
          
          .permission-info {
            .permission-name {
              font-weight: 500;
              color: #303133;
              margin-bottom: 2px;
            }
            
            .permission-code {
              font-size: 12px;
              color: #909399;
              font-family: monospace;
            }
          }
        }
      }
      
      .placeholder-card {
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 400px;
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .role-permission-manager {
    .manager-content {
      grid-template-columns: 1fr;
      
      .roles-panel {
        .roles-list .role-item .role-info .role-meta {
          flex-direction: column;
          gap: 8px;
        }
      }
      
      .permissions-panel .permissions-card .permission-categories .permission-category .category-permissions {
        grid-template-columns: 1fr;
      }
    }
  }
}

@media (max-width: 768px) {
  .role-permission-manager {
    .manager-header .header-content {
      flex-direction: column;
      gap: 16px;
      align-items: stretch;
    }
    
    .manager-content .permissions-panel .permissions-card {
      .card-header {
        flex-direction: column;
        gap: 12px;
        align-items: stretch;
        
        .permission-actions {
          justify-content: center;
        }
      }
      
      .permission-stats {
        flex-direction: column;
        gap: 12px;
      }
    }
  }
}
</style>