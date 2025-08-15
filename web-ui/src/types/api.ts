/**
 * 系统API响应类型定义 - 与后端Rust API保持一致
 */
export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  error?: ApiError
  meta?: ResponseMeta
  timestamp: string
  request_id?: string
}

export interface ApiError {
  code: string
  message: string
  details?: any
  trace_id?: string
}

export interface ResponseMeta {
  pagination?: PaginationMeta
  performance?: PerformanceMeta
  version: string
  extra?: any
}

export interface PaginationMeta {
  page: number
  size: number
  total: number
  pages: number
  has_next: boolean
  has_prev: boolean
}

export interface PerformanceMeta {
  query_time_ms: number
  processing_time_ms: number
  total_time_ms: number
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  size: number
  pages: number
}

// ========== 枚举类型 ==========

export type ProtocolKind = 'ModbusTcp' | 'OpcUa' | 'Mqtt'
export type TagDataType = 'Float' | 'Int' | 'Bool' | 'String'
export type CompareOp = 'GT' | 'LT' | 'GTE' | 'LTE' | 'EQ' | 'NE'
export type AlertLevel = 'INFO' | 'WARN' | 'CRIT'
export type StatsKind = 'mean' | 'min' | 'max' | 'sum'

// ========== 设备相关类型 ==========

export interface DeviceVO {
  id: string
  name: string
  protocol: ProtocolKind
  location?: string
  endpoint?: string
  config?: any
  enabled: boolean
  created_at: string
  updated_at: string
}

export interface DeviceCreateReq {
  name: string
  protocol: ProtocolKind
  location?: string
  endpoint?: string
  config?: any
  enabled?: boolean
}

export interface DevicePatchReq {
  name?: string
  location?: string
  endpoint?: string
  config?: any
  enabled?: boolean
}

export interface DeviceQuery {
  protocol?: ProtocolKind
  enabled?: boolean
  name_contains?: string
  location_contains?: string
  endpoint_contains?: string
  page?: number
  size?: number
}

export interface DeviceTestResult {
  success: boolean
  message?: string
  latency_ms?: number
  error_details?: any
}

// 标签相关类型
export interface Tag {
  id: number
  device_id: number
  name: string
  address: string
  data_type:
    | 'boolean'
    | 'int16'
    | 'int32'
    | 'int64'
    | 'float32'
    | 'float64'
    | 'string'
  unit?: string
  scaling?: number
  offset?: number
  read_only: boolean
  created_at: string
  updated_at: string
  device?: Device
}

export interface TagValue {
  tag_id: number
  value: any
  timestamp: string
  quality: 'good' | 'uncertain' | 'bad'
}

export interface CreateTagRequest {
  device_id: number
  name: string
  address: string
  data_type: string
  unit?: string
  scaling?: number
  offset?: number
  read_only?: boolean
}

export interface UpdateTagRequest {
  name?: string
  address?: string
  data_type?: string
  unit?: string
  scaling?: number
  offset?: number
  read_only?: boolean
}

// 报警相关类型
export interface Alert {
  id: number
  message: string
  level: 'info' | 'warning' | 'error' | 'critical'
  source: string
  source_id?: number
  created_at: string
  acknowledged: boolean
  acknowledged_at?: string
  acknowledged_by?: string
}

export interface CreateAlertRequest {
  message: string
  level: 'info' | 'warning' | 'error' | 'critical'
  source: string
  source_id?: number
}

export interface UpdateAlertRequest {
  acknowledged?: boolean
}

// 连接器相关类型
export interface Connector {
  id: number
  name: string
  connector_type: string
  config: Record<string, unknown>
  enabled: boolean
  status: 'running' | 'stopped' | 'error'
  created_at: string
  updated_at: string
  last_activity?: string
}

export interface CreateConnectorRequest {
  name: string
  connector_type: string
  config: Record<string, unknown>
  enabled?: boolean
}

export interface UpdateConnectorRequest {
  name?: string
  config?: Record<string, unknown>
  enabled?: boolean
}

// 统计分析类型
export interface SystemStats {
  devices: {
    total: number
    online: number
    offline: number
    error: number
  }
  tags: {
    total: number
    active: number
  }
  alerts: {
    total: number
    unacknowledged: number
    by_level: Record<string, number>
  }
  data_points: {
    last_hour: number
    last_day: number
    total: number
  }
}

export interface PerformanceMetrics {
  cpu_usage: number
  memory_usage: number
  disk_usage: number
  network_io: {
    bytes_in: number
    bytes_out: number
  }
  frame_bus_stats: {
    messages_per_second: number
    queue_depth: number
    latency_ms: number
  }
}

// 数据查询类型
export interface TimeSeriesQuery {
  tag_ids: number[]
  start_time: string
  end_time: string
  aggregation?: 'avg' | 'min' | 'max' | 'sum' | 'count'
  interval?: string
}

export interface TimeSeriesDataPoint {
  timestamp: string
  value: number
  tag_id: number
}

export interface TimeSeriesResponse {
  data: TimeSeriesDataPoint[]
  meta: {
    query: TimeSeriesQuery
    total_points: number
    duration_ms: number
  }
}

// 用户认证类型
export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  token: string
  user: User
  expires_at: string
}

export interface User {
  id: number
  username: string
  email?: string
  role: 'admin' | 'operator' | 'viewer'
  created_at: string
  updated_at: string
  last_login?: string
}

// WebSocket消息类型
export interface WebSocketMessage<T = any> {
  type: string
  payload: T
  timestamp?: string
}

export interface DeviceDataMessage extends WebSocketMessage {
  type: 'device_data'
  payload: {
    device_id: number
    tag_values: TagValue[]
  }
}

export interface AlertMessage extends WebSocketMessage {
  type: 'alert'
  payload: Alert
}

export interface SystemStatusMessage extends WebSocketMessage {
  type: 'system_status'
  payload: {
    status: 'healthy' | 'degraded' | 'error'
    message?: string
    details?: Record<string, any>
  }
}

// 表格查询参数
export interface QueryParams {
  page?: number
  limit?: number
  search?: string
  sort_by?: string
  sort_order?: 'asc' | 'desc'
  filters?: Record<string, any>
}

// 导出导入类型
export interface ExportConfig {
  format: 'json' | 'csv' | 'excel'
  include_data?: boolean
  date_range?: {
    start: string
    end: string
  }
}

export interface ImportResult {
  success: boolean
  imported_count: number
  failed_count: number
  errors?: string[]
}

// 表单验证类型
export interface ValidationError {
  field: string
  message: string
  code?: string
}

export interface FormState<T> {
  data: T
  loading: boolean
  errors: ValidationError[]
  dirty: boolean
  valid: boolean
}

// 通用工具类型
export type LoadingState = 'idle' | 'loading' | 'success' | 'error'

export interface SelectOption {
  label: string
  value: string | number
  disabled?: boolean
  children?: SelectOption[]
}

export interface TableColumn {
  key: string
  title: string
  sortable?: boolean
  width?: number
  align?: 'left' | 'center' | 'right'
  formatter?: (value: any, record: any) => string
}

export interface ChartOptions {
  title?: string
  subtitle?: string
  x_axis?: {
    title: string
    type: 'datetime' | 'category' | 'value'
  }
  y_axis?: {
    title: string
    type: 'value' | 'log'
  }
  series?: Array<{
    name: string
    type: 'line' | 'bar' | 'area' | 'scatter'
    data: Array<[string | number, number]>
  }>
}
