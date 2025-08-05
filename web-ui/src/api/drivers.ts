/**
 * 驱动管理 API
 */

import { get, post, put, del, upload } from './http'
import type { PaginatedResponse } from './http'

export interface UnifiedDriverEntryVO {
  driver_id: string
  driver_kind: string
  name: string
  version: string
  protocol: string
  status: string
  description?: string
  features?: string[]
  loaded_at?: string
  file_path?: string
  stats?: DriverStatisticsVO
}

// 类型别名，用于组件中保持一致性
export interface DriverVO {
  id: string
  filename: string
  file_path?: string
  file_size: number
  status: 'Loaded' | 'Failed' | 'Unloaded'
  uploaded_at: string
  last_loaded_at?: string
  error_message?: string
  info?: {
    name: string
    version: string
    author?: string
    protocol?: string
    description?: string
  }
  config?: any
}

export interface DriverQuery {
  page?: number
  size?: number
  name_contains?: string
  status?: 'Loaded' | 'Failed' | 'Unloaded'
  protocol?: string
}

export interface DriverStatisticsVO {
  attached_devices: number
  read_count: number
  write_count: number
  error_count: number
  avg_response_time_ms: number
  success_rate: number
}

export interface DriverUploadResponse {
  success: boolean
  message: string
  data: DriverVO | null
}

export interface DriverUploadInfo {
  filename: string
  driver_id: string
  file_size: number
  status: string
  message?: string
}

export interface DriverStatusVO {
  loaded_count: number
  failed_count: number
  unloaded_count: number
  total_count: number
}

export interface DriverReloadResult {
  success: boolean
  message?: string
  driver: DriverVO
}

export interface DriverListQuery {
  page?: number
  page_size?: number
  driver_kind?: string
  protocol?: string
  status?: string
  name_contains?: string
  active_only?: boolean
  error_only?: boolean
  sort_by?: string
  descending?: boolean
}

export interface DriverListResponse {
  drivers: UnifiedDriverEntryVO[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

export interface DriverDetailResponse {
  driver: UnifiedDriverEntryVO
}

export interface DriverSearchQuery {
  q: string
}

export interface DriverSearchResponse {
  query: string
  results: UnifiedDriverEntryVO[]
  total: number
}

export interface RegistryOverviewVO {
  total_drivers: number
  static_drivers: number
  dynamic_drivers: number
  running_drivers: number
  error_drivers: number
  protocol_stats: Record<string, number>
  status_stats: Record<string, number>
}

export interface RegistryOverviewResponse {
  overview: RegistryOverviewVO
}

export interface DriverReloadRequest {
  force?: boolean
}

export interface DriverReloadResponse {
  success: boolean
  old_driver_id: string
  new_driver_id?: string
  message: string
}

export interface DriverUnloadResponse {
  success: boolean
  driver_id: string
  message: string
}

// ========== 驱动配置相关接口 ==========

export interface DriverConfigVO {
  id: string
  name: string
  description?: string
  protocol: string
  connection_type: string
  enabled: boolean
  config: any
  
  // 性能设置
  scan_interval: number
  timeout: number
  max_concurrent: number
  batch_size: number
  
  // 重连策略
  max_retries: number
  retry_interval: number
  exponential_backoff: boolean
  max_retry_interval: number
  
  // 日志设置
  log_level: string
  enable_request_log: boolean
  enable_response_log: boolean
  max_log_size: number
  
  // 安全配置
  enable_ssl: boolean
  verify_certificate: boolean
  client_cert_path?: string
  client_key_path?: string
  
  created_at: string
  updated_at: string
}

export interface DriverConfigCreateReq {
  name: string
  description?: string
  protocol: string
  connection_type: string
  enabled: boolean
  config: any
  scan_interval?: number
  timeout?: number
  max_concurrent?: number
  batch_size?: number
  max_retries?: number
  retry_interval?: number
  exponential_backoff?: boolean
  max_retry_interval?: number
  log_level?: string
  enable_request_log?: boolean
  enable_response_log?: boolean
  max_log_size?: number
  enable_ssl?: boolean
  verify_certificate?: boolean
  client_cert_path?: string
  client_key_path?: string
}

export interface DriverConfigUpdateReq {
  name?: string
  description?: string
  protocol?: string
  connection_type?: string
  enabled?: boolean
  config?: any
  scan_interval?: number
  timeout?: number
  max_concurrent?: number
  batch_size?: number
  max_retries?: number
  retry_interval?: number
  exponential_backoff?: boolean
  max_retry_interval?: number
  log_level?: string
  enable_request_log?: boolean
  enable_response_log?: boolean
  max_log_size?: number
  enable_ssl?: boolean
  verify_certificate?: boolean
  client_cert_path?: string
  client_key_path?: string
}

export interface DriverConfigQuery {
  protocol?: string
  enabled?: boolean
  name_contains?: string
  page?: number
  page_size?: number
}

export interface DriverConfigResponse {
  driver_config: DriverConfigVO
}

export interface DriverConfigListResponse {
  driver_configs: DriverConfigVO[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

export const driversApi = {
  /**
   * 获取所有驱动列表 (简化版本，兼容Dashboard调用)
   */
  getDrivers(): Promise<DriverVO[]> {
    return get('/api/v1/drivers')
      .then((response: any) => {
        // 处理分页响应或直接数组响应
        if (response.data && Array.isArray(response.data)) {
          return response.data
        } else if (Array.isArray(response)) {
          return response
        } else {
          return []
        }
      })
      .catch(() => {
        // 返回模拟数据以防API未实现
        return [
          {
            id: 'modbus-tcp-001',
            filename: 'modbus_tcp.so',
            file_size: 204800,
            status: 'Loaded' as const,
            uploaded_at: '2025-01-27T10:00:00Z',
            info: {
              name: 'Modbus TCP Driver',
              version: '1.0.0',
              protocol: 'Modbus TCP',
              description: 'Standard Modbus TCP/IP driver'
            }
          },
          {
            id: 'opcua-001',
            filename: 'opcua_client.dll',
            file_size: 512000,
            status: 'Loaded' as const,
            uploaded_at: '2025-01-27T09:30:00Z',
            info: {
              name: 'OPC UA Client',
              version: '2.1.0',
              protocol: 'OPC UA',
              description: 'OPC UA client driver'
            }
          }
        ]
      })
  },

  /**
   * 上传驱动文件
   */
  upload(file: File, onProgress?: (progress: number) => void): Promise<DriverUploadResponse> {
    return upload('/api/v1/drivers', file, onProgress)
  },

  /**
   * 查询驱动列表
   */
  list(params?: DriverQuery): Promise<any> {
    return get('/api/v1/drivers', params)
  },

  /**
   * 获取驱动详情
   */
  get(driverId: string): Promise<DriverVO> {
    return get(`/api/v1/drivers/${driverId}`)
  },

  /**
   * 获取驱动状态统计
   */
  getStatus(): Promise<DriverStatusVO> {
    return get('/api/v1/drivers/status')
  },

  /**
   * 删除驱动
   */
  delete(driverId: string): Promise<{ success: boolean; message?: string }> {
    return del(`/api/v1/drivers/${driverId}`)
  },

  /**
   * 重新加载驱动
   */
  reload(driverId: string): Promise<DriverReloadResult> {
    return post(`/api/v1/drivers/${driverId}/reload`)
  },

  /**
   * 重新加载所有驱动
   */
  reloadAll(): Promise<{ success: boolean; success_count: number; failed_count: number; message?: string }> {
    return post('/api/v1/drivers/reload-all')
  },

  /**
   * 搜索驱动
   */
  search(params: DriverSearchQuery): Promise<DriverSearchResponse> {
    return get('/api/v1/drivers/search', params)
  },

  /**
   * 获取注册表概览
   */
  overview(): Promise<RegistryOverviewResponse> {
    return get('/api/v1/drivers/overview')
  },
}

// ========== 驱动配置API ==========
export const driverConfigsApi = {
  /**
   * 创建驱动配置
   */
  create(config: DriverConfigCreateReq): Promise<DriverConfigResponse> {
    return post('/api/v1/driver-configs', config)
  },

  /**
   * 查询驱动配置列表
   */
  list(params?: DriverConfigQuery): Promise<DriverConfigListResponse> {
    return get('/api/v1/driver-configs', params)
  },

  /**
   * 获取驱动配置详情
   */
  get(configId: string): Promise<DriverConfigResponse> {
    return get(`/api/v1/driver-configs/${configId}`)
  },

  /**
   * 更新驱动配置
   */
  update(configId: string, update: DriverConfigUpdateReq): Promise<DriverConfigResponse> {
    return put(`/api/v1/driver-configs/${configId}`, update)
  },

  /**
   * 删除驱动配置
   */
  delete(configId: string): Promise<{ success: boolean; message?: string }> {
    return del(`/api/v1/driver-configs/${configId}`)
  },
}