/**
 * 报警管理 API
 * 
 * 注意：这些API是代理到独立的alert-engine服务
 */

import { get, post, put, del } from './http'
import type { PaginatedResponse } from './http'

export interface AlertRuleVO {
  id: string
  name: string
  description?: string
  tag_id: string
  condition: AlertCondition
  severity: AlertSeverity
  enabled: boolean
  cooldown_seconds: number
  notification_channels: string[]
  created_at: string
  updated_at: string
}

export interface AlertRuleCreateReq {
  name: string
  description?: string
  tag_id: string
  condition: AlertCondition
  severity: AlertSeverity
  enabled?: boolean
  cooldown_seconds?: number
  notification_channels?: string[]
}

export interface AlertRulePatchReq {
  name?: string
  description?: string
  tag_id?: string
  condition?: AlertCondition
  severity?: AlertSeverity
  enabled?: boolean
  cooldown_seconds?: number
  notification_channels?: string[]
}

export interface AlertCondition {
  operator: AlertOperator
  threshold: number
  duration_seconds?: number
}

export enum AlertOperator {
  GreaterThan = 'gt',
  LessThan = 'lt',
  Equal = 'eq',
  NotEqual = 'ne',
  GreaterThanOrEqual = 'gte',
  LessThanOrEqual = 'lte',
}

export enum AlertSeverity {
  Critical = 'critical',
  Major = 'major',
  Minor = 'minor',
  Warning = 'warning',
  Info = 'info',
}

export interface AlertEventVO {
  id: string
  rule_id: string
  rule_name: string
  tag_id: string
  tag_name: string
  severity: AlertSeverity
  message: string
  triggered_at: string
  resolved_at?: string
  acknowledged_at?: string
  current_value: any
  threshold_value: number
  operator: AlertOperator
}

export interface AlertRuleQuery {
  page?: number
  size?: number
  enabled?: boolean
  severity?: AlertSeverity
  tag_id?: string
  name_contains?: string
}

export interface AlertEventQuery {
  page?: number
  size?: number
  rule_id?: string
  severity?: AlertSeverity
  resolved?: boolean
  acknowledged?: boolean
  start_time?: string
  end_time?: string
}

export interface AlertStatistics {
  total_rules: number
  enabled_rules: number
  total_events: number
  unresolved_events: number
  events_by_severity: Record<AlertSeverity, number>
  events_last_24h: number
}

export const alertsApi = {
  // 报警规则管理
  rules: {
    /**
     * 创建报警规则
     */
    create(data: AlertRuleCreateReq): Promise<AlertRuleVO> {
      return post('/alerts/rules', data)
    },

    /**
     * 查询报警规则列表
     */
    list(params?: AlertRuleQuery): Promise<PaginatedResponse<AlertRuleVO>> {
      return get('/alerts/rules', params)
    },

    /**
     * 获取报警规则详情
     */
    get(id: string): Promise<AlertRuleVO> {
      return get(`/alerts/rules/${id}`)
    },

    /**
     * 更新报警规则
     */
    update(id: string, data: AlertRulePatchReq): Promise<AlertRuleVO> {
      return put(`/alerts/rules/${id}`, data)
    },

    /**
     * 删除报警规则
     */
    delete(id: string): Promise<void> {
      return del(`/alerts/rules/${id}`)
    },

    /**
     * 启用/禁用报警规则
     */
    toggle(id: string, enabled: boolean): Promise<AlertRuleVO> {
      return put(`/alerts/rules/${id}`, { enabled })
    },

    /**
     * 测试报警规则
     */
    test(data: AlertRuleCreateReq): Promise<{ success: boolean; message?: string }> {
      return post('/alerts/rules/test', data)
    },
  },

  // 报警事件管理
  events: {
    /**
     * 查询报警事件列表
     */
    list(params?: AlertEventQuery): Promise<PaginatedResponse<AlertEventVO>> {
      return get('/alerts/events', params)
    },

    /**
     * 获取报警事件详情
     */
    get(id: string): Promise<AlertEventVO> {
      return get(`/alerts/events/${id}`)
    },

    /**
     * 确认报警事件
     */
    acknowledge(id: string, comment?: string): Promise<void> {
      return post(`/alerts/events/${id}/acknowledge`, { comment })
    },

    /**
     * 解决报警事件
     */
    resolve(id: string, comment?: string): Promise<void> {
      return post(`/alerts/events/${id}/resolve`, { comment })
    },

    /**
     * 批量确认报警事件
     */
    batchAcknowledge(ids: string[], comment?: string): Promise<{ success: number; failed: number }> {
      return post('/alerts/events/batch/acknowledge', { ids, comment })
    },

    /**
     * 批量解决报警事件
     */
    batchResolve(ids: string[], comment?: string): Promise<{ success: number; failed: number }> {
      return post('/alerts/events/batch/resolve', { ids, comment })
    },
  },

  // 统计信息
  statistics: {
    /**
     * 获取报警统计信息
     */
    get(): Promise<AlertStatistics> {
      return get('/alerts/statistics')
    },

    /**
     * 获取报警趋势数据
     */
    getTrends(period: '1h' | '24h' | '7d' | '30d'): Promise<{
      period: string
      data: Array<{
        timestamp: string
        total: number
        by_severity: Record<AlertSeverity, number>
      }>
    }> {
      return get('/alerts/statistics/trends', { period })
    },
  },

  // 通知通道管理
  notifications: {
    /**
     * 获取通知通道列表
     */
    getChannels(): Promise<Array<{
      id: string
      name: string
      type: 'email' | 'webhook' | 'websocket'
      config: any
      enabled: boolean
    }>> {
      return get('/alerts/notifications/channels')
    },

    /**
     * 测试通知通道
     */
    testChannel(channelId: string): Promise<{ success: boolean; message?: string }> {
      return post(`/alerts/notifications/channels/${channelId}/test`)
    },
  },
}