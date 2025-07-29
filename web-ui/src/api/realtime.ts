/**
 * Gateway Rust - 实时数据API模块
 */

import { http } from './http'
import { wsClient } from './index'

export interface RealtimeDataPoint {
  tag_id: string
  device_id: string
  tag_name: string
  value: any
  quality: 'good' | 'bad' | 'uncertain'
  timestamp: string
  unit?: string
}

export interface RealtimeSubscription {
  subscription_id: string
  device_ids?: string[]
  tag_ids?: string[]
  sample_rate?: number // 采样率，毫秒
  filter?: {
    quality?: string[]
    value_range?: { min: number; max: number }
  }
}

export interface DeviceStatus {
  device_id: string
  device_name: string
  status: 'online' | 'offline' | 'error' | 'maintenance'
  connection_time?: string
  last_data_time?: string
  data_points_count: number
  error_message?: string
  protocol: string
  ip_address?: string
}

export interface TagValue {
  tag_id: string
  device_id: string
  tag_name: string
  value: any
  quality: string
  timestamp: string
  unit?: string
  description?: string
}

export interface RealtimeAlert {
  alert_id: string
  rule_name: string
  device_name: string
  tag_name: string
  current_value: any
  threshold_value: any
  condition: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  message: string
  triggered_at: string
  acknowledged: boolean
}

export interface ConnectionStatistics {
  total_devices: number
  online_devices: number
  offline_devices: number
  error_devices: number
  total_tags: number
  active_tags: number
  data_rate: number // 数据点/秒
  error_rate: number // 错误率 %
  avg_response_time: number // 平均响应时间 ms
}

/**
 * 实时数据API类
 */
class RealtimeApi {
  private subscriptions: Map<string, RealtimeSubscription> = new Map()

  /**
   * 获取当前所有设备状态
   */
  async getDeviceStatuses(): Promise<DeviceStatus[]> {
    const response = await http.get('/realtime/devices/status')
    return response.data
  }

  /**
   * 获取指定设备状态
   */
  async getDeviceStatus(deviceId: string): Promise<DeviceStatus> {
    const response = await http.get(`/realtime/devices/${deviceId}/status`)
    return response.data
  }

  /**
   * 获取当前标签值
   */
  async getCurrentTagValues(deviceId?: string, tagIds?: string[]): Promise<TagValue[]> {
    const params: any = {}
    if (deviceId) params.device_id = deviceId
    if (tagIds && tagIds.length > 0) params.tag_ids = tagIds.join(',')

    const response = await http.get('/realtime/tags/current', { params })
    return response.data
  }

  /**
   * 获取指定标签的当前值
   */
  async getTagValue(tagId: string): Promise<TagValue> {
    const response = await http.get(`/realtime/tags/${tagId}/value`)
    return response.data
  }

  /**
   * 写入标签值（对于可写标签）
   */
  async writeTagValue(tagId: string, value: any): Promise<void> {
    await http.put(`/realtime/tags/${tagId}/value`, { value })
  }

  /**
   * 批量写入标签值
   */
  async writeTagValues(values: Array<{ tag_id: string; value: any }>): Promise<void> {
    await http.put('/realtime/tags/values', { values })
  }

  /**
   * 获取实时报警
   */
  async getActiveAlerts(): Promise<RealtimeAlert[]> {
    const response = await http.get('/realtime/alerts')
    return response.data
  }

  /**
   * 确认报警
   */
  async acknowledgeAlert(alertId: string, message?: string): Promise<void> {
    await http.post(`/realtime/alerts/${alertId}/acknowledge`, { 
      message 
    })
  }

  /**
   * 获取连接统计信息
   */
  async getConnectionStatistics(): Promise<ConnectionStatistics> {
    const response = await http.get('/realtime/statistics')
    return response.data
  }

  /**
   * 创建实时数据订阅
   */
  async subscribe(subscription: Omit<RealtimeSubscription, 'subscription_id'>): Promise<string> {
    const subscriptionId = `sub_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    
    const fullSubscription: RealtimeSubscription = {
      subscription_id: subscriptionId,
      ...subscription
    }

    // 发送订阅请求到WebSocket
    wsClient.send('subscribe', fullSubscription)
    
    // 保存订阅信息
    this.subscriptions.set(subscriptionId, fullSubscription)
    
    return subscriptionId
  }

  /**
   * 取消实时数据订阅
   */
  async unsubscribe(subscriptionId: string): Promise<void> {
    // 发送取消订阅请求到WebSocket
    wsClient.send('unsubscribe', { subscription_id: subscriptionId })
    
    // 删除本地订阅信息
    this.subscriptions.delete(subscriptionId)
  }

  /**
   * 订阅设备状态变化
   */
  onDeviceStatusChange(callback: (status: DeviceStatus) => void): void {
    wsClient.on('device_status', callback)
  }

  /**
   * 取消设备状态变化订阅
   */
  offDeviceStatusChange(callback?: (status: DeviceStatus) => void): void {
    wsClient.off('device_status', callback)
  }

  /**
   * 订阅标签数据变化
   */
  onTagDataChange(callback: (data: RealtimeDataPoint) => void): void {
    wsClient.on('tag_data', callback)
  }

  /**
   * 取消标签数据变化订阅
   */
  offTagDataChange(callback?: (data: RealtimeDataPoint) => void): void {
    wsClient.off('tag_data', callback)
  }

  /**
   * 订阅实时报警
   */
  onAlertTriggered(callback: (alert: RealtimeAlert) => void): void {
    wsClient.on('alert_triggered', callback)
  }

  /**
   * 取消实时报警订阅
   */
  offAlertTriggered(callback?: (alert: RealtimeAlert) => void): void {
    wsClient.off('alert_triggered', callback)
  }

  /**
   * 订阅连接统计变化
   */
  onStatisticsUpdate(callback: (stats: ConnectionStatistics) => void): void {
    wsClient.on('statistics_update', callback)
  }

  /**
   * 取消连接统计变化订阅
   */
  offStatisticsUpdate(callback?: (stats: ConnectionStatistics) => void): void {
    wsClient.off('statistics_update', callback)
  }

  /**
   * 获取所有活跃订阅
   */
  getActiveSubscriptions(): RealtimeSubscription[] {
    return Array.from(this.subscriptions.values())
  }

  /**
   * 获取指定订阅信息
   */
  getSubscription(subscriptionId: string): RealtimeSubscription | undefined {
    return this.subscriptions.get(subscriptionId)
  }

  /**
   * 清除所有订阅
   */
  clearAllSubscriptions(): void {
    for (const subscriptionId of this.subscriptions.keys()) {
      this.unsubscribe(subscriptionId)
    }
  }

  /**
   * 刷新设备连接
   */
  async refreshDeviceConnection(deviceId: string): Promise<void> {
    await http.post(`/realtime/devices/${deviceId}/refresh`)
  }

  /**
   * 重启设备连接
   */
  async restartDeviceConnection(deviceId: string): Promise<void> {
    await http.post(`/realtime/devices/${deviceId}/restart`)
  }

  /**
   * 测试设备连通性
   */
  async testDeviceConnection(deviceId: string): Promise<{
    success: boolean
    response_time: number
    error_message?: string
  }> {
    const response = await http.post(`/realtime/devices/${deviceId}/test`)
    return response.data
  }
}

// 创建并导出API实例
export const realtimeApi = new RealtimeApi()

export default realtimeApi