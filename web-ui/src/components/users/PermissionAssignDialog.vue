<template>
  <el-dialog
    v-model="dialogVisible"
    title="权限分配"
    width="900px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @close="handleClose"
  >
    <div class="permission-assign-dialog">
      <!-- 用户信息头部 -->
      <div class="user-info-header">
        <div class="user-avatar">
          <el-avatar :size="50" :src="userData?.avatar">
            <el-icon><UserFilled /></el-icon>
          </el-avatar>
        </div>
        <div class="user-details">
          <h3 class="user-name">{{ userData?.name }}</h3>
          <div class="user-meta">
            <el-tag type="info" size="small">{{ userData?.username }}</el-tag>
            <el-tag :type="getRoleType(userData?.role)" size="small">
              {{ getRoleName(userData?.role) }}
            </el-tag>
            <el-tag type="warning" size="small">{{ getDepartmentName(userData?.departmentId) }}</el-tag>
          </div>
        </div>
      </div>

      <el-divider />

      <!-- 权限分配主体 -->
      <div class="permission-content">
        <!-- 左侧权限分类 -->
        <div class="permission-categories">
          <div class="category-header">
            <h4>权限分类</h4>
            <div class="category-actions">
              <el-button 
                size="small" 
                @click="expandAllCategories"
                :icon="Expand"
              >
                展开全部
              </el-button>
              <el-button 
                size="small" 
                @click="collapseAllCategories"
                :icon="Fold"
              >
                收起全部
              </el-button>
            </div>
          </div>
          
          <div class="category-list">
            <div
              v-for="category in permissionCategories"
              :key="category.id"
              :class="['category-item', { active: selectedCategoryId === category.id }]"
              @click="selectCategory(category.id)"
            >
              <div class="category-main">
                <el-icon class="category-icon">
                  <component :is="category.icon" />
                </el-icon>
                <span class="category-name">{{ category.name }}</span>
                <div class="category-stats">
                  <span class="granted-count">{{ getGrantedCount(category.id) }}</span>
                  <span class="total-count">/{{ category.permissions.length }}</span>
                </div>
              </div>
              <div class="category-progress">
                <el-progress
                  :percentage="getPermissionProgress(category.id)"
                  :stroke-width="3"
                  :show-text="false"
                  :color="getProgressColor(category.id)"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧权限详情 -->
        <div class="permission-details">
          <div class="details-header">
            <h4>{{ getSelectedCategoryName() }} 权限</h4>
            <div class="details-actions">
              <el-button 
                size="small" 
                @click="grantAllInCategory"
                :icon="Check"
                type="success"
              >
                全部授予
              </el-button>
              <el-button 
                size="small" 
                @click="revokeAllInCategory"
                :icon="Close"
                type="danger"
              >
                全部撤销
              </el-button>
            </div>
          </div>

          <div class="permission-grid">
            <div
              v-for="permission in getSelectedCategoryPermissions()"
              :key="permission.id"
              class="permission-item"
            >
              <div class="permission-main">
                <div class="permission-info">
                  <div class="permission-header">
                    <span class="permission-name">{{ permission.name }}</span>
                    <el-tag
                      :type="permission.level === 'high' ? 'danger' : permission.level === 'medium' ? 'warning' : 'info'"
                      size="small"
                      class="permission-level"
                    >
                      {{ getLevelText(permission.level) }}
                    </el-tag>
                  </div>
                  <div class="permission-desc">{{ permission.description }}</div>
                  <div class="permission-source">
                    <span v-if="permission.fromRole" class="source-tag role-source">
                      角色权限
                    </span>
                    <span v-if="permission.fromUser" class="source-tag user-source">
                      个人权限
                    </span>
                  </div>
                </div>
                <div class="permission-control">
                  <el-switch
                    v-model="permission.granted"
                    :disabled="permission.fromRole && !permission.canOverride"
                    @change="handlePermissionChange(permission)"
                    :active-color="permission.level === 'high' ? '#F56C6C' : '#67C23A'"
                  />
                </div>
              </div>
              
              <!-- 权限依赖 -->
              <div v-if="permission.dependencies?.length" class="permission-dependencies">
                <div class="dependencies-label">依赖权限:</div>
                <div class="dependencies-list">
                  <el-tag
                    v-for="dep in permission.dependencies"
                    :key="dep"
                    size="small"
                    type="info"
                    class="dependency-tag"
                  >
                    {{ getPermissionName(dep) }}
                  </el-tag>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 权限变更预览 -->
      <div v-if="hasChanges" class="changes-preview">
        <el-divider />
        <div class="changes-header">
          <h4>权限变更预览</h4>
          <el-tag :type="changeCount > 0 ? 'warning' : 'info'">
            {{ changeCount }} 项变更
          </el-tag>
        </div>
        <div class="changes-list">
          <div
            v-for="change in permissionChanges"
            :key="change.permissionId"
            :class="['change-item', change.action]"
          >
            <el-icon class="change-icon">
              <Plus v-if="change.action === 'grant'" />
              <Minus v-if="change.action === 'revoke'" />
            </el-icon>
            <span class="change-text">
              {{ change.action === 'grant' ? '授予' : '撤销' }} {{ change.permissionName }}
            </span>
            <el-tag
              :type="change.action === 'grant' ? 'success' : 'danger'"
              size="small"
              class="change-tag"
            >
              {{ change.action === 'grant' ? '新增' : '移除' }}
            </el-tag>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button @click="resetChanges" :disabled="!hasChanges">重置</el-button>
        <el-button
          type="primary"
          @click="savePermissions"
          :loading="saving"
          :disabled="!hasChanges"
        >
          保存权限
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * PermissionAssignDialog —— 权限分配对话框组件
 *
 * 📝 Responsibilities:
 *  1. 个人权限的精细化分配和管理
 *  2. 角色权限的查看和覆盖控制
 *  3. 权限变更的预览和批量操作
 *  4. 权限依赖关系的展示和验证
 *  5. 权限分类的导航和过滤
 *
 * 📦 Dependencies:
 *  - Element Plus 组件库
 *  - 权限管理相关API
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, watch, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  UserFilled,
  Monitor,
  Setting,
  Document,
  Connection,
  Bell,
  Expand,
  Fold,
  Check,
  Close,
  Plus,
  Minus
} from '@element-plus/icons-vue'

// ===== Props & Emits =====
const props = defineProps<{
  visible: boolean
  userData?: any
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  'save': [permissionData: any]
}>()

// ===== 响应式数据 =====
const dialogVisible = ref(props.visible)
const saving = ref(false)
const selectedCategoryId = ref('device')

// 权限数据
const userPermissions = ref<any[]>([])
const originalPermissions = ref<any[]>([])
const availableRoles = ref<any[]>([])
const departments = ref<any[]>([])

// ===== 权限分类配置 =====
const permissionCategories = ref([
  {
    id: 'device',
    name: '设备管理',
    icon: 'Monitor',
    permissions: [
      {
        id: 'device_read',
        name: '设备查看',
        description: '查看设备信息、状态和配置',
        level: 'low',
        granted: false,
        fromRole: true,
        fromUser: false,
        canOverride: false,
        dependencies: []
      },
      {
        id: 'device_write',
        name: '设备编辑',
        description: '修改设备配置和参数',
        level: 'medium',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true,
        dependencies: ['device_read']
      },
      {
        id: 'device_control',
        name: '设备控制',
        description: '启动、停止、重启设备',
        level: 'high',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true,
        dependencies: ['device_read', 'device_write']
      },
      {
        id: 'device_delete',
        name: '设备删除',
        description: '删除设备及其相关数据',
        level: 'high',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true,
        dependencies: ['device_read', 'device_write']
      }
    ]
  },
  {
    id: 'system',
    name: '系统设置',
    icon: 'Setting',
    permissions: [
      {
        id: 'system_config',
        name: '系统配置',
        description: '修改系统全局配置',
        level: 'high',
        granted: false,
        fromRole: true,
        fromUser: false,
        canOverride: false
      },
      {
        id: 'user_manage',
        name: '用户管理',
        description: '创建、编辑、删除用户',
        level: 'high',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true
      },
      {
        id: 'role_manage',
        name: '角色管理',
        description: '创建、编辑、删除角色',
        level: 'high',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true
      },
      {
        id: 'backup_restore',
        name: '备份恢复',
        description: '系统数据备份和恢复',
        level: 'medium',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true
      }
    ]
  },
  {
    id: 'data',
    name: '数据管理',
    icon: 'Document',
    permissions: [
      {
        id: 'data_read',
        name: '数据查看',
        description: '查看历史数据和实时数据',
        level: 'low',
        granted: false,
        fromRole: true,
        fromUser: false,
        canOverride: false
      },
      {
        id: 'data_export',
        name: '数据导出',
        description: '导出数据到文件',
        level: 'medium',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true,
        dependencies: ['data_read']
      },
      {
        id: 'data_import',
        name: '数据导入',
        description: '从文件导入数据',
        level: 'medium',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true
      },
      {
        id: 'data_delete',
        name: '数据删除',
        description: '删除历史数据',
        level: 'high',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true,
        dependencies: ['data_read']
      }
    ]
  },
  {
    id: 'network',
    name: '网络连接',
    icon: 'Connection',
    permissions: [
      {
        id: 'network_config',
        name: '网络配置',
        description: '配置网络连接参数',
        level: 'medium',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true
      },
      {
        id: 'protocol_manage',
        name: '协议管理',
        description: '管理通讯协议配置',
        level: 'medium',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true
      },
      {
        id: 'driver_manage',
        name: '驱动管理',
        description: '安装、更新、删除驱动',
        level: 'high',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true
      }
    ]
  },
  {
    id: 'alert',
    name: '报警管理',
    icon: 'Bell',
    permissions: [
      {
        id: 'alert_read',
        name: '报警查看',
        description: '查看报警信息和历史',
        level: 'low',
        granted: false,
        fromRole: true,
        fromUser: false,
        canOverride: false
      },
      {
        id: 'alert_ack',
        name: '报警确认',
        description: '确认和处理报警',
        level: 'medium',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true,
        dependencies: ['alert_read']
      },
      {
        id: 'alert_config',
        name: '报警配置',
        description: '配置报警规则和阈值',
        level: 'medium',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true
      },
      {
        id: 'alert_delete',
        name: '报警删除',
        description: '删除报警记录',
        level: 'high',
        granted: false,
        fromRole: false,
        fromUser: false,
        canOverride: true,
        dependencies: ['alert_read']
      }
    ]
  }
])

// ===== 计算属性 =====
const hasChanges = computed(() => {
  return permissionChanges.value.length > 0
})

const changeCount = computed(() => {
  return permissionChanges.value.length
})

const permissionChanges = computed(() => {
  const changes: any[] = []
  
  permissionCategories.value.forEach(category => {
    category.permissions.forEach(permission => {
      const original = originalPermissions.value.find(p => p.id === permission.id)
      if (original && original.granted !== permission.granted) {
        changes.push({
          permissionId: permission.id,
          permissionName: permission.name,
          action: permission.granted ? 'grant' : 'revoke'
        })
      }
    })
  })
  
  return changes
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
    
    // 加载用户权限
    loadUserPermissions()
    
  } catch (error) {
    console.error('初始化权限分配对话框失败:', error)
    ElMessage.error('初始化失败')
  }
}

/**
 * 加载用户权限
 */
function loadUserPermissions() {
  // 模拟从用户角色和个人权限加载数据
  const userRole = availableRoles.value.find(r => r.id === props.userData?.role)
  const rolePermissions = userRole?.permissions || []
  
  // 更新权限状态
  permissionCategories.value.forEach(category => {
    category.permissions.forEach(permission => {
      const rolePermission = rolePermissions.find((p: any) => p.id === permission.id)
      
      if (rolePermission) {
        permission.granted = rolePermission.granted
        permission.fromRole = true
        permission.canOverride = rolePermission.canOverride !== false
      } else {
        permission.granted = false
        permission.fromRole = false
        permission.canOverride = true
      }
      
      // 模拟一些个人权限
      if (Math.random() > 0.8) {
        permission.fromUser = true
        permission.granted = true
      }
    })
  })
  
  // 保存原始状态
  saveOriginalState()
}

/**
 * 保存原始状态
 */
function saveOriginalState() {
  originalPermissions.value = []
  permissionCategories.value.forEach(category => {
    category.permissions.forEach(permission => {
      originalPermissions.value.push({
        id: permission.id,
        granted: permission.granted
      })
    })
  })
}

/**
 * 生成模拟角色数据
 */
function generateMockRoles() {
  return [
    {
      id: 'admin',
      name: '系统管理员',
      permissions: [
        { id: 'device_read', granted: true, canOverride: false },
        { id: 'device_write', granted: true, canOverride: true },
        { id: 'system_config', granted: true, canOverride: false },
        { id: 'user_manage', granted: true, canOverride: true },
        { id: 'data_read', granted: true, canOverride: false },
        { id: 'data_export', granted: true, canOverride: true },
        { id: 'alert_read', granted: true, canOverride: false },
        { id: 'alert_ack', granted: true, canOverride: true }
      ]
    },
    {
      id: 'operator',
      name: '操作员',
      permissions: [
        { id: 'device_read', granted: true, canOverride: false },
        { id: 'device_write', granted: true, canOverride: true },
        { id: 'data_read', granted: true, canOverride: false },
        { id: 'alert_read', granted: true, canOverride: false },
        { id: 'alert_ack', granted: true, canOverride: true }
      ]
    },
    {
      id: 'viewer',
      name: '观察员',
      permissions: [
        { id: 'device_read', granted: true, canOverride: false },
        { id: 'data_read', granted: true, canOverride: false },
        { id: 'alert_read', granted: true, canOverride: false }
      ]
    }
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
    { id: 'management', name: '管理部门' }
  ]
}

/**
 * 获取角色类型
 */
function getRoleType(roleId: string): string {
  switch (roleId) {
    case 'admin': return 'danger'
    case 'manager': return 'warning'
    case 'operator': return 'success'
    case 'viewer': return 'info'
    default: return 'info'
  }
}

/**
 * 获取角色名称
 */
function getRoleName(roleId: string): string {
  const role = availableRoles.value.find(r => r.id === roleId)
  return role?.name || '未知角色'
}

/**
 * 获取部门名称
 */
function getDepartmentName(departmentId: string): string {
  const department = departments.value.find(d => d.id === departmentId)
  return department?.name || '未分配部门'
}

/**
 * 获取已授予权限数量
 */
function getGrantedCount(categoryId: string): number {
  const category = permissionCategories.value.find(c => c.id === categoryId)
  return category?.permissions.filter(p => p.granted).length || 0
}

/**
 * 获取权限进度
 */
function getPermissionProgress(categoryId: string): number {
  const category = permissionCategories.value.find(c => c.id === categoryId)
  if (!category || category.permissions.length === 0) return 0
  
  const grantedCount = category.permissions.filter(p => p.granted).length
  return Math.round((grantedCount / category.permissions.length) * 100)
}

/**
 * 获取进度条颜色
 */
function getProgressColor(categoryId: string): string {
  const progress = getPermissionProgress(categoryId)
  if (progress >= 80) return '#67C23A'
  if (progress >= 50) return '#E6A23C'
  return '#F56C6C'
}

/**
 * 选择分类
 */
function selectCategory(categoryId: string) {
  selectedCategoryId.value = categoryId
}

/**
 * 获取选中分类名称
 */
function getSelectedCategoryName(): string {
  const category = permissionCategories.value.find(c => c.id === selectedCategoryId.value)
  return category?.name || ''
}

/**
 * 获取选中分类权限
 */
function getSelectedCategoryPermissions() {
  const category = permissionCategories.value.find(c => c.id === selectedCategoryId.value)
  return category?.permissions || []
}

/**
 * 获取级别文本
 */
function getLevelText(level: string): string {
  switch (level) {
    case 'high': return '高风险'
    case 'medium': return '中风险'
    case 'low': return '低风险'
    default: return '未知'
  }
}

/**
 * 获取权限名称
 */
function getPermissionName(permissionId: string): string {
  for (const category of permissionCategories.value) {
    const permission = category.permissions.find(p => p.id === permissionId)
    if (permission) return permission.name
  }
  return permissionId
}

/**
 * 展开所有分类
 */
function expandAllCategories() {
  // 实现分类展开逻辑
  ElMessage.success('已展开所有分类')
}

/**
 * 收起所有分类
 */
function collapseAllCategories() {
  // 实现分类收起逻辑
  ElMessage.success('已收起所有分类')
}

/**
 * 授予分类下所有权限
 */
function grantAllInCategory() {
  const category = permissionCategories.value.find(c => c.id === selectedCategoryId.value)
  if (!category) return
  
  category.permissions.forEach(permission => {
    if (permission.canOverride || !permission.fromRole) {
      permission.granted = true
    }
  })
  
  ElMessage.success(`已授予 ${category.name} 分类下所有可用权限`)
}

/**
 * 撤销分类下所有权限
 */
function revokeAllInCategory() {
  const category = permissionCategories.value.find(c => c.id === selectedCategoryId.value)
  if (!category) return
  
  category.permissions.forEach(permission => {
    if (permission.canOverride || !permission.fromRole) {
      permission.granted = false
    }
  })
  
  ElMessage.success(`已撤销 ${category.name} 分类下所有可撤销权限`)
}

/**
 * 处理权限变更
 */
function handlePermissionChange(permission: any) {
  // 检查依赖权限
  if (permission.granted && permission.dependencies?.length) {
    for (const depId of permission.dependencies) {
      const depPermission = findPermissionById(depId)
      if (depPermission && !depPermission.granted) {
        ElMessageBox.confirm(
          `此权限依赖 "${getPermissionName(depId)}" 权限，是否同时授予？`,
          '权限依赖',
          {
            type: 'warning'
          }
        ).then(() => {
          depPermission.granted = true
        }).catch(() => {
          permission.granted = false
        })
        return
      }
    }
  }
  
  // 检查被依赖权限
  if (!permission.granted) {
    const dependentPermissions = findDependentPermissions(permission.id)
    if (dependentPermissions.length > 0) {
      const dependentNames = dependentPermissions.map(p => p.name).join('、')
      ElMessageBox.confirm(
        `撤销此权限将同时撤销依赖它的权限: ${dependentNames}，是否继续？`,
        '权限依赖',
        {
          type: 'warning'
        }
      ).then(() => {
        dependentPermissions.forEach(p => p.granted = false)
      }).catch(() => {
        permission.granted = true
      })
    }
  }
}

/**
 * 根据ID查找权限
 */
function findPermissionById(permissionId: string) {
  for (const category of permissionCategories.value) {
    const permission = category.permissions.find(p => p.id === permissionId)
    if (permission) return permission
  }
  return null
}

/**
 * 查找依赖指定权限的权限列表
 */
function findDependentPermissions(permissionId: string) {
  const dependents: any[] = []
  
  for (const category of permissionCategories.value) {
    for (const permission of category.permissions) {
      if (permission.dependencies?.includes(permissionId) && permission.granted) {
        dependents.push(permission)
      }
    }
  }
  
  return dependents
}

/**
 * 重置变更
 */
function resetChanges() {
  originalPermissions.value.forEach(original => {
    const permission = findPermissionById(original.id)
    if (permission) {
      permission.granted = original.granted
    }
  })
  
  ElMessage.success('已重置所有变更')
}

/**
 * 保存权限
 */
async function savePermissions() {
  try {
    saving.value = true
    
    // 模拟保存操作
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    const permissionData = {
      userId: props.userData?.id,
      permissions: []
    }
    
    permissionCategories.value.forEach(category => {
      category.permissions.forEach(permission => {
        if (permission.fromUser || (permission.canOverride && permission.granted !== permission.fromRole)) {
          permissionData.permissions.push({
            id: permission.id,
            granted: permission.granted,
            source: 'user'
          })
        }
      })
    })
    
    emit('save', permissionData)
    dialogVisible.value = false
    
    ElMessage.success('权限保存成功')
    
  } catch (error) {
    console.error('保存权限失败:', error)
    ElMessage.error('保存权限失败')
  } finally {
    saving.value = false
  }
}

/**
 * 处理对话框关闭
 */
function handleClose() {
  if (hasChanges.value) {
    ElMessageBox.confirm(
      '你有未保存的权限更改，确定要关闭吗？',
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
})
</script>

<style scoped lang="scss">
.permission-assign-dialog {
  .user-info-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: #f8f9fa;
    border-radius: 8px;
    
    .user-avatar {
      flex-shrink: 0;
    }
    
    .user-details {
      flex: 1;
      
      .user-name {
        margin: 0 0 8px 0;
        font-size: 18px;
        font-weight: 600;
        color: #303133;
      }
      
      .user-meta {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
      }
    }
  }
  
  .permission-content {
    display: flex;
    gap: 24px;
    margin-top: 24px;
    
    .permission-categories {
      width: 300px;
      flex-shrink: 0;
      
      .category-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 16px;
        
        h4 {
          margin: 0;
          font-size: 16px;
          font-weight: 600;
          color: #303133;
        }
        
        .category-actions {
          display: flex;
          gap: 8px;
        }
      }
      
      .category-list {
        .category-item {
          padding: 12px;
          border: 1px solid #ebeef5;
          border-radius: 8px;
          margin-bottom: 8px;
          cursor: pointer;
          transition: all 0.2s;
          
          &:hover {
            border-color: #409eff;
            background: #f0f9ff;
          }
          
          &.active {
            border-color: #409eff;
            background: #409eff;
            color: white;
            
            .category-stats {
              color: rgba(255, 255, 255, 0.8);
            }
          }
          
          .category-main {
            display: flex;
            align-items: center;
            gap: 12px;
            margin-bottom: 8px;
            
            .category-icon {
              font-size: 18px;
            }
            
            .category-name {
              flex: 1;
              font-weight: 500;
            }
            
            .category-stats {
              font-size: 12px;
              color: #909399;
              
              .granted-count {
                font-weight: 600;
              }
            }
          }
          
          .category-progress {
            margin-top: 8px;
          }
        }
      }
    }
    
    .permission-details {
      flex: 1;
      
      .details-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 20px;
        
        h4 {
          margin: 0;
          font-size: 16px;
          font-weight: 600;
          color: #303133;
        }
        
        .details-actions {
          display: flex;
          gap: 8px;
        }
      }
      
      .permission-grid {
        .permission-item {
          border: 1px solid #ebeef5;
          border-radius: 8px;
          padding: 16px;
          margin-bottom: 12px;
          transition: all 0.2s;
          
          &:hover {
            border-color: #409eff;
            box-shadow: 0 2px 12px rgba(64, 158, 255, 0.1);
          }
          
          .permission-main {
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            
            .permission-info {
              flex: 1;
              
              .permission-header {
                display: flex;
                align-items: center;
                gap: 12px;
                margin-bottom: 8px;
                
                .permission-name {
                  font-weight: 500;
                  color: #303133;
                }
                
                .permission-level {
                  margin: 0;
                }
              }
              
              .permission-desc {
                font-size: 14px;
                color: #606266;
                margin-bottom: 8px;
                line-height: 1.4;
              }
              
              .permission-source {
                display: flex;
                gap: 8px;
                
                .source-tag {
                  padding: 2px 8px;
                  border-radius: 4px;
                  font-size: 12px;
                  
                  &.role-source {
                    background: #e1f3d8;
                    color: #67c23a;
                  }
                  
                  &.user-source {
                    background: #fdf6ec;
                    color: #e6a23c;
                  }
                }
              }
            }
            
            .permission-control {
              flex-shrink: 0;
              margin-left: 16px;
            }
          }
          
          .permission-dependencies {
            margin-top: 12px;
            padding-top: 12px;
            border-top: 1px solid #f0f0f0;
            
            .dependencies-label {
              font-size: 12px;
              color: #909399;
              margin-bottom: 8px;
            }
            
            .dependencies-list {
              display: flex;
              gap: 6px;
              flex-wrap: wrap;
              
              .dependency-tag {
                margin: 0;
              }
            }
          }
        }
      }
    }
  }
  
  .changes-preview {
    margin-top: 20px;
    
    .changes-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;
      
      h4 {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        color: #303133;
      }
    }
    
    .changes-list {
      max-height: 200px;
      overflow-y: auto;
      
      .change-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 8px 12px;
        border-radius: 6px;
        margin-bottom: 4px;
        
        &.grant {
          background: #f0f9ff;
          border-left: 3px solid #67c23a;
        }
        
        &.revoke {
          background: #fef0f0;
          border-left: 3px solid #f56c6c;
        }
        
        .change-icon {
          font-size: 14px;
          
          .change-item.grant & {
            color: #67c23a;
          }
          
          .change-item.revoke & {
            color: #f56c6c;
          }
        }
        
        .change-text {
          flex: 1;
          font-size: 14px;
          color: #303133;
        }
        
        .change-tag {
          margin: 0;
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
  .permission-assign-dialog {
    .permission-content {
      flex-direction: column;
      gap: 16px;
      
      .permission-categories {
        width: 100%;
      }
    }
    
    .user-info-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }
  }
}
</style>