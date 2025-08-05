/**
 * 历史数据 Pinia Store
 * 
 * 管理时间序列数据查询、统计分析、导出等操作
 * 包含智能缓存机制以提升查询性能
 */

import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import { ElMessage } from 'element-plus'
import { historyApi } from '@/api'
import type { 
  HistoryDataVO, 
  HistoryQuery, 
  StatisticsVO,
  TimeSeriesData,
  ExportFormat,
  AggregationFunction,
  DateRange
} from '@/api/history'

// ===== 缓存相关类型定义 =====
interface CacheEntry {
  key: string
  data: any
  timestamp: number
  expireTime: number
  size: number
  hits: number
}

interface QueryCacheEntry extends CacheEntry {
  data: {
    timeSeriesData?: TimeSeriesData[]
    rawData?: HistoryDataVO[]
    statistics?: StatisticsVO
    total: number
  }
  query: HistoryQuery
}

interface CacheStatistics {
  totalEntries: number
  totalSize: number
  hitRate: number
  totalHits: number
  totalMisses: number
}

export interface HistoryState {
  // 历史数据
  timeSeriesData: TimeSeriesData[]
  rawData: HistoryDataVO[]
  
  // 统计数据
  statistics: StatisticsVO | null
  
  // 查询参数
  query: HistoryQuery
  
  // 时间范围
  dateRange: DateRange
  
  // 选中的点位
  selectedTags: string[]
  
  // 加载状态
  loading: boolean
  exporting: boolean
  
  // 数据总数
  total: number
  
  // 图表配置
  chartConfig: {
    title: string
    showLegend: boolean
    smoothLine: boolean
    autoRefresh: boolean
    refreshInterval: number // 秒
  }
  
  // 分页
  currentPage: number
  pageSize: number
  
  // 缓存配置
  cacheConfig: {
    enabled: boolean
    maxSize: number // MB
    maxEntries: number
    defaultTTL: number // 秒
    enablePersist: boolean
  }
}

export const useHistoryStore = defineStore('history', () => {
  // ===== 状态定义 =====
  const state = ref<HistoryState>({
    timeSeriesData: [],
    rawData: [],
    statistics: null,
    query: {
      tag_ids: [],
      start_time: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(), // 默认最近24小时
      end_time: new Date().toISOString(),
      aggregation: 'avg',
      interval: '1m',
      page: 1,
      size: 1000,
    },
    dateRange: {
      start: new Date(Date.now() - 24 * 60 * 60 * 1000),
      end: new Date(),
    },
    selectedTags: [],
    loading: false,
    exporting: false,
    total: 0,
    chartConfig: {
      title: '历史数据趋势',
      showLegend: true,
      smoothLine: true,
      autoRefresh: false,
      refreshInterval: 30,
    },
    currentPage: 1,
    pageSize: 1000,
    cacheConfig: {
      enabled: true,
      maxSize: 50, // 50MB
      maxEntries: 100,
      defaultTTL: 900, // 15分钟
      enablePersist: false
    }
  })

  // ===== 缓存相关状态 =====
  const cache = ref<Map<string, QueryCacheEntry>>(new Map())
  const cacheStats = ref({
    totalHits: 0,
    totalMisses: 0
  })

  // 自动刷新定时器
  let refreshTimer: NodeJS.Timeout | null = null
  
  // 缓存清理定时器
  let cacheCleanupTimer: NodeJS.Timeout | null = null

  // ===== 计算属性 =====
  const hasData = computed(() => {
    return state.value.timeSeriesData.length > 0 || state.value.rawData.length > 0
  })

  const isLoading = computed(() => {
    return state.value.loading
  })

  const isExporting = computed(() => {
    return state.value.exporting
  })

  const totalPages = computed(() => {
    return Math.ceil(state.value.total / state.value.pageSize)
  })

  const timeRange = computed(() => {
    const { start_time, end_time } = state.value.query
    const duration = new Date(end_time).getTime() - new Date(start_time).getTime()
    const hours = Math.round(duration / (1000 * 60 * 60))
    
    if (hours < 1) {
      const minutes = Math.round(duration / (1000 * 60))
      return `${minutes}分钟`
    } else if (hours < 24) {
      return `${hours}小时`
    } else {
      const days = Math.round(hours / 24)
      return `${days}天`
    }
  })

  const dataCount = computed(() => {
    return state.value.timeSeriesData.reduce((total, series) => total + series.data.length, 0)
  })

  const cacheStatistics = computed<CacheStatistics>(() => {
    const totalRequests = cacheStats.value.totalHits + cacheStats.value.totalMisses
    const hitRate = totalRequests > 0 ? (cacheStats.value.totalHits / totalRequests) * 100 : 0
    
    let totalSize = 0
    cache.value.forEach(entry => {
      totalSize += entry.size
    })
    
    return {
      totalEntries: cache.value.size,
      totalSize,
      hitRate: Math.round(hitRate * 100) / 100,
      totalHits: cacheStats.value.totalHits,
      totalMisses: cacheStats.value.totalMisses
    }
  })

  const cacheEntries = computed(() => {
    return Array.from(cache.value.values()).sort((a, b) => b.timestamp - a.timestamp)
  })

  // ===== 缓存辅助方法 =====

  /**
   * 生成缓存键
   */
  function generateCacheKey(query: HistoryQuery, type: 'timeseries' | 'raw' | 'statistics'): string {
    const keyObj = {
      type,
      tag_ids: [...query.tag_ids].sort(),
      start_time: query.start_time,
      end_time: query.end_time,
      aggregation: query.aggregation,
      interval: query.interval,
      page: query.page,
      size: query.size
    }
    
    return btoa(JSON.stringify(keyObj)).replace(/[^a-zA-Z0-9]/g, '')
  }

  /**
   * 计算数据大小（估算）
   */
  function calculateDataSize(data: any): number {
    try {
      return new Blob([JSON.stringify(data)]).size
    } catch {
      // 回退到估算
      const jsonStr = JSON.stringify(data)
      return jsonStr.length * 2 // UTF-16编码大致每字符2字节
    }
  }

  /**
   * 检查缓存是否有效
   */
  function isCacheValid(entry: QueryCacheEntry): boolean {
    return Date.now() < entry.expireTime
  }

  /**
   * 清理过期缓存
   */
  function cleanExpiredCache() {
    const now = Date.now()
    const keysToDelete: string[] = []
    
    cache.value.forEach((entry, key) => {
      if (now >= entry.expireTime) {
        keysToDelete.push(key)
      }
    })
    
    keysToDelete.forEach(key => {
      cache.value.delete(key)
    })
    
    if (keysToDelete.length > 0) {
      console.log(`清理了 ${keysToDelete.length} 个过期缓存项`)
    }
  }

  /**
   * 强制执行缓存大小限制
   */
  function enforceCacheLimits() {
    // 检查条目数量限制
    if (cache.value.size > state.value.cacheConfig.maxEntries) {
      const entries = Array.from(cache.value.entries())
      // 按访问次数和时间排序，删除最少使用的
      entries.sort((a, b) => {
        const aScore = a[1].hits / Math.log(Date.now() - a[1].timestamp + 1)
        const bScore = b[1].hits / Math.log(Date.now() - b[1].timestamp + 1)
        return aScore - bScore
      })
      
      const toDelete = entries.slice(0, entries.length - state.value.cacheConfig.maxEntries)
      toDelete.forEach(([key]) => {
        cache.value.delete(key)
      })
      
      console.log(`清理了 ${toDelete.length} 个缓存项以满足条目数量限制`)
    }
    
    // 检查大小限制
    let totalSize = 0
    cache.value.forEach(entry => {
      totalSize += entry.size
    })
    
    const maxSizeBytes = state.value.cacheConfig.maxSize * 1024 * 1024
    if (totalSize > maxSizeBytes) {
      const entries = Array.from(cache.value.entries())
      entries.sort((a, b) => a[1].timestamp - b[1].timestamp) // 按时间排序，删除最旧的
      
      let deletedSize = 0
      for (const [key, entry] of entries) {
        cache.value.delete(key)
        deletedSize += entry.size
        
        if (totalSize - deletedSize <= maxSizeBytes * 0.8) {
          break
        }
      }
      
      console.log(`清理了 ${formatBytes(deletedSize)} 的缓存以满足大小限制`)
    }
  }

  /**
   * 格式化字节大小
   */
  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
  }

  /**
   * 从缓存获取数据
   */
  function getFromCache(cacheKey: string): QueryCacheEntry | null {
    const entry = cache.value.get(cacheKey)
    if (entry && isCacheValid(entry)) {
      entry.hits++
      entry.timestamp = Date.now() // 更新访问时间
      cacheStats.value.totalHits++
      return entry
    }
    
    if (entry && !isCacheValid(entry)) {
      cache.value.delete(cacheKey)
    }
    
    cacheStats.value.totalMisses++
    return null
  }

  /**
   * 存储到缓存
   */
  function setCache(cacheKey: string, data: any, query: HistoryQuery) {
    if (!state.value.cacheConfig.enabled) return
    
    const size = calculateDataSize(data)
    const now = Date.now()
    
    const entry: QueryCacheEntry = {
      key: cacheKey,
      data,
      query: { ...query },
      timestamp: now,
      expireTime: now + (state.value.cacheConfig.defaultTTL * 1000),
      size,
      hits: 0
    }
    
    cache.value.set(cacheKey, entry)
    
    // 清理缓存
    enforceCacheLimits()
  }

  // ===== 操作方法 =====

  /**
   * 查询时间序列数据（带缓存）
   */
  async function fetchTimeSeriesData(params?: Partial<HistoryQuery>) {
    try {
      state.value.loading = true
      
      const query = {
        ...state.value.query,
        ...params,
      }

      // 先清理过期缓存
      cleanExpiredCache()

      // 检查缓存
      const cacheKey = generateCacheKey(query, 'timeseries')
      const cachedData = getFromCache(cacheKey)
      
      if (cachedData?.data.timeSeriesData) {
        console.log('从缓存返回时间序列数据:', `${cacheKey.substring(0, 16)  }...`)
        state.value.timeSeriesData = cachedData.data.timeSeriesData
        state.value.total = cachedData.data.total
        state.value.query = { ...query }
        return
      }

      console.log('缓存未命中，发起API请求 - 时间序列数据')
      const response = await historyApi.getTimeSeries(query)
      
      state.value.timeSeriesData = response.series
      state.value.total = response.total
      
      // 缓存结果
      setCache(cacheKey, {
        timeSeriesData: response.series,
        total: response.total
      }, query)
      
      // 更新查询条件
      state.value.query = { ...query }
      
    } catch (error) {
      console.error('获取时间序列数据失败:', error)
      ElMessage.error('获取历史数据失败')
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 查询原始数据（带缓存）
   */
  async function fetchRawData(params?: Partial<HistoryQuery>) {
    try {
      state.value.loading = true
      
      const query = {
        ...state.value.query,
        ...params,
      }

      // 先清理过期缓存
      cleanExpiredCache()

      // 检查缓存
      const cacheKey = generateCacheKey(query, 'raw')
      const cachedData = getFromCache(cacheKey)
      
      if (cachedData?.data.rawData) {
        console.log('从缓存返回原始数据:', `${cacheKey.substring(0, 16)  }...`)
        state.value.rawData = cachedData.data.rawData
        state.value.total = cachedData.data.total
        state.value.currentPage = query.page || 1
        state.value.pageSize = query.size || 1000
        state.value.query = { ...query }
        return
      }

      console.log('缓存未命中，发起API请求 - 原始数据')
      const response = await historyApi.getRawData(query)
      
      state.value.rawData = response.items
      state.value.total = response.total
      state.value.currentPage = response.page
      state.value.pageSize = response.size
      
      // 缓存结果
      setCache(cacheKey, {
        rawData: response.items,
        total: response.total
      }, query)
      
      // 更新查询条件
      state.value.query = { ...query }
      
    } catch (error) {
      console.error('获取原始数据失败:', error)
      ElMessage.error('获取原始数据失败')
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 获取统计数据（带缓存）
   */
  async function fetchStatistics(params?: Partial<HistoryQuery>) {
    try {
      const query = {
        ...state.value.query,
        ...params,
      }

      // 先清理过期缓存
      cleanExpiredCache()

      // 检查缓存
      const cacheKey = generateCacheKey(query, 'statistics')
      const cachedData = getFromCache(cacheKey)
      
      if (cachedData?.data.statistics) {
        console.log('从缓存返回统计数据:', `${cacheKey.substring(0, 16)  }...`)
        state.value.statistics = cachedData.data.statistics
        return
      }

      console.log('缓存未命中，发起API请求 - 统计数据')
      const statistics = await historyApi.getStatistics(query)
      state.value.statistics = statistics
      
      // 缓存结果
      setCache(cacheKey, {
        statistics,
        total: 0 // 统计数据没有total字段
      }, query)
      
    } catch (error) {
      console.error('获取统计数据失败:', error)
      ElMessage.error('获取统计数据失败')
    }
  }

  /**
   * 设置时间范围
   */
  function setDateRange(start: Date, end: Date) {
    state.value.dateRange = { start, end }
    state.value.query.start_time = start.toISOString()
    state.value.query.end_time = end.toISOString()
  }

  /**
   * 设置快速时间范围
   */
  function setQuickDateRange(range: 'last1h' | 'last6h' | 'last24h' | 'last7d' | 'last30d') {
    const now = new Date()
    let start: Date

    switch (range) {
      case 'last1h':
        start = new Date(now.getTime() - 60 * 60 * 1000)
        break
      case 'last6h':
        start = new Date(now.getTime() - 6 * 60 * 60 * 1000)
        break
      case 'last24h':
        start = new Date(now.getTime() - 24 * 60 * 60 * 1000)
        break
      case 'last7d':
        start = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
        break
      case 'last30d':
        start = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000)
        break
      default:
        start = new Date(now.getTime() - 24 * 60 * 60 * 1000)
    }

    setDateRange(start, now)
  }

  /**
   * 设置聚合函数
   */
  function setAggregation(aggregation: AggregationFunction) {
    state.value.query.aggregation = aggregation
  }

  /**
   * 设置时间间隔
   */
  function setInterval(interval: string) {
    state.value.query.interval = interval
  }

  /**
   * 选择点位
   */
  function selectTags(tagIds: string[]) {
    state.value.selectedTags = tagIds
    state.value.query.tag_ids = tagIds
  }

  /**
   * 添加点位
   */
  function addTag(tagId: string) {
    if (!state.value.selectedTags.includes(tagId)) {
      state.value.selectedTags.push(tagId)
      state.value.query.tag_ids = [...state.value.selectedTags]
    }
  }

  /**
   * 移除点位
   */
  function removeTag(tagId: string) {
    state.value.selectedTags = state.value.selectedTags.filter(id => id !== tagId)
    state.value.query.tag_ids = [...state.value.selectedTags]
  }

  /**
   * 导出数据
   */
  async function exportData(format: ExportFormat = 'csv'): Promise<boolean> {
    try {
      state.value.exporting = true
      
      const blob = await historyApi.exportData(state.value.query, format)
      
      // 创建下载链接
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `history_data_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.${format}`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      window.URL.revokeObjectURL(url)
      
      ElMessage.success('数据导出成功')
      return true
      
    } catch (error) {
      console.error('导出数据失败:', error)
      ElMessage.error('导出数据失败')
      return false
    } finally {
      state.value.exporting = false
    }
  }

  /**
   * 刷新数据
   */
  async function refresh() {
    // 同时获取时间序列数据和统计数据
    await Promise.all([
      fetchTimeSeriesData(state.value.query),
      fetchStatistics(state.value.query),
    ])
  }

  /**
   * 启动自动刷新
   */
  function startAutoRefresh() {
    stopAutoRefresh() // 先停止现有定时器
    
    state.value.chartConfig.autoRefresh = true
    
    refreshTimer = setInterval(() => {
      refresh()
    }, state.value.chartConfig.refreshInterval * 1000)
  }

  /**
   * 停止自动刷新
   */
  function stopAutoRefresh() {
    state.value.chartConfig.autoRefresh = false
    
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
  }

  /**
   * 设置自动刷新间隔
   */
  function setRefreshInterval(seconds: number) {
    state.value.chartConfig.refreshInterval = seconds
    
    if (state.value.chartConfig.autoRefresh) {
      startAutoRefresh() // 重新启动定时器
    }
  }

  /**
   * 更新图表配置
   */
  function updateChartConfig(config: Partial<typeof state.value.chartConfig>) {
    state.value.chartConfig = {
      ...state.value.chartConfig,
      ...config,
    }
  }

  /**
   * 分页跳转
   */
  async function changePage(page: number) {
    await fetchRawData({
      ...state.value.query,
      page,
    })
  }

  /**
   * 更改分页大小
   */
  async function changePageSize(size: number) {
    await fetchRawData({
      ...state.value.query,
      size,
      page: 1,
    })
  }

  /**
   * 重置状态
   */
  function reset() {
    // 停止自动刷新
    stopAutoRefresh()
    
    // 重置状态
    state.value.timeSeriesData = []
    state.value.rawData = []
    state.value.statistics = null
    state.value.query = {
      tag_ids: [],
      start_time: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
      end_time: new Date().toISOString(),
      aggregation: 'avg',
      interval: '1m',
      page: 1,
      size: 1000,
    }
    state.value.dateRange = {
      start: new Date(Date.now() - 24 * 60 * 60 * 1000),
      end: new Date(),
    }
    state.value.selectedTags = []
    state.value.loading = false
    state.value.exporting = false
    state.value.total = 0
    state.value.currentPage = 1
    state.value.pageSize = 1000
    state.value.chartConfig = {
      title: '历史数据趋势',
      showLegend: true,
      smoothLine: true,
      autoRefresh: false,
      refreshInterval: 30,
    }
  }

  // ===== 缓存管理方法 =====

  /**
   * 清空所有缓存
   */
  function clearCache() {
    const count = cache.value.size
    cache.value.clear()
    cacheStats.value.totalHits = 0
    cacheStats.value.totalMisses = 0
    ElMessage.success(`已清空 ${count} 个缓存项`)
  }

  /**
   * 删除特定缓存项
   */
  function deleteCacheEntry(cacheKey: string): boolean {
    const deleted = cache.value.delete(cacheKey)
    if (deleted) {
      ElMessage.success('缓存项已删除')
    }
    return deleted
  }

  /**
   * 更新缓存配置
   */
  function updateCacheConfig(config: Partial<typeof state.value.cacheConfig>) {
    state.value.cacheConfig = {
      ...state.value.cacheConfig,
      ...config
    }
    
    // 如果禁用缓存，清空所有缓存
    if (!state.value.cacheConfig.enabled) {
      cache.value.clear()
    }
    
    // 重新强制执行缓存限制
    enforceCacheLimits()
  }

  /**
   * 预热缓存
   */
  async function warmupCache(queries: Partial<HistoryQuery>[]) {
    console.log('开始预热缓存...')
    
    const promises = queries.map(async (queryParams) => {
      try {
        await fetchTimeSeriesData(queryParams)
        await fetchStatistics(queryParams)
      } catch (error) {
        console.warn('预热缓存失败:', error)
      }
    })
    
    await Promise.allSettled(promises)
    ElMessage.success('缓存预热完成')
  }

  // 在组件卸载时清理定时器
  function cleanup() {
    stopAutoRefresh()
    
    // 停止缓存清理定时器
    if (cacheCleanupTimer) {
      clearInterval(cacheCleanupTimer)
      cacheCleanupTimer = null
    }
  }

  // ===== 初始化缓存清理定时器 =====
  
  // 每5分钟清理一次过期缓存
  cacheCleanupTimer = setInterval(() => {
    if (state.value.cacheConfig.enabled) {
      cleanExpiredCache()
    }
  }, 5 * 60 * 1000)

  // ===== 返回 Store API =====
  return {
    // 状态
    state: readonly(state),
    cache: readonly(cache),
    
    // 计算属性
    hasData,
    isLoading,
    isExporting,
    totalPages,
    timeRange,
    dataCount,
    cacheStatistics,
    cacheEntries,
    
    // 方法
    fetchTimeSeriesData,
    fetchRawData,
    fetchStatistics,
    setDateRange,
    setQuickDateRange,
    setAggregation,
    setInterval,
    selectTags,
    addTag,
    removeTag,
    exportData,
    refresh,
    startAutoRefresh,
    stopAutoRefresh,
    setRefreshInterval,
    updateChartConfig,
    changePage,
    changePageSize,
    reset,
    cleanup,
    
    // 缓存方法
    clearCache,
    deleteCacheEntry,
    updateCacheConfig,
    warmupCache,
  }
})

// 类型导出
export type HistoryStore = ReturnType<typeof useHistoryStore>