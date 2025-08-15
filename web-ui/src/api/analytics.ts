/**
 * 数据分析 API
 */

import { get, post, put, del } from './http'

export interface KpiMetric {
  id: string
  title: string
  value: number
  unit: string
  trend: 'up' | 'down' | 'stable'
  trendText: string
  comparison: string
  icon: string
  iconClass: string
}

export interface DataPoint {
  id: string
  name: string
  dataType: string
  currentValue: string | number | boolean
}

export interface Report {
  id: string
  title: string
  description: string
  type: 'trend' | 'distribution' | 'performance' | 'custom'
  chartType: string
  status: 'active' | 'inactive' | 'error'
  lastRunTime: Date
  dataSource?: string
}

export interface DataSource {
  id: string
  name: string
  type: 'datapoints' | 'drivers' | 'alarms'
}

export interface DataQualityMetric {
  id: string
  name: string
  status: string
  percentage: number
}

export interface PredictionRequest {
  datapointId: string
  timeRange: {
    startTime: Date
    endTime: Date
  }
}

export interface PredictionResult {
  accuracy: number
  confidenceRange: {
    min: number
    max: number
  }
  anomalyRisk: 'low' | 'medium' | 'high'
}

export const analyticsApi = {
  /**
   * 获取KPI指标
   */
  async getKpiMetrics(): Promise<{ data: KpiMetric[] }> {
    try {
      return await get('/api/v1/analytics/kpi')
    } catch (error) {
      console.warn('KPI指标API未实现，返回空数据')
      return { data: [] }
    }
  },

  /**
   * 获取数据点列表
   */
  async getDatapoints(): Promise<{ data: DataPoint[] }> {
    try {
      return await get('/api/v1/analytics/datapoints')
    } catch (error) {
      console.warn('数据点API未实现，返回空数据')
      return { data: [] }
    }
  },

  /**
   * 获取数据质量指标
   */
  async getDataQualityMetrics(): Promise<{ data: DataQualityMetric[] }> {
    try {
      return await get('/api/v1/analytics/data-quality')
    } catch (error) {
      console.warn('数据质量指标API未实现，返回空数据')
      return { data: [] }
    }
  },

  /**
   * 获取报表列表
   */
  async getReports(): Promise<{ data: Report[] }> {
    try {
      return await get('/api/v1/analytics/reports')
    } catch (error) {
      console.warn('报表列表API未实现，返回空数据')
      return { data: [] }
    }
  },

  /**
   * 获取数据源列表
   */
  async getDataSources(): Promise<{ data: DataSource[] }> {
    try {
      return await get('/api/v1/analytics/data-sources')
    } catch (error) {
      console.warn('数据源API未实现，返回空数据')
      return { data: [] }
    }
  },

  /**
   * 创建报表
   */
  async createReport(report: Partial<Report>): Promise<{ data: Report }> {
    try {
      return await post('/api/v1/analytics/reports', report)
    } catch (error) {
      console.warn('创建报表API未实现')
      throw error
    }
  },

  /**
   * 更新报表
   */
  async updateReport(
    id: string,
    report: Partial<Report>
  ): Promise<{ data: Report }> {
    try {
      return await put(`/api/v1/analytics/reports/${id}`, report)
    } catch (error) {
      console.warn('更新报表API未实现')
      throw error
    }
  },

  /**
   * 删除报表
   */
  async deleteReport(id: string): Promise<void> {
    try {
      return await del(`/api/v1/analytics/reports/${id}`)
    } catch (error) {
      console.warn('删除报表API未实现')
      throw error
    }
  },

  /**
   * 复制报表
   */
  async duplicateReport(id: string): Promise<{ data: Report }> {
    try {
      return await post(`/api/v1/analytics/reports/${id}/duplicate`)
    } catch (error) {
      console.warn('复制报表API未实现')
      throw error
    }
  },

  /**
   * 运行报表
   */
  async runReport(id: string): Promise<void> {
    try {
      return await post(`/api/v1/analytics/reports/${id}/run`)
    } catch (error) {
      console.warn('运行报表API未实现')
      throw error
    }
  },

  /**
   * 导出报表
   */
  async exportReport(id: string): Promise<void> {
    try {
      return await post(`/api/v1/analytics/reports/${id}/export`)
    } catch (error) {
      console.warn('导出报表API未实现')
      throw error
    }
  },

  /**
   * 运行预测分析
   */
  async runPrediction(request: PredictionRequest): Promise<PredictionResult> {
    try {
      return await post('/api/v1/analytics/prediction', request)
    } catch (error) {
      console.warn('预测分析API未实现')
      // 返回默认预测结果
      return {
        accuracy: 75,
        confidenceRange: { min: 50, max: 90 },
        anomalyRisk: 'low',
      }
    }
  },

  /**
   * 获取趋势图表数据
   */
  async getTrendData(datapointIds: string[], timeRange: any): Promise<any> {
    try {
      return await post('/api/v1/analytics/trend', {
        datapointIds,
        timeRange,
      })
    } catch (error) {
      console.warn('趋势数据API未实现')
      return { series: [] }
    }
  },

  /**
   * 获取分布图表数据
   */
  async getDistributionData(datapointId: string, timeRange: any): Promise<any> {
    try {
      return await post('/api/v1/analytics/distribution', {
        datapointId,
        timeRange,
      })
    } catch (error) {
      console.warn('分布数据API未实现')
      return { series: [] }
    }
  },

  /**
   * 获取设备性能数据
   */
  async getDevicePerformanceData(timeRange: any): Promise<any> {
    try {
      return await post('/api/v1/analytics/device-performance', { timeRange })
    } catch (error) {
      console.warn('设备性能数据API未实现')
      return { series: [] }
    }
  },

  /**
   * 获取报警统计数据
   */
  async getAlarmStatsData(timeRange: any): Promise<any> {
    try {
      return await post('/api/v1/analytics/alarm-stats', { timeRange })
    } catch (error) {
      console.warn('报警统计数据API未实现')
      return { series: [] }
    }
  },
}
