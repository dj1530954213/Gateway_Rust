/**
 * Gateway Rust - HTTP 客户端基础配置
 *
 * 提供 Axios 实例配置、请求/响应拦截器、错误处理等基础功能
 */

import axios from 'axios'
import type {
  AxiosInstance,
  AxiosRequestConfig,
  AxiosResponse,
  InternalAxiosRequestConfig,
} from 'axios'
import { ElMessage, ElMessageBox, ElLoading } from 'element-plus'
import NProgress from 'nprogress'

import { useAuthStore } from '@/stores/auth'
import { useSystemStore } from '@/stores/system'

import 'nprogress/nprogress.css'
import CryptoJS from 'crypto-js'

// 基础配置
const IS_DEV = import.meta.env.DEV
// 在开发环境中，如果没有设置VITE_API_BASE_URL或设置为空，则使用相对路径通过Vite代理
const BASE_URL = import.meta.env.VITE_API_BASE_URL !== undefined && import.meta.env.VITE_API_BASE_URL !== '' 
  ? import.meta.env.VITE_API_BASE_URL 
  : (IS_DEV ? '' : 'http://localhost:8080')
const REQUEST_TIMEOUT = 30000
const RETRY_COUNT = 3
const RETRY_DELAY = 1000

// 配置 NProgress
NProgress.configure({ showSpinner: false })

// 创建 Axios 实例
const http: AxiosInstance = axios.create({
  baseURL: BASE_URL,
  timeout: REQUEST_TIMEOUT,
  headers: {
    'Content-Type': 'application/json',
    Accept: 'application/json',
  },
  withCredentials: false,
})

// 请求计数器（用于显示加载状态）
let requestCount = 0
let loadingInstance: any = null

// 生成请求ID
function generateRequestId(): string {
  return CryptoJS.lib.WordArray.random(16).toString()
}

// 显示全局加载
function showLoading() {
  if (requestCount === 0) {
    loadingInstance = ElLoading.service({
      lock: true,
      text: '加载中...',
      background: 'rgba(0, 0, 0, 0.7)',
    })
  }
  requestCount++
}

// 隐藏全局加载
function hideLoading() {
  requestCount--
  if (requestCount <= 0) {
    requestCount = 0
    if (loadingInstance) {
      loadingInstance.close()
      loadingInstance = null
    }
  }
}

// 请求拦截器
http.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    // 显示进度条
    NProgress.start()

    // 添加请求ID用于追踪
    const requestId = generateRequestId()
    config.headers['X-Request-ID'] = requestId

    // 添加时间戳防止缓存
    if (config.method?.toLowerCase() === 'get') {
      config.params = {
        ...config.params,
        _t: Date.now(),
      }
    }

    // 添加认证token
    const authStore = useAuthStore()
    if (authStore.token) {
      config.headers.Authorization = `Bearer ${authStore.token}`
    }

    // 添加语言头
    config.headers['Accept-Language'] = 'zh-CN,zh;q=0.9,en;q=0.8'

    // 如果是文件上传，更新Content-Type
    if (config.data instanceof FormData) {
      config.headers['Content-Type'] = 'multipart/form-data'
    }

    // 显示全局加载（可通过配置禁用）
    if (config.headers['X-Show-Loading'] !== 'false') {
      showLoading()
    }

    // 调试日志
    if (import.meta.env.VITE_DEBUG === 'true') {
      console.group(`🚀 API请求: ${config.method?.toUpperCase()} ${config.url}`)
      console.log('Request ID:', requestId)
      console.log('Config:', config)
      console.groupEnd()
    }

    return config
  },
  error => {
    NProgress.done()
    hideLoading()
    console.error('请求拦截器错误:', error)
    return Promise.reject(error)
  }
)

// 响应拦截器
http.interceptors.response.use(
  (response: AxiosResponse) => {
    NProgress.done()
    hideLoading()

    const config = response.config as InternalAxiosRequestConfig
    const requestId = config.headers['X-Request-ID']

    // 调试日志
    if (import.meta.env.VITE_DEBUG === 'true') {
      console.group(
        `✅ API响应: ${response.status} ${config.method?.toUpperCase()} ${config.url}`
      )
      console.log('Request ID:', requestId)
      console.log('Response:', response.data)
      console.groupEnd()
    }

    // 检查业务状态码
    if (response.data && typeof response.data === 'object') {
      // Gateway Rust 使用标准HTTP状态码，不需要额外检查
      return response
    }

    return response
  },
  async error => {
    NProgress.done()
    hideLoading()

    const { config, response } = error
    const requestId = config?.headers['X-Request-ID']

    // 调试日志
    if (import.meta.env.VITE_DEBUG === 'true') {
      console.group(
        `❌ API错误: ${response?.status || 'Network'} ${config?.method?.toUpperCase()} ${config?.url}`
      )
      console.log('Request ID:', requestId)
      console.log('Error:', error)
      console.groupEnd()
    }

    // 网络错误重试
    if (!response && config && config.retryCount < RETRY_COUNT) {
      config.retryCount = config.retryCount || 0
      config.retryCount++

      console.warn(`网络请求失败，第${config.retryCount}次重试...`)

      await new Promise(resolve =>
        setTimeout(resolve, RETRY_DELAY * config.retryCount)
      )
      return http(config)
    }

    // 错误处理
    if (response) {
      const status = response.status
      const data = response.data

      switch (status) {
        case 400:
          ElMessage.error(data?.detail || '请求参数错误')
          break
        case 401:
          ElMessage.error('认证失败，请重新登录')
          // 清除认证状态并跳转登录
          const authStore = useAuthStore()
          authStore.logout()
          // 避免在已经在登录页时重复跳转
          if (window.location.pathname !== '/login') {
            window.location.href = '/login'
          }
          break
        case 403:
          ElMessage.error('权限不足，无法访问该资源')
          break
        case 404:
          ElMessage.error(data?.detail || '请求的资源不存在')
          break
        case 409:
          ElMessage.error(data?.detail || '资源冲突')
          break
        case 422:
          ElMessage.error(data?.detail || '数据验证失败')
          break
        case 429:
          ElMessage.error('请求过于频繁，请稍后重试')
          break
        case 500:
        case 502:
        case 503:
        case 504:
          // 对于API未实现的情况，不显示错误消息，让API层自己处理
          if (
            !config?.url?.includes('/metrics/') &&
            !config?.url?.includes('/system/health')
          ) {
            ElMessage.error('服务器错误，请稍后重试')
          }
          break
        default:
          ElMessage.error(data?.detail || `请求失败 (${status})`)
      }
    } else {
      // 网络错误
      if (error.code === 'ECONNABORTED') {
        ElMessage.error('请求超时，请检查网络连接')
      } else if (error.message === 'Network Error') {
        ElMessage.error('网络连接失败，请检查网络')
      } else {
        ElMessage.error(`请求失败: ${error.message}`)
      }
    }

    return Promise.reject(error)
  }
)

// 通用请求方法
export interface ApiResponse<T = any> {
  data?: T
  message?: string
  success?: boolean
  [key: string]: any
}

export interface PaginatedResponse<T = any> {
  items: T[]
  total: number
  page: number
  size: number
  pages: number
}

// 基础请求方法
export async function request<T = any>(config: AxiosRequestConfig): Promise<T> {
  try {
    const response = await http(config)
    return response.data
  } catch (error: any) {
    throw error
  }
}

// GET 请求
export function get<T = any>(
  url: string,
  params?: any,
  config?: AxiosRequestConfig
): Promise<T> {
  return request<T>({
    method: 'GET',
    url,
    params,
    ...config,
  })
}

// POST 请求
export function post<T = any>(
  url: string,
  data?: any,
  config?: AxiosRequestConfig
): Promise<T> {
  return request<T>({
    method: 'POST',
    url,
    data,
    ...config,
  })
}

// PUT 请求
export function put<T = any>(
  url: string,
  data?: any,
  config?: AxiosRequestConfig
): Promise<T> {
  return request<T>({
    method: 'PUT',
    url,
    data,
    ...config,
  })
}

// DELETE 请求
export function del<T = any>(
  url: string,
  config?: AxiosRequestConfig
): Promise<T> {
  return request<T>({
    method: 'DELETE',
    url,
    ...config,
  })
}

// PATCH 请求
export function patch<T = any>(
  url: string,
  data?: any,
  config?: AxiosRequestConfig
): Promise<T> {
  return request<T>({
    method: 'PATCH',
    url,
    data,
    ...config,
  })
}

// 文件上传
export function upload<T = any>(
  url: string,
  file: File | FormData,
  onProgress?: (progress: number) => void,
  config?: AxiosRequestConfig
): Promise<T> {
  const formData = file instanceof FormData ? file : new FormData()
  if (file instanceof File) {
    formData.append('file', file)
  }

  return request<T>({
    method: 'POST',
    url,
    data: formData,
    headers: {
      'Content-Type': 'multipart/form-data',
    },
    onUploadProgress: onProgress
      ? progressEvent => {
          if (progressEvent.total) {
            const progress = Math.round(
              (progressEvent.loaded * 100) / progressEvent.total
            )
            onProgress(progress)
          }
        }
      : undefined,
    ...config,
  })
}

// 文件下载
export async function download(
  url: string,
  filename?: string,
  params?: any
): Promise<void> {
  try {
    const response = await http({
      method: 'GET',
      url,
      params,
      responseType: 'blob',
      headers: {
        'X-Show-Loading': 'false', // 不显示全局加载
      },
    })

    // 创建下载链接
    const blob = new Blob([response.data])
    const downloadUrl = window.URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = downloadUrl

    // 从响应头获取文件名
    const contentDisposition = response.headers['content-disposition']
    if (contentDisposition && !filename) {
      const matches = contentDisposition.match(
        /filename[^;=\n]*=((['"]).*?\2|[^;\n]*)/
      )?.[1]
      if (matches) {
        filename = matches.replace(/['"]/g, '')
      }
    }

    link.download = filename || `download_${Date.now()}`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    window.URL.revokeObjectURL(downloadUrl)

    ElMessage.success('文件下载成功')
  } catch (error) {
    console.error('文件下载失败:', error)
    ElMessage.error('文件下载失败')
    throw error
  }
}

// 取消请求用的 CancelToken
export const CancelToken = axios.CancelToken
export const isCancel = axios.isCancel

// 导出http实例
export { http }

export default http
