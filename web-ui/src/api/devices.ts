/**
 * 设备管理 API
 */

import { get, post, put, del } from './http'
import type { PaginatedResponse } from './http'

export interface DeviceVO {
  id: string
  name: string
  protocol: ProtocolKind
  location?: string
  endpoint?: string
  config?: any
  enabled: boolean
  created_at: string
  updated_at: string
}

export interface DeviceCreateReq {
  name: string
  protocol: ProtocolKind
  location?: string
  endpoint?: string
  config?: any
  enabled?: boolean
}

export interface DevicePatchReq {
  name?: string
  location?: string
  endpoint?: string
  config?: any
  enabled?: boolean
}

export interface DeviceQuery {
  page?: number
  size?: number
  protocol?: ProtocolKind
  enabled?: boolean
  name_contains?: string
}

export enum ProtocolKind {
  ModbusTcp = 'ModbusTcp',
  OpcUa = 'OpcUa',
  Mqtt = 'Mqtt',
}

export const devicesApi = {
  /**
   * 创建设备
   */
  create(data: DeviceCreateReq): Promise<DeviceVO> {
    return post('/devices', data)
  },

  /**
   * 查询设备列表
   */
  list(params?: DeviceQuery): Promise<PaginatedResponse<DeviceVO>> {
    return get('/devices', params)
  },

  /**
   * 获取设备详情
   */
  get(id: string): Promise<DeviceVO> {
    return get(`/devices/${id}`)
  },

  /**
   * 更新设备
   */
  update(id: string, data: DevicePatchReq): Promise<DeviceVO> {
    return put(`/devices/${id}`, data)
  },

  /**
   * 删除设备
   */
  delete(id: string): Promise<void> {
    return del(`/devices/${id}`)
  },

  /**
   * 测试设备连接
   */
  testConnection(data: DeviceCreateReq): Promise<{ success: boolean; message?: string }> {
    return post('/devices/test-connection', data)
  },
}