/**
 * Pinia Stores 统一导出
 */

// Auth Store
export { useAuthStore } from './auth'
export type { User, LoginRequest, LoginResponse } from '@/types/auth'

// Devices Store
export { useDevicesStore } from './devices'
export type { DevicesStore, DeviceState } from './devices'

// Tags Store
export { useTagsStore } from './tags'
export type { TagsStore, TagState } from './tags'

// Drivers Store
export { useDriversStore } from './drivers'
export type { DriversStore, DriverState } from './drivers'

// History Store
export { useHistoryStore } from './history'
export type { HistoryStore, HistoryState } from './history'

// Alerts Store
export { useAlertsStore } from './alerts'
export type { AlertsStore, AlertState } from './alerts'