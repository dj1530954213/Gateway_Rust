/**
 * 连接器管理 API
 */

import { get, post, put, del } from './http'

export interface Connector {
  id: string
  name: string
  type: 'mqtt' | 'http' | 'influxdb' | 'websocket' | 'tcp' | 'opcua'
  status: 'connected' | 'disconnected' | 'connecting' | 'error'
  config: Record<string, any>
  configValid: boolean
  protocol?: string
  host: string
  port: number
  messagesSent: number
  errorCount: number
  avgLatency: number
  uptime?: number
  lastError?: string
  createdAt: Date
  updatedAt: Date
}

export interface ConnectorCreateReq {
  name: string
  type: string
  config: Record<string, any>
  host: string
  port: number
}

export interface ConnectorTestResult {
  success: boolean
  message: string
  responseTime: number
  details?: string
}

export const connectorApi = {
  /**
   * 获取连接器列表
   */
  async getConnectors(): Promise<{ data: Connector[] }> {
    try {
      return await get('/api/v1/connectors')
    } catch (error) {
      console.warn('连接器API未实现，返回空列表')
      return { data: [] }
    }
  },

  /**
   * 获取连接器详情
   */
  async getConnector(id: string): Promise<{ data: Connector }> {
    try {
      return await get(`/api/v1/connectors/${id}`)
    } catch (error) {
      console.warn('连接器详情API未实现')
      throw error
    }
  },

  /**
   * 创建连接器
   */
  async createConnector(
    connector: ConnectorCreateReq
  ): Promise<{ data: Connector }> {
    try {
      return await post('/api/v1/connectors', connector)
    } catch (error) {
      console.warn('创建连接器API未实现')
      throw error
    }
  },

  /**
   * 更新连接器
   */
  async updateConnector(
    id: string,
    connector: Partial<Connector>
  ): Promise<{ data: Connector }> {
    try {
      return await put(`/api/v1/connectors/${id}`, connector)
    } catch (error) {
      console.warn('更新连接器API未实现')
      throw error
    }
  },

  /**
   * 删除连接器
   */
  async deleteConnector(id: string): Promise<void> {
    try {
      return await del(`/api/v1/connectors/${id}`)
    } catch (error) {
      console.warn('删除连接器API未实现')
      throw error
    }
  },

  /**
   * 启动连接器
   */
  async startConnector(id: string): Promise<void> {
    try {
      return await post(`/api/v1/connectors/${id}/start`)
    } catch (error) {
      console.warn('启动连接器API未实现')
      throw error
    }
  },

  /**
   * 停止连接器
   */
  async stopConnector(id: string): Promise<void> {
    try {
      return await post(`/api/v1/connectors/${id}/stop`)
    } catch (error) {
      console.warn('停止连接器API未实现')
      throw error
    }
  },

  /**
   * 测试连接器连接
   */
  async testConnection(
    connector: string | Connector
  ): Promise<ConnectorTestResult> {
    try {
      if (typeof connector === 'string') {
        return await post(`/api/v1/connectors/${connector}/test`)
      } else {
        return await post('/api/v1/connectors/test', connector)
      }
    } catch (error) {
      console.warn('连接器测试API未实现')
      // 返回默认失败结果
      return {
        success: false,
        message: 'API未实现',
        responseTime: 0,
        details: '连接器测试API尚未实现',
      }
    }
  },

  /**
   * 批量启动连接器
   */
  async batchStartConnectors(ids: string[]): Promise<void> {
    try {
      return await post('/api/v1/connectors/batch/start', { ids })
    } catch (error) {
      console.warn('批量启动连接器API未实现')
      throw error
    }
  },

  /**
   * 批量停止连接器
   */
  async batchStopConnectors(ids: string[]): Promise<void> {
    try {
      return await post('/api/v1/connectors/batch/stop', { ids })
    } catch (error) {
      console.warn('批量停止连接器API未实现')
      throw error
    }
  },

  /**
   * 批量重启连接器
   */
  async batchRestartConnectors(ids: string[]): Promise<void> {
    try {
      return await post('/api/v1/connectors/batch/restart', { ids })
    } catch (error) {
      console.warn('批量重启连接器API未实现')
      throw error
    }
  },

  /**
   * 批量删除连接器
   */
  async batchDeleteConnectors(ids: string[]): Promise<void> {
    try {
      return await post('/api/v1/connectors/batch/delete', { ids })
    } catch (error) {
      console.warn('批量删除连接器API未实现')
      throw error
    }
  },
}
