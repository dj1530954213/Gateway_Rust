/**
 * Format utilities for display values
 */

/**
 * Format byte size to human readable string
 */
export function formatBytes(bytes: number, decimals: number = 2): string {
  if (bytes === 0) return '0 B'
  if (bytes < 0) return '-'

  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']

  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`
}

/**
 * Alias for formatBytes - Format file size to human readable string
 */
export function formatFileSize(bytes: number, decimals: number = 2): string {
  return formatBytes(bytes, decimals)
}

/**
 * Format percentage with specified decimals
 */
export function formatPercentage(value: number, decimals: number = 1): string {
  if (typeof value !== 'number' || isNaN(value)) return '-'

  return `${value.toFixed(decimals)}%`
}

/**
 * Format number with thousands separators
 */
export function formatNumber(value: number, decimals?: number): string {
  if (typeof value !== 'number' || isNaN(value)) return '-'

  const options: Intl.NumberFormatOptions = {
    useGrouping: true,
  }

  if (typeof decimals === 'number') {
    options.minimumFractionDigits = decimals
    options.maximumFractionDigits = decimals
  }

  return new Intl.NumberFormat('zh-CN', options).format(value)
}

/**
 * Format large numbers with unit suffixes (K, M, B)
 */
export function formatLargeNumber(value: number, decimals: number = 1): string {
  if (typeof value !== 'number' || isNaN(value)) return '-'
  if (value === 0) return '0'

  const absValue = Math.abs(value)
  const sign = value < 0 ? '-' : ''

  if (absValue >= 1e9) {
    return `${sign}${(absValue / 1e9).toFixed(decimals)}B`
  } else if (absValue >= 1e6) {
    return `${sign}${(absValue / 1e6).toFixed(decimals)}M`
  } else if (absValue >= 1e3) {
    return `${sign}${(absValue / 1e3).toFixed(decimals)}K`
  } else {
    return `${sign}${absValue.toFixed(decimals)}`
  }
}

/**
 * Format currency value
 */
export function formatCurrency(
  value: number,
  currency: string = 'CNY',
  locale: string = 'zh-CN'
): string {
  if (typeof value !== 'number' || isNaN(value)) return '-'

  return new Intl.NumberFormat(locale, {
    style: 'currency',
    currency,
  }).format(value)
}

/**
 * Format network speed (bits per second)
 */
export function formatNetworkSpeed(bps: number): string {
  if (typeof bps !== 'number' || isNaN(bps) || bps < 0) return '-'

  const units = ['bps', 'Kbps', 'Mbps', 'Gbps', 'Tbps']
  let value = bps
  let unitIndex = 0

  while (value >= 1000 && unitIndex < units.length - 1) {
    value /= 1000
    unitIndex++
  }

  const decimals = value >= 100 ? 0 : value >= 10 ? 1 : 2
  return `${value.toFixed(decimals)} ${units[unitIndex]}`
}

/**
 * Format latency in milliseconds
 */
export function formatLatency(ms: number): string {
  if (typeof ms !== 'number' || isNaN(ms) || ms < 0) return '-'

  if (ms < 1) {
    return `${(ms * 1000).toFixed(0)}μs`
  } else if (ms < 1000) {
    return `${ms.toFixed(ms < 10 ? 2 : ms < 100 ? 1 : 0)}ms`
  } else {
    return `${(ms / 1000).toFixed(ms < 10000 ? 2 : 1)}s`
  }
}

/**
 * Format temperature with unit
 */
export function formatTemperature(
  celsius: number,
  unit: 'C' | 'F' = 'C'
): string {
  if (typeof celsius !== 'number' || isNaN(celsius)) return '-'

  if (unit === 'F') {
    const fahrenheit = (celsius * 9) / 5 + 32
    return `${fahrenheit.toFixed(1)}°F`
  }

  return `${celsius.toFixed(1)}°C`
}

/**
 * Format IP address for display
 */
export function formatIPAddress(ip: string): string {
  if (!ip || typeof ip !== 'string') return '-'

  // IPv6 compression (basic)
  if (ip.includes(':')) {
    return ip
      .toLowerCase()
      .replace(/^0+|:0+/g, ':')
      .replace(/::+/g, '::')
  }

  // IPv4 validation
  const ipv4Regex = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/
  if (ipv4Regex.test(ip)) {
    return ip
  }

  return ip
}

/**
 * Format MAC address for display
 */
export function formatMACAddress(mac: string): string {
  if (!mac || typeof mac !== 'string') return '-'

  // Remove any existing separators and normalize
  const clean = mac.replace(/[:-]/g, '').toLowerCase()

  if (clean.length !== 12) return mac

  // Add colons every 2 characters
  return clean.match(/.{2}/g)?.join(':') || mac
}

/**
 * Format error message for display
 */
export function formatErrorMessage(error: any): string {
  if (!error) return '未知错误'

  if (typeof error === 'string') return error

  if (error.message) return error.message

  if (error.error) return error.error

  return JSON.stringify(error)
}

/**
 * Format API endpoint for display
 */
export function formatEndpoint(endpoint: string): string {
  if (!endpoint) return '-'

  try {
    const url = new URL(endpoint)
    return `${url.protocol}//${url.host}${url.pathname}`
  } catch {
    return endpoint
  }
}

/**
 * Format version string
 */
export function formatVersion(version: string): string {
  if (!version) return '-'

  // Remove 'v' prefix if present
  const clean = version.replace(/^v/, '')

  // Split by dots and ensure numeric parts
  const parts = clean.split('.')
  if (parts.length >= 3) {
    return parts.slice(0, 3).join('.')
  }

  return clean
}

/**
 * Format status for display with icon
 */
export function formatStatus(status: string): {
  text: string
  type: 'success' | 'warning' | 'danger' | 'info'
  icon: string
} {
  const statusMap: Record<string, any> = {
    running: { text: '运行中', type: 'success', icon: 'CircleCheck' },
    stopped: { text: '已停止', type: 'info', icon: 'CircleClose' },
    error: { text: '错误', type: 'danger', icon: 'Warning' },
    warning: { text: '警告', type: 'warning', icon: 'Warning' },
    connecting: { text: '连接中', type: 'warning', icon: 'Loading' },
    connected: { text: '已连接', type: 'success', icon: 'Connection' },
    disconnected: { text: '已断开', type: 'danger', icon: 'Connection' },
    online: { text: '在线', type: 'success', icon: 'CircleCheck' },
    offline: { text: '离线', type: 'danger', icon: 'CircleClose' },
    enabled: { text: '启用', type: 'success', icon: 'Check' },
    disabled: { text: '禁用', type: 'info', icon: 'Close' },
  }

  return (
    statusMap[status?.toLowerCase()] || {
      text: status || '未知',
      type: 'info',
      icon: 'QuestionFilled',
    }
  )
}

/**
 * Format log level for display
 */
export function formatLogLevel(level: string): {
  text: string
  type: 'success' | 'warning' | 'danger' | 'info'
} {
  const levelMap: Record<string, any> = {
    trace: { text: 'TRACE', type: 'info' },
    debug: { text: 'DEBUG', type: 'info' },
    info: { text: 'INFO', type: 'success' },
    warn: { text: 'WARN', type: 'warning' },
    error: { text: 'ERROR', type: 'danger' },
  }

  return (
    levelMap[level?.toLowerCase()] || {
      text: level?.toUpperCase() || 'UNKNOWN',
      type: 'info',
    }
  )
}

/**
 * Format health level for display
 */
export function formatHealthLevel(level: string): {
  text: string
  type: 'success' | 'warning' | 'danger' | 'info'
  color: string
} {
  const healthMap: Record<string, any> = {
    excellent: { text: '优秀', type: 'success', color: '#67c23a' },
    good: { text: '良好', type: 'success', color: '#95d475' },
    warning: { text: '警告', type: 'warning', color: '#e6a23c' },
    critical: { text: '严重', type: 'danger', color: '#f56c6c' },
    failed: { text: '失败', type: 'danger', color: '#ff4757' },
  }

  return (
    healthMap[level?.toLowerCase()] || {
      text: level || '未知',
      type: 'info',
      color: '#909399',
    }
  )
}

/**
 * Truncate text with ellipsis
 */
export function truncateText(text: string, maxLength: number = 50): string {
  if (!text || typeof text !== 'string') return '-'

  if (text.length <= maxLength) return text

  return `${text.substring(0, maxLength - 3)}...`
}

/**
 * Format array as comma-separated string
 */
export function formatArray(arr: any[], maxItems: number = 3): string {
  if (!Array.isArray(arr) || arr.length === 0) return '-'

  const items = arr.slice(0, maxItems)
  const result = items.join(', ')

  if (arr.length > maxItems) {
    return `${result} (+${arr.length - maxItems})`
  }

  return result
}

/**
 * Format boolean for display
 */
export function formatBoolean(value: boolean): string {
  return value ? '是' : '否'
}

/**
 * Format null/undefined values
 */
export function formatEmpty(value: any, placeholder: string = '-'): string {
  if (value === null || value === undefined || value === '') {
    return placeholder
  }

  return String(value)
}
