/**
 * 认证相关类型定义
 */

export interface User {
  id: number
  username: string
  full_name: string
  email: string
  role: 'admin' | 'operator' | 'viewer'
  status: 'active' | 'inactive'
  last_login?: string
  created_at: string
  updated_at: string
}

export interface LoginRequest {
  username: string
  password: string
  remember?: boolean
}

export interface LoginResponse {
  token: string
  user: User
  expires_in: number
}

export interface UpdatePasswordRequest {
  current_password: string
  new_password: string
}

export interface UpdateProfileRequest {
  full_name?: string
  email?: string
}
