/**
 * Gateway Rust - 系统API模块
 */

import { http } from './http'

export interface SystemInfo {
  version: string
  uptime: number
  memory_usage: number
  cpu_usage: number
  disk_usage: number
  connected_devices: number
  active_tags: number
  alert_count: number
}

export interface SystemConfig {
  general: {
    instance_id: string
    environment: string
    enable_debug: boolean
  }
  network: {
    rest_api_bind: string
    web_ui_bind: string
    metrics_bind: string
    websocket_bind: string
  }
  database: {
    influxdb: {
      url: string
      username: string
      org: string
      bucket: string
    }
    postgres: {
      host: string
      port: number
      database: string
      username: string
    }
  }
  mqtt: {
    broker_host: string
    broker_port: number
    client_id: string
    username: string
  }
  monitoring: {
    prometheus_url: string
    grafana_url: string
    enable_metrics: boolean
    metrics_interval: number
  }
  logging: {
    level: string
    format: string
    output: string
  }
}

export interface SystemHealthStatus {
  status: 'healthy' | 'warning' | 'critical'
  components: {
    [key: string]: {
      status: 'healthy' | 'warning' | 'critical'
      message?: string
      last_check: string
    }
  }
  uptime: number
  version: string
}

export interface SystemMetrics {
  timestamp: string
  cpu_usage: number
  memory_usage: number
  disk_usage: number
  network_io: {
    bytes_in: number
    bytes_out: number
  }
  active_connections: number
  request_rate: number
  error_rate: number
}

export interface BackupInfo {
  id: string
  name: string
  type: 'full' | 'incremental'
  size: number
  created_at: string
  status: 'completed' | 'failed' | 'in_progress'
  file_path?: string
}

export interface LogEntry {
  timestamp: string
  level: 'debug' | 'info' | 'warn' | 'error'
  module: string
  message: string
  metadata?: Record<string, any>
}

/**
 * 系统API类
 */
class SystemApi {
  /**
   * 获取系统信息
   */
  async getInfo(): Promise<SystemInfo> {
    const response = await http.get('/system/info')
    return response.data
  }

  /**
   * 获取系统配置
   */
  async getConfig(): Promise<SystemConfig> {
    const response = await http.get('/system/config')
    return response.data
  }

  /**
   * 更新系统配置
   */
  async updateConfig(config: Partial<SystemConfig>): Promise<void> {
    await http.put('/system/config', config)
  }

  /**
   * 获取系统健康状态
   */
  async getHealth(): Promise<SystemHealthStatus> {
    const response = await http.get('/system/health')
    return response.data
  }

  /**
   * 获取系统指标
   */
  async getMetrics(timeRange?: { start: string; end: string }): Promise<SystemMetrics[]> {
    const params = timeRange ? { 
      start: timeRange.start, 
      end: timeRange.end 
    } : {}
    
    const response = await http.get('/system/metrics', { params })
    return response.data
  }

  /**
   * 获取系统日志
   */
  async getLogs(params?: {
    level?: string
    module?: string
    limit?: number
    offset?: number
    start_time?: string
    end_time?: string
  }): Promise<{ logs: LogEntry[]; total: number }> {
    const response = await http.get('/system/logs', { params })
    return response.data
  }

  /**
   * 清理系统日志
   */
  async clearLogs(before?: string): Promise<void> {
    await http.delete('/system/logs', { 
      params: before ? { before } : {} 
    })
  }

  /**
   * 创建系统备份
   */
  async createBackup(type: 'full' | 'incremental' = 'full'): Promise<BackupInfo> {
    const response = await http.post('/system/backup', { type })
    return response.data
  }

  /**
   * 获取备份列表
   */
  async getBackups(): Promise<BackupInfo[]> {
    const response = await http.get('/system/backups')
    return response.data
  }

  /**
   * 恢复系统备份
   */
  async restoreBackup(backupId: string): Promise<void> {
    await http.post(`/system/backup/${backupId}/restore`)
  }

  /**
   * 删除系统备份
   */
  async deleteBackup(backupId: string): Promise<void> {
    await http.delete(`/system/backup/${backupId}`)
  }

  /**
   * 重启系统
   */
  async restart(): Promise<void> {
    await http.post('/system/restart')
  }

  /**
   * 关闭系统
   */
  async shutdown(): Promise<void> {
    await http.post('/system/shutdown')
  }

  /**
   * 检查系统更新
   */
  async checkUpdate(): Promise<{
    available: boolean
    current_version: string
    latest_version?: string
    release_notes?: string
  }> {
    const response = await http.get('/system/update/check')
    return response.data
  }

  /**
   * 执行系统更新
   */
  async performUpdate(): Promise<void> {
    await http.post('/system/update/perform')
  }
}

// 创建并导出API实例
export const systemApi = new SystemApi()

export default systemApi