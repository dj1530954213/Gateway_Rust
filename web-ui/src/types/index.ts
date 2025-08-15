// Type definitions for Edge Gateway Management System

export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  message?: string
  code?: number
}

export interface PaginationParams {
  page: number
  size: number
  total?: number
}

export interface BaseEntity {
  id: string
  created_at: string
  updated_at: string
}

// System types
export interface SystemInfo {
  version: string
  build_time: string
  git_commit: string
  uptime: number
  platform: string
  arch: string
}

export interface SystemMetrics {
  cpu_usage: number
  memory_usage: number
  disk_usage: number
  network_in: number
  network_out: number
  timestamp: string
}

export interface HealthStatus {
  overall: HealthLevel
  components: ComponentHealth[]
  last_check: string
}

export interface ComponentHealth {
  name: string
  status: HealthLevel
  uptime: number
  error_count: number
  last_error?: string
  metrics: Record<string, any>
}

export enum HealthLevel {
  Excellent = 'excellent',
  Good = 'good',
  Warning = 'warning',
  Critical = 'critical',
  Failed = 'failed',
}

// Driver types
export interface Driver extends BaseEntity {
  name: string
  driver_type: DriverType
  config: DriverConfig
  status: DriverStatus
  description?: string
  enabled: boolean
  last_poll_time?: string
  error_count: number
  data_points_count: number
}

export enum DriverType {
  ModbusTcp = 'modbus_tcp',
  ModbusRtu = 'modbus_rtu',
  OpcUa = 'opc_ua',
  EthernetIp = 'ethernet_ip',
  Custom = 'custom',
}

export enum DriverStatus {
  Running = 'running',
  Stopped = 'stopped',
  Error = 'error',
  Connecting = 'connecting',
}

export interface DriverConfig {
  host?: string
  port?: number
  device_id?: number
  endpoint_url?: string
  username?: string
  password?: string
  polling_interval: number
  timeout: number
  retry_count: number
  data_points: DataPoint[]
  [key: string]: any
}

// Data Point types
export interface DataPoint extends BaseEntity {
  name: string
  address: string
  data_type: DataType
  access_mode: AccessMode
  scale_factor?: number
  offset?: number
  unit?: string
  description?: string
  min_value?: number
  max_value?: number
  tags: string[]
}

export enum DataType {
  Bool = 'bool',
  Int16 = 'int16',
  Uint16 = 'uint16',
  Int32 = 'int32',
  Uint32 = 'uint32',
  Float32 = 'float32',
  Float64 = 'float64',
  String = 'string',
  Bytes = 'bytes',
}

export enum AccessMode {
  Read = 'read',
  Write = 'write',
  ReadWrite = 'read_write',
}

// Data Value types
export interface DataValue {
  point_id: string
  value: any
  quality: DataQuality
  timestamp: string
  source_timestamp?: string
}

export enum DataQuality {
  Good = 'good',
  Bad = 'bad',
  Uncertain = 'uncertain',
}

// Connector types
export interface Connector extends BaseEntity {
  name: string
  connector_type: ConnectorType
  config: ConnectorConfig
  status: ConnectorStatus
  description?: string
  enabled: boolean
  last_publish_time?: string
  published_count: number
  error_count: number
}

export enum ConnectorType {
  Mqtt = 'mqtt',
  Http = 'http',
  InfluxDB = 'influxdb',
  Database = 'database',
  Custom = 'custom',
}

export enum ConnectorStatus {
  Connected = 'connected',
  Disconnected = 'disconnected',
  Error = 'error',
  Connecting = 'connecting',
}

export interface ConnectorConfig {
  endpoint: string
  username?: string
  password?: string
  topic?: string
  headers?: Record<string, string>
  batch_size?: number
  flush_interval?: number
  compression?: boolean
  [key: string]: any
}

// Alert types
export interface Alert extends BaseEntity {
  name: string
  level: AlertLevel
  source: string
  message: string
  acknowledged: boolean
  acknowledged_by?: string
  acknowledged_at?: string
  resolved: boolean
  resolved_at?: string
  metadata: Record<string, any>
}

export enum AlertLevel {
  Info = 'info',
  Warning = 'warning',
  Error = 'error',
  Critical = 'critical',
}

export interface AlertRule extends BaseEntity {
  name: string
  description?: string
  enabled: boolean
  condition: AlertCondition
  actions: AlertAction[]
}

export interface AlertCondition {
  point_id?: string
  metric?: string
  operator: ComparisonOperator
  threshold: number
  duration?: number
}

export enum ComparisonOperator {
  GreaterThan = 'gt',
  LessThan = 'lt',
  Equal = 'eq',
  NotEqual = 'ne',
  GreaterThanOrEqual = 'gte',
  LessThanOrEqual = 'lte',
}

export interface AlertAction {
  type: AlertActionType
  config: Record<string, any>
}

export enum AlertActionType {
  Email = 'email',
  Webhook = 'webhook',
  Log = 'log',
}

// User and auth types
export interface User extends BaseEntity {
  username: string
  email: string
  full_name: string
  role: UserRole
  enabled: boolean
  last_login?: string
}

export enum UserRole {
  Admin = 'admin',
  Operator = 'operator',
  Viewer = 'viewer',
}

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  token: string
  expires_in: number
  user: User
}

// Configuration types
export interface GatewayConfig {
  system: SystemConfig
  drivers: Record<string, DriverConfig>
  connectors: Record<string, ConnectorConfig>
  security: SecurityConfig
  monitoring: MonitoringConfig
  advanced: AdvancedConfig
}

export interface SystemConfig {
  name: string
  description?: string
  log_level: LogLevel
  thread_pool_size: number
  max_connections: number
  api_port: number
  web_port: number
}

export enum LogLevel {
  Trace = 'trace',
  Debug = 'debug',
  Info = 'info',
  Warn = 'warn',
  Error = 'error',
}

export interface SecurityConfig {
  auth_enabled: boolean
  jwt_secret: string
  token_expiry: number
  tls_enabled: boolean
  cert_file?: string
  key_file?: string
}

export interface MonitoringConfig {
  enabled: boolean
  metrics_port: number
  health_check_interval: number
  alert_rules: AlertRule[]
}

export interface AdvancedConfig {
  ml_enabled: boolean
  analytics_enabled: boolean
  edge_compute_enabled: boolean
  optimization_enabled: boolean
}

// Chart and visualization types
export interface ChartData {
  labels: string[]
  datasets: ChartDataset[]
}

export interface ChartDataset {
  label: string
  data: number[]
  backgroundColor?: string
  borderColor?: string
  fill?: boolean
}

export interface TimeSeriesPoint {
  timestamp: string
  value: number
}

// Table types
export interface TableColumn {
  key: string
  label: string
  width?: number
  sortable?: boolean
  filterable?: boolean
  formatter?: (value: any, row: any) => string
}

export interface TableOptions {
  pagination?: boolean
  selection?: boolean
  search?: boolean
  export?: boolean
}

// Form types
export interface FormField {
  key: string
  label: string
  type: FormFieldType
  required?: boolean
  options?: SelectOption[]
  validation?: FormValidation
  placeholder?: string
  help?: string
}

export enum FormFieldType {
  Text = 'text',
  Number = 'number',
  Password = 'password',
  Select = 'select',
  MultiSelect = 'multi-select',
  Boolean = 'boolean',
  Date = 'date',
  DateTime = 'datetime',
  TextArea = 'textarea',
  Json = 'json',
}

export interface SelectOption {
  label: string
  value: any
  disabled?: boolean
}

export interface FormValidation {
  min?: number
  max?: number
  pattern?: string
  custom?: (value: any) => boolean | string
}

// Theme types
export interface ThemeConfig {
  mode: ThemeMode
  primaryColor: string
  accentColor: string
  fontSize: number
}

export enum ThemeMode {
  Light = 'light',
  Dark = 'dark',
  Auto = 'auto',
}

// WebSocket types
export interface WebSocketMessage {
  type: string
  data: any
  timestamp: string
}

export interface RealtimeData {
  data_values: DataValue[]
  system_metrics: SystemMetrics
  alerts: Alert[]
}
