/**
 * Gateway Rust - 认证API模块
 */

import { http } from './http'

export interface LoginRequest {
  username: string
  password: string
  remember_me?: boolean
}

export interface LoginResponse {
  access_token: string
  refresh_token?: string
  token_type: string
  expires_in: number
  user: UserInfo
}

export interface UserInfo {
  id: string
  username: string
  email?: string
  full_name?: string
  roles: string[]
  permissions: string[]
  is_active: boolean
  last_login_at?: string
  created_at: string
}

export interface RefreshTokenRequest {
  refresh_token: string
}

export interface ChangePasswordRequest {
  old_password: string
  new_password: string
  confirm_password: string
}

export interface UpdateProfileRequest {
  full_name?: string
  email?: string
  phone?: string
  avatar?: string
}

export interface ResetPasswordRequest {
  email: string
}

export interface ConfirmResetPasswordRequest {
  token: string
  new_password: string
  confirm_password: string
}

/**
 * 认证API类
 */
class AuthApi {
  /**
   * 用户登录
   */
  async login(credentials: LoginRequest): Promise<LoginResponse> {
    const response = await http.post('/auth/login', credentials)
    return response.data
  }

  /**
   * 用户登出
   */
  async logout(): Promise<void> {
    await http.post('/auth/logout')
  }

  /**
   * 刷新访问令牌
   */
  async refreshToken(request: RefreshTokenRequest): Promise<LoginResponse> {
    const response = await http.post('/auth/refresh', request)
    return response.data
  }

  /**
   * 获取当前用户信息
   */
  async getCurrentUser(): Promise<UserInfo> {
    const response = await http.get('/auth/me')
    return response.data
  }

  /**
   * 更新用户资料
   */
  async updateProfile(profile: UpdateProfileRequest): Promise<UserInfo> {
    const response = await http.put('/auth/profile', profile)
    return response.data
  }

  /**
   * 修改密码
   */
  async changePassword(request: ChangePasswordRequest): Promise<void> {
    await http.put('/auth/change-password', request)
  }

  /**
   * 请求重置密码
   */
  async requestPasswordReset(request: ResetPasswordRequest): Promise<void> {
    await http.post('/auth/reset-password', request)
  }

  /**
   * 确认重置密码
   */
  async confirmPasswordReset(request: ConfirmResetPasswordRequest): Promise<void> {
    await http.post('/auth/reset-password/confirm', request)
  }

  /**
   * 验证令牌有效性
   */
  async verifyToken(): Promise<{ valid: boolean; expires_at: string }> {
    const response = await http.get('/auth/verify')
    return response.data
  }

  /**
   * 获取用户权限列表
   */
  async getUserPermissions(): Promise<string[]> {
    const response = await http.get('/auth/permissions')
    return response.data
  }

  /**
   * 检查用户是否具有指定权限
   */
  async hasPermission(permission: string): Promise<boolean> {
    const response = await http.get(`/auth/permissions/${permission}/check`)
    return response.data.has_permission
  }

  /**
   * 获取用户角色列表
   */
  async getUserRoles(): Promise<string[]> {
    const response = await http.get('/auth/roles')
    return response.data
  }

  /**
   * 上传用户头像
   */
  async uploadAvatar(file: File): Promise<{ avatar_url: string }> {
    const formData = new FormData()
    formData.append('avatar', file)
    
    const response = await http.post('/auth/avatar', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    })
    return response.data
  }

  /**
   * 删除用户头像
   */
  async deleteAvatar(): Promise<void> {
    await http.delete('/auth/avatar')
  }

  /**
   * 获取登录历史
   */
  async getLoginHistory(params?: {
    limit?: number
    offset?: number
    start_date?: string
    end_date?: string
  }): Promise<{
    items: Array<{
      id: string
      ip_address: string
      user_agent: string
      location?: string
      login_at: string
      logout_at?: string
      status: 'success' | 'failed'
    }>
    total: number
  }> {
    const response = await http.get('/auth/login-history', { params })
    return response.data
  }

  /**
   * 获取活跃会话列表
   */
  async getActiveSessions(): Promise<Array<{
    id: string
    ip_address: string
    user_agent: string
    location?: string
    created_at: string
    expires_at: string
    is_current: boolean
  }>> {
    const response = await http.get('/auth/sessions')
    return response.data
  }

  /**
   * 终止指定会话
   */
  async terminateSession(sessionId: string): Promise<void> {
    await http.delete(`/auth/sessions/${sessionId}`)
  }

  /**
   * 终止所有其他会话
   */
  async terminateOtherSessions(): Promise<void> {
    await http.delete('/auth/sessions/others')
  }

  /**
   * 记录用户活动日志
   */
  async logActivity(activity: {
    type: string
    action: string
    target?: string
    details?: any
  }): Promise<{ success: boolean; message?: string }> {
    try {
      const response = await http.post('/auth/activity-log', activity)
      return response.data
    } catch (error) {
      console.warn('Failed to log activity:', error)
      return { success: false, message: 'Failed to log activity' }
    }
  }
}

// 创建并导出API实例
export const authApi = new AuthApi()

export default authApi