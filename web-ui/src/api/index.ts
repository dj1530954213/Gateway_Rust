/**
 * Gateway Rust - API 统一导出
 */

// HTTP 基础工具
export * from './http'

// 各模块 API
export * from './http'
export * from './auth'
export * from './health'
export * from './devices'
export * from './tags'
export * from './drivers'
export * from './history'
export * from './alerts'
export * from './system'
export * from './realtime'
export * from './datapoints'
export * from './users'
export * from './systemlogs'
export * from './backup'
export * from './metrics'

// Config API (使用 system API)
export { systemApi as configApi } from './system'

// 导入API实例以供便捷方法使用
import { authApi } from './auth'
import { healthApi } from './health'
import { devicesApi } from './devices'
import { tagsApi } from './tags'
import { driversApi } from './drivers'
import { historyApi } from './history'
import { alertsApi } from './alerts'
import { systemApi } from './system'
import { realtimeApi } from './realtime'
import { datapointsApi } from './datapoints'
import { usersApi } from './users'
import { systemLogsApi } from './systemlogs'
import { backupApi } from './backup'
import { metricsApi } from './metrics'

// 重新导出常用的 API 实例
export { authApi } from './auth'
export { healthApi } from './health'
export { devicesApi } from './devices'
export { tagsApi } from './tags'
export { driversApi } from './drivers'
export { historyApi } from './history'
export { alertsApi } from './alerts'
export { systemApi } from './system'
export { realtimeApi } from './realtime'
export { datapointsApi } from './datapoints'
export { usersApi } from './users'
export { systemLogsApi } from './systemlogs'
export { backupApi } from './backup'
export { metricsApi } from './metrics'

// WebSocket 连接管理
export interface WebSocketMessage {
  type: string
  data: any
  timestamp: string
}

export interface TelemetryData {
  tag_id: string
  value: any
  quality: string
  timestamp: string
}

export interface AlertNotification {
  event_id: string
  rule_name: string
  severity: string
  message: string
  timestamp: string
}

export class WebSocketClient {
  private ws?: WebSocket
  private url: string
  private reconnectAttempts = 0
  private maxReconnectAttempts: number
  private reconnectInterval: number
  private heartbeatInterval?: number
  private heartbeatTimer?: NodeJS.Timeout
  private listeners: Map<string, Set<Function>> = new Map()

  constructor(
    url?: string,
    options: {
      maxReconnectAttempts?: number
      reconnectInterval?: number
      heartbeatInterval?: number
    } = {}
  ) {
    this.url = url || `${import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080'}/ws/telemetry`
    this.maxReconnectAttempts = options.maxReconnectAttempts || 
      parseInt(import.meta.env.VITE_WS_MAX_RECONNECT_ATTEMPTS || '10')
    this.reconnectInterval = options.reconnectInterval || 
      parseInt(import.meta.env.VITE_WS_RECONNECT_INTERVAL || '5000')
    this.heartbeatInterval = options.heartbeatInterval || 
      parseInt(import.meta.env.VITE_WS_HEARTBEAT_INTERVAL || '30000')
  }

  /**
   * 连接 WebSocket
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.url)

        this.ws.onopen = () => {
          console.log('WebSocket 连接已建立')
          this.reconnectAttempts = 0
          this.startHeartbeat()
          resolve()
        }

        this.ws.onmessage = (event) => {
          try {
            const message: WebSocketMessage = JSON.parse(event.data)
            this.emit(message.type, message.data)
          } catch (error) {
            console.error('解析 WebSocket 消息失败:', error)
          }
        }

        this.ws.onclose = (event) => {
          console.log('WebSocket 连接已关闭:', event.code, event.reason)
          this.stopHeartbeat()
          
          if (!event.wasClean && this.reconnectAttempts < this.maxReconnectAttempts) {
            setTimeout(() => {
              this.reconnectAttempts++
              console.log(`尝试重连 WebSocket (${this.reconnectAttempts}/${this.maxReconnectAttempts})`)
              this.connect()
            }, this.reconnectInterval)
          }
        }

        this.ws.onerror = (error) => {
          console.error('WebSocket 错误:', error)
          reject(error)
        }

      } catch (error) {
        reject(error)
      }
    })
  }

  /**
   * 断开连接
   */
  disconnect(): void {
    this.stopHeartbeat()
    if (this.ws) {
      this.ws.close()
      this.ws = undefined
    }
  }

  /**
   * 发送消息
   */
  send(type: string, data: any): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const message: WebSocketMessage = {
        type,
        data,
        timestamp: new Date().toISOString(),
      }
      this.ws.send(JSON.stringify(message))
    }
  }

  /**
   * 订阅消息
   */
  on(type: string, callback: Function): void {
    if (!this.listeners.has(type)) {
      this.listeners.set(type, new Set())
    }
    this.listeners.get(type)!.add(callback)
  }

  /**
   * 取消订阅
   */
  off(type: string, callback?: Function): void {
    if (!this.listeners.has(type)) return
    
    if (callback) {
      this.listeners.get(type)!.delete(callback)
    } else {
      this.listeners.get(type)!.clear()
    }
  }

  /**
   * 触发事件
   */
  private emit(type: string, data: any): void {
    if (this.listeners.has(type)) {
      this.listeners.get(type)!.forEach(callback => {
        try {
          callback(data)
        } catch (error) {
          console.error(`WebSocket 事件处理器错误 [${type}]:`, error)
        }
      })
    }
  }

  /**
   * 开始心跳
   */
  private startHeartbeat(): void {
    if (this.heartbeatInterval && this.heartbeatInterval > 0) {
      this.heartbeatTimer = setInterval(() => {
        this.send('ping', { timestamp: Date.now() })
      }, this.heartbeatInterval)
    }
  }

  /**
   * 停止心跳
   */
  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = undefined
    }
  }

  /**
   * 获取连接状态
   */
  get readyState(): number {
    return this.ws?.readyState ?? WebSocket.CLOSED
  }

  /**
   * 是否已连接
   */
  get isConnected(): boolean {
    return this.readyState === WebSocket.OPEN
  }
}

// 创建全局 WebSocket 实例
export const wsClient = new WebSocketClient()

// 便捷方法
export const api = {
  auth: authApi,
  health: healthApi,
  devices: devicesApi,
  tags: tagsApi,
  drivers: driversApi,
  history: historyApi,
  alerts: alertsApi,
  system: systemApi,
  realtime: realtimeApi,
  datapoints: datapointsApi,
  users: usersApi,
  systemLogs: systemLogsApi,
  backup: backupApi,
  metrics: metricsApi,
  ws: wsClient,
}

export default api