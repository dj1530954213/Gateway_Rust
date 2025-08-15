/**
 * 报警管理 Pinia Store
 *
 * 管理报警规则、报警历史、实时通知等状态
 */

import { ElMessage, ElMessageBox, ElNotification } from 'element-plus'
import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'

import { alertsApi } from '@/api'
import type {
  AlertRuleVO,
  AlertRuleCreateReq,
  AlertRulePatchReq,
  AlertRuleQuery,
  AlertEventVO,
  AlertEventQuery,
  AlertSeverity,
  AlertStatus,
  NotificationChannelType,
  PaginatedResponse,
} from '@/api/alerts'

export interface AlertState {
  // 报警规则
  rules: AlertRuleVO[]
  rulesTotal: number
  rulesCurrentPage: number
  rulesPageSize: number

  // 报警历史
  events: AlertEventVO[]
  eventsTotal: number
  eventsCurrentPage: number
  eventsPageSize: number

  // 查询条件
  rulesQuery: AlertRuleQuery
  eventsQuery: AlertEventQuery

  // 选中项
  selectedRules: AlertRuleVO[]
  selectedEvents: AlertEventVO[]

  // 当前编辑项
  currentRule: AlertRuleVO | null
  currentEvent: AlertEventVO | null

  // 加载状态
  rulesLoading: boolean
  eventsLoading: boolean

  // 实时通知
  realtimeAlerts: AlertEventVO[]
  notificationsEnabled: boolean

  // 统计数据
  statistics: {
    totalRules: number
    activeRules: number
    triggeredToday: number
    criticalCount: number
    warningCount: number
    infoCount: number
  }

  // WebSocket连接状态
  wsConnected: boolean
}

export const useAlertsStore = defineStore('alerts', () => {
  // ===== 状态定义 =====
  const state = ref<AlertState>({
    rules: [],
    rulesTotal: 0,
    rulesCurrentPage: 1,
    rulesPageSize: 20,

    events: [],
    eventsTotal: 0,
    eventsCurrentPage: 1,
    eventsPageSize: 50,

    rulesQuery: {
      page: 1,
      size: 20,
    },
    eventsQuery: {
      page: 1,
      size: 50,
      start_time: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(), // 默认最近7天
      end_time: new Date().toISOString(),
    },

    selectedRules: [],
    selectedEvents: [],

    currentRule: null,
    currentEvent: null,

    rulesLoading: false,
    eventsLoading: false,

    realtimeAlerts: [],
    notificationsEnabled: true,

    statistics: {
      totalRules: 0,
      activeRules: 0,
      triggeredToday: 0,
      criticalCount: 0,
      warningCount: 0,
      infoCount: 0,
    },

    wsConnected: false,
  })

  // WebSocket 连接
  let alertWebSocket: WebSocket | null = null

  // ===== 计算属性 =====
  const hasRules = computed(() => {
    return state.value.rules.length > 0
  })

  const hasEvents = computed(() => {
    return state.value.events.length > 0
  })

  const rulesTotalPages = computed(() => {
    return Math.ceil(state.value.rulesTotal / state.value.rulesPageSize)
  })

  const eventsTotalPages = computed(() => {
    return Math.ceil(state.value.eventsTotal / state.value.eventsPageSize)
  })

  const activeRules = computed(() => {
    return state.value.rules.filter(rule => rule.enabled)
  })

  const inactiveRules = computed(() => {
    return state.value.rules.filter(rule => !rule.enabled)
  })

  const rulesBySeverity = computed(() => {
    const grouped: Record<AlertSeverity, AlertRuleVO[]> = {
      Critical: [],
      Warning: [],
      Info: [],
    }
    state.value.rules.forEach(rule => {
      grouped[rule.severity].push(rule)
    })
    return grouped
  })

  const eventsBySeverity = computed(() => {
    const grouped: Record<AlertSeverity, AlertEventVO[]> = {
      Critical: [],
      Warning: [],
      Info: [],
    }
    state.value.events.forEach(event => {
      grouped[event.severity].push(event)
    })
    return grouped
  })

  const unreadAlertsCount = computed(() => {
    return state.value.realtimeAlerts.filter(alert => !alert.acknowledged)
      .length
  })

  // 为兼容DashboardPage中的使用，添加state.unreadCount
  const stateWithUnreadCount = computed(() => ({
    ...state.value,
    unreadCount: unreadAlertsCount.value,
    alerts: state.value.events, // 别名
    total: state.value.eventsTotal, // 别名
  }))

  // ===== 报警规则方法 =====

  /**
   * 获取报警规则列表
   */
  async function fetchRules(params?: Partial<AlertRuleQuery>) {
    try {
      state.value.rulesLoading = true

      const query = {
        ...state.value.rulesQuery,
        ...params,
      }

      const response: PaginatedResponse<AlertRuleVO> =
        await alertsApi.listRules(query)

      state.value.rules = response.items
      state.value.rulesTotal = response.total
      state.value.rulesCurrentPage = response.page
      state.value.rulesPageSize = response.size

      // 更新查询条件
      state.value.rulesQuery = { ...query }

      // 更新统计数据
      updateRulesStatistics()
    } catch (error) {
      console.error('获取报警规则失败:', error)
      ElMessage.error('获取报警规则失败')
    } finally {
      state.value.rulesLoading = false
    }
  }

  /**
   * 获取报警规则详情
   */
  async function fetchRule(id: string): Promise<AlertRuleVO | null> {
    try {
      const rule = await alertsApi.getRule(id)
      state.value.currentRule = rule
      return rule
    } catch (error) {
      console.error('获取报警规则详情失败:', error)
      ElMessage.error('获取报警规则详情失败')
      return null
    }
  }

  /**
   * 创建报警规则
   */
  async function createRule(
    data: AlertRuleCreateReq
  ): Promise<AlertRuleVO | null> {
    try {
      state.value.rulesLoading = true

      const rule = await alertsApi.createRule(data)

      // 添加到列表开头
      state.value.rules.unshift(rule)
      state.value.rulesTotal += 1

      // 更新统计数据
      updateRulesStatistics()

      ElMessage.success('报警规则创建成功')
      return rule
    } catch (error) {
      console.error('创建报警规则失败:', error)
      ElMessage.error('创建报警规则失败')
      return null
    } finally {
      state.value.rulesLoading = false
    }
  }

  /**
   * 更新报警规则
   */
  async function updateRule(
    id: string,
    data: AlertRulePatchReq
  ): Promise<AlertRuleVO | null> {
    try {
      state.value.rulesLoading = true

      const updatedRule = await alertsApi.updateRule(id, data)

      // 更新列表中的规则
      const index = state.value.rules.findIndex(r => r.id === id)
      if (index !== -1) {
        state.value.rules[index] = updatedRule
      }

      // 更新当前规则
      if (state.value.currentRule?.id === id) {
        state.value.currentRule = updatedRule
      }

      // 更新统计数据
      updateRulesStatistics()

      ElMessage.success('报警规则更新成功')
      return updatedRule
    } catch (error) {
      console.error('更新报警规则失败:', error)
      ElMessage.error('更新报警规则失败')
      return null
    } finally {
      state.value.rulesLoading = false
    }
  }

  /**
   * 删除报警规则
   */
  async function deleteRule(id: string): Promise<boolean> {
    try {
      const rule = state.value.rules.find(r => r.id === id)
      if (!rule) return false

      await ElMessageBox.confirm(
        `确定要删除报警规则 "${rule.name}" 吗？删除后无法恢复。`,
        '确认删除',
        {
          type: 'warning',
          confirmButtonText: '删除',
          cancelButtonText: '取消',
          confirmButtonClass: 'el-button--danger',
        }
      )

      state.value.rulesLoading = true

      await alertsApi.deleteRule(id)

      // 从列表中移除
      state.value.rules = state.value.rules.filter(r => r.id !== id)
      state.value.rulesTotal -= 1

      // 清除当前规则
      if (state.value.currentRule?.id === id) {
        state.value.currentRule = null
      }

      // 从选中列表中移除
      state.value.selectedRules = state.value.selectedRules.filter(
        r => r.id !== id
      )

      // 更新统计数据
      updateRulesStatistics()

      ElMessage.success('报警规则删除成功')
      return true
    } catch (error) {
      if (error !== 'cancel') {
        console.error('删除报警规则失败:', error)
        ElMessage.error('删除报警规则失败')
      }
      return false
    } finally {
      state.value.rulesLoading = false
    }
  }

  /**
   * 启用/禁用报警规则
   */
  async function toggleRule(id: string, enabled: boolean): Promise<boolean> {
    try {
      const rule = await alertsApi.updateRule(id, { enabled })

      // 更新列表中的规则
      const index = state.value.rules.findIndex(r => r.id === id)
      if (index !== -1) {
        state.value.rules[index] = rule
      }

      // 更新统计数据
      updateRulesStatistics()

      ElMessage.success(enabled ? '报警规则已启用' : '报警规则已禁用')
      return true
    } catch (error) {
      console.error('切换报警规则状态失败:', error)
      ElMessage.error('操作失败')
      return false
    }
  }

  // ===== 报警历史方法 =====

  /**
   * 获取报警历史
   */
  async function fetchEvents(params?: Partial<AlertEventQuery>) {
    try {
      state.value.eventsLoading = true

      const query = {
        ...state.value.eventsQuery,
        ...params,
      }

      const response: PaginatedResponse<AlertEventVO> =
        await alertsApi.listEvents(query)

      state.value.events = response.items
      state.value.eventsTotal = response.total
      state.value.eventsCurrentPage = response.page
      state.value.eventsPageSize = response.size

      // 更新查询条件
      state.value.eventsQuery = { ...query }

      // 更新统计数据
      updateEventsStatistics()
    } catch (error) {
      console.error('获取报警历史失败:', error)
      ElMessage.error('获取报警历史失败')
    } finally {
      state.value.eventsLoading = false
    }
  }

  /**
   * 确认报警事件
   */
  async function acknowledgeEvent(
    id: string,
    comment?: string
  ): Promise<boolean> {
    try {
      await alertsApi.acknowledgeEvent(id, comment)

      // 更新列表中的事件
      const index = state.value.events.findIndex(e => e.id === id)
      if (index !== -1) {
        state.value.events[index].acknowledged = true
        state.value.events[index].ack_time = new Date().toISOString()
        if (comment) {
          state.value.events[index].ack_comment = comment
        }
      }

      // 更新实时报警列表
      const realtimeIndex = state.value.realtimeAlerts.findIndex(
        e => e.id === id
      )
      if (realtimeIndex !== -1) {
        state.value.realtimeAlerts[realtimeIndex].acknowledged = true
      }

      ElMessage.success('报警事件已确认')
      return true
    } catch (error) {
      console.error('确认报警事件失败:', error)
      ElMessage.error('确认报警事件失败')
      return false
    }
  }

  /**
   * 批量确认报警事件
   */
  async function batchAcknowledgeEvents(
    ids: string[],
    comment?: string
  ): Promise<boolean> {
    try {
      state.value.eventsLoading = true

      const results = await Promise.allSettled(
        ids.map(id => alertsApi.acknowledgeEvent(id, comment))
      )

      const successCount = results.filter(r => r.status === 'fulfilled').length
      const failedCount = results.filter(r => r.status === 'rejected').length

      // 更新列表中的事件
      const successIds = ids.slice(0, successCount)
      state.value.events.forEach(event => {
        if (successIds.includes(event.id)) {
          event.acknowledged = true
          event.ack_time = new Date().toISOString()
          if (comment) {
            event.ack_comment = comment
          }
        }
      })

      // 更新实时报警列表
      state.value.realtimeAlerts.forEach(alert => {
        if (successIds.includes(alert.id)) {
          alert.acknowledged = true
        }
      })

      // 清空选中列表
      state.value.selectedEvents = []

      if (failedCount === 0) {
        ElMessage.success(`成功确认 ${successCount} 个报警事件`)
      } else {
        ElMessage.warning(
          `确认完成：成功 ${successCount} 个，失败 ${failedCount} 个`
        )
      }

      return failedCount === 0
    } catch (error) {
      console.error('批量确认报警事件失败:', error)
      ElMessage.error('批量确认报警事件失败')
      return false
    } finally {
      state.value.eventsLoading = false
    }
  }

  // ===== 实时通知方法 =====

  /**
   * 连接WebSocket接收实时报警
   */
  function connectAlertWebSocket() {
    if (alertWebSocket?.readyState === WebSocket.OPEN) {
      return
    }

    const wsUrl = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws/alerts`

    alertWebSocket = new WebSocket(wsUrl)

    alertWebSocket.onopen = () => {
      state.value.wsConnected = true
      console.log('Alert WebSocket connected')
    }

    alertWebSocket.onmessage = event => {
      try {
        const alertEvent: AlertEventVO = JSON.parse(event.data)
        handleRealtimeAlert(alertEvent)
      } catch (error) {
        console.error('解析WebSocket消息失败:', error)
      }
    }

    alertWebSocket.onclose = () => {
      state.value.wsConnected = false
      console.log('Alert WebSocket disconnected')

      // 尝试重连
      setTimeout(() => {
        if (state.value.notificationsEnabled) {
          connectAlertWebSocket()
        }
      }, 5000)
    }

    alertWebSocket.onerror = error => {
      console.error('Alert WebSocket error:', error)
    }
  }

  /**
   * 断开WebSocket连接
   */
  function disconnectAlertWebSocket() {
    if (alertWebSocket) {
      alertWebSocket.close()
      alertWebSocket = null
    }
    state.value.wsConnected = false
  }

  /**
   * 处理实时报警
   */
  function handleRealtimeAlert(alertEvent: AlertEventVO) {
    // 添加到实时报警列表
    state.value.realtimeAlerts.unshift(alertEvent)

    // 限制实时报警列表长度
    if (state.value.realtimeAlerts.length > 100) {
      state.value.realtimeAlerts = state.value.realtimeAlerts.slice(0, 100)
    }

    // 显示通知
    if (state.value.notificationsEnabled) {
      showAlertNotification(alertEvent)
    }

    // 更新报警历史列表（如果在第一页）
    if (state.value.eventsCurrentPage === 1) {
      state.value.events.unshift(alertEvent)
      state.value.eventsTotal += 1

      // 限制列表长度
      if (state.value.events.length > state.value.eventsPageSize) {
        state.value.events = state.value.events.slice(
          0,
          state.value.eventsPageSize
        )
      }
    }
  }

  /**
   * 显示报警通知
   */
  function showAlertNotification(alertEvent: AlertEventVO) {
    const title = `${alertEvent.severity} 报警`
    const message = `${alertEvent.rule_name}: ${alertEvent.message}`

    ElNotification({
      title,
      message,
      type:
        alertEvent.severity === 'Critical'
          ? 'error'
          : alertEvent.severity === 'Warning'
            ? 'warning'
            : 'info',
      duration: alertEvent.severity === 'Critical' ? 0 : 5000, // Critical报警不自动关闭
      showClose: true,
      onClick: () => {
        // 跳转到报警详情
        state.value.currentEvent = alertEvent
      },
    })
  }

  /**
   * 启用/禁用通知
   */
  function toggleNotifications(enabled: boolean) {
    state.value.notificationsEnabled = enabled

    if (enabled) {
      connectAlertWebSocket()
    } else {
      disconnectAlertWebSocket()
    }
  }

  /**
   * 清空实时报警列表
   */
  function clearRealtimeAlerts() {
    state.value.realtimeAlerts = []
  }

  /**
   * 添加告警到实时列表
   */
  function addAlert(alert: AlertEventVO) {
    handleRealtimeAlert(alert)
  }

  /**
   * 获取最近告警列表
   */
  async function fetchAlerts(params?: { page: number; size: number }) {
    return await fetchEvents(params)
  }

  /**
   * 确认告警
   */
  async function acknowledgeAlert(alertId: string, comment?: string) {
    return await acknowledgeEvent(alertId, comment)
  }

  /**
   * 忽略告警
   */
  async function dismissAlert(alertId: string) {
    // 这里可以实现忽略逻辑，暂时使用确认代替
    return await acknowledgeEvent(alertId, '已忽略')
  }

  // ===== 统计方法 =====

  /**
   * 更新规则统计数据
   */
  function updateRulesStatistics() {
    state.value.statistics.totalRules = state.value.rules.length
    state.value.statistics.activeRules = state.value.rules.filter(
      r => r.enabled
    ).length
  }

  /**
   * 更新事件统计数据
   */
  function updateEventsStatistics() {
    const today = new Date().toDateString()

    state.value.statistics.triggeredToday = state.value.events.filter(
      e => new Date(e.triggered_at).toDateString() === today
    ).length

    state.value.statistics.criticalCount = state.value.events.filter(
      e => e.severity === 'Critical'
    ).length
    state.value.statistics.warningCount = state.value.events.filter(
      e => e.severity === 'Warning'
    ).length
    state.value.statistics.infoCount = state.value.events.filter(
      e => e.severity === 'Info'
    ).length
  }

  // ===== 工具方法 =====

  /**
   * 搜索报警规则
   */
  async function searchRules(keyword: string) {
    await fetchRules({
      ...state.value.rulesQuery,
      name_contains: keyword || undefined,
      page: 1,
    })
  }

  /**
   * 按严重级别筛选规则
   */
  async function filterRulesBySeverity(severity: AlertSeverity | undefined) {
    await fetchRules({
      ...state.value.rulesQuery,
      severity,
      page: 1,
    })
  }

  /**
   * 按状态筛选规则
   */
  async function filterRulesByStatus(enabled: boolean | undefined) {
    await fetchRules({
      ...state.value.rulesQuery,
      enabled,
      page: 1,
    })
  }

  /**
   * 按严重级别筛选事件
   */
  async function filterEventsBySeverity(severity: AlertSeverity | undefined) {
    await fetchEvents({
      ...state.value.eventsQuery,
      severity,
      page: 1,
    })
  }

  /**
   * 按确认状态筛选事件
   */
  async function filterEventsByAck(acknowledged: boolean | undefined) {
    await fetchEvents({
      ...state.value.eventsQuery,
      acknowledged,
      page: 1,
    })
  }

  /**
   * 重置状态
   */
  function reset() {
    // 断开WebSocket连接
    disconnectAlertWebSocket()

    // 重置状态
    state.value.rules = []
    state.value.rulesTotal = 0
    state.value.rulesCurrentPage = 1
    state.value.rulesPageSize = 20

    state.value.events = []
    state.value.eventsTotal = 0
    state.value.eventsCurrentPage = 1
    state.value.eventsPageSize = 50

    state.value.rulesQuery = { page: 1, size: 20 }
    state.value.eventsQuery = {
      page: 1,
      size: 50,
      start_time: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
      end_time: new Date().toISOString(),
    }

    state.value.selectedRules = []
    state.value.selectedEvents = []
    state.value.currentRule = null
    state.value.currentEvent = null

    state.value.rulesLoading = false
    state.value.eventsLoading = false

    state.value.realtimeAlerts = []
    state.value.notificationsEnabled = true

    state.value.statistics = {
      totalRules: 0,
      activeRules: 0,
      triggeredToday: 0,
      criticalCount: 0,
      warningCount: 0,
      infoCount: 0,
    }

    state.value.wsConnected = false
  }

  // ===== 返回 Store API =====
  return {
    // 状态
    state: readonly(stateWithUnreadCount),

    // 计算属性
    hasRules,
    hasEvents,
    rulesTotalPages,
    eventsTotalPages,
    activeRules,
    inactiveRules,
    rulesBySeverity,
    eventsBySeverity,
    unreadAlertsCount,

    // 报警规则方法
    fetchRules,
    fetchRule,
    createRule,
    updateRule,
    deleteRule,
    toggleRule,

    // 报警历史方法
    fetchEvents,
    fetchAlerts, // 别名方法
    acknowledgeEvent,
    acknowledgeAlert, // 别名方法
    dismissAlert,
    addAlert,
    batchAcknowledgeEvents,

    // 实时通知方法
    connectAlertWebSocket,
    disconnectAlertWebSocket,
    toggleNotifications,
    clearRealtimeAlerts,

    // 工具方法
    searchRules,
    filterRulesBySeverity,
    filterRulesByStatus,
    filterEventsBySeverity,
    filterEventsByAck,
    reset,
  }
})

// 类型导出
export type AlertsStore = ReturnType<typeof useAlertsStore>
