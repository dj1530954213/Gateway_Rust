/**
 * 数据点管理 API
 */

import { get, post, put, del } from './http'
import type { PaginatedResponse } from './http'

export interface DataPoint {
  id: string
  name: string
  description?: string
  address: string
  dataType: 'boolean' | 'integer' | 'float' | 'string' | 'datetime'
  driverId: string
  groupId?: string
  status: 'active' | 'inactive' | 'error'
  quality: 'good' | 'uncertain' | 'bad' | 'invalid'
  currentValue: any
  lastReadTime: Date
  unit?: string
  scaling?: {
    factor: number
    offset: number
  }
  limits?: {
    min?: number
    max?: number
  }
  createTime: Date
  updateTime: Date
  config?: any
}

export interface DataPointGroup {
  id: string
  name: string
  parentId?: string
  description?: string
  children?: DataPoint[]
  groups?: DataPointGroup[]
  createTime: Date
}

export interface DatapointsStats {
  total: number
  active: number
  goodQuality: number
  alarming: number
}

export interface AlarmRule {
  id: string
  datapointId: string
  name: string
  condition: 'gt' | 'lt' | 'eq' | 'ne' | 'range'
  value: number | string
  highValue?: number
  priority: 'low' | 'medium' | 'high' | 'critical'
  enabled: boolean
  createTime: Date
}

export interface DataPointQuery {
  page?: number
  size?: number
  keyword?: string
  status?: 'active' | 'inactive' | 'error'
  quality?: 'good' | 'uncertain' | 'bad' | 'invalid'
  dataType?: 'boolean' | 'integer' | 'float' | 'string' | 'datetime'
  driverId?: string
  groupId?: string
}

export interface DataPointCreateReq {
  name: string
  description?: string
  address: string
  dataType: 'boolean' | 'integer' | 'float' | 'string' | 'datetime'
  driverId: string
  groupId?: string
  unit?: string
  scaling?: {
    factor: number
    offset: number
  }
  limits?: {
    min?: number
    max?: number
  }
}

export interface DataPointUpdateReq extends Partial<DataPointCreateReq> {
  id: string
}

export interface HistoryQuery {
  datapointId: string
  startTime: Date
  endTime: Date
  interval?: string
}

export interface HistoryPoint {
  timestamp: Date
  value: any
  quality: string
}

export const datapointsApi = {
  /**
   * 获取数据点列表
   */
  list(params?: DataPointQuery): Promise<PaginatedResponse<DataPoint>> {
    return get('/datapoints', params)
  },

  /**
   * 获取数据点详情
   */
  get(id: string): Promise<DataPoint> {
    return get(`/datapoints/${id}`)
  },

  /**
   * 创建数据点
   */
  create(data: DataPointCreateReq): Promise<DataPoint> {
    return post('/datapoints', data)
  },

  /**
   * 更新数据点
   */
  update(id: string, data: DataPointUpdateReq): Promise<DataPoint> {
    return put(`/datapoints/${id}`, data)
  },

  /**
   * 删除数据点
   */
  delete(id: string): Promise<{ success: boolean; message?: string }> {
    return del(`/datapoints/${id}`)
  },

  /**
   * 激活数据点
   */
  activate(id: string): Promise<{ success: boolean; message?: string }> {
    return post(`/datapoints/${id}/activate`)
  },

  /**
   * 停用数据点
   */
  deactivate(id: string): Promise<{ success: boolean; message?: string }> {
    return post(`/datapoints/${id}/deactivate`)
  },

  /**
   * 批量激活数据点
   */
  batchActivate(ids: string[]): Promise<{ success: boolean; successCount: number; failedCount: number }> {
    return post('/datapoints/batch/activate', { ids })
  },

  /**
   * 批量停用数据点
   */
  batchDeactivate(ids: string[]): Promise<{ success: boolean; successCount: number; failedCount: number }> {
    return post('/datapoints/batch/deactivate', { ids })
  },

  /**
   * 批量删除数据点
   */
  batchDelete(ids: string[]): Promise<{ success: boolean; successCount: number; failedCount: number }> {
    return del('/datapoints/batch', { ids })
  },

  /**
   * 获取数据点统计信息
   */
  getStats(): Promise<DatapointsStats> {
    return get('/datapoints/stats')
  },

  /**
   * 获取数据点分组列表
   */
  getGroups(): Promise<DataPointGroup[]> {
    return get('/datapoints/groups')
  },

  /**
   * 创建数据点分组
   */
  createGroup(data: { name: string; description?: string; parentId?: string }): Promise<DataPointGroup> {
    return post('/datapoints/groups', data)
  },

  /**
   * 获取数据点历史数据
   */
  getHistory(query: HistoryQuery): Promise<HistoryPoint[]> {
    return get(`/datapoints/${query.datapointId}/history`, {
      startTime: query.startTime.toISOString(),
      endTime: query.endTime.toISOString(),
      interval: query.interval
    })
  },

  /**
   * 获取数据点报警规则
   */
  getAlarmRules(datapointId: string): Promise<AlarmRule[]> {
    return get(`/datapoints/${datapointId}/alarm-rules`)
  },

  /**
   * 创建报警规则
   */
  createAlarmRule(data: Omit<AlarmRule, 'id' | 'createTime'>): Promise<AlarmRule> {
    return post(`/datapoints/${data.datapointId}/alarm-rules`, data)
  },

  /**
   * 更新报警规则
   */
  updateAlarmRule(datapointId: string, ruleId: string, data: Partial<AlarmRule>): Promise<AlarmRule> {
    return put(`/datapoints/${datapointId}/alarm-rules/${ruleId}`, data)
  },

  /**
   * 删除报警规则
   */
  deleteAlarmRule(datapointId: string, ruleId: string): Promise<{ success: boolean }> {
    return del(`/datapoints/${datapointId}/alarm-rules/${ruleId}`)
  }
}