import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import { ElMessage } from 'element-plus'
import type { User, LoginRequest, LoginResponse } from '@/types/auth'
import { authApi } from '@/api'

export const useAuthStore = defineStore('auth', () => {
  // 测试模式检查
  const isTestMode = import.meta.env.VITE_BYPASS_AUTH === 'true'
  
  // State
  const token = ref<string | null>(localStorage.getItem('token'))
  const user = ref<User | null>(
    localStorage.getItem('user') 
      ? JSON.parse(localStorage.getItem('user')!) 
      : null
  )
  const loading = ref(false)

  // 如果是测试模式，创建虚拟用户
  if (isTestMode && !user.value) {
    const testUser = {
      id: 'test-user-001',
      username: 'test-admin',
      email: 'test@gateway.local',
      full_name: '测试管理员',
      role: 'admin',
      permissions: ['*'], // 所有权限
      is_active: true,
      created_at: new Date().toISOString()
    }
    user.value = testUser
    token.value = `test-token-${  Date.now()}`
    localStorage.setItem('user', JSON.stringify(testUser))
    localStorage.setItem('token', token.value)
  }

  // Getters
  const isAuthenticated = computed(() => {
    if (isTestMode) return true // 测试模式下始终认为已认证
    return !!token.value && !!user.value
  })
  const userRole = computed(() => user.value?.role)
  const userName = computed(() => user.value?.full_name || user.value?.username)

  // Actions
  const login = async (credentials: LoginRequest): Promise<void> => {
    loading.value = true
    try {
      // 测试模式下直接模拟成功登录
      if (isTestMode) {
        // 模拟API延迟
        await new Promise(resolve => setTimeout(resolve, 500))
        
        const testUser = {
          id: 'test-user-001',
          username: credentials.username || 'test-admin',
          email: 'test@gateway.local',
          full_name: `测试用户-${credentials.username}`,
          role: 'admin',
          permissions: ['*'], // 所有权限
          is_active: true,
          created_at: new Date().toISOString()
        }
        
        const testToken = `test-token-${  Date.now()}`
        
        token.value = testToken
        user.value = testUser
        
        localStorage.setItem('token', testToken)
        localStorage.setItem('user', JSON.stringify(testUser))
        
        ElMessage.success('登录成功 (测试模式)')
        return
      }
      
      // 正常模式下调用真实API
      const response = await authApi.login(credentials)
      
      if (response.success && response.data) {
        const loginData = response.data as LoginResponse
        token.value = loginData.token
        user.value = loginData.user
        
        // Store token and user in localStorage
        localStorage.setItem('token', loginData.token)
        localStorage.setItem('user', JSON.stringify(loginData.user))
        
        ElMessage.success('登录成功')
      } else {
        throw new Error(response.message || '登录失败')
      }
    } catch (error: any) {
      if (!isTestMode) {
        ElMessage.error(error.message || '登录失败')
        throw error
      }
    } finally {
      loading.value = false
    }
  }

  const logout = async (): Promise<void> => {
    try {
      await authApi.logout()
    } catch (error) {
      console.warn('Logout API call failed:', error)
    } finally {
      // Clear local state regardless of API call result
      token.value = null
      user.value = null
      localStorage.removeItem('token')
      localStorage.removeItem('user')
      ElMessage.success('已退出登录')
    }
  }

  const fetchUserInfo = async (): Promise<void> => {
    if (!token.value) return
    
    try {
      const response = await authApi.getUserInfo()
      if (response.success && response.data) {
        user.value = response.data as User
        localStorage.setItem('user', JSON.stringify(user.value))
      }
    } catch (error) {
      console.error('Failed to fetch user info:', error)
      // Clear invalid authentication state
      token.value = null
      user.value = null
      localStorage.removeItem('token')
      localStorage.removeItem('user')
      throw error
    }
  }

  const updatePassword = async (currentPassword: string, newPassword: string): Promise<void> => {
    loading.value = true
    try {
      const response = await authApi.updatePassword({
        current_password: currentPassword,
        new_password: newPassword,
      })
      
      if (response.success) {
        ElMessage.success('密码修改成功')
      } else {
        throw new Error(response.message || '密码修改失败')
      }
    } catch (error: any) {
      ElMessage.error(error.message || '密码修改失败')
      throw error
    } finally {
      loading.value = false
    }
  }

  const updateProfile = async (profileData: Partial<User>): Promise<void> => {
    loading.value = true
    try {
      const response = await authApi.updateProfile(profileData)
      
      if (response.success && response.data) {
        user.value = response.data as User
        ElMessage.success('个人信息更新成功')
      } else {
        throw new Error(response.message || '信息更新失败')
      }
    } catch (error: any) {
      ElMessage.error(error.message || '信息更新失败')
      throw error
    } finally {
      loading.value = false
    }
  }

  const hasPermission = (permission: string): boolean => {
    if (!user.value) return false
    
    // Admin has all permissions
    if (user.value.role === 'admin') return true
    
    // Define role-based permissions
    const rolePermissions: Record<string, string[]> = {
      admin: ['*'], // All permissions
      operator: [
        'dashboard:read',
        'realtime:read',
        'drivers:read',
        'drivers:write',
        'connectors:read',
        'connectors:write',
        'data-points:read',
        'data-points:write',
        'alerts:read',
        'alerts:write',
        'analytics:read',
        'monitoring:read',
        'system:config:read',
        'system:logs:read',
      ],
      viewer: [
        'dashboard:read',
        'realtime:read',
        'drivers:read',
        'connectors:read',
        'data-points:read',
        'alerts:read',
        'analytics:read',
        'monitoring:read',
        'system:logs:read',
      ],
    }
    
    const userPermissions = rolePermissions[user.value.role] || []
    return userPermissions.includes('*') || userPermissions.includes(permission)
  }

  const hasRole = (role: string): boolean => {
    // 测试模式下，admin角色始终返回 true，其他角色基于测试用户配置
    if (isTestMode) {
      return role === 'admin' || user.value?.role === role
    }
    
    return user.value?.role === role
  }

  // Session management
  const sessions = ref<any[]>([])
  const sessionLoading = ref(false)

  const getSessions = async (): Promise<void> => {
    if (!user.value) return
    
    // 测试模式下返回模拟会话数据
    if (isTestMode) {
      sessions.value = [{
        id: 'test-session-001',
        ip_address: '127.0.0.1',
        user_agent: 'Test Browser',
        location: '本地测试',
        is_active: true,
        created_at: new Date().toISOString(),
        last_used_at: new Date().toISOString()
      }]
      console.log('🔧 测试模式 - 会话列表: 返回测试数据')
      return
    }
    
    sessionLoading.value = true
    try {
      const response = await authApi.getSessions()
      if (response.success && response.data) {
        sessions.value = response.data
      }
    } catch (error) {
      console.error('Failed to fetch sessions:', error)
      ElMessage.error('获取会话信息失败')
    } finally {
      sessionLoading.value = false
    }
  }

  const terminateSession = async (sessionId: string): Promise<void> => {
    try {
      const response = await authApi.terminateSession(sessionId)
      if (response.success) {
        await getSessions() // Refresh sessions list
        ElMessage.success('会话已终止')
      } else {
        throw new Error(response.message || '终止会话失败')
      }
    } catch (error: any) {
      console.error('Failed to terminate session:', error)
      ElMessage.error(error.message || '终止会话失败')
    }
  }

  const terminateAllOtherSessions = async (): Promise<void> => {
    try {
      const response = await authApi.terminateAllOtherSessions()
      if (response.success) {
        await getSessions() // Refresh sessions list
        ElMessage.success('已终止其他所有会话')
      } else {
        throw new Error(response.message || '终止其他会话失败')
      }
    } catch (error: any) {
      console.error('Failed to terminate other sessions:', error)
      ElMessage.error(error.message || '终止其他会话失败')
    }
  }

  // Activity logging
  const activities = ref<any[]>([])
  const activityLoading = ref(false)

  const logActivity = async (activity: {
    type: string
    action: string
    target?: string
    details?: any
  }): Promise<void> => {
    // 测试模式下跳过API调用，只在控制台记录
    if (isTestMode) {
      console.log('🔧 测试模式 - 活动记录:', activity)
      return
    }
    
    try {
      const response = await authApi.logActivity(activity)
      if (!response.success) {
        console.warn('Failed to log activity:', response.message)
      }
    } catch (error) {
      console.error('Failed to log activity:', error)
    }
  }

  const getActivities = async (filters?: {
    type?: string
    dateRange?: [Date, Date]
    limit?: number
  }): Promise<void> => {
    // 测试模式下返回测试活动记录
    if (isTestMode) {
      activities.value = [{
        id: 'test-activity-001',
        activity_type: 'login',
        action: '用户登录',
        target: 'test-admin',
        details: { message: '测试模式登录' },
        ip_address: '127.0.0.1',
        created_at: new Date().toISOString()
      }]
      console.log('🔧 测试模式 - 活动记录: 返回测试数据')
      return
    }
    
    activityLoading.value = true
    try {
      const response = await authApi.getActivities(filters)
      if (response.success && response.data) {
        activities.value = response.data
      }
    } catch (error) {
      console.error('Failed to fetch activities:', error)
      ElMessage.error('获取活动记录失败')
    } finally {
      activityLoading.value = false
    }
  }

  // Permission management with more granular control
  const permissions = ref<string[]>([])

  const loadUserPermissions = async (): Promise<void> => {
    if (!user.value) return
    
    // 测试模式下直接设置所有权限
    if (isTestMode) {
      permissions.value = ['*']
      console.log('🔧 测试模式 - 权限加载: 已授予所有权限')
      return
    }
    
    try {
      const response = await authApi.getUserPermissions()
      if (response.success && response.data) {
        permissions.value = response.data
      }
    } catch (error) {
      console.error('Failed to load permissions:', error)
      permissions.value = []
    }
  }

  const checkPermission = (permission: string): boolean => {
    // 测试模式下始终返回 true
    if (isTestMode) return true
    
    if (!user.value) return false
    
    // Admin has all permissions
    if (user.value.role === 'admin') return true
    
    // Check specific permission
    return permissions.value.includes(permission)
  }

  const checkMultiplePermissions = (requiredPermissions: string[], requireAll = true): boolean => {
    // 测试模式下始终返回 true
    if (isTestMode) return true
    
    if (!user.value) return false
    
    if (user.value.role === 'admin') return true
    
    if (requireAll) {
      return requiredPermissions.every(permission => permissions.value.includes(permission))
    } else {
      return requiredPermissions.some(permission => permissions.value.includes(permission))
    }
  }

  // Initialize store
  const init = async (): Promise<void> => {
    // 测试模式下跳过初始化API调用
    if (isTestMode) {
      console.log('🔧 测试模式 - 跳过认证初始化')
      await loadUserPermissions() // 仍然需要加载权限以设置测试权限
      return
    }
    
    if (token.value) {
      try {
        await fetchUserInfo()
        await loadUserPermissions()
        await getSessions()
        await getActivities()
      } catch (error) {
        console.error('Failed to initialize auth store:', error)
        // Clear invalid authentication state
        token.value = null
        user.value = null
        localStorage.removeItem('token')
        localStorage.removeItem('user')
      }
    }
  }

  return {
    // State
    token: readonly(token),
    user: readonly(user),
    loading: readonly(loading),
    sessions: readonly(sessions),
    sessionLoading: readonly(sessionLoading),
    activities: readonly(activities),
    activityLoading: readonly(activityLoading),
    permissions: readonly(permissions),
    
    // Getters
    isAuthenticated,
    userRole,
    userName,
    
    // Actions
    login,
    logout,
    fetchUserInfo,
    updatePassword,
    updateProfile,
    hasPermission,
    hasRole,
    init,
    
    // Session management
    getSessions,
    terminateSession,
    terminateAllOtherSessions,
    
    // Activity logging
    logActivity,
    getActivities,
    
    // Enhanced permission management
    loadUserPermissions,
    checkPermission,
    checkMultiplePermissions,
  }
})