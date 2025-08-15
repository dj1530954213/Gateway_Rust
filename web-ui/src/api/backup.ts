/**
 * 系统备份与恢复 API
 */

import { http } from './http'

export interface BackupItem {
  id: string
  name: string
  type: 'full' | 'config' | 'data' | 'incremental'
  status: 'success' | 'failed' | 'running' | 'pending'
  size: number
  createdAt: string
  compressionRatio: number
  encrypted: boolean
  contents: Array<{
    name: string
    size: number
  }>
  verification?: {
    md5: string
    integrity: boolean
    lastVerified: string
  }
}

export interface BackupStats {
  totalBackups: number
  totalSize: number
  successRate: number
  scheduledTasks: number
}

export interface BackupCreateReq {
  name: string
  type: string
  contents: string[]
  compress: boolean
  encrypt: boolean
  password?: string
  description?: string
}

export interface BackupRestoreReq {
  backupId: string
  password?: string
}

export interface BackupSchedule {
  id: string
  name: string
  backupType: string
  cron: string
  enabled: boolean
  retention: number
}

export interface BackupScheduleCreateReq {
  name: string
  backupType: string
  cron: string
  retention: number
}

export interface BackupOperation {
  id: string
  name: string
  type: string
  progress: number
  status: 'running' | 'completed' | 'error' | 'cancelled'
  currentStep: string
  startedAt: string
  estimatedCompletion?: string
}

class BackupApi {
  /**
   * 获取备份列表
   */
  async list(params?: {
    search?: string
    type?: string
    page?: number
    size?: number
  }) {
    return http.get('/api/v1/backup/list', { params })
  }

  /**
   * 获取备份统计信息
   */
  async getStats(): Promise<{ data: BackupStats }> {
    return http.get('/api/v1/backup/stats')
  }

  /**
   * 创建备份
   */
  async create(data: BackupCreateReq) {
    return http.post('/api/v1/backup/create', data)
  }

  /**
   * 删除备份
   */
  async delete(id: string) {
    return http.delete(`/api/v1/backup/${id}`)
  }

  /**
   * 恢复备份
   */
  async restore(data: BackupRestoreReq) {
    return http.post('/api/v1/backup/restore', data)
  }

  /**
   * 下载备份
   */
  async download(id: string) {
    return http.get(`/api/v1/backup/${id}/download`, {
      responseType: 'blob',
    })
  }

  /**
   * 验证备份完整性
   */
  async verify(id: string) {
    return http.post(`/api/v1/backup/${id}/verify`)
  }

  /**
   * 获取备份详情
   */
  async getDetail(id: string) {
    return http.get(`/api/v1/backup/${id}`)
  }

  /**
   * 获取正在进行的操作列表
   */
  async getOperations(): Promise<{ data: BackupOperation[] }> {
    return http.get('/api/v1/backup/operations')
  }

  /**
   * 取消操作
   */
  async cancelOperation(operationId: string) {
    return http.post(`/api/v1/backup/operations/${operationId}/cancel`)
  }

  /**
   * 获取操作进度
   */
  async getOperationProgress(
    operationId: string
  ): Promise<{ data: BackupOperation }> {
    return http.get(`/api/v1/backup/operations/${operationId}`)
  }

  /**
   * 获取计划任务列表
   */
  async getSchedules(): Promise<{ data: BackupSchedule[] }> {
    return http.get('/api/v1/backup/schedules')
  }

  /**
   * 创建计划任务
   */
  async createSchedule(data: BackupScheduleCreateReq) {
    return http.post('/api/v1/backup/schedules', data)
  }

  /**
   * 更新计划任务
   */
  async updateSchedule(id: string, data: Partial<BackupScheduleCreateReq>) {
    return http.put(`/api/v1/backup/schedules/${id}`, data)
  }

  /**
   * 删除计划任务
   */
  async deleteSchedule(id: string) {
    return http.delete(`/api/v1/backup/schedules/${id}`)
  }

  /**
   * 启用/禁用计划任务
   */
  async toggleSchedule(id: string, enabled: boolean) {
    return http.put(`/api/v1/backup/schedules/${id}/toggle`, { enabled })
  }

  /**
   * 手动执行计划任务
   */
  async executeSchedule(id: string) {
    return http.post(`/api/v1/backup/schedules/${id}/execute`)
  }

  /**
   * 获取备份配置选项
   */
  async getBackupOptions() {
    return http.get('/api/v1/backup/options')
  }

  /**
   * 更新备份配置
   */
  async updateBackupConfig(config: any) {
    return http.put('/api/v1/backup/config', config)
  }

  /**
   * 获取存储使用情况
   */
  async getStorageUsage() {
    return http.get('/api/v1/backup/storage')
  }

  /**
   * 清理过期备份
   */
  async cleanup(olderThanDays: number) {
    return http.post('/api/v1/backup/cleanup', {
      older_than_days: olderThanDays,
    })
  }

  /**
   * 导出备份配置
   */
  async exportConfig() {
    return http.get('/api/v1/backup/export-config', {
      responseType: 'blob',
    })
  }

  /**
   * 导入备份配置
   */
  async importConfig(file: File) {
    const formData = new FormData()
    formData.append('config', file)
    return http.post('/api/v1/backup/import-config', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    })
  }

  /**
   * 测试备份连通性
   */
  async testConnection() {
    return http.post('/api/v1/backup/test-connection')
  }
}

export const backupApi = new BackupApi()
export default backupApi
