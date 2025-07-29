import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import { ElMessage } from 'element-plus'
import type { User, LoginRequest, LoginResponse } from '@/types/auth'
import { authApi } from '@/api'

export const useAuthStore = defineStore('auth', () => {
  // State
  const token = ref<string | null>(localStorage.getItem('token'))
  const user = ref<User | null>(
    localStorage.getItem('user') 
      ? JSON.parse(localStorage.getItem('user')!) 
      : null
  )
  const loading = ref(false)

  // Getters
  const isAuthenticated = computed(() => !!token.value && !!user.value)
  const userRole = computed(() => user.value?.role)
  const userName = computed(() => user.value?.full_name || user.value?.username)

  // Actions
  const login = async (credentials: LoginRequest): Promise<void> => {
    loading.value = true
    try {
      // Use mock authentication when API is not available
      const useMockAuth = true // Set to false when backend is ready
      
      console.log('Login attempt:', credentials)
      
      if (useMockAuth) {
        // Mock login validation
        if (credentials.username && credentials.password) {
          const mockToken = `mock-jwt-token-${  Date.now()}`
          const mockUser = {
            id: 1,
            username: credentials.username,
            full_name: credentials.username === 'admin' ? '管理员' : '用户',
            email: `${credentials.username}@example.com`,
            role: credentials.username === 'admin' ? 'admin' : 'operator',
            status: 'active',
            last_login: new Date().toISOString(),
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString()
          }
          
          token.value = mockToken
          user.value = mockUser
          
          // Store token and user in localStorage
          localStorage.setItem('token', mockToken)
          localStorage.setItem('user', JSON.stringify(mockUser))
          
          ElMessage.success('登录成功')
          return
        } else {
          throw new Error('用户名和密码不能为空')
        }
      }
      
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
      ElMessage.error(error.message || '登录失败')
      throw error
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
      // Use mock data when API is not available
      const useMockData = true // Set to false when backend is ready
      
      if (useMockData) {
        // Mock user data for development
        if (!user.value && token.value) {
          user.value = {
            id: 1,
            username: 'admin',
            full_name: '管理员',
            email: 'admin@example.com',
            role: 'admin',
            status: 'active',
            last_login: new Date().toISOString(),
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString()
          }
        }
        return
      }
      
      const response = await authApi.getUserInfo()
      if (response.success && response.data) {
        user.value = response.data as User
      }
    } catch (error) {
      console.error('Failed to fetch user info:', error)
      // Use mock data instead of clearing auth state
      if (!user.value && token.value) {
        user.value = {
          id: 1,
          username: 'admin',
          full_name: '管理员',
          email: 'admin@example.com',
          role: 'admin',
          status: 'active',
          last_login: new Date().toISOString(),
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        }
      }
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
    return user.value?.role === role
  }

  // Session management
  const sessions = ref<any[]>([])
  const sessionLoading = ref(false)

  const getSessions = async (): Promise<void> => {
    if (!user.value) return
    
    sessionLoading.value = true
    try {
      // Mock session data for development
      const mockSessions = [
        {
          id: 'session_1',
          userId: user.value.id,
          deviceInfo: {
            type: 'desktop',
            name: 'Chrome on Windows',
            browser: 'Chrome 119.0',
            os: 'Windows 10'
          },
          ip: '192.168.1.100',
          location: '北京市',
          loginTime: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
          lastActivity: new Date().toISOString(),
          status: 'active',
          current: true
        },
        {
          id: 'session_2',
          userId: user.value.id,
          deviceInfo: {
            type: 'mobile',
            name: 'Safari on iPhone',
            browser: 'Safari 17.0',
            os: 'iOS 17'
          },
          ip: '192.168.1.101',
          location: '上海市',
          loginTime: new Date(Date.now() - 12 * 60 * 60 * 1000).toISOString(),
          lastActivity: new Date(Date.now() - 3 * 60 * 60 * 1000).toISOString(),
          status: 'active',
          current: false
        }
      ]
      
      sessions.value = mockSessions
    } catch (error) {
      console.error('Failed to fetch sessions:', error)
      ElMessage.error('获取会话信息失败')
    } finally {
      sessionLoading.value = false
    }
  }

  const terminateSession = async (sessionId: string): Promise<void> => {
    try {
      // Mock terminate session
      await new Promise(resolve => setTimeout(resolve, 500))
      
      const sessionIndex = sessions.value.findIndex(s => s.id === sessionId)
      if (sessionIndex > -1) {
        sessions.value[sessionIndex].status = 'terminated'
      }
      
      ElMessage.success('会话已终止')
    } catch (error) {
      console.error('Failed to terminate session:', error)
      ElMessage.error('终止会话失败')
    }
  }

  const terminateAllOtherSessions = async (): Promise<void> => {
    try {
      // Mock terminate all other sessions
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      sessions.value = sessions.value.map(session => {
        if (!session.current) {
          return { ...session, status: 'terminated' }
        }
        return session
      })
      
      ElMessage.success('已终止其他所有会话')
    } catch (error) {
      console.error('Failed to terminate other sessions:', error)
      ElMessage.error('终止其他会话失败')
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
    try {
      const activityRecord = {
        id: `activity_${Date.now()}`,
        userId: user.value?.id,
        type: activity.type,
        action: activity.action,
        target: activity.target,
        details: activity.details,
        timestamp: new Date().toISOString(),
        ip: '192.168.1.100', // Should be detected from request
        userAgent: navigator.userAgent
      }
      
      // Add to local activities (in real app, send to server)
      activities.value.unshift(activityRecord)
      
      // Keep only last 100 activities in memory
      if (activities.value.length > 100) {
        activities.value = activities.value.slice(0, 100)
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
    activityLoading.value = true
    try {
      // Mock activity data
      const mockActivities = [
        {
          id: 'activity_1',
          userId: user.value?.id,
          type: 'auth',
          action: 'login',
          target: 'system',
          timestamp: new Date().toISOString(),
          ip: '192.168.1.100',
          userAgent: navigator.userAgent,
          details: { method: 'password' }
        },
        {
          id: 'activity_2',
          userId: user.value?.id,
          type: 'device',
          action: 'view',
          target: 'device_list',
          timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
          ip: '192.168.1.100',
          userAgent: navigator.userAgent,
          details: { page: 'devices' }
        }
      ]
      
      activities.value = mockActivities
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
    
    try {
      // Mock permissions based on role
      const rolePermissions: Record<string, string[]> = {
        admin: [
          'users:create', 'users:read', 'users:update', 'users:delete',
          'roles:create', 'roles:read', 'roles:update', 'roles:delete',
          'permissions:assign', 'permissions:revoke',
          'devices:create', 'devices:read', 'devices:update', 'devices:delete',
          'devices:control', 'devices:monitor',
          'data:read', 'data:export', 'data:import', 'data:delete',
          'alerts:create', 'alerts:read', 'alerts:update', 'alerts:delete',
          'alerts:acknowledge', 'alerts:configure',
          'system:config', 'system:logs', 'system:backup', 'system:restore',
          'reports:generate', 'reports:export'
        ],
        manager: [
          'users:read', 'users:update',
          'roles:read',
          'devices:read', 'devices:update', 'devices:control',
          'data:read', 'data:export',
          'alerts:read', 'alerts:acknowledge', 'alerts:configure',
          'reports:generate', 'reports:export'
        ],
        operator: [
          'devices:read', 'devices:control', 'devices:monitor',
          'data:read',
          'alerts:read', 'alerts:acknowledge'
        ],
        viewer: [
          'devices:read',
          'data:read',
          'alerts:read'
        ]
      }
      
      permissions.value = rolePermissions[user.value.role] || []
    } catch (error) {
      console.error('Failed to load permissions:', error)
    }
  }

  const checkPermission = (permission: string): boolean => {
    if (!user.value) return false
    
    // Admin has all permissions
    if (user.value.role === 'admin') return true
    
    // Check specific permission
    return permissions.value.includes(permission)
  }

  const checkMultiplePermissions = (requiredPermissions: string[], requireAll = true): boolean => {
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
    // 在Mock模式下自动登录
    if (import.meta.env.VITE_ENABLE_MOCK === 'true' && !token.value) {
      console.log('Auto-login in Mock mode')
      const mockToken = `mock-jwt-token-${Date.now()}`
      const mockUser = {
        id: 1,
        username: 'admin',
        full_name: '管理员',
        email: 'admin@example.com',
        role: 'admin',
        status: 'active',
        last_login: new Date().toISOString(),
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
      
      token.value = mockToken
      user.value = mockUser
      
      localStorage.setItem('token', mockToken)
      localStorage.setItem('user', JSON.stringify(mockUser))
    }
    
    if (token.value) {
      await fetchUserInfo()
      await loadUserPermissions()
      await getSessions()
      await getActivities()
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