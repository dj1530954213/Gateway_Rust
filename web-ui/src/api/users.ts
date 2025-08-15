/**
 * 用户管理 API
 */

import { http } from './http'

export interface User {
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
  updatedAt?: string
  loginCount: number
  permissions: string[]
  recentActivities: Array<{
    id: string
    action: string
    timestamp: string
    result: 'success' | 'failed'
  }>
}

export interface UserCreateReq {
  username: string
  email: string
  password: string
  role: string
  department: string
  status: string
  realName?: string
  phone?: string
  note?: string
}

export interface UserUpdateReq {
  username?: string
  email?: string
  role?: string
  department?: string
  status?: string
  realName?: string
  phone?: string
  note?: string
}

export interface UserQuery {
  page?: number
  size?: number
  search?: string
  role?: string
  status?: string
  sort_by?: string
  sort_order?: 'asc' | 'desc'
}

export interface UserStats {
  totalUsers: number
  activeUsers: number
  onlineUsers: number
  adminUsers: number
}

export interface BatchUserOperation {
  userIds: string[]
  operation: 'enable' | 'disable' | 'delete' | 'changeRole'
  newRole?: string
}

class UsersApi {
  /**
   * 获取用户列表
   */
  async list(params?: UserQuery) {
    return http.get('/api/v1/users', { params })
  }

  /**
   * 获取单个用户详情
   */
  async get(id: string) {
    return http.get(`/api/v1/users/${id}`)
  }

  /**
   * 创建用户
   */
  async create(data: UserCreateReq) {
    return http.post('/api/v1/users', data)
  }

  /**
   * 更新用户
   */
  async update(id: string, data: UserUpdateReq) {
    return http.put(`/api/v1/users/${id}`, data)
  }

  /**
   * 删除用户
   */
  async delete(id: string) {
    return http.delete(`/api/v1/users/${id}`)
  }

  /**
   * 批量删除用户
   */
  async batchDelete(userIds: string[]) {
    return http.post('/api/v1/users/batch/delete', { userIds })
  }

  /**
   * 批量操作用户
   */
  async batchOperation(data: BatchUserOperation) {
    return http.post('/api/v1/users/batch/operation', data)
  }

  /**
   * 切换用户状态
   */
  async toggleStatus(id: string, status: 'active' | 'disabled') {
    return http.put(`/api/v1/users/${id}/status`, { status })
  }

  /**
   * 重置用户密码
   */
  async resetPassword(id: string) {
    return http.post(`/api/v1/users/${id}/reset-password`)
  }

  /**
   * 获取用户统计信息
   */
  async getStats(): Promise<{ data: UserStats }> {
    return http.get('/api/v1/users/stats')
  }

  /**
   * 获取用户权限
   */
  async getPermissions(id: string) {
    return http.get(`/api/v1/users/${id}/permissions`)
  }

  /**
   * 更新用户权限
   */
  async updatePermissions(id: string, permissions: string[]) {
    return http.put(`/api/v1/users/${id}/permissions`, { permissions })
  }

  /**
   * 获取用户会话信息
   */
  async getSessions(id: string) {
    return http.get(`/api/v1/users/${id}/sessions`)
  }

  /**
   * 踢出用户会话
   */
  async kickSession(id: string, sessionId: string) {
    return http.delete(`/api/v1/users/${id}/sessions/${sessionId}`)
  }

  /**
   * 获取用户活动记录
   */
  async getActivities(id: string, params?: { page?: number; size?: number }) {
    return http.get(`/api/v1/users/${id}/activities`, { params })
  }

  /**
   * 导出用户数据
   */
  async export(params?: UserQuery) {
    return http.get('/api/v1/users/export', {
      params,
      responseType: 'blob',
    })
  }
}

export const usersApi = new UsersApi()
export default usersApi
