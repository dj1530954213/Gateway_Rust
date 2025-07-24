import axios from 'axios'
import type { AxiosInstance, AxiosRequestConfig } from 'axios'
import { ElMessage } from 'element-plus'
import type { 
  ApiResponse, 
  LoginRequest, 
  User, 
  SystemInfo, 
  SystemMetrics,
  HealthStatus,
  GatewayConfig,
  Driver,
  Connector,
  DataPoint,
  Alert,
  AlertRule,
  PaginationParams 
} from '@/types'

// Create axios instance
const api: AxiosInstance = axios.create({
  baseURL: '/api/v1',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor to add auth token
api.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor to handle errors
api.interceptors.response.use(
  (response) => {
    return response
  },
  (error) => {
    if (error.response?.status === 401) {
      // Clear auth state and redirect to login
      localStorage.removeItem('token')
      window.location.href = '/login'
    } else if (error.response?.status >= 500) {
      ElMessage.error('服务器错误，请稍后重试')
    } else if (error.response?.data?.message) {
      ElMessage.error(error.response.data.message)
    }
    return Promise.reject(error)
  }
)

// Generic API request wrapper
async function request<T = any>(config: AxiosRequestConfig): Promise<ApiResponse<T>> {
  try {
    const response = await api(config)
    return response.data
  } catch (error: any) {
    throw new Error(error.response?.data?.message || error.message || 'Request failed')
  }
}

// Auth API
export const authApi = {
  login: (credentials: LoginRequest) =>
    request({
      method: 'POST',
      url: '/auth/login',
      data: credentials,
    }),

  logout: () =>
    request({
      method: 'POST',
      url: '/auth/logout',
    }),

  getUserInfo: () =>
    request<User>({
      method: 'GET',
      url: '/auth/user',
    }),

  updatePassword: (data: { current_password: string; new_password: string }) =>
    request({
      method: 'PUT',
      url: '/auth/password',
      data,
    }),

  updateProfile: (data: Partial<User>) =>
    request<User>({
      method: 'PUT',
      url: '/auth/profile',
      data,
    }),
}

// System API
export const systemApi = {
  getSystemInfo: () =>
    request<SystemInfo>({
      method: 'GET',
      url: '/system/info',
    }),

  getSystemMetrics: () =>
    request<SystemMetrics>({
      method: 'GET',
      url: '/system/metrics',
    }),

  getHealthStatus: () =>
    request<HealthStatus>({
      method: 'GET',
      url: '/system/health',
    }),

  getGatewayConfig: () =>
    request<GatewayConfig>({
      method: 'GET',
      url: '/system/config',
    }),

  updateGatewayConfig: (config: Partial<GatewayConfig>) =>
    request<GatewayConfig>({
      method: 'PUT',
      url: '/system/config',
      data: config,
    }),

  restartSystem: () =>
    request({
      method: 'POST',
      url: '/system/restart',
    }),

  backupConfig: (): Promise<Blob> =>
    api.get('/system/config/backup', { responseType: 'blob' }).then(res => res.data),

  restoreConfig: (file: File) => {
    const formData = new FormData()
    formData.append('file', file)
    return request({
      method: 'POST',
      url: '/system/config/restore',
      data: formData,
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  },

  getLogs: (params: { level?: string; limit?: number; offset?: number }) =>
    request({
      method: 'GET',
      url: '/system/logs',
      params,
    }),
}

// Drivers API
export const driversApi = {
  getDrivers: (params?: PaginationParams & { status?: string; type?: string }) =>
    request<{ items: Driver[]; total: number }>({
      method: 'GET',
      url: '/drivers',
      params,
    }),

  getDriver: (id: string) =>
    request<Driver>({
      method: 'GET',
      url: `/drivers/${id}`,
    }),

  createDriver: (data: Partial<Driver>) =>
    request<Driver>({
      method: 'POST',
      url: '/drivers',
      data,
    }),

  updateDriver: (id: string, data: Partial<Driver>) =>
    request<Driver>({
      method: 'PUT',
      url: `/drivers/${id}`,
      data,
    }),

  deleteDriver: (id: string) =>
    request({
      method: 'DELETE',
      url: `/drivers/${id}`,
    }),

  startDriver: (id: string) =>
    request({
      method: 'POST',
      url: `/drivers/${id}/start`,
    }),

  stopDriver: (id: string) =>
    request({
      method: 'POST',
      url: `/drivers/${id}/stop`,
    }),

  testConnection: (config: any) =>
    request({
      method: 'POST',
      url: '/drivers/test-connection',
      data: config,
    }),
}

// Connectors API
export const connectorsApi = {
  getConnectors: (params?: PaginationParams & { status?: string; type?: string }) =>
    request<{ items: Connector[]; total: number }>({
      method: 'GET',
      url: '/connectors',
      params,
    }),

  getConnector: (id: string) =>
    request<Connector>({
      method: 'GET',
      url: `/connectors/${id}`,
    }),

  createConnector: (data: Partial<Connector>) =>
    request<Connector>({
      method: 'POST',
      url: '/connectors',
      data,
    }),

  updateConnector: (id: string, data: Partial<Connector>) =>
    request<Connector>({
      method: 'PUT',
      url: `/connectors/${id}`,
      data,
    }),

  deleteConnector: (id: string) =>
    request({
      method: 'DELETE',
      url: `/connectors/${id}`,
    }),

  startConnector: (id: string) =>
    request({
      method: 'POST',
      url: `/connectors/${id}/start`,
    }),

  stopConnector: (id: string) =>
    request({
      method: 'POST',
      url: `/connectors/${id}/stop`,
    }),

  testConnection: (config: any) =>
    request({
      method: 'POST',
      url: '/connectors/test-connection',
      data: config,
    }),
}

// Data Points API
export const dataPointsApi = {
  getDataPoints: (params?: PaginationParams & { driver_id?: string; type?: string }) =>
    request<{ items: DataPoint[]; total: number }>({
      method: 'GET',
      url: '/data-points',
      params,
    }),

  getDataPoint: (id: string) =>
    request<DataPoint>({
      method: 'GET',
      url: `/data-points/${id}`,
    }),

  createDataPoint: (data: Partial<DataPoint>) =>
    request<DataPoint>({
      method: 'POST',
      url: '/data-points',
      data,
    }),

  updateDataPoint: (id: string, data: Partial<DataPoint>) =>
    request<DataPoint>({
      method: 'PUT',
      url: `/data-points/${id}`,
      data,
    }),

  deleteDataPoint: (id: string) =>
    request({
      method: 'DELETE',
      url: `/data-points/${id}`,
    }),

  bulkImport: (file: File) => {
    const formData = new FormData()
    formData.append('file', file)
    return request({
      method: 'POST',
      url: '/data-points/import',
      data: formData,
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  },

  bulkExport: (params?: { driver_id?: string; format?: 'csv' | 'xlsx' }): Promise<Blob> =>
    api.get('/data-points/export', { 
      params, 
      responseType: 'blob' 
    }).then(res => res.data),
}

// Realtime Data API
export const realtimeApi = {
  getCurrentValues: (pointIds?: string[]) =>
    request({
      method: 'GET',
      url: '/realtime/current',
      params: pointIds ? { point_ids: pointIds.join(',') } : undefined,
    }),

  getHistoricalData: (params: {
    point_ids: string[]
    start_time: string
    end_time: string
    interval?: string
  }) =>
    request({
      method: 'GET',
      url: '/realtime/history',
      params,
    }),

  writeDataPoint: (pointId: string, value: any) =>
    request({
      method: 'POST',
      url: `/realtime/write/${pointId}`,
      data: { value },
    }),
}

// Alerts API
export const alertsApi = {
  getAlerts: (params?: PaginationParams & { 
    level?: string
    acknowledged?: boolean
    resolved?: boolean 
  }) =>
    request<{ items: Alert[]; total: number }>({
      method: 'GET',
      url: '/alerts',
      params,
    }),

  getAlert: (id: string) =>
    request<Alert>({
      method: 'GET',
      url: `/alerts/${id}`,
    }),

  acknowledgeAlert: (id: string) =>
    request({
      method: 'POST',
      url: `/alerts/${id}/acknowledge`,
    }),

  resolveAlert: (id: string) =>
    request({
      method: 'POST',
      url: `/alerts/${id}/resolve`,
    }),

  getAlertRules: () =>
    request<AlertRule[]>({
      method: 'GET',
      url: '/alert-rules',
    }),

  createAlertRule: (data: Partial<AlertRule>) =>
    request<AlertRule>({
      method: 'POST',
      url: '/alert-rules',
      data,
    }),

  updateAlertRule: (id: string, data: Partial<AlertRule>) =>
    request<AlertRule>({
      method: 'PUT',
      url: `/alert-rules/${id}`,
      data,
    }),

  deleteAlertRule: (id: string) =>
    request({
      method: 'DELETE',
      url: `/alert-rules/${id}`,
    }),
}

// Analytics API
export const analyticsApi = {
  getStatistics: (params?: { period?: string }) =>
    request({
      method: 'GET',
      url: '/analytics/statistics',
      params,
    }),

  getDataTrends: (params: {
    point_ids: string[]
    start_time: string
    end_time: string
    aggregation?: 'avg' | 'min' | 'max' | 'sum'
  }) =>
    request({
      method: 'GET',
      url: '/analytics/trends',
      params,
    }),

  getPerformanceMetrics: (params?: { period?: string }) =>
    request({
      method: 'GET',
      url: '/analytics/performance',
      params,
    }),
}

// Users API (Admin only)
export const usersApi = {
  getUsers: (params?: PaginationParams) =>
    request<{ items: User[]; total: number }>({
      method: 'GET',
      url: '/users',
      params,
    }),

  getUser: (id: string) =>
    request<User>({
      method: 'GET',
      url: `/users/${id}`,
    }),

  createUser: (data: Partial<User> & { password: string }) =>
    request<User>({
      method: 'POST',
      url: '/users',
      data,
    }),

  updateUser: (id: string, data: Partial<User>) =>
    request<User>({
      method: 'PUT',
      url: `/users/${id}`,
      data,
    }),

  deleteUser: (id: string) =>
    request({
      method: 'DELETE',
      url: `/users/${id}`,
    }),

  resetPassword: (id: string, newPassword: string) =>
    request({
      method: 'POST',
      url: `/users/${id}/reset-password`,
      data: { password: newPassword },
    }),
}

export default api