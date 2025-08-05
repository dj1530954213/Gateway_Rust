/**
 * 系统日志 API
 */

import { http } from './http'

export interface SystemLog {
  id: string
  timestamp: string
  level: 'error' | 'warn' | 'info' | 'debug' | 'trace'
  source: string
  thread: string
  message: string
  details?: {
    file?: string
    line?: number
    function?: string
    context?: Record<string, any>
  }
}

export interface LogQuery {
  page?: number
  size?: number
  search?: string
  level?: string
  source?: string
  timeRange?: [string, string]
  sort_by?: string
  sort_order?: 'asc' | 'desc'
}

export interface LogStats {
  todayCount: number
  todayTrend: number
  errorCount: number
  errorTrend: number
  warningCount: number
  warningTrend: number
  storageSize: number
  storageTrend: number
}

export interface LogSettings {
  enabledLevels: string[]
  refreshInterval: number
  maxLogLines: number
  retentionPeriod: number
  rotationSize: string
  enableArchiving: boolean
}

export interface LogSource {
  label: string
  value: string
}

class SystemLogsApi {
  /**
   * 获取系统日志列表
   */
  async list(params?: LogQuery) {
    return http.get('/api/v1/system/logs', { params })
  }

  /**
   * 获取日志统计信息
   */
  async getStats(): Promise<{ data: LogStats }> {
    return http.get('/api/v1/system/logs/stats')
  }

  /**
   * 获取日志来源列表
   */
  async getSources(): Promise<{ data: LogSource[] }> {
    return http.get('/api/v1/system/logs/sources')
  }

  /**
   * 清空所有日志
   */
  async clearAll() {
    return http.delete('/api/v1/system/logs')
  }

  /**
   * 导出日志
   */
  async export(params?: LogQuery) {
    return http.get('/api/v1/system/logs/export', {
      params,
      responseType: 'blob'
    })
  }

  /**
   * 获取日志设置
   */
  async getSettings(): Promise<{ data: LogSettings }> {
    return http.get('/api/v1/system/logs/settings')
  }

  /**
   * 更新日志设置
   */
  async updateSettings(settings: LogSettings) {
    return http.put('/api/v1/system/logs/settings', settings)
  }

  /**
   * 获取实时日志流
   */
  getRealtimeStream() {
    // 这里应该返回一个 WebSocket 连接或 Server-Sent Events 连接
    // 简化实现，返回一个模拟的 EventSource 类似对象
    return {
      onMessage: (callback: (log: SystemLog) => void) => {
        // 在真实实现中，这里会设置 WebSocket 或 SSE 监听器
        console.log('设置实时日志监听器', callback)
      },
      close: () => {
        console.log('关闭实时日志流')
      }
    }
  }

  /**
   * 搜索日志
   */
  async search(query: string, params?: Omit<LogQuery, 'search'>) {
    return http.get('/api/v1/system/logs/search', {
      params: { ...params, q: query }
    })
  }

  /**
   * 获取日志详情
   */
  async getDetail(id: string) {
    return http.get(`/api/v1/system/logs/${id}`)
  }

  /**
   * 按时间范围获取日志
   */
  async getByTimeRange(startTime: string, endTime: string, params?: Omit<LogQuery, 'timeRange'>) {
    return http.get('/api/v1/system/logs/range', {
      params: { ...params, start_time: startTime, end_time: endTime }
    })
  }

  /**
   * 获取日志存储使用情况
   */
  async getStorageUsage() {
    return http.get('/api/v1/system/logs/storage')
  }

  /**
   * 执行日志清理任务
   */
  async cleanup(olderThan: string) {
    return http.post('/api/v1/system/logs/cleanup', { older_than: olderThan })
  }

  /**
   * 获取日志文件列表
   */
  async getLogFiles() {
    return http.get('/api/v1/system/logs/files')
  }

  /**
   * 下载日志文件
   */
  async downloadFile(filename: string) {
    return http.get(`/api/v1/system/logs/files/${filename}`, {
      responseType: 'blob'
    })
  }
}

export const systemLogsApi = new SystemLogsApi()
export default systemLogsApi