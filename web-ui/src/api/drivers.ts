/**
 * 驱动管理 API
 */

import { get, post, del, upload } from './http'
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

export const driversApi = {
  /**
   * 上传驱动文件
   */
  upload(file: File, onProgress?: (progress: number) => void): Promise<DriverUploadResponse> {
    return upload('/drivers', file, onProgress)
  },

  /**
   * 查询驱动列表
   */
  list(params?: DriverQuery): Promise<PaginatedResponse<DriverVO>> {
    return get('/drivers', params)
  },

  /**
   * 获取驱动详情
   */
  get(driverId: string): Promise<DriverVO> {
    return get(`/drivers/${driverId}`)
  },

  /**
   * 获取驱动状态统计
   */
  getStatus(): Promise<DriverStatusVO> {
    return get('/drivers/status')
  },

  /**
   * 删除驱动
   */
  delete(driverId: string): Promise<{ success: boolean; message?: string }> {
    return del(`/drivers/${driverId}`)
  },

  /**
   * 重新加载驱动
   */
  reload(driverId: string): Promise<DriverReloadResult> {
    return post(`/drivers/${driverId}/reload`)
  },

  /**
   * 重新加载所有驱动
   */
  reloadAll(): Promise<{ success: boolean; success_count: number; failed_count: number; message?: string }> {
    return post('/drivers/reload-all')
  },

  /**
   * 搜索驱动
   */
  search(params: DriverSearchQuery): Promise<DriverSearchResponse> {
    return get('/drivers/search', params)
  },

  /**
   * 获取注册表概览
   */
  overview(): Promise<RegistryOverviewResponse> {
    return get('/drivers/overview')
  },
}