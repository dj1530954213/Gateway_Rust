/**
 * 健康检查 API
 */

import { get } from './http'

export interface HealthResponse {
  status: 'healthy' | 'unhealthy'
  timestamp: string
  version: string
  services: Record<string, string>
}

export interface ReadinessResponse {
  status: 'ready' | 'not_ready'
  timestamp: string
  reason?: string
}

export interface LivenessResponse {
  status: 'alive'
  timestamp: string
}

export const healthApi = {
  /**
   * 基础健康检查
   */
  check(): Promise<HealthResponse> {
    return get('/health')
  },

  /**
   * 就绪探针
   */
  readiness(): Promise<ReadinessResponse> {
    return get('/health/ready')
  },

  /**
   * 存活探针
   */
  liveness(): Promise<LivenessResponse> {
    return get('/health/live')
  },
}