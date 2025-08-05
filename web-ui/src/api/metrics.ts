/**
 * 系统性能指标 API
 */

import { http } from './http'

export interface SystemMetrics {
  cpuUsage: number
  memoryUsage: number
  diskUsage: number
  networkSpeed: string
  uploadSpeed: string
  downloadSpeed: string
  cpuTemperature?: number
  availableMemory?: string
  diskReadSpeed?: string
  diskWriteSpeed?: string
  networkLatency?: number
  packetLoss?: number
  timestamp: string
}

export interface MetricsHistory {
  timeLabels: string[]
  cpuHistory: number[]
  memoryHistory: number[]
  networkUpHistory: number[]
  networkDownHistory: number[]
  diskReadHistory: number[]
  diskWriteHistory: number[]
  loadAverage1m: number[]
  loadAverage5m: number[]
  loadAverage15m: number[]
}

export interface DetailedMetric {
  category: string
  metric: string
  current: string
  average: string
  peak: string
  status: 'normal' | 'warning' | 'critical'
  lastUpdate: string
}

export interface SystemHealthStatus {
  overall: 'healthy' | 'warning' | 'critical'
  cpuStatus: 'normal' | 'high' | 'critical'
  memoryStatus: 'normal' | 'high' | 'critical'
  diskStatus: 'normal' | 'high' | 'critical'
  networkStatus: 'normal' | 'slow' | 'error'
}

export interface MetricsQuery {
  timeRange: '5m' | '30m' | '1h' | '6h' | '24h'
  interval?: string
  metrics?: string[]
}

class MetricsApi {
  /**
   * 获取当前系统指标
   */
  async getCurrentMetrics(): Promise<{ data: SystemMetrics }> {
    try {
      return await http.get('/api/v1/metrics/current')
    } catch (error) {
      // 如果API不存在，返回默认数据
      console.warn('Metrics API 未实现，使用默认数据')
      return {
        data: {
          cpuUsage: 0,
          memoryUsage: 0,
          diskUsage: 0,
          networkSpeed: '0 MB/s',
          uploadSpeed: '0 MB/s',
          downloadSpeed: '0 MB/s',
          timestamp: new Date().toISOString()
        }
      }
    }
  }

  /**
   * 获取指标历史数据
   */
  async getMetricsHistory(params: MetricsQuery): Promise<{ data: MetricsHistory }> {
    try {
      return await http.get('/api/v1/metrics/history', { params })
    } catch (error) {
      console.warn('Metrics History API 未实现，使用默认数据')
      return {
        data: {
          timeLabels: [],
          cpuHistory: [],
          memoryHistory: [],
          networkUpHistory: [],
          networkDownHistory: [],
          diskReadHistory: [],
          diskWriteHistory: [],
          loadAverage1m: [],
          loadAverage5m: [],
          loadAverage15m: []
        }
      }
    }
  }

  /**
   * 获取详细指标数据
   */
  async getDetailedMetrics(): Promise<{ data: DetailedMetric[] }> {
    try {
      return await http.get('/api/v1/metrics/detailed')
    } catch (error) {
      console.warn('Detailed Metrics API 未实现，使用默认数据')
      return { data: [] }
    }
  }

  /**
   * 获取系统健康状态
   */
  async getSystemHealth(): Promise<{ data: SystemHealthStatus }> {
    try {
      return await http.get('/api/v1/metrics/health')
    } catch (error) {
      console.warn('System Health API 未实现，使用默认数据')
      return {
        data: {
          overall: 'healthy',
          cpuStatus: 'normal',
          memoryStatus: 'normal',
          diskStatus: 'normal',
          networkStatus: 'normal'
        }
      }
    }
  }

  /**
   * 获取CPU信息
   */
  async getCpuInfo() {
    return http.get('/api/v1/metrics/cpu')
  }

  /**
   * 获取内存信息
   */
  async getMemoryInfo() {
    return http.get('/api/v1/metrics/memory')
  }

  /**
   * 获取磁盘信息
   */
  async getDiskInfo() {
    return http.get('/api/v1/metrics/disk')
  }

  /**
   * 获取网络信息
   */
  async getNetworkInfo() {
    return http.get('/api/v1/metrics/network')
  }

  /**
   * 获取系统负载信息
   */
  async getLoadInfo() {
    return http.get('/api/v1/metrics/load')
  }

  /**
   * 获取进程信息
   */
  async getProcessInfo() {
    return http.get('/api/v1/metrics/processes')
  }

  /**
   * 获取服务状态
   */
  async getServicesStatus() {
    return http.get('/api/v1/metrics/services')
  }

  /**
   * 获取系统信息概览
   */
  async getSystemInfo() {
    return http.get('/api/v1/metrics/system-info')
  }

  /**
   * 导出指标数据
   */
  async exportMetrics(params: MetricsQuery & { format?: 'csv' | 'json' }) {
    return http.get('/api/v1/metrics/export', {
      params,
      responseType: 'blob'
    })
  }

  /**
   * 获取指标警报规则
   */
  async getAlertRules() {
    return http.get('/api/v1/metrics/alert-rules')
  }

  /**
   * 更新指标警报规则
   */
  async updateAlertRules(rules: any) {
    return http.put('/api/v1/metrics/alert-rules', rules)
  }

  /**
   * 获取指标收集配置
   */
  async getMetricsConfig() {
    return http.get('/api/v1/metrics/config')
  }

  /**
   * 更新指标收集配置
   */
  async updateMetricsConfig(config: any) {
    return http.put('/api/v1/metrics/config', config)
  }

  /**
   * 清理历史指标数据
   */
  async cleanupMetrics(olderThanDays: number) {
    return http.post('/api/v1/metrics/cleanup', { older_than_days: olderThanDays })
  }

  /**
   * 获取指标存储使用情况
   */
  async getMetricsStorage() {
    return http.get('/api/v1/metrics/storage')
  }

  /**
   * 执行系统性能测试
   */
  async runPerformanceTest(testType: 'cpu' | 'memory' | 'disk' | 'network') {
    return http.post(`/api/v1/metrics/test/${testType}`)
  }

  /**
   * 获取指标收集器状态
   */
  async getCollectorStatus() {
    return http.get('/api/v1/metrics/collectors/status')
  }

  /**
   * 重启指标收集器
   */
  async restartCollector(collectorName: string) {
    return http.post(`/api/v1/metrics/collectors/${collectorName}/restart`)
  }
}

export const metricsApi = new MetricsApi()
export default metricsApi