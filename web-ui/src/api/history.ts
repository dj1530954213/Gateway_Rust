/**
 * 历史数据查询 API
 */

import { get, post } from './http'

export interface TimeSeriesQuery {
  tag_ids: string[]
  start_time: string
  end_time: string
  interval?: string
  limit?: number
}

export interface TimeSeriesPoint {
  timestamp: string
  tag_id: string
  value: any
  quality?: string
}

export interface TimeSeriesData {
  points: TimeSeriesPoint[]
  total: number
  interval: string
}

export interface AggregatedQuery {
  tag_ids: string[]
  start_time: string
  end_time: string
  aggregation: AggregationType
  window?: string
  limit?: number
}

export interface AggregatedPoint {
  timestamp: string
  tag_id: string
  value: number
  count?: number
}

export interface AggregatedData {
  points: AggregatedPoint[]
  total: number
  aggregation: AggregationType
  window: string
}

export enum AggregationType {
  Avg = 'avg',
  Min = 'min',
  Max = 'max',
  Sum = 'sum',
  Count = 'count',
  First = 'first',
  Last = 'last',
}

export interface CsvExportRequest {
  tag_ids: string[]
  start_time: string
  end_time: string
  interval?: string
  include_quality?: boolean
}

export const historyApi = {
  /**
   * 查询时间序列数据
   */
  queryTimeSeries(params: TimeSeriesQuery): Promise<TimeSeriesData> {
    return get('/history/time-series', params)
  },

  /**
   * 查询聚合数据
   */
  queryAggregated(params: AggregatedQuery): Promise<AggregatedData> {
    return get('/history/aggregated', params)
  },

  /**
   * 导出 CSV 数据
   */
  exportCsv(params: CsvExportRequest): Promise<void> {
    return post('/history/export/csv', params, {
      responseType: 'blob',
    })
  },

  /**
   * 获取标签统计信息
   */
  getTagStats(
    tagId: string,
    startTime: string,
    endTime: string
  ): Promise<{
    count: number
    min: number
    max: number
    avg: number
    first_timestamp: string
    last_timestamp: string
  }> {
    return get('/history/stats', {
      tag_id: tagId,
      start_time: startTime,
      end_time: endTime,
    })
  },

  /**
   * 获取可用时间范围
   */
  getTimeRange(tagId?: string): Promise<{
    earliest: string
    latest: string
    total_points: number
  }> {
    return get('/history/time-range', tagId ? { tag_id: tagId } : undefined)
  },
}
