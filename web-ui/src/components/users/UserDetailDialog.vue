<template>
  <el-dialog
    v-model="dialogVisible"
    title="用户详情"
    width="900px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div v-if="userData" class="user-detail-dialog">
      <!-- 用户基本信息卡片 -->
      <el-card class="user-info-card" shadow="never">
        <div class="user-header">
          <div class="user-avatar">
            <el-avatar :size="80" :src="userData.avatar">
              <el-icon><UserFilled /></el-icon>
            </el-avatar>
            <div v-if="userData.isOnline" class="online-indicator">
              <el-badge is-dot type="success" />
            </div>
          </div>

          <div class="user-basic">
            <div class="user-name">{{ userData.name }}</div>
            <div class="user-username">@{{ userData.username }}</div>
            <div class="user-tags">
              <el-tag :type="getRoleType(userData.role)" size="small">
                {{ getRoleName(userData.role) }}
              </el-tag>
              <el-tag :type="getStatusType(userData.status)" size="small">
                {{ getStatusLabel(userData.status) }}
              </el-tag>
              <el-tag v-if="userData.isOnline" type="success" size="small">
                在线
              </el-tag>
            </div>
          </div>

          <div class="user-actions">
            <el-button type="primary" @click="editUser">
              <el-icon><Edit /></el-icon>
              编辑
            </el-button>
            <el-button @click="sendMessage">
              <el-icon><Message /></el-icon>
              发送消息
            </el-button>
          </div>
        </div>
      </el-card>

      <!-- 详细信息标签页 -->
      <el-tabs v-model="activeTab" class="detail-tabs">
        <!-- 基本信息 -->
        <el-tab-pane label="基本信息" name="basic">
          <div class="detail-content">
            <div class="detail-sections">
              <!-- 个人信息 -->
              <div class="detail-section">
                <h4 class="section-title">个人信息</h4>
                <div class="detail-grid">
                  <div class="detail-item">
                    <label>真实姓名：</label>
                    <span>{{ userData.name }}</span>
                  </div>
                  <div class="detail-item">
                    <label>用户名：</label>
                    <span>{{ userData.username }}</span>
                  </div>
                  <div class="detail-item">
                    <label>邮箱地址：</label>
                    <span>{{ userData.email }}</span>
                  </div>
                  <div class="detail-item">
                    <label>手机号码：</label>
                    <span>{{ userData.phone || '-' }}</span>
                  </div>
                  <div class="detail-item">
                    <label>工号：</label>
                    <span>{{ userData.employeeId || '-' }}</span>
                  </div>
                  <div class="detail-item">
                    <label>职位：</label>
                    <span>{{ userData.position || '-' }}</span>
                  </div>
                </div>
              </div>

              <!-- 组织信息 -->
              <div class="detail-section">
                <h4 class="section-title">组织信息</h4>
                <div class="detail-grid">
                  <div class="detail-item">
                    <label>所属部门：</label>
                    <span>{{ getDepartmentName(userData.departmentId) }}</span>
                  </div>
                  <div class="detail-item">
                    <label>用户角色：</label>
                    <el-tag :type="getRoleType(userData.role)" size="small">
                      {{ getRoleName(userData.role) }}
                    </el-tag>
                  </div>
                  <div class="detail-item">
                    <label>直属上级：</label>
                    <span>{{ getManagerName(userData.managerId) }}</span>
                  </div>
                  <div class="detail-item">
                    <label>入职时间：</label>
                    <span>{{ formatDate(userData.joinDate) }}</span>
                  </div>
                </div>
              </div>

              <!-- 账户状态 -->
              <div class="detail-section">
                <h4 class="section-title">账户状态</h4>
                <div class="detail-grid">
                  <div class="detail-item">
                    <label>账户状态：</label>
                    <el-tag :type="getStatusType(userData.status)" size="small">
                      {{ getStatusLabel(userData.status) }}
                    </el-tag>
                  </div>
                  <div class="detail-item">
                    <label>在线状态：</label>
                    <el-tag
                      :type="userData.isOnline ? 'success' : 'info'"
                      size="small"
                    >
                      {{ userData.isOnline ? '在线' : '离线' }}
                    </el-tag>
                  </div>
                  <div class="detail-item">
                    <label>账户有效期：</label>
                    <span>{{
                      formatDate(userData.expireDate) || '永久有效'
                    }}</span>
                  </div>
                  <div class="detail-item">
                    <label>创建时间：</label>
                    <span>{{ formatDateTime(userData.createdAt) }}</span>
                  </div>
                  <div class="detail-item">
                    <label>最后更新：</label>
                    <span>{{ formatDateTime(userData.updatedAt) }}</span>
                  </div>
                </div>
              </div>

              <!-- 备注信息 -->
              <div v-if="userData.remark" class="detail-section">
                <h4 class="section-title">备注信息</h4>
                <div class="remark-content">
                  <p>{{ userData.remark }}</p>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- 权限信息 -->
        <el-tab-pane label="权限信息" name="permissions">
          <div class="detail-content">
            <div class="permissions-overview">
              <div class="role-info">
                <h4 class="section-title">当前角色</h4>
                <div class="role-card">
                  <div class="role-header">
                    <el-tag :type="getRoleType(userData.role)" size="large">
                      {{ getRoleName(userData.role) }}
                    </el-tag>
                  </div>
                  <div class="role-description">
                    {{ getRoleDescription(userData.role) }}
                  </div>
                </div>
              </div>

              <div class="permissions-list">
                <h4 class="section-title">具体权限</h4>
                <div class="permission-categories">
                  <div
                    v-for="category in userPermissionCategories"
                    :key="category.name"
                    class="permission-category"
                  >
                    <div class="category-header">
                      <el-icon class="category-icon">
                        <component :is="category.icon" />
                      </el-icon>
                      <span class="category-name">{{ category.name }}</span>
                      <el-tag
                        :type="getPermissionSummaryType(category.permissions)"
                        size="small"
                      >
                        {{ getPermissionSummary(category.permissions) }}
                      </el-tag>
                    </div>
                    <div class="category-permissions">
                      <div
                        v-for="permission in category.permissions"
                        :key="permission.id"
                        class="permission-item"
                      >
                        <el-icon
                          :class="permission.granted ? 'granted' : 'denied'"
                        >
                          <component
                            :is="permission.granted ? 'Check' : 'Close'"
                          />
                        </el-icon>
                        <span
                          :class="permission.granted ? 'granted' : 'denied'"
                        >
                          {{ permission.name }}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- 登录记录 -->
        <el-tab-pane label="登录记录" name="login">
          <div class="detail-content">
            <div class="login-overview">
              <div class="login-stats">
                <div class="stat-item">
                  <div class="stat-number">{{ userData.loginCount || 0 }}</div>
                  <div class="stat-label">总登录次数</div>
                </div>
                <div class="stat-item">
                  <div class="stat-number">
                    {{ formatDate(userData.lastLoginTime) || '从未' }}
                  </div>
                  <div class="stat-label">最后登录</div>
                </div>
                <div class="stat-item">
                  <div class="stat-number">
                    {{ userData.lastLoginIp || '-' }}
                  </div>
                  <div class="stat-label">最后登录IP</div>
                </div>
              </div>

              <div class="login-history">
                <h4 class="section-title">最近登录记录</h4>
                <el-table :data="loginHistory" :loading="loadingHistory">
                  <el-table-column label="登录时间" width="180">
                    <template #default="{ row }">
                      {{ formatDateTime(row.loginTime) }}
                    </template>
                  </el-table-column>
                  <el-table-column label="IP地址" width="150">
                    <template #default="{ row }">
                      <span class="ip-address">{{ row.ipAddress }}</span>
                    </template>
                  </el-table-column>
                  <el-table-column label="登录位置">
                    <template #default="{ row }">
                      {{ row.location || '未知' }}
                    </template>
                  </el-table-column>
                  <el-table-column label="设备类型" width="120">
                    <template #default="{ row }">
                      <el-tag size="small">{{ row.deviceType }}</el-tag>
                    </template>
                  </el-table-column>
                  <el-table-column label="浏览器" width="120">
                    <template #default="{ row }">
                      {{ row.browser }}
                    </template>
                  </el-table-column>
                  <el-table-column label="状态" width="100">
                    <template #default="{ row }">
                      <el-tag
                        :type="row.success ? 'success' : 'danger'"
                        size="small"
                      >
                        {{ row.success ? '成功' : '失败' }}
                      </el-tag>
                    </template>
                  </el-table-column>
                </el-table>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- 活动日志 -->
        <el-tab-pane label="活动日志" name="activity">
          <div class="detail-content">
            <div class="activity-timeline">
              <h4 class="section-title">最近活动</h4>
              <el-timeline>
                <el-timeline-item
                  v-for="activity in userActivities"
                  :key="activity.id"
                  :timestamp="formatDateTime(activity.timestamp)"
                  :type="getActivityType(activity.type)"
                >
                  <div class="activity-item">
                    <div class="activity-header">
                      <el-icon class="activity-icon">
                        <component :is="getActivityIcon(activity.type)" />
                      </el-icon>
                      <span class="activity-title">{{ activity.title }}</span>
                      <el-tag
                        :type="getActivityTagType(activity.type)"
                        size="small"
                      >
                        {{ getActivityTypeLabel(activity.type) }}
                      </el-tag>
                    </div>
                    <div class="activity-description">
                      {{ activity.description }}
                    </div>
                    <div v-if="activity.details" class="activity-details">
                      <el-descriptions :column="2" size="small">
                        <el-descriptions-item
                          v-for="(value, key) in activity.details"
                          :key="key"
                          :label="key"
                        >
                          {{ value }}
                        </el-descriptions-item>
                      </el-descriptions>
                    </div>
                  </div>
                </el-timeline-item>
              </el-timeline>
            </div>
          </div>
        </el-tab-pane>

        <!-- 会话管理 -->
        <el-tab-pane label="会话管理" name="sessions">
          <div class="detail-content">
            <div class="sessions-list">
              <h4 class="section-title">当前会话</h4>
              <el-table :data="userSessions" :loading="loadingSessions">
                <el-table-column label="会话ID" width="200">
                  <template #default="{ row }">
                    <span class="session-id">{{ row.sessionId }}</span>
                  </template>
                </el-table-column>
                <el-table-column label="IP地址" width="150">
                  <template #default="{ row }">
                    <span class="ip-address">{{ row.ipAddress }}</span>
                  </template>
                </el-table-column>
                <el-table-column label="设备信息">
                  <template #default="{ row }">
                    <div class="device-info">
                      <div>{{ row.deviceType }} - {{ row.browser }}</div>
                      <div class="device-os">{{ row.operatingSystem }}</div>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column label="登录时间" width="180">
                  <template #default="{ row }">
                    {{ formatDateTime(row.loginTime) }}
                  </template>
                </el-table-column>
                <el-table-column label="最后活动" width="180">
                  <template #default="{ row }">
                    {{ formatDateTime(row.lastActivity) }}
                  </template>
                </el-table-column>
                <el-table-column label="状态" width="100">
                  <template #default="{ row }">
                    <el-tag
                      :type="row.isActive ? 'success' : 'warning'"
                      size="small"
                    >
                      {{ row.isActive ? '活跃' : '空闲' }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column label="操作" width="120">
                  <template #default="{ row }">
                    <el-button
                      type="link"
                      size="small"
                      :disabled="row.isCurrent"
                      @click="terminateSession(row)"
                    >
                      {{ row.isCurrent ? '当前会话' : '终止' }}
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">关闭</el-button>
        <el-button type="primary" @click="editUser"> 编辑用户 </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * UserDetailDialog —— 用户详情对话框组件
 *
 * 📝 Responsibilities:
 *  1. 展示用户详细信息
 *  2. 显示用户权限和角色信息
 *  3. 展示登录记录和活动日志
 *  4. 管理用户会话信息
 *  5. 提供快捷操作入口
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
  Edit,
  Message,
  Monitor,
  Setting,
  Document,
  Connection,
  Bell,
  Check,
  Close,
  User,
  Lock,
  View,
  Operation,
  ChatDotRound,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, watch, onMounted } from 'vue'

// ===== Props & Emits =====
const props = defineProps<{
  visible: boolean
  userData?: any
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  edit: [userData: any]
}>()

// ===== 响应式数据 =====
const dialogVisible = ref(props.visible)
const activeTab = ref('basic')
const loadingHistory = ref(false)
const loadingSessions = ref(false)

// 数据
const loginHistory = ref<any[]>([])
const userActivities = ref<any[]>([])
const userSessions = ref<any[]>([])

// 模拟数据
const availableRoles = ref<any[]>([])
const departments = ref<any[]>([])
const managers = ref<any[]>([])

// ===== 计算属性 =====
const userPermissionCategories = computed(() => {
  if (!props.userData?.role) return []

  const role = availableRoles.value.find(r => r.id === props.userData.role)
  const permissions = role?.permissions || []

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
    // 生成模拟数据
    availableRoles.value = generateMockRoles()
    departments.value = generateMockDepartments()
    managers.value = generateMockManagers()

    // 加载用户相关数据
    await Promise.all([
      loadLoginHistory(),
      loadUserActivities(),
      loadUserSessions(),
    ])
  } catch (error) {
    console.error('初始化用户详情失败:', error)
    ElMessage.error('初始化失败')
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
      permissions: [
        {
          id: 'device_read',
          name: '设备查看',
          category: 'device',
          granted: true,
        },
        {
          id: 'device_write',
          name: '设备编辑',
          category: 'device',
          granted: true,
        },
        {
          id: 'device_delete',
          name: '设备删除',
          category: 'device',
          granted: true,
        },
        {
          id: 'system_config',
          name: '系统配置',
          category: 'system',
          granted: true,
        },
        {
          id: 'user_manage',
          name: '用户管理',
          category: 'system',
          granted: true,
        },
        {
          id: 'data_export',
          name: '数据导出',
          category: 'data',
          granted: true,
        },
        {
          id: 'network_config',
          name: '网络配置',
          category: 'network',
          granted: true,
        },
        {
          id: 'alert_manage',
          name: '报警管理',
          category: 'alert',
          granted: true,
        },
      ],
    },
    {
      id: 'operator',
      name: '操作员',
      description: '负责日常设备操作和监控',
      permissions: [
        {
          id: 'device_read',
          name: '设备查看',
          category: 'device',
          granted: true,
        },
        {
          id: 'device_write',
          name: '设备编辑',
          category: 'device',
          granted: true,
        },
        {
          id: 'device_delete',
          name: '设备删除',
          category: 'device',
          granted: false,
        },
        {
          id: 'data_export',
          name: '数据导出',
          category: 'data',
          granted: true,
        },
        {
          id: 'alert_manage',
          name: '报警管理',
          category: 'alert',
          granted: true,
        },
      ],
    },
    {
      id: 'viewer',
      name: '观察员',
      description: '只能查看数据，无法进行修改操作',
      permissions: [
        {
          id: 'device_read',
          name: '设备查看',
          category: 'device',
          granted: true,
        },
        {
          id: 'device_write',
          name: '设备编辑',
          category: 'device',
          granted: false,
        },
        {
          id: 'device_delete',
          name: '设备删除',
          category: 'device',
          granted: false,
        },
        {
          id: 'data_export',
          name: '数据导出',
          category: 'data',
          granted: false,
        },
      ],
    },
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
 * 生成模拟管理员数据
 */
function generateMockManagers() {
  return [
    { id: 'manager1', name: '张经理' },
    { id: 'manager2', name: '李主管' },
    { id: 'manager3', name: '王总监' },
  ]
}

/**
 * 加载登录历史
 */
async function loadLoginHistory() {
  loadingHistory.value = true
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 800))

    loginHistory.value = Array.from({ length: 15 }, (_, i) => ({
      id: i + 1,
      loginTime: new Date(
        Date.now() -
          i * 24 * 60 * 60 * 1000 -
          Math.random() * 12 * 60 * 60 * 1000
      ).toISOString(),
      ipAddress: `192.168.1.${100 + Math.floor(Math.random() * 50)}`,
      location: ['上海', '北京', '深圳', '杭州'][Math.floor(Math.random() * 4)],
      deviceType: ['Desktop', 'Mobile', 'Tablet'][
        Math.floor(Math.random() * 3)
      ],
      browser: ['Chrome', 'Firefox', 'Safari', 'Edge'][
        Math.floor(Math.random() * 4)
      ],
      success: Math.random() > 0.1,
    }))
  } finally {
    loadingHistory.value = false
  }
}

/**
 * 加载用户活动
 */
async function loadUserActivities() {
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 600))

    const activityTypes = [
      'login',
      'logout',
      'create',
      'update',
      'delete',
      'view',
      'export',
    ]
    const activityTitles = {
      login: '登录系统',
      logout: '退出系统',
      create: '创建设备',
      update: '更新配置',
      delete: '删除数据',
      view: '查看报告',
      export: '导出数据',
    }

    userActivities.value = Array.from({ length: 20 }, (_, i) => {
      const type =
        activityTypes[Math.floor(Math.random() * activityTypes.length)]
      return {
        id: i + 1,
        type,
        title: activityTitles[type as keyof typeof activityTitles],
        description: `用户执行了${activityTitles[type as keyof typeof activityTitles]}操作`,
        timestamp: new Date(
          Date.now() - i * 2 * 60 * 60 * 1000 - Math.random() * 60 * 60 * 1000
        ).toISOString(),
        details:
          type === 'create' ? { 设备名称: '设备-001', 设备类型: 'PLC' } : null,
      }
    })
  } catch (error) {
    console.error('加载用户活动失败:', error)
  }
}

/**
 * 加载用户会话
 */
async function loadUserSessions() {
  loadingSessions.value = true
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 500))

    userSessions.value = Array.from({ length: 3 }, (_, i) => ({
      sessionId: `sess_${Date.now()}_${i}`,
      ipAddress: `192.168.1.${100 + i}`,
      deviceType: ['Desktop', 'Mobile', 'Tablet'][i % 3],
      browser: ['Chrome 120', 'Firefox 115', 'Safari 17'][i % 3],
      operatingSystem: ['Windows 11', 'macOS 14', 'iOS 17'][i % 3],
      loginTime: new Date(
        Date.now() - (i + 1) * 24 * 60 * 60 * 1000
      ).toISOString(),
      lastActivity: new Date(Date.now() - i * 60 * 60 * 1000).toISOString(),
      isActive: i === 0,
      isCurrent: i === 0,
    }))
  } finally {
    loadingSessions.value = false
  }
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
 * 获取角色描述
 */
function getRoleDescription(roleId: string): string {
  const role = availableRoles.value.find(r => r.id === roleId)
  return role ? role.description : ''
}

/**
 * 获取部门名称
 */
function getDepartmentName(deptId: string): string {
  const dept = departments.value.find(d => d.id === deptId)
  return dept ? dept.name : deptId
}

/**
 * 获取管理员名称
 */
function getManagerName(managerId: string): string {
  if (!managerId) return '-'
  const manager = managers.value.find(m => m.id === managerId)
  return manager ? manager.name : managerId
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
 * 获取权限汇总类型
 */
function getPermissionSummaryType(permissions: any[]): string {
  const granted = permissions.filter(p => p.granted).length
  const total = permissions.length

  if (granted === total) return 'success'
  if (granted === 0) return 'danger'
  return 'warning'
}

/**
 * 获取权限汇总
 */
function getPermissionSummary(permissions: any[]): string {
  const granted = permissions.filter(p => p.granted).length
  const total = permissions.length
  return `${granted}/${total}`
}

/**
 * 获取活动类型
 */
function getActivityType(type: string): string {
  const typeMap: { [key: string]: string } = {
    login: 'success',
    logout: 'info',
    create: 'primary',
    update: 'warning',
    delete: 'danger',
    view: 'info',
    export: 'success',
  }
  return typeMap[type] || 'info'
}

/**
 * 获取活动图标
 */
function getActivityIcon(type: string): string {
  const iconMap: { [key: string]: string } = {
    login: 'User',
    logout: 'Lock',
    create: 'Plus',
    update: 'Edit',
    delete: 'Delete',
    view: 'View',
    export: 'Download',
  }
  return iconMap[type] || 'Operation'
}

/**
 * 获取活动标签类型
 */
function getActivityTagType(type: string): string {
  return getActivityType(type)
}

/**
 * 获取活动类型标签
 */
function getActivityTypeLabel(type: string): string {
  const labelMap: { [key: string]: string } = {
    login: '登录',
    logout: '登出',
    create: '创建',
    update: '更新',
    delete: '删除',
    view: '查看',
    export: '导出',
  }
  return labelMap[type] || type
}

/**
 * 终止会话
 */
async function terminateSession(session: any) {
  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    ElMessage.success('会话已终止')
    await loadUserSessions()
  } catch (error) {
    ElMessage.error('终止会话失败')
  }
}

/**
 * 编辑用户
 */
function editUser() {
  emit('edit', props.userData)
  dialogVisible.value = false
}

/**
 * 发送消息
 */
function sendMessage() {
  ElMessage.info('发送消息功能开发中...')
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

/**
 * 处理对话框关闭
 */
function handleClose() {
  dialogVisible.value = false
}

// ===== 监听器 =====
watch(
  () => props.visible,
  visible => {
    dialogVisible.value = visible
    if (visible && props.userData) {
      initializeData()
    }
  }
)

watch(dialogVisible, visible => {
  emit('update:visible', visible)
  if (visible) {
    activeTab.value = 'basic'
  }
})

// ===== 生命周期 =====
onMounted(() => {
  if (props.visible && props.userData) {
    initializeData()
  }
})
</script>

<style scoped lang="scss">
.user-detail-dialog {
  .user-info-card {
    margin-bottom: 16px;

    .user-header {
      display: flex;
      align-items: flex-start;
      gap: 20px;

      .user-avatar {
        position: relative;
        flex-shrink: 0;

        .online-indicator {
          position: absolute;
          bottom: 0;
          right: 0;
        }
      }

      .user-basic {
        flex: 1;

        .user-name {
          font-size: 24px;
          font-weight: 600;
          color: #303133;
          margin-bottom: 8px;
        }

        .user-username {
          font-size: 14px;
          color: #909399;
          margin-bottom: 12px;
        }

        .user-tags {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
        }
      }

      .user-actions {
        display: flex;
        gap: 12px;
        flex-shrink: 0;
      }
    }
  }

  .detail-tabs {
    .detail-content {
      .detail-sections {
        .detail-section {
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

          .detail-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 16px;

            .detail-item {
              display: flex;
              align-items: center;

              label {
                font-weight: 500;
                color: #606266;
                width: 100px;
                flex-shrink: 0;
              }

              span {
                color: #303133;
              }
            }
          }

          .remark-content {
            padding: 16px;
            background: #f8f9fa;
            border-radius: 6px;

            p {
              margin: 0;
              color: #606266;
              line-height: 1.6;
            }
          }
        }
      }

      .permissions-overview {
        .role-info {
          margin-bottom: 32px;

          .role-card {
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;

            .role-header {
              margin-bottom: 12px;
            }

            .role-description {
              color: #606266;
              line-height: 1.6;
            }
          }
        }

        .permissions-list {
          .permission-categories {
            .permission-category {
              margin-bottom: 24px;

              &:last-child {
                margin-bottom: 0;
              }

              .category-header {
                display: flex;
                align-items: center;
                gap: 12px;
                margin-bottom: 16px;

                .category-icon {
                  font-size: 18px;
                  color: #409eff;
                }

                .category-name {
                  font-weight: 500;
                  color: #303133;
                  flex: 1;
                }
              }

              .category-permissions {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                gap: 12px;

                .permission-item {
                  display: flex;
                  align-items: center;
                  gap: 8px;
                  padding: 8px 12px;
                  background: #fff;
                  border: 1px solid #e4e7ed;
                  border-radius: 4px;

                  .el-icon {
                    font-size: 16px;

                    &.granted {
                      color: #67c23a;
                    }

                    &.denied {
                      color: #f56c6c;
                    }
                  }

                  span {
                    font-size: 13px;

                    &.granted {
                      color: #303133;
                    }

                    &.denied {
                      color: #909399;
                    }
                  }
                }
              }
            }
          }
        }
      }

      .login-overview {
        .login-stats {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 16px;
          margin-bottom: 32px;

          .stat-item {
            text-align: center;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;

            .stat-number {
              font-size: 20px;
              font-weight: 600;
              color: #303133;
              margin-bottom: 8px;
            }

            .stat-label {
              font-size: 14px;
              color: #606266;
            }
          }
        }

        .login-history {
          .ip-address {
            font-family: monospace;
            color: #409eff;
          }
        }
      }

      .activity-timeline {
        .activity-item {
          .activity-header {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-bottom: 8px;

            .activity-icon {
              font-size: 16px;
              color: #409eff;
            }

            .activity-title {
              font-weight: 500;
              color: #303133;
              flex: 1;
            }
          }

          .activity-description {
            color: #606266;
            margin-bottom: 12px;
            line-height: 1.5;
          }

          .activity-details {
            padding: 12px;
            background: #f8f9fa;
            border-radius: 4px;
          }
        }
      }

      .sessions-list {
        .session-id {
          font-family: monospace;
          font-size: 12px;
          color: #909399;
        }

        .ip-address {
          font-family: monospace;
          color: #409eff;
        }

        .device-info {
          .device-os {
            font-size: 12px;
            color: #909399;
            margin-top: 2px;
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
  .user-detail-dialog {
    .user-info-card .user-header {
      flex-direction: column;
      align-items: center;
      text-align: center;

      .user-actions {
        width: 100%;
        justify-content: center;
      }
    }

    .detail-content {
      .detail-sections .detail-section .detail-grid {
        grid-template-columns: 1fr;
        gap: 12px;
      }

      .permissions-overview
        .permissions-list
        .permission-categories
        .permission-category
        .category-permissions {
        grid-template-columns: 1fr;
      }

      .login-overview .login-stats {
        grid-template-columns: 1fr;
      }
    }
  }
}
</style>
