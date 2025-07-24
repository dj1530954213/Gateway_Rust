import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import customParseFormat from 'dayjs/plugin/customParseFormat'
import timezone from 'dayjs/plugin/timezone'
import utc from 'dayjs/plugin/utc'
import 'dayjs/locale/zh-cn'

// Configure dayjs
dayjs.extend(relativeTime)
dayjs.extend(customParseFormat)
dayjs.extend(timezone)
dayjs.extend(utc)
dayjs.locale('zh-cn')

export const DATE_FORMATS = {
  DATE: 'YYYY-MM-DD',
  TIME: 'HH:mm:ss',
  DATETIME: 'YYYY-MM-DD HH:mm:ss',
  DATETIME_MINUTE: 'YYYY-MM-DD HH:mm',
  ISO: 'YYYY-MM-DDTHH:mm:ss.SSSZ',
  DISPLAY: 'YYYY年MM月DD日 HH:mm:ss',
  MONTH_DAY: 'MM-DD',
  HOUR_MINUTE: 'HH:mm',
} as const

/**
 * Format timestamp to human readable string
 */
export function formatTime(
  timestamp: string | number | Date,
  format: string = DATE_FORMATS.DATETIME
): string {
  if (!timestamp) return '-'
  
  try {
    return dayjs(timestamp).format(format)
  } catch (error) {
    console.error('Date formatting error:', error)
    return '-'
  }
}

/**
 * Get relative time (e.g., "2 hours ago")
 */
export function getRelativeTime(timestamp: string | number | Date): string {
  if (!timestamp) return '-'
  
  try {
    return dayjs(timestamp).fromNow()
  } catch (error) {
    console.error('Relative time error:', error)
    return '-'
  }
}

/**
 * Format duration in seconds to human readable string
 */
export function formatDuration(seconds: number): string {
  if (!seconds || seconds < 0) return '0秒'
  
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const remainingSeconds = seconds % 60
  
  const parts: string[] = []
  
  if (days > 0) parts.push(`${days}天`)
  if (hours > 0) parts.push(`${hours}小时`)
  if (minutes > 0) parts.push(`${minutes}分钟`)
  if (remainingSeconds > 0 || parts.length === 0) {
    parts.push(`${Math.floor(remainingSeconds)}秒`)
  }
  
  return parts.join('')
}

/**
 * Format uptime in seconds to readable string
 */
export function formatUptime(seconds: number): string {
  return formatDuration(seconds)
}

/**
 * Get start and end of day for given date
 */
export function getDayRange(date?: string | Date): { start: string; end: string } {
  const day = date ? dayjs(date) : dayjs()
  
  return {
    start: day.startOf('day').toISOString(),
    end: day.endOf('day').toISOString(),
  }
}

/**
 * Get start and end of week for given date
 */
export function getWeekRange(date?: string | Date): { start: string; end: string } {
  const day = date ? dayjs(date) : dayjs()
  
  return {
    start: day.startOf('week').toISOString(),
    end: day.endOf('week').toISOString(),
  }
}

/**
 * Get start and end of month for given date
 */
export function getMonthRange(date?: string | Date): { start: string; end: string } {
  const day = date ? dayjs(date) : dayjs()
  
  return {
    start: day.startOf('month').toISOString(),
    end: day.endOf('month').toISOString(),
  }
}

/**
 * Get time range for specified period
 */
export function getTimeRange(period: 'hour' | 'day' | 'week' | 'month' | 'year', count: number = 1): {
  start: string
  end: string
} {
  const now = dayjs()
  const start = now.subtract(count, period)
  
  return {
    start: start.toISOString(),
    end: now.toISOString(),
  }
}

/**
 * Check if timestamp is today
 */
export function isToday(timestamp: string | number | Date): boolean {
  if (!timestamp) return false
  
  try {
    return dayjs(timestamp).isSame(dayjs(), 'day')
  } catch {
    return false
  }
}

/**
 * Check if timestamp is within last N minutes
 */
export function isWithinMinutes(timestamp: string | number | Date, minutes: number): boolean {
  if (!timestamp) return false
  
  try {
    const diff = dayjs().diff(dayjs(timestamp), 'minute')
    return diff <= minutes
  } catch {
    return false
  }
}

/**
 * Generate time series labels for charts
 */
export function generateTimeLabels(
  start: string | Date,
  end: string | Date,
  interval: 'minute' | 'hour' | 'day' | 'week' | 'month' = 'hour'
): string[] {
  const startTime = dayjs(start)
  const endTime = dayjs(end)
  const labels: string[] = []
  
  let current = startTime
  let format: string
  
  switch (interval) {
    case 'minute':
      format = DATE_FORMATS.HOUR_MINUTE
      break
    case 'hour':
      format = DATE_FORMATS.HOUR_MINUTE
      break
    case 'day':
      format = DATE_FORMATS.MONTH_DAY
      break
    case 'week':
    case 'month':
      format = DATE_FORMATS.DATE
      break
    default:
      format = DATE_FORMATS.DATETIME_MINUTE
  }
  
  while (current.isBefore(endTime) || current.isSame(endTime)) {
    labels.push(current.format(format))
    current = current.add(1, interval)
  }
  
  return labels
}

/**
 * Parse ISO string to local timezone
 */
export function parseISOToLocal(isoString: string): Date {
  return dayjs(isoString).toDate()
}

/**
 * Convert local date to ISO string
 */
export function localToISO(date: Date): string {
  return dayjs(date).toISOString()
}

/**
 * Format file timestamp for display
 */
export function formatFileTime(timestamp: string | number | Date): string {
  if (!timestamp) return '-'
  
  const now = dayjs()
  const time = dayjs(timestamp)
  const diffDays = now.diff(time, 'day')
  
  if (diffDays === 0) {
    return time.format('今天 HH:mm')
  } else if (diffDays === 1) {
    return time.format('昨天 HH:mm')
  } else if (diffDays < 7) {
    return time.format('MM-DD HH:mm')
  } else {
    return time.format(DATE_FORMATS.DATETIME_MINUTE)
  }
}

/**
 * Get business hours for scheduling
 */
export function getBusinessHours(): { start: string; end: string } {
  return {
    start: '09:00',
    end: '18:00',
  }
}

/**
 * Check if current time is within business hours
 */
export function isBusinessHours(): boolean {
  const now = dayjs()
  const { start, end } = getBusinessHours()
  const startTime = dayjs(start, 'HH:mm')
  const endTime = dayjs(end, 'HH:mm')
  
  return now.isAfter(startTime) && now.isBefore(endTime)
}

/**
 * Format timestamp for export filename
 */
export function formatFilename(prefix: string = '', extension: string = ''): string {
  const timestamp = dayjs().format('YYYYMMDD_HHmmss')
  return `${prefix}${prefix ? '_' : ''}${timestamp}${extension ? '.' + extension : ''}`
}

/**
 * Calculate age from date
 */
export function calculateAge(date: string | Date): number {
  return dayjs().diff(dayjs(date), 'year')
}

/**
 * Get timezone information
 */
export function getTimezoneInfo(): {
  name: string
  offset: string
  abbreviation: string
} {
  const now = dayjs()
  
  return {
    name: Intl.DateTimeFormat().resolvedOptions().timeZone,
    offset: now.format('Z'),
    abbreviation: now.format('z'),
  }
}