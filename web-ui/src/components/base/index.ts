// 基础组件统一导出
export { default as BaseTable } from './BaseTable.vue'
export { default as BaseForm } from './BaseForm.vue'
export { default as StatusTag } from './StatusTag.vue'
export { default as ActionButtons } from './ActionButtons.vue'
export { default as SearchBox } from './SearchBox.vue'
export { default as FilterPanel } from './FilterPanel.vue'
export { default as ConfirmDialog } from './ConfirmDialog.vue'
export { default as LoadingCard } from './LoadingCard.vue'

// 导出类型定义
export type {
  TableColumn,
  TableAction,
  TableData,
  PaginationConfig,
  SortConfig,
} from './BaseTable.vue'

export type {
  FormField,
  FormValue,
  ValidationRule,
  FormConfig,
} from './BaseForm.vue'

export type { StatusMapping } from './StatusTag.vue'

export type { ActionButton, DropdownItem } from './ActionButtons.vue'

export type { SearchSuggestion, AdvancedField } from './SearchBox.vue'

export type { FilterItem, QuickFilter } from './FilterPanel.vue'

export type { ConfirmType, AutoAction } from './ConfirmDialog.vue'

export type { LoadingStatus, AnimationType } from './LoadingCard.vue'
