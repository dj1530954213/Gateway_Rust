/**
 * 点位管理 API
 */

import { get, post, put, del } from './http'
import type { PaginatedResponse } from './http'

export interface TagVO {
  id: string
  device_id: string
  name: string
  address: string
  data_type: TagDataType
  scaling?: number
  offset?: number
  unit?: string
  description?: string
  enabled: boolean
  created_at: string
}

export interface TagCreateReq {
  device_id: string
  name: string
  address: string
  data_type: TagDataType
  scaling?: number
  offset?: number
  unit?: string
  description?: string
  enabled?: boolean
}

export interface TagPatchReq {
  name?: string
  address?: string
  data_type?: TagDataType
  scaling?: number
  offset?: number
  unit?: string
  description?: string
  enabled?: boolean
}

export interface TagQuery {
  page?: number
  size?: number
  device_id?: string
  data_type?: TagDataType
  enabled?: boolean
  name_contains?: string
}

export enum TagDataType {
  Float = 'Float',
  Int = 'Int',
  Bool = 'Bool',
  String = 'String',
}

export const tagsApi = {
  /**
   * 创建点位
   */
  create(data: TagCreateReq): Promise<TagVO> {
    return post('/tags', data)
  },

  /**
   * 查询点位列表
   */
  list(params?: TagQuery): Promise<PaginatedResponse<TagVO>> {
    return get('/tags', params)
  },

  /**
   * 获取点位详情
   */
  get(id: string): Promise<TagVO> {
    return get(`/tags/${id}`)
  },

  /**
   * 更新点位
   */
  update(id: string, data: TagPatchReq): Promise<TagVO> {
    return put(`/tags/${id}`, data)
  },

  /**
   * 删除点位
   */
  delete(id: string): Promise<void> {
    return del(`/tags/${id}`)
  },

  /**
   * 批量导入点位
   */
  bulkImport(
    file: File
  ): Promise<{ success: number; failed: number; errors?: string[] }> {
    const formData = new FormData()
    formData.append('file', file)
    return post('/tags/import', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  },

  /**
   * 批量导出点位
   */
  bulkExport(params?: {
    device_id?: string
    format?: 'csv' | 'xlsx'
  }): Promise<void> {
    return get('/tags/export', params, {
      responseType: 'blob',
    })
  },
}
