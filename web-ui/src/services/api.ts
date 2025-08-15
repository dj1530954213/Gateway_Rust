/**
 * @deprecated 请使用 @/api 模块的新 API 结构
 * 此文件保留用于兼容性，新代码应使用新的 API 模块
 */

import { api } from '@/api'

// 重新导出新的 API 以保持兼容性
export const {
  auth: authApi,
  health: healthApi,
  devices: devicesApi,
  tags: tagsApi,
  drivers: driversApi,
  history: historyApi,
  alerts: alertsApi,
  system: systemApi,
  realtime: realtimeApi,
  ws: wsClient,
} = api

// 兼容性封装
export { get, post, put, del, upload, download } from '@/api/http'

export default api
