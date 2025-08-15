/**
 * 增强的WebSocket组合式函数
 * 
 * 提供完整的WebSocket连接管理、断线重连、消息队列等功能
 */

import { ref, computed, reactive, onUnmounted, watch, nextTick } from 'vue'
import { ElMessage, ElNotification } from 'element-plus'
import type { 
  TelemetryMsg, 
  AlertNotification, 
  ClientSubscription,
  WsMessage,
  DeviceVO 
} from '@/types/api'

// ========== 类型定义 ==========

export interface WebSocketConfig {
  url?: string
  protocols?: string | string[]
  maxReconnectAttempts?: number
  reconnectInterval?: number
  heartbeatInterval?: number
  messageQueueMaxSize?: number
  enableLogging?: boolean
  autoReconnect?: boolean
}

export interface ConnectionState {
  status: 'connecting' | 'connected' | 'disconnected' | 'reconnecting' | 'error'
  readyState: number
  lastConnected?: Date
  lastError?: Error
  reconnectAttempts: number
  latency?: number
}

export interface MessageStats {
  sent: number
  received: number
  errors: number
  queuedMessages: number
  lastMessageTime?: Date
}

export interface QueuedMessage {
  message: WsMessage
  timestamp: Date
  priority: 'low' | 'normal' | 'high' | 'critical'
  retries: number
  maxRetries: number
}

export interface SubscriptionOptions {
  deviceIds: string[]
  tagIds?: string[]
  alerts?: boolean
  sampleIntervalMs?: number
}

// ========== 主要组合式函数 ==========

export function useWebSocketEnhanced(initialConfig: WebSocketConfig = {}) {
  // 配置
  const config = reactive<Required<WebSocketConfig>>({
    url: initialConfig.url || `${import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:50016'}/ws`,
    protocols: initialConfig.protocols || [],
    maxReconnectAttempts: initialConfig.maxReconnectAttempts || 10,
    reconnectInterval: initialConfig.reconnectInterval || 5000,
    heartbeatInterval: initialConfig.heartbeatInterval || 30000,
    messageQueueMaxSize: initialConfig.messageQueueMaxSize || 1000,
    enableLogging: initialConfig.enableLogging ?? true,
    autoReconnect: initialConfig.autoReconnect ?? true,
  })

  // WebSocket实例
  let ws: WebSocket | null = null
  let heartbeatTimer: NodeJS.Timeout | null = null
  let reconnectTimer: NodeJS.Timeout | null = null

  // 状态管理
  const connectionState = reactive<ConnectionState>({
    status: 'disconnected',
    readyState: WebSocket.CLOSED,
    reconnectAttempts: 0,
  })

  const messageStats = reactive<MessageStats>({
    sent: 0,
    received: 0,
    errors: 0,
    queuedMessages: 0,
  })

  // 消息队列
  const messageQueue = ref<QueuedMessage[]>([])
  
  // 事件监听器
  const eventListeners = new Map<string, Set<Function>>()
  
  // 订阅状态
  const currentSubscription = ref<ClientSubscription | null>(null)
  
  // 实时数据存储
  const telemetryData = reactive<Map<string, TelemetryMsg>>(new Map())
  const alertNotifications = ref<AlertNotification[]>([])

  // ========== 计算属性 ==========

  const isConnected = computed(() => connectionState.status === 'connected')
  const isConnecting = computed(() => connectionState.status === 'connecting')
  const isReconnecting = computed(() => connectionState.status === 'reconnecting')
  const hasError = computed(() => connectionState.status === 'error')
  
  const connectionInfo = computed(() => ({
    ...connectionState,
    uptime: connectionState.lastConnected 
      ? Date.now() - connectionState.lastConnected.getTime() 
      : 0,
    config: { ...config },
    stats: { ...messageStats },
  }))

  // ========== 核心方法 ==========

  /**
   * 建立WebSocket连接
   */
  async function connect(): Promise<boolean> {
    if (ws && (ws.readyState === WebSocket.CONNECTING || ws.readyState === WebSocket.OPEN)) {
      log('WebSocket already connecting or connected')
      return true
    }

    return new Promise((resolve) => {
      try {
        connectionState.status = 'connecting'
        connectionState.readyState = WebSocket.CONNECTING
        
        log(`Connecting to WebSocket: ${config.url}`)
        
        ws = new WebSocket(config.url, config.protocols)
        
        ws.onopen = () => {
          connectionState.status = 'connected'
          connectionState.readyState = WebSocket.OPEN
          connectionState.lastConnected = new Date()
          connectionState.reconnectAttempts = 0
          connectionState.lastError = undefined
          
          log('WebSocket connected successfully')
          
          // 开始心跳
          startHeartbeat()
          
          // 处理队列中的消息
          processMessageQueue()
          
          // 重新订阅（如果有的话）
          if (currentSubscription.value) {
            subscribe(currentSubscription.value)
          }
          
          emit('connected', { timestamp: new Date() })
          ElMessage.success('WebSocket连接成功')
          
          resolve(true)
        }

        ws.onmessage = (event) => {
          handleMessage(event.data)
        }

        ws.onerror = (error) => {
          connectionState.status = 'error'
          connectionState.lastError = new Error('WebSocket connection error')
          messageStats.errors++
          
          log('WebSocket error:', error)
          emit('error', error)
          
          resolve(false)
        }

        ws.onclose = (event) => {
          const wasConnected = connectionState.status === 'connected'
          
          connectionState.status = 'disconnected'
          connectionState.readyState = WebSocket.CLOSED
          
          stopHeartbeat()
          
          log(`WebSocket closed: code=${event.code}, reason=${event.reason}`)
          
          emit('disconnected', { 
            code: event.code, 
            reason: event.reason,
            wasClean: event.wasClean 
          })
          
          // 自动重连
          if (config.autoReconnect && wasConnected && !event.wasClean) {
            attemptReconnect()
          }
          
          if (!wasConnected) {
            resolve(false)
          }
        }
        
      } catch (error) {
        connectionState.status = 'error'
        connectionState.lastError = error as Error
        log('Failed to create WebSocket:', error)
        emit('error', error)
        resolve(false)
      }
    })
  }

  /**
   * 断开WebSocket连接
   */
  function disconnect() {
    config.autoReconnect = false
    
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
    
    stopHeartbeat()
    
    if (ws) {
      ws.close(1000, 'Manual disconnect')
      ws = null
    }
    
    connectionState.status = 'disconnected'
    connectionState.readyState = WebSocket.CLOSED
    
    log('WebSocket manually disconnected')
  }

  /**
   * 发送消息
   */
  function send(message: WsMessage, priority: QueuedMessage['priority'] = 'normal'): boolean {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      // 将消息添加到队列
      queueMessage({
        message,
        timestamp: new Date(),
        priority,
        retries: 0,
        maxRetries: 3,
      })
      return false
    }

    try {
      const messageStr = JSON.stringify(message)
      ws.send(messageStr)
      
      messageStats.sent++
      messageStats.lastMessageTime = new Date()
      
      log('Message sent:', message)
      emit('messageSent', message)
      
      return true
    } catch (error) {
      log('Failed to send message:', error)
      messageStats.errors++
      
      // 将消息添加到队列重试
      queueMessage({
        message,
        timestamp: new Date(),
        priority,
        retries: 0,
        maxRetries: 3,
      })
      
      return false
    }
  }

  /**
   * 订阅数据
   */
  function subscribe(options: SubscriptionOptions) {
    const subscription: ClientSubscription = {
      device_ids: options.deviceIds,
      tag_ids: options.tagIds,
      alerts: options.alerts ?? true,
      sample_interval_ms: options.sampleIntervalMs,
    }

    currentSubscription.value = subscription
    
    return send({
      type: 'Subscribe',
      data: subscription,
    }, 'high')
  }

  /**
   * 取消订阅
   */
  function unsubscribe() {
    currentSubscription.value = null
    
    return send({
      type: 'Unsubscribe',
      data: null,
    }, 'high')
  }

  /**
   * 订阅设备数据
   */
  function subscribeToDevices(devices: DeviceVO[], options: Partial<SubscriptionOptions> = {}) {
    return subscribe({
      deviceIds: devices.map(d => d.id),
      alerts: true,
      sampleIntervalMs: 1000,
      ...options,
    })
  }

  // ========== 内部方法 ==========

  /**
   * 处理接收到的消息
   */
  function handleMessage(data: string) {
    try {
      const message = JSON.parse(data) as WsMessage
      
      messageStats.received++
      messageStats.lastMessageTime = new Date()
      
      log('Message received:', message)
      
      switch (message.type) {
        case 'Telemetry':
          handleTelemetryMessage(message.data)
          break
          
        case 'TelemetryBatch':
          message.data.forEach(handleTelemetryMessage)
          break
          
        case 'Alert':
          handleAlertMessage(message.data)
          break
          
        case 'Status':
          handleStatusMessage(message.data)
          break
          
        case 'Pong':
          handlePongMessage()
          break
          
        case 'Error':
          handleErrorMessage(message.data)
          break
          
        default:
          log('Unknown message type:', message.type)
      }
      
      emit('message', message)
      emit(message.type.toLowerCase(), message.data)
      
    } catch (error) {
      log('Failed to parse message:', error)
      messageStats.errors++
      emit('parseError', error)
    }
  }

  /**
   * 处理遥测数据
   */
  function handleTelemetryMessage(data: TelemetryMsg) {
    const key = `${data.device_id}_${data.tag_id}`
    telemetryData.set(key, data)
    
    emit('telemetry', data)
  }

  /**
   * 处理报警消息
   */
  function handleAlertMessage(data: AlertNotification) {
    alertNotifications.value.unshift(data)
    
    // 限制存储的报警数量
    if (alertNotifications.value.length > 100) {
      alertNotifications.value.splice(100)
    }
    
    // 显示通知
    ElNotification({
      title: '系统报警',
      message: `${data.rule_name}: ${data.message}`,
      type: data.level === 'CRIT' ? 'error' : data.level === 'WARN' ? 'warning' : 'info',
      duration: data.level === 'CRIT' ? 0 : 5000,
    })
    
    emit('alert', data)
  }

  /**
   * 处理状态消息
   */
  function handleStatusMessage(data: any) {
    emit('status', data)
  }

  /**
   * 处理Pong消息
   */
  function handlePongMessage() {
    const now = Date.now()
    if (connectionState.lastPingTime) {
      connectionState.latency = now - connectionState.lastPingTime
    }
  }

  /**
   * 处理错误消息
   */
  function handleErrorMessage(data: any) {
    ElMessage.error(`WebSocket错误: ${data.message}`)
    emit('serverError', data)
  }

  /**
   * 开始心跳
   */
  function startHeartbeat() {
    if (heartbeatTimer || config.heartbeatInterval <= 0) return
    
    heartbeatTimer = setInterval(() => {
      if (ws && ws.readyState === WebSocket.OPEN) {
        ;(connectionState as any).lastPingTime = Date.now()
        send({ type: 'Ping', data: null }, 'low')
      }
    }, config.heartbeatInterval)
  }

  /**
   * 停止心跳
   */
  function stopHeartbeat() {
    if (heartbeatTimer) {
      clearInterval(heartbeatTimer)
      heartbeatTimer = null
    }
  }

  /**
   * 尝试重连
   */
  function attemptReconnect() {
    if (connectionState.reconnectAttempts >= config.maxReconnectAttempts) {
      log('Max reconnect attempts reached')
      connectionState.status = 'error'
      connectionState.lastError = new Error('Max reconnect attempts reached')
      ElMessage.error('WebSocket连接失败，已达到最大重试次数')
      return
    }

    connectionState.status = 'reconnecting'
    connectionState.reconnectAttempts++
    
    const delay = Math.min(
      config.reconnectInterval * Math.pow(1.5, connectionState.reconnectAttempts - 1),
      30000
    )
    
    log(`Attempting reconnect ${connectionState.reconnectAttempts}/${config.maxReconnectAttempts} in ${delay}ms`)
    
    reconnectTimer = setTimeout(async () => {
      await connect()
    }, delay)
  }

  /**
   * 消息排队
   */
  function queueMessage(queuedMessage: QueuedMessage) {
    if (messageQueue.value.length >= config.messageQueueMaxSize) {
      // 移除最旧的低优先级消息
      const lowPriorityIndex = messageQueue.value.findIndex(m => m.priority === 'low')
      if (lowPriorityIndex !== -1) {
        messageQueue.value.splice(lowPriorityIndex, 1)
      } else {
        messageQueue.value.shift() // 移除最旧的消息
      }
    }
    
    messageQueue.value.push(queuedMessage)
    messageQueue.value.sort((a, b) => {
      const priorityOrder = { critical: 4, high: 3, normal: 2, low: 1 }
      return priorityOrder[b.priority] - priorityOrder[a.priority]
    })
    
    messageStats.queuedMessages = messageQueue.value.length
  }

  /**
   * 处理消息队列
   */
  function processMessageQueue() {
    if (!ws || ws.readyState !== WebSocket.OPEN) return
    
    const messagesToSend = [...messageQueue.value]
    messageQueue.value.length = 0
    
    for (const queuedMessage of messagesToSend) {
      const success = send(queuedMessage.message, queuedMessage.priority)
      
      if (!success && queuedMessage.retries < queuedMessage.maxRetries) {
        queuedMessage.retries++
        queueMessage(queuedMessage)
      }
    }
    
    messageStats.queuedMessages = messageQueue.value.length
  }

  /**
   * 事件发射器
   */
  function emit(eventName: string, data?: any) {
    const listeners = eventListeners.get(eventName)
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(data)
        } catch (error) {
          log(`Error in event listener for ${eventName}:`, error)
        }
      })
    }
  }

  /**
   * 添加事件监听器
   */
  function on(eventName: string, callback: Function) {
    if (!eventListeners.has(eventName)) {
      eventListeners.set(eventName, new Set())
    }
    eventListeners.get(eventName)!.add(callback)
  }

  /**
   * 移除事件监听器
   */
  function off(eventName: string, callback?: Function) {
    const listeners = eventListeners.get(eventName)
    if (!listeners) return
    
    if (callback) {
      listeners.delete(callback)
    } else {
      listeners.clear()
    }
  }

  /**
   * 日志记录
   */
  function log(...args: any[]) {
    if (config.enableLogging) {
      console.log(`[WebSocket Enhanced]`, ...args)
    }
  }

  // ========== 生命周期管理 ==========

  onUnmounted(() => {
    disconnect()
  })

  // ========== 公共API ==========

  return {
    // 状态
    connectionState: readonly(connectionState),
    messageStats: readonly(messageStats),
    connectionInfo,
    isConnected,
    isConnecting,
    isReconnecting,
    hasError,
    
    // 数据
    telemetryData: readonly(telemetryData),
    alertNotifications: readonly(alertNotifications),
    currentSubscription: readonly(currentSubscription),
    
    // 方法
    connect,
    disconnect,
    send,
    subscribe,
    unsubscribe,
    subscribeToDevices,
    
    // 事件
    on,
    off,
    
    // 配置
    updateConfig: (newConfig: Partial<WebSocketConfig>) => {
      Object.assign(config, newConfig)
    },
    
    // 工具方法
    clearTelemetryData: () => telemetryData.clear(),
    clearAlertNotifications: () => alertNotifications.value.length = 0,
    getLatestTelemetry: (deviceId: string, tagId: string) => {
      return telemetryData.get(`${deviceId}_${tagId}`)
    },
  }
}

// ========== 辅助类型 ==========

export type WebSocketEnhanced = ReturnType<typeof useWebSocketEnhanced>

// ========== 预设配置 ==========

export const websocketPresets = {
  development: {
    url: 'ws://localhost:50016/ws',
    enableLogging: true,
    heartbeatInterval: 10000,
    maxReconnectAttempts: 5,
  },
  
  production: {
    url: `ws://${window.location.host}/ws/telemetry`,
    enableLogging: false,
    heartbeatInterval: 30000,
    maxReconnectAttempts: 10,
  },
  
  testing: {
    url: 'ws://localhost:50016/ws',
    enableLogging: true,
    heartbeatInterval: 5000,
    maxReconnectAttempts: 3,
    autoReconnect: false,
  },
} as const