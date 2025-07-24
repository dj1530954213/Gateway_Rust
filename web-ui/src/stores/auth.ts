import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import { ElMessage } from 'element-plus'
import type { User, LoginRequest, LoginResponse } from '@/types'
import { authApi } from '@/services/api'

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
          const mockToken = 'mock-jwt-token-' + Date.now()
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

  // Initialize store
  const init = async (): Promise<void> => {
    if (token.value) {
      await fetchUserInfo()
    }
  }

  return {
    // State
    token: readonly(token),
    user: readonly(user),
    loading: readonly(loading),
    
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
  }
})