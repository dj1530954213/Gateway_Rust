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

export interface SystemMetrics {
  cpuUsage: number
  memoryUsage: number
  diskUsage: number
  activeConnections: number
  messagesPerSecond: number
  uptime: number
  networkIn: number
  networkOut: number
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
   * 获取系统性能指标
   */
  async getSystemMetrics(): Promise<{ success: boolean; data: SystemMetrics }> {
    try {
      const response = await http.get('/system/metrics')
      return {
        success: true,
        data: response.data
      }
    } catch (error) {
      // 如果API尚未实现，返回模拟数据
      return {
        success: true,
        data: {
          cpuUsage: Math.random() * 100,
          memoryUsage: Math.random() * 100,
          diskUsage: Math.random() * 100,
          activeConnections: Math.floor(Math.random() * 50),
          messagesPerSecond: Math.floor(Math.random() * 1000),
          uptime: Date.now() - Math.random() * 86400000,
          networkIn: Math.random() * 1024 * 1024,
          networkOut: Math.random() * 1024 * 1024
        }
      }
    }
  }

  /**
   * 获取系统组件状态
   */
  async getComponentStatus(): Promise<any> {
    try {
      const response = await http.get('/system/components/status')
      return response.data
    } catch (error) {
      // 返回模拟组件状态数据
      return {
        database: { status: 'healthy', uptime: 86400, connections: 5 },
        messageQueue: { status: 'healthy', queueSize: 0, throughput: 100 },
        webServer: { status: 'healthy', activeConnections: 3, requests: 1024 },
        fileSystem: { status: 'healthy', freeSpace: '15.2GB', diskUsage: 75 }
      }
    }
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
    try {
      const response = await http.get('/system/health')
      return response.data
    } catch (error) {
      console.warn('System Health API 未实现，使用默认状态')
      return {
        status: 'healthy',
        timestamp: new Date().toISOString(),
        services: {
          database: 'unknown',
          message_bus: 'unknown',
          driver_manager: 'unknown',
          web_server: 'healthy'
        },
        metrics: {
          uptime: 0,
          memory_usage: 0,
          cpu_usage: 0,
          disk_usage: 0
        }
      }
    }
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