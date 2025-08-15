// 业务组件统一导出
export { default as ConnectionStatus } from './ConnectionStatus.vue'
export { default as DataPointSelector } from './DataPointSelector.vue'
export { default as ProtocolConfig } from './ProtocolConfig.vue'
export { default as TimeRangePicker } from './TimeRangePicker.vue'
export { default as ChartContainer } from './ChartContainer.vue'
export { default as ConfigValidation } from './ConfigValidation.vue'
export { default as LogViewer } from './LogViewer.vue'
export { default as FileUploader } from './FileUploader.vue'

// 导出类型定义
export type {
  ConnectionInfo,
  ConnectionHistoryRecord,
} from './ConnectionStatus.vue'
export type { DataPoint } from './DataPointSelector.vue'
export type {
  ProtocolType,
  ProtocolConfig,
  DataPointMapping,
} from './ProtocolConfig.vue'
export type {
  TimeRange,
  QuickRange,
  TimeTemplate,
  TimeHistoryRecord,
} from './TimeRangePicker.vue'
export type { ChartType, ChartData, ChartConfig } from './ChartContainer.vue'
export type {
  ValidationType,
  ValidationResult,
  ValidationRule,
  ValidationHistoryRecord,
} from './ConfigValidation.vue'
export type { LogLevel, LogEntry } from './LogViewer.vue'
export type {
  UploadStatus,
  FileItem,
  UploadType,
  UploadHistoryRecord,
} from './FileUploader.vue'
